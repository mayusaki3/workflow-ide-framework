//! P0-2 WebView 技術検証
//!
//! 役割:
//! - 技術検証アプリケーションのエントリーポイント。
//! - WV-11-02 CEF OSR 最小構成検証用の `--cef-probe` を提供する。
//!
//! 注意:
//! - P0-2 WebView 技術検証用のPoCコード。
//! - WV-03以降の検証結果により、モジュール構成は変更される可能性がある。
//! - `--cef-probe` は技術検証用であり、正式 API 仕様ではない。

mod app;
mod layout_storage;
mod panel_tab;
mod platform;

use std::path::PathBuf;

/// アプリケーションの起動処理。
///
/// # 役割
/// - 通常起動時は eframe アプリケーションを起動する。
/// - `--cef-probe` 指定時は CEF ライブラリのロードと主要シンボル解決のみを行い終了する。
///
/// # 戻り値
/// - 成功時: `Ok(())`。
/// - 失敗時: eframe または検証処理のエラー。
///
/// # 注意点
/// - `--cef-probe` では `cef_initialize` は呼び出さない。
/// - `--cef-path` には CEF ライブラリファイル、または `libcef` を含むディレクトリを指定できる。
fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|arg| arg == "--cef-probe") {
        run_cef_probe_from_args(&args);
        return Ok(());
    }

    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "P0-2 WebView Validation",
        options,
        Box::new(|cc| Ok(Box::new(app::DockingValidationApp::new(cc)))),
    )
}

/// コマンドライン引数から CEF Probe を実行する。
///
/// # 役割
/// - `--cef-path` を解析する。
/// - `platform::cef::run_symbol_probe` を呼び出す。
/// - 検証結果を標準出力または標準エラーへ出力する。
///
/// # 引数
/// - `args`: `std::env::args()` から取得した引数一覧。
///
/// # 注意点
/// - 検証失敗時はプロセスを `1` で終了する。
fn run_cef_probe_from_args(args: &[String]) {
    let cef_path = find_option_value(args, "--cef-path").map(PathBuf::from);

    println!("WV-11-02 CEF symbol probe start");

    match platform::cef::run_symbol_probe(cef_path) {
        Ok(message) => {
            println!("{message}");
            println!("WV-11-02 CEF symbol probe OK");
        }
        Err(error) => {
            eprintln!("WV-11-02 CEF symbol probe failed");
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}

/// 指定オプションの値を取得する。
///
/// # 役割
/// - `--cef-path <path>` 形式を解析する。
///
/// # 引数
/// - `args`: コマンドライン引数一覧。
/// - `name`: 取得対象のオプション名。
///
/// # 戻り値
/// - 値が存在する場合は `Some(&str)`。
/// - オプションが存在しない、または値が存在しない場合は `None`。
fn find_option_value<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.windows(2)
        .find(|window| window[0] == name)
        .map(|window| window[1].as_str())
}
