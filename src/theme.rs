use std::borrow::Cow;

use gpui_kit::App;
use gpui_kit::component::{Theme, ThemeRegistry};

const MONO_THEMES: &str = include_str!("../assets/themes/mono.json");

/// Loads the embedded `mono-light` / `mono-dark` theme set and makes it the
/// active light/dark pair. Call once in `main()` after `gpui_kit::init(cx)`.
pub fn install(cx: &mut App) {
    ThemeRegistry::global_mut(cx)
        .load_themes_from_str(MONO_THEMES)
        .expect("failed to parse embedded assets/themes/mono.json");

    let themes = ThemeRegistry::global(cx).themes();
    let light_theme = themes
        .get("mono-light")
        .expect("mono-light missing from embedded theme set")
        .clone();
    let dark_theme = themes
        .get("mono-dark")
        .expect("mono-dark missing from embedded theme set")
        .clone();

    let theme = Theme::global_mut(cx);
    theme.light_theme = light_theme;
    theme.dark_theme = dark_theme;

    cx.text_system()
        .add_fonts(vec![
            Cow::Borrowed(&include_bytes!("../assets/fonts/HankenGrotesk-Regular.ttf")[..]),
            Cow::Borrowed(&include_bytes!("../assets/fonts/HankenGrotesk-Medium.ttf")[..]),
            Cow::Borrowed(&include_bytes!("../assets/fonts/HankenGrotesk-SemiBold.ttf")[..]),
            Cow::Borrowed(&include_bytes!("../assets/fonts/HankenGrotesk-Bold.ttf")[..]),
            Cow::Borrowed(&include_bytes!("../assets/fonts/InclusiveSans-Regular.ttf")[..]),
            Cow::Borrowed(&include_bytes!("../assets/fonts/InclusiveSans-Medium.ttf")[..]),
            Cow::Borrowed(&include_bytes!("../assets/fonts/InclusiveSans-SemiBold.ttf")[..]),
            Cow::Borrowed(&include_bytes!("../assets/fonts/InclusiveSans-Bold.ttf")[..]),
            Cow::Borrowed(&include_bytes!("../assets/fonts/GoogleSansCode-Regular.ttf")[..]),
            Cow::Borrowed(&include_bytes!("../assets/fonts/GoogleSansCode-Medium.ttf")[..]),
            Cow::Borrowed(&include_bytes!("../assets/fonts/XanhMono-Regular.ttf")[..]),
            Cow::Borrowed(&include_bytes!("../assets/fonts/XanhMono-Italic.ttf")[..]),
        ])
        .expect("failed to register embedded fonts");

    let mode = Theme::global(cx).mode;
    Theme::change(mode, None, cx);
}

#[cfg(test)]
mod tests {
    use super::MONO_THEMES;
    use gpui_kit::component::ThemeSet;

    #[test]
    fn embedded_theme_set_has_required_colors() {
        let set: ThemeSet =
            serde_json::from_str(MONO_THEMES).expect("embedded mono.json must deserialize");

        for name in ["mono-light", "mono-dark"] {
            let config = set
                .themes
                .iter()
                .find(|t| t.name.as_ref() == name)
                .unwrap_or_else(|| panic!("{name} missing from theme set"));
            let colors = &config.colors;
            assert!(colors.background.is_some(), "{name}: background");
            assert!(colors.foreground.is_some(), "{name}: foreground");
            assert!(colors.border.is_some(), "{name}: border");
            assert!(colors.primary.is_some(), "{name}: primary.background");
            assert!(colors.sidebar.is_some(), "{name}: sidebar.background");
            assert!(
                colors.sidebar_accent.is_some(),
                "{name}: sidebar.accent.background"
            );
            assert!(colors.danger.is_some(), "{name}: danger.background");
            assert!(colors.ring.is_some(), "{name}: ring");
            assert!(config.font_family.is_some(), "{name}: font.family");
            assert!(
                config.mono_font_family.is_some(),
                "{name}: mono_font.family"
            );
        }
    }
}
