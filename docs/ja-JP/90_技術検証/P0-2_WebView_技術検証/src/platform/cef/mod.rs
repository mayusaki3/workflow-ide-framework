//! WV-11 Browser Surface 検証用 CEF モジュール。
//!
//! 役割:
//! - CEF OSR 検証コードの公開口を提供する。
//! - CEF 依存を Framework 内部の実験用境界に閉じ込める。
//!
//! 注意点:
//! - 本モジュールは技術検証用であり、正式 API 仕様ではない。
//! - WV-11-02 では、まず CEF ライブラリの動的ロードと主要シンボル解決を確認する。
//! - Browser 作成、OSR、OnPaint、RGBA バッファ取得は後続ステップで追加する。

pub mod ffi;

use std::path::PathBuf;

/// CEF OSR 検証の初期段階を実行する。
///
/// # 役割
/// - CEF ライブラリをロードする。
/// - WV-11-02 の初期検証に必要な主要シンボルを解決する。
///
/// # 引数
/// - `library_path`: CEF ライブラリへの明示パス。`None` の場合は OS ごとの既定ファイル名を使用する。
///
/// # 戻り値
/// - 成功時: 検証ログ文字列。
/// - 失敗時: 失敗理由を含むエラー文字列。
///
/// # 注意点
/// - 現段階では `cef_initialize` を呼び出さない。
/// - CEF ABI 構造体定義を追加した後、初期化呼び出しへ進む。
pub fn run_symbol_probe(library_path: Option<PathBuf>) -> Result<String, String> {
    let path = library_path.unwrap_or_else(|| PathBuf::from(ffi::default_cef_library_name()));
    let probe = ffi::CefLibraryProbe::load(&path)?;
    probe.resolve_required_symbols()?;

    Ok(format!(
        "CEF library loaded and required symbols resolved: {}",
        probe.path().display()
    ))
}
