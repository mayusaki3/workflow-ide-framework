//! CEF 自前 FFI 検証用の最小定義。
//!
//! 役割:
//! - WV-11-02 CEF OSR 最小構成検証で、CEF ライブラリのロードと主要シンボル解決を確認する。
//! - 既存 Rust CEF クレートに依存せず、Framework 内部に CEF 接続境界を閉じ込める方針を検証する。
//!
//! 注意点:
//! - 本ファイルは技術検証用であり、正式 API 仕様ではない。
//! - CEF の完全な構造体定義はまだ含めない。
//! - `cef_initialize` 等を直接呼び出す段階では、CEF ヘッダと一致する ABI 定義を追加する必要がある。
//! - 現段階では、動的ロードとシンボル解決までを安全に確認する。

use libloading::{Library, Symbol};
use std::path::{Path, PathBuf};

/// CEF の `cef_initialize` シンボル型。
///
/// 役割:
/// - シンボル存在確認に使用する。
///
/// 注意点:
/// - 現段階では呼び出しに使用しない。
/// - 引数 ABI は後続の WV-11-02 詳細実装で CEF ヘッダに合わせて確定する。
pub type CefInitializeSymbol = unsafe extern "C" fn() -> i32;

/// CEF の `cef_shutdown` シンボル型。
///
/// 役割:
/// - CEF 終了処理シンボルの存在確認に使用する。
///
/// 注意点:
/// - `cef_initialize` を呼び出していない段階では、この関数も呼び出さない。
pub type CefShutdownSymbol = unsafe extern "C" fn();

/// CEF の `cef_do_message_loop_work` シンボル型。
///
/// 役割:
/// - 外部メッセージループ方式の成立性確認に使用する。
///
/// 注意点:
/// - OSR 実装時は、メインループ統合方式の候補として評価する。
pub type CefDoMessageLoopWorkSymbol = unsafe extern "C" fn();

/// CEF ライブラリの動的ロード結果。
///
/// 役割:
/// - CEF ライブラリハンドルを保持する。
/// - 主要シンボルが解決可能であることを確認する。
///
/// 注意点:
/// - `Library` はシンボルより長く生存させる必要がある。
/// - 本構造体は検証用であり、正式な Browser Surface 実装ではない。
pub struct CefLibraryProbe {
    library: Library,
    path: PathBuf,
}

impl CefLibraryProbe {
    /// CEF ライブラリを指定パスからロードする。
    ///
    /// # 役割
    /// - CEF ライブラリの存在とロード可否を確認する。
    ///
    /// # 引数
    /// - `path`: `libcef.dll` または `libcef.so` のパス。
    ///
    /// # 戻り値
    /// - 成功時: ロード済みライブラリを保持する `CefLibraryProbe`。
    /// - 失敗時: `libloading` のエラー文字列。
    ///
    /// # 注意点
    /// - CEF の依存 DLL / so が同じディレクトリまたは検索パス上にない場合、ロードに失敗する。
    pub fn load(path: impl AsRef<Path>) -> Result<Self, String> {
        let path_ref = path.as_ref();
        let library = unsafe { Library::new(path_ref) }
            .map_err(|error| format!("CEF library load failed: {error}"))?;

        Ok(Self {
            library,
            path: path_ref.to_path_buf(),
        })
    }

    /// ロードした CEF ライブラリのパスを返す。
    ///
    /// # 役割
    /// - 検証ログへロード対象を出力する。
    ///
    /// # 戻り値
    /// - CEF ライブラリのパス。
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// WV-11-02 の初期段階で必要な CEF シンボルを解決する。
    ///
    /// # 役割
    /// - `cef_initialize`
    /// - `cef_shutdown`
    /// - `cef_do_message_loop_work`
    ///   の存在確認を行う。
    ///
    /// # 戻り値
    /// - 成功時: `Ok(())`。
    /// - 失敗時: 解決できなかったシンボルを含むエラー文字列。
    ///
    /// # 注意点
    /// - 現段階ではシンボル解決のみを行い、呼び出しは行わない。
    pub fn resolve_required_symbols(&self) -> Result<(), String> {
        self.resolve_symbol::<CefInitializeSymbol>(b"cef_initialize\0")?;
        self.resolve_symbol::<CefShutdownSymbol>(b"cef_shutdown\0")?;
        self.resolve_symbol::<CefDoMessageLoopWorkSymbol>(b"cef_do_message_loop_work\0")?;
        Ok(())
    }

    /// 指定された CEF シンボルを解決する。
    ///
    /// # 役割
    /// - `libloading` の unsafe 境界を本モジュール内へ閉じ込める。
    ///
    /// # 引数
    /// - `name`: null 終端された CEF シンボル名。
    ///
    /// # 戻り値
    /// - 成功時: 解決済みシンボル。
    /// - 失敗時: `libloading` のエラー文字列。
    fn resolve_symbol<T>(&self, name: &[u8]) -> Result<Symbol<'_, T>, String> {
        unsafe { self.library.get::<T>(name) }.map_err(|error| {
            let symbol_name = String::from_utf8_lossy(name).trim_end_matches('\0').to_string();
            format!("CEF symbol resolve failed: {symbol_name}: {error}")
        })
    }
}

/// OS ごとの既定 CEF ライブラリ名を返す。
///
/// # 役割
/// - コマンドライン引数が指定されない場合の探索名を決定する。
///
/// # 戻り値
/// - Windows: `libcef.dll`
/// - Linux: `libcef.so`
/// - その他: `libcef`
#[must_use]
pub fn default_cef_library_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "libcef.dll"
    } else if cfg!(target_os = "linux") {
        "libcef.so"
    } else {
        "libcef"
    }
}
