use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{OnceLock, RwLock};

pub const EN_US: &str = "en-US";
pub const JA_JP: &str = "ja-JP";

#[derive(Debug, Clone)]
pub struct LocalizationConfig {
    pub default_locale: String,
    pub resource_directory: PathBuf,
    pub application_resource_directories: Vec<PathBuf>,
}

impl Default for LocalizationConfig {
    fn default() -> Self {
        Self {
            default_locale: EN_US.to_owned(),
            resource_directory: PathBuf::from("resources/locales"),
            application_resource_directories: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct LocaleResource {
    locale: String,
    name: String,
    #[serde(default)]
    strings: HashMap<String, String>,
}

#[derive(Debug)]
struct LocalizationState {
    current: String,
    resources: HashMap<String, LocaleResource>,
    application: HashMap<String, HashMap<String, String>>,
}

static STATE: OnceLock<RwLock<LocalizationState>> = OnceLock::new();

pub fn init(config: &LocalizationConfig) {
    let mut resources = load_resources(&config.resource_directory);
    for directory in &config.application_resource_directories {
        merge_resources(&mut resources, load_resources(directory));
    }
    let os_locale = sys_locale::get_locale();
    let current = select_initial_locale(
        resources.keys().map(String::as_str),
        os_locale.as_deref(),
        &config.default_locale,
    );
    let _ = STATE.set(RwLock::new(LocalizationState {
        current,
        resources,
        application: HashMap::new(),
    }));
}

fn select_initial_locale<'a>(
    available: impl Iterator<Item = &'a str>,
    os_locale: Option<&str>,
    fallback: &str,
) -> String {
    let available = available.map(str::to_owned).collect::<Vec<_>>();
    if let Some(os) = os_locale {
        let normalized = os.replace('_', "-");
        if let Some(found) = available
            .iter()
            .find(|locale| locale.eq_ignore_ascii_case(&normalized))
        {
            return found.clone();
        }
        if let Some(language) = normalized.split('-').next() {
            if let Some(found) = available.iter().find(|locale| {
                locale
                    .split('-')
                    .next()
                    .is_some_and(|part| part.eq_ignore_ascii_case(language))
            }) {
                return found.clone();
            }
        }
    }
    if let Some(found) = available.iter().find(|locale| locale.as_str() == fallback) {
        return found.clone();
    }
    if let Some(found) = available.iter().find(|locale| locale.as_str() == EN_US) {
        return found.clone();
    }
    available
        .into_iter()
        .next()
        .unwrap_or_else(|| EN_US.to_owned())
}

fn load_resources(directory: &Path) -> HashMap<String, LocaleResource> {
    let mut resources = HashMap::new();
    let Ok(entries) = std::fs::read_dir(directory) else {
        tracing::warn!(target: "wfide::i18n", path = %directory.display(), "locale resource directory not found");
        return resources;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("toml") {
            continue;
        }
        match std::fs::read_to_string(&path)
            .map_err(|error| error.to_string())
            .and_then(|content| {
                toml::from_str::<LocaleResource>(&content).map_err(|error| error.to_string())
            }) {
            Ok(resource) => {
                resources.insert(resource.locale.clone(), resource);
            }
            Err(error) => {
                tracing::warn!(target: "wfide::i18n", path = %path.display(), %error, "failed to load locale resource")
            }
        }
    }
    resources
}

fn merge_resources(
    target: &mut HashMap<String, LocaleResource>,
    sources: HashMap<String, LocaleResource>,
) {
    for (locale, source) in sources {
        match target.get_mut(&locale) {
            Some(target_resource) => {
                for (key, value) in source.strings {
                    target_resource.strings.insert(key, value);
                }
            }
            None => {
                target.insert(locale, source);
            }
        }
    }
}

pub fn current_locale() -> String {
    STATE
        .get()
        .and_then(|state| state.read().ok().map(|state| state.current.clone()))
        .unwrap_or_else(|| EN_US.to_owned())
}

pub fn locales() -> Vec<(String, String)> {
    let mut locales = STATE
        .get()
        .and_then(|state| {
            state.read().ok().map(|state| {
                state
                    .resources
                    .values()
                    .map(|resource| (resource.locale.clone(), resource.name.clone()))
                    .collect::<Vec<_>>()
            })
        })
        .unwrap_or_default();
    locales.sort_by(|a, b| a.0.cmp(&b.0));
    locales
}

pub fn set_locale(locale: &str) -> Result<(), &'static str> {
    let state = STATE.get().ok_or("WFIDE localization is not initialized")?;
    let mut state = state
        .write()
        .map_err(|_| "WFIDE localization lock is poisoned")?;
    if !state.resources.contains_key(locale) {
        return Err("unsupported locale");
    }
    if state.current != locale {
        let previous = state.current.clone();
        state.current = locale.to_owned();
        tracing::info!(target: "wfide::i18n", from = %previous, to = %locale, "locale changed");
    }
    Ok(())
}

pub fn register(
    locale: &str,
    key: impl Into<String>,
    value: impl Into<String>,
) -> Result<(), &'static str> {
    let state = STATE.get().ok_or("WFIDE localization is not initialized")?;
    let mut state = state
        .write()
        .map_err(|_| "WFIDE localization lock is poisoned")?;
    state
        .application
        .entry(locale.to_owned())
        .or_default()
        .insert(key.into(), value.into());
    Ok(())
}

pub fn text(key: &str) -> String {
    let locale = current_locale();
    let Some(state) = STATE.get().and_then(|state| state.read().ok()) else {
        return key.to_owned();
    };
    if let Some(value) = state
        .application
        .get(&locale)
        .and_then(|values| values.get(key))
    {
        return value.clone();
    }
    if let Some(value) = state
        .resources
        .get(&locale)
        .and_then(|resource| resource.strings.get(key))
    {
        return value.clone();
    }
    if locale != EN_US {
        if let Some(value) = state
            .application
            .get(EN_US)
            .and_then(|values| values.get(key))
        {
            return value.clone();
        }
        if let Some(value) = state
            .resources
            .get(EN_US)
            .and_then(|resource| resource.strings.get(key))
        {
            return value.clone();
        }
    }
    key.to_owned()
}

#[cfg(test)]
mod tests {
    use super::select_initial_locale;

    #[test]
    fn os_locale_selects_matching_supported_locale() {
        let available = ["en-US", "ja-JP"];
        assert_eq!(
            select_initial_locale(available.into_iter(), Some("ja_JP"), "en-US"),
            "ja-JP"
        );
    }

    #[test]
    fn os_language_can_match_supported_region_variant() {
        let available = ["en-US", "ja-JP"];
        assert_eq!(
            select_initial_locale(available.into_iter(), Some("ja"), "en-US"),
            "ja-JP"
        );
    }

    #[test]
    fn unsupported_os_locale_uses_configured_fallback() {
        let available = ["en-US", "ja-JP"];
        assert_eq!(
            select_initial_locale(available.into_iter(), Some("fr-FR"), "ja-JP"),
            "ja-JP"
        );
    }
}
