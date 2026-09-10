//! WV-11 Browser Surface 検証用 CEF モジュール。
//!
//! 役割:
//! - CEF OSR 検証コードの公開口を提供する。
//! - CEF 依存を Framework 内部の実験用境界に閉じ込める。
//! - `third_party/cef/<os>` 配置の CEF ランタイムを探索する。
//!
//! 注意点:
//! - 本モジュールは技術検証用であり、正式 API 仕様ではない。
//! - WV-11-02 の Runtime / Symbol 検証は `libloading` による既存 Probe を使用する。
//! - CEF Initialize 以降は、検証基準 CEF と一致する `cef-rs` バインディングを使用し、
//!   CEF C API ABI の手書き複製を避ける。
//! - Browser 作成、OSR、OnPaint、描画バッファ取得は後続ステップで追加する。

pub mod ffi;

use cef::{args::Args, api_hash, execute_process, initialize, shutdown, Settings};
use std::path::{Path, PathBuf};
use std::ptr;

/// WV-11-02 Runtime / Symbol Probe を実行する。
///
/// @hldocs.ref doc-20260628-000011Z-WV11#sec_10811wkg708i
/// @hldocs.ref doc-20260628-000011Z-WV11#sec_pkl5oq8fkmc6
///
/// # 役割
/// - CEF ライブラリをロードする。
/// - WV-11-02 の初期検証に必要な主要シンボルを解決する。
///
/// # 引数
/// - `library_path`: CEF ライブラリまたは CEF 配置ディレクトリへの明示パス。
///   - ファイルの場合はそのままロードする。
///   - ディレクトリの場合は OS ごとの CEF ライブラリ名を連結してロードする。
///   - `None` の場合は `third_party/cef/<os>/<libcef>` を使用する。
///
/// # 戻り値
/// - 成功時: 検証ログ文字列。
/// - 失敗時: 失敗理由を含むエラー文字列。
///
/// # 注意点
/// - 本 Probe は `cef_initialize` を呼び出さない。
/// - CEF 初期化は `run_initialize_probe` で独立して検証する。
pub fn run_symbol_probe(library_path: Option<PathBuf>) -> Result<String, String> {
    let path = resolve_cef_library_path(library_path)?;
    let probe = ffi::CefLibraryProbe::load(&path)?;
    probe.resolve_required_symbols()?;

    Ok(format!(
        "CEF library loaded and required symbols resolved: {}",
        probe.path().display()
    ))
}

/// WV-11-02 CEF Initialize Probe を実行する。
///
/// @hldocs.ref doc-20260628-000011Z-WV11#sec_aj1rfgg9rguj
///
/// # 役割
/// - `cef-rs` が使用する CEF API バージョンを初期化する。
/// - `cef_execute_process` により CEF subprocess を先に処理する。
/// - Browser Process では Windowless Rendering を有効にした最小設定で CEF を初期化する。
/// - 初期化成功後、Browser を作成せず直ちに CEF を shutdown する。
///
/// # 戻り値
/// - Browser Process で成功時: CEF 初期化と shutdown 成功を示す文字列。
/// - CEF subprocess で成功時: subprocess の終了コードを示す文字列。
/// - 失敗時: CEF 初期化失敗を示すエラー文字列。
///
/// # 注意点
/// - 本 Probe は CEF-IT-SPEC-003 のみを対象とする。
/// - `cef_execute_process` が 0 以上を返した場合、そのプロセスは subprocess なので
///   `cef_initialize` / `cef_shutdown` を呼び出してはならない。
/// - Browser 作成は CEF-IT-SPEC-004 以降で行う。
pub fn run_initialize_probe() -> Result<String, String> {
    // cef-rs の生成バインディングが対象 CEF と同じ API バージョンを使用するよう初期化する。
    let _ = api_hash(cef::sys::CEF_API_VERSION_LAST, 0);

    let args = Args::new();

    // CEF はマルチプロセス構成を前提とする。
    // subprocess では execute_process が 0 以上を返すため、アプリ本体の初期化へ進まず終了する。
    let subprocess_exit_code = execute_process(
        Some(args.as_main_args()),
        None,
        ptr::null_mut(),
    );

    if subprocess_exit_code >= 0 {
        return Ok(format!(
            "CEF subprocess completed with exit code {subprocess_exit_code}"
        ));
    }

    let settings = Settings {
        windowless_rendering_enabled: 1,
        external_message_pump: 1,
        no_sandbox: 1,
        ..Default::default()
    };

    let initialized = initialize(
        Some(args.as_main_args()),
        Some(&settings),
        None,
        ptr::null_mut(),
    );

    if initialized != 1 {
        return Err(format!(
            "CEF initialize failed: cef_initialize returned {initialized}"
        ));
    }

    shutdown();

    Ok("CEF initialize and shutdown succeeded".to_string())
}

/// CEF ライブラリパスを解決する。
///
/// # 役割
/// - `--cef-path` にファイルまたはディレクトリのどちらを渡しても扱えるようにする。
/// - 未指定時は `third_party/cef/<os>` 配下を既定配置として扱う。
///
/// # 引数
/// - `path`: 明示指定された CEF ライブラリまたは CEF 配置ディレクトリ。
///
/// # 戻り値
/// - CEF ライブラリファイルのパス。
///
/// # 注意点
/// - ファイル存在確認は行う。
/// - CEF の依存ファイル存在確認は後続ステップで追加する。
pub fn resolve_cef_library_path(path: Option<PathBuf>) -> Result<PathBuf, String> {
    let candidate = match path {
        Some(path) => to_library_path(path),
        None => default_cef_library_path(),
    };

    if candidate.exists() {
        Ok(candidate)
    } else {
        Err(format!(
            "CEF library not found: {}",
            candidate.display()
        ))
    }
}

/// 指定パスを CEF ライブラリファイルパスへ変換する。
///
/// # 役割
/// - ディレクトリ指定時に OS ごとの CEF ライブラリ名を付与する。
///
/// # 引数
/// - `path`: ファイルまたはディレクトリのパス。
///
/// # 戻り値
/// - CEF ライブラリファイルパス。
fn to_library_path(path: PathBuf) -> PathBuf {
    if path.is_dir() {
        path.join(ffi::default_cef_library_name())
    } else {
        path
    }
}

/// 既定の CEF ライブラリファイルパスを返す。
///
/// # 役割
/// - 技術検証用に `third_party/cef/<os>` 配置を標準探索先とする。
///
/// # 戻り値
/// - OS ごとの CEF ライブラリファイルパス。
#[must_use]
pub fn default_cef_library_path() -> PathBuf {
    Path::new("third_party")
        .join("cef")
        .join(default_cef_platform_dir())
        .join(ffi::default_cef_library_name())
}

/// CEF ランタイム配置用の OS 別ディレクトリ名を返す。
///
/// # 役割
/// - `third_party/cef/<os>` の `<os>` を決定する。
///
/// # 戻り値
/// - Windows: `windows`
/// - Linux: `linux`
/// - macOS: `macos`
/// - その他: `unknown`
#[must_use]
pub fn default_cef_platform_dir() -> &'static str {
    if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else {
        "unknown"
    }
}
