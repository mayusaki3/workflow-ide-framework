//! WV-11 Browser Surface 検証用 CEF モジュール。
//!
//! 役割:
//! - CEF OSR 検証コードの公開口を提供する。
//! - CEF 依存を Framework 内部の実験用境界に閉じ込める。
//! - `cef-dll-sys` が実行バイナリ出力先へ配置した CEF ランタイムを検証対象とする。
//!
//! 注意点:
//! - 本モジュールは技術検証用であり、正式 API 仕様ではない。
//! - WV-11-02 の Runtime / Symbol 検証は `libloading` による既存 Probe を使用する。
//! - CEF Initialize 以降は、検証基準 CEF と一致する `cef-rs` バインディングを使用し、
//!   CEF C API ABI の手書き複製を避ける。
//! - Paint、描画バッファ取得、継続描画更新は後続ステップで追加する。

pub mod ffi;

use cef::*;
use std::path::PathBuf;
use std::ptr;
use std::thread::sleep;
use std::time::Duration;

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
///   - `None` の場合は、現在の実行ファイルと同じディレクトリにある CEF ライブラリを使用する。
///
/// # 戻り値
/// - 成功時: 検証ログ文字列。
/// - 失敗時: 失敗理由を含むエラー文字列。
///
/// # 注意点
/// - 本 Probe は `cef_initialize` を呼び出さない。
/// - CEF 初期化は `run_initialize_probe` で独立して検証する。
/// - Windows / Linux では `cef-dll-sys` が CEF ランタイムを実行バイナリ出力先へ配置するため、
///   既定 Probe でも Initialize Probe と同じランタイムを確認できる。
pub fn run_symbol_probe(library_path: Option<PathBuf>) -> Result<String, String> {
    let path = resolve_cef_library_path(library_path)?;
    let probe = ffi::CefLibraryProbe::load(&path)?;
    probe.resolve_required_symbols()?;

    Ok(format!(
        "CEF library loaded and required symbols resolved: {}",
        probe.path().display()
    ))
}

/// CEF subprocess を実行する。
///
/// @hldocs.ref doc-20260628-000011Z-WV11#sec_aj1rfgg9rguj
///
/// # 役割
/// - `--type=<process>` を持つ CEF subprocess から `cef_execute_process` を実行する。
/// - renderer / gpu-process 等を通常の eframe アプリ起動経路へ入れない。
///
/// # 戻り値
/// - CEF subprocess の終了コード。
/// - `cef_execute_process` が負値を返した異常時は `1`。
///
/// # 注意点
/// - この関数は `main` の通常アプリ起動判定より前から呼び出す。
/// - CEF subprocess では `cef_initialize` / `cef_shutdown` を呼び出さない。
pub fn run_subprocess() -> i32 {
    let _ = api_hash(sys::CEF_API_VERSION_LAST, 0);
    let args = args::Args::new();
    let exit_code = execute_process(
        Some(args.as_main_args()),
        None,
        ptr::null_mut(),
    );

    if exit_code >= 0 {
        exit_code
    } else {
        eprintln!(
            "WV-11-02 CEF subprocess failed: cef_execute_process returned {exit_code}"
        );
        1
    }
}

/// WV-11-02 CEF Initialize Probe を実行する。
///
/// @hldocs.ref doc-20260628-000011Z-WV11#sec_aj1rfgg9rguj
///
/// # 役割
/// - `cef-rs` が使用する CEF API バージョンを初期化する。
/// - Browser Process では Windowless Rendering のみを有効にした最小設定で CEF を初期化する。
/// - 初期化成功後、Browser を作成せず直ちに CEF を shutdown する。
///
/// # 戻り値
/// - Browser Process で成功時: CEF 初期化と shutdown 成功を示す文字列。
/// - 失敗時: CEF 初期化失敗を示すエラー文字列。
///
/// # 注意点
/// - 本 Probe は CEF-IT-SPEC-003 のみを対象とする。
/// - CEF subprocess は `main` 冒頭で `run_subprocess` へ分岐済みであることを前提とする。
/// - `external_message_pump` は Browser / UI イベントループ統合時に検証するため、
///   初期化単体 Probe では有効化しない。
/// - Browser 作成は CEF-IT-SPEC-004 以降で行う。
pub fn run_initialize_probe() -> Result<String, String> {
    let _ = api_hash(sys::CEF_API_VERSION_LAST, 0);
    let args = args::Args::new();

    let process_result = execute_process(
        Some(args.as_main_args()),
        None,
        ptr::null_mut(),
    );

    if process_result >= 0 {
        return Ok(format!(
            "CEF subprocess completed with exit code {process_result}"
        ));
    }

    let settings = Settings {
        windowless_rendering_enabled: 1,
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

#[derive(Clone)]
struct BrowserProbeRenderHandler;

wrap_render_handler! {
    struct BrowserProbeRenderHandlerBuilder {
        handler: BrowserProbeRenderHandler,
    }

    impl RenderHandler {
        fn view_rect(&self, _browser: Option<&mut Browser>, rect: Option<&mut Rect>) {
            if let Some(rect) = rect {
                rect.width = 800;
                rect.height = 600;
            }
        }
    }
}

impl BrowserProbeRenderHandlerBuilder {
    /// CEF-IT-SPEC-004 用の固定サイズ RenderHandler を生成する。
    ///
    /// # 戻り値
    /// - 800x600 の view rect を返す RenderHandler。
    fn build() -> RenderHandler {
        Self::new(BrowserProbeRenderHandler)
    }
}

wrap_client! {
    struct BrowserProbeClientBuilder {
        render_handler: RenderHandler,
    }

    impl Client {
        fn render_handler(&self) -> Option<RenderHandler> {
            Some(self.render_handler.clone())
        }
    }
}

impl BrowserProbeClientBuilder {
    /// CEF-IT-SPEC-004 用の最小 Client を生成する。
    ///
    /// # 戻り値
    /// - Windowless Rendering に必要な RenderHandler を持つ Client。
    fn build() -> Client {
        Self::new(BrowserProbeRenderHandlerBuilder::build())
    }
}

/// WV-11-02 Windowless Browser Probe を実行する。
///
/// @hldocs.ref doc-20260628-000011Z-WV11#sec_fjanz0cmlgpv
///
/// # 役割
/// - CEF を Windowless Rendering 有効で初期化する。
/// - 独立 Native Window を生成しない Windowless Browser の作成を確認する。
/// - Paint / Buffer の成立性は判定せず CEF-IT-SPEC-005 以降へ分離する。
///
/// # 戻り値
/// - 成功時: Windowless Browser 作成成功を示す文字列。
/// - 失敗時: 初期化または Browser 作成失敗理由。
///
/// # 注意点
/// - Browser は検証後に close 要求し、CEF メッセージ処理を短時間継続してから shutdown する。
/// - この終了処理は Probe の後始末であり、CEF-IT-SPEC-008 の合格判定には使用しない。
pub fn run_browser_probe() -> Result<String, String> {
    let _ = api_hash(sys::CEF_API_VERSION_LAST, 0);
    let args = args::Args::new();

    let process_result = execute_process(
        Some(args.as_main_args()),
        None,
        ptr::null_mut(),
    );

    if process_result >= 0 {
        return Ok(format!(
            "CEF subprocess completed with exit code {process_result}"
        ));
    }

    let settings = Settings {
        windowless_rendering_enabled: 1,
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
            "CEF initialize failed before browser creation: cef_initialize returned {initialized}"
        ));
    }

    let window_info = WindowInfo {
        windowless_rendering_enabled: 1,
        ..Default::default()
    };
    let browser_settings = BrowserSettings::default();
    let mut client = BrowserProbeClientBuilder::build();
    let browser = browser_host_create_browser_sync(
        Some(&window_info),
        Some(&mut client),
        Some(&"about:blank".into()),
        Some(&browser_settings),
        None,
        None,
    );

    let Some(mut browser) = browser else {
        shutdown();
        return Err("CEF windowless browser creation returned None".to_string());
    };

    if let Some(host) = browser.host() {
        host.close_browser(true.into());
    }

    for _ in 0..20 {
        do_message_loop_work();
        sleep(Duration::from_millis(10));
    }

    shutdown();

    Ok("CEF windowless browser creation succeeded".to_string())
}

/// CEF ライブラリパスを解決する。
///
/// # 役割
/// - `--cef-path` にファイルまたはディレクトリのどちらを渡しても扱えるようにする。
/// - 未指定時は現在の実行ファイルと同じディレクトリを既定配置として扱う。
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
        None => default_cef_library_path()?,
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
/// - 現在実行中の検証バイナリと同じディレクトリにある CEF ライブラリを返す。
/// - Runtime / Symbol Probe と Initialize Probe が同一ランタイムを対象にする。
///
/// # 戻り値
/// - 成功時: 実行ファイルと同じディレクトリにある OS ごとの CEF ライブラリパス。
/// - 失敗時: 実行ファイルパスを取得または解決できなかった理由。
///
/// # 注意点
/// - Windows / Linux の `cef-dll-sys` は CEF ランタイムを Cargo の実行バイナリ出力先へ配置する。
/// - macOS のランタイム配置は別構造のため、WV-11-06 で個別に検証する。
pub fn default_cef_library_path() -> Result<PathBuf, String> {
    let executable = std::env::current_exe()
        .map_err(|error| format!("Failed to resolve current executable path: {error}"))?;
    let executable_dir = executable.parent().ok_or_else(|| {
        format!(
            "Failed to resolve current executable directory: {}",
            executable.display()
        )
    })?;

    Ok(executable_dir.join(ffi::default_cef_library_name()))
}
