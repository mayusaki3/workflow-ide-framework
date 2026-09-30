use eframe::egui;
use std::path::{Path, PathBuf};

const LOCALE_FONT_NAME: &str = "wfide_locale_fallback";
const OS_FONT_NAME: &str = "wfide_os_fallback";

pub fn install_for_locale(
    ctx: &egui::Context,
    locale: &str,
    application_font: Option<&Path>,
) -> Result<Vec<PathBuf>, String> {
    let mut fonts = egui::FontDefinitions::default();
    let mut loaded = Vec::new();

    if let Some(path) = application_font {
        install_font(&mut fonts, "wfide_application_font", path, true)?;
    }

    if let Some(path) = find_os_fallback_font() {
        install_font(&mut fonts, OS_FONT_NAME, &path, false)?;
        loaded.push(path);
    }

    if let Some(path) = find_locale_font(locale) {
        if !loaded.contains(&path) {
            install_font(&mut fonts, LOCALE_FONT_NAME, &path, false)?;
            loaded.push(path);
        }
    }

    ctx.set_fonts(fonts);
    Ok(loaded)
}

pub fn find_os_fallback_font() -> Option<PathBuf> {
    // File-system and project names may contain local-script characters even
    // when the Framework UI locale is English. Keep an OS-appropriate font
    // available independently from the selected UI locale.
    locale_font_candidates().into_iter().find(|path| path.is_file())
}

fn install_font(
    fonts: &mut egui::FontDefinitions,
    name: &str,
    path: &Path,
    primary: bool,
) -> Result<(), String> {
    let bytes = std::fs::read(path)
        .map_err(|error| format!("failed to read font {}: {error}", path.display()))?;
    fonts.font_data.insert(
        name.to_owned(),
        egui::FontData::from_owned(bytes).into(),
    );
    for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
        let entries = fonts.families.entry(family).or_default();
        if primary {
            entries.insert(0, name.to_owned());
        } else {
            entries.push(name.to_owned());
        }
    }
    Ok(())
}

pub fn find_locale_font(locale: &str) -> Option<PathBuf> {
    if !locale.eq_ignore_ascii_case("ja-JP") {
        return None;
    }
    locale_font_candidates().into_iter().find(|path| path.is_file())
}

#[cfg(target_os = "windows")]
fn locale_font_candidates() -> Vec<PathBuf> {
    let root = std::env::var_os("WINDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\\Windows"));
    let fonts = root.join("Fonts");
    [
        "YuGothM.ttc",
        "YuGothR.ttc",
        "meiryo.ttc",
        "msgothic.ttc",
    ]
    .into_iter()
    .map(|name| fonts.join(name))
    .collect()
}

#[cfg(target_os = "macos")]
fn locale_font_candidates() -> Vec<PathBuf> {
    [
        "/System/Library/Fonts/ヒラギノ角ゴシック W3.ttc",
        "/System/Library/Fonts/ヒラギノ角ゴシック W6.ttc",
        "/System/Library/Fonts/AppleSDGothicNeo.ttc",
    ]
    .into_iter()
    .map(PathBuf::from)
    .collect()
}

#[cfg(target_os = "linux")]
fn locale_font_candidates() -> Vec<PathBuf> {
    [
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/opentype/noto/NotoSansCJKjp-Regular.otf",
        "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
    ]
    .into_iter()
    .map(PathBuf::from)
    .collect()
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
fn locale_font_candidates() -> Vec<PathBuf> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsupported_locale_does_not_request_locale_fallback() {
        assert_eq!(find_locale_font("en-US"), None);
    }

    #[test]
    fn japanese_has_platform_candidates() {
        if cfg!(any(target_os = "windows", target_os = "macos", target_os = "linux")) {
            assert!(!locale_font_candidates().is_empty());
        }
    }
}
