use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

pub const EN_US: &str = "en-US";
pub const JA_JP: &str = "ja-JP";

#[derive(Debug, Clone)]
pub struct LocalizationConfig {
    pub default_locale: String,
    pub supported_locales: Vec<String>,
}

impl Default for LocalizationConfig {
    fn default() -> Self {
        Self {
            default_locale: EN_US.to_owned(),
            supported_locales: vec![EN_US.to_owned(), JA_JP.to_owned()],
        }
    }
}

#[derive(Debug)]
struct LocalizationState {
    current: String,
    supported: Vec<String>,
    application: HashMap<String, HashMap<String, String>>,
}

static STATE: OnceLock<RwLock<LocalizationState>> = OnceLock::new();

pub fn init(config: &LocalizationConfig) {
    let supported = if config.supported_locales.is_empty() {
        vec![EN_US.to_owned()]
    } else {
        config.supported_locales.clone()
    };
    let current = if supported.iter().any(|locale| locale == &config.default_locale) {
        config.default_locale.clone()
    } else {
        supported[0].clone()
    };
    let _ = STATE.set(RwLock::new(LocalizationState {
        current,
        supported,
        application: HashMap::new(),
    }));
}

pub fn current_locale() -> String {
    STATE
        .get()
        .and_then(|state| state.read().ok().map(|state| state.current.clone()))
        .unwrap_or_else(|| EN_US.to_owned())
}

pub fn supported_locales() -> Vec<String> {
    STATE
        .get()
        .and_then(|state| state.read().ok().map(|state| state.supported.clone()))
        .unwrap_or_else(|| vec![EN_US.to_owned(), JA_JP.to_owned()])
}

pub fn set_locale(locale: &str) -> Result<(), &'static str> {
    let state = STATE.get().ok_or("WFIDE localization is not initialized")?;
    let mut state = state.write().map_err(|_| "WFIDE localization lock is poisoned")?;
    if !state.supported.iter().any(|supported| supported == locale) {
        return Err("unsupported locale");
    }
    if state.current != locale {
        let previous = state.current.clone();
        state.current = locale.to_owned();
        tracing::info!(target: "wfide::i18n", from = %previous, to = %locale, "locale changed");
    }
    Ok(())
}

pub fn register(locale: &str, key: impl Into<String>, value: impl Into<String>) -> Result<(), &'static str> {
    let state = STATE.get().ok_or("WFIDE localization is not initialized")?;
    let mut state = state.write().map_err(|_| "WFIDE localization lock is poisoned")?;
    state
        .application
        .entry(locale.to_owned())
        .or_default()
        .insert(key.into(), value.into());
    Ok(())
}

pub fn text(key: &str) -> String {
    let locale = current_locale();
    if let Some(value) = framework_text(&locale, key) {
        return value.to_owned();
    }
    if let Some(state) = STATE.get().and_then(|state| state.read().ok()) {
        if let Some(value) = state.application.get(&locale).and_then(|values| values.get(key)) {
            return value.clone();
        }
        if locale != EN_US {
            if let Some(value) = state.application.get(EN_US).and_then(|values| values.get(key)) {
                return value.clone();
            }
        }
    }
    framework_text(EN_US, key).unwrap_or(key).to_owned()
}

fn framework_text(locale: &str, key: &str) -> Option<&'static str> {
    let ja = locale == JA_JP;
    Some(match key {
        "logging.title" => if ja { "ログ設定" } else { "Logging Settings" },
        "logging.framework_panel" => if ja { "Framework 標準パネル" } else { "Framework standard panel" },
        "logging.runtime_level" => if ja { "実行時ログレベル" } else { "Runtime Log Level" },
        "logging.description" => if ja { "実行中に記録する最小重要度を指定します。変更はコンソール、ファイル、メモリ出力へ即時反映されます。" } else { "Controls the minimum severity recorded at runtime. Changes apply immediately to console, file, and in-memory output." },
        "logging.error" => if ja { "ERROR: エラーのみ" } else { "ERROR: errors only" },
        "logging.warn" => if ja { "WARN: 警告とエラー" } else { "WARN: warnings and errors" },
        "logging.info" => if ja { "INFO: 通常の動作情報、警告、エラー（既定値）" } else { "INFO: normal operational information, warnings, and errors (default)" },
        "logging.debug" => if ja { "DEBUG: INFOに加えてデバッグ情報" } else { "DEBUG: INFO plus debugging information" },
        "logging.trace" => if ja { "TRACE: DEBUGに加えて最も詳細な内部処理" } else { "TRACE: DEBUG plus the most detailed internal processing" },
        "logging.load_warning" => if ja { "DEBUG/TRACEはログ量と処理負荷が大きくなる場合があります。" } else { "DEBUG/TRACE can significantly increase log volume and processing load." },
        "language.title" => if ja { "言語設定" } else { "Language Settings" },
        "language.description" => if ja { "表示言語を実行中に切り替えます。" } else { "Change the display language at runtime." },
        "language.english" => "English",
        "language.japanese" => "日本語",
        _ => return None,
    })
}
