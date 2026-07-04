//! WV-11 Browser Surface 検証用 CEF モジュール。
//!
//! 役割:
//! - CEF OSR 検証コードの公開口を提供する。
//! - CEF 依存を Framework 内部の実験用境界に閉じ込める。
//! - `third_party/cef/<os>` 配置の CEF ランタイムを探索する。
//!
//! 注意点:
//! - 本モジュールは技術検証用であり、正式 API 仕様ではない。
//! - WV-11-02 では、まず CEF ライブラリの動的ロードと主要シンボル解決を確認する。
//! - Browser 作成、OSR、OnPaint、RGBA バッファ取得は後続ステップで追加する。

pub mod ffi;

use std::path::{Path, PathBuf};

/// CEF OSR 検証の初期段階を実行する。
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
/// - 現段階では `cef_initialize` を呼び出さない。
/// - CEF ABI 構造体定義を追加した後、初期化呼び出しへ進む。
pub fn run_symbol_probe(library_path: Option<PathBuf>) -> Result<String, String> {
    let path = resolve_cef_library_path(library_path)?;
    let probe = ffi::CefLibraryProbe::load(&path)?;
    probe.resolve_required_symbols()?;

    Ok(format!(
        "CEF library loaded and required symbols resolved: {}",
        probe.path().display()
    ))
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
/// - その他: `unknown`
#[must_use]
pub fn default_cef_platform_dir() -> &'static str {
    if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "unknown"
    }
}
