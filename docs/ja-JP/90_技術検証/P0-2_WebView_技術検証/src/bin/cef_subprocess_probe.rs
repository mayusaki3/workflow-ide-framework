//! WV-11-03 CEF subprocess 専用 helper。
//!
//! 役割:
//! - CEF renderer / gpu-process 等の subprocess だけを処理する。
//! - eframe / egui の GUI 初期化経路へ入れない。
//!
//! 注意点:
//! - 本ファイルは技術検証用であり、正式 Runtime 実装ではない。
//! - 親 Browser Process から `Settings.browser_subprocess_path` で明示指定される。

use cef::*;
use std::ptr;

/// CEF subprocess helper のエントリーポイント。
///
/// # 戻り値
/// - CEF subprocess の終了コードをそのままプロセス終了コードとして返す。
fn main() {
    let _ = api_hash(sys::CEF_API_VERSION_LAST, 0);
    let args = args::Args::new();
    let exit_code = execute_process(Some(args.as_main_args()), None, ptr::null_mut());

    if exit_code >= 0 {
        std::process::exit(exit_code);
    }

    eprintln!(
        "WV-11-03 CEF subprocess helper failed: cef_execute_process returned {exit_code}"
    );
    std::process::exit(1);
}
