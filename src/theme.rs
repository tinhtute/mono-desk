use std::borrow::Cow;

use gpui_kit::component::{Theme, ThemeRegistry};
use gpui_kit::{App, px};

const MONO_THEMES: &str = include_str!("../assets/themes/mono.json");

/// Lower bound of the Font Size setting, in px.
pub const FONT_SIZE_MIN: f64 = 12.0;
/// Upper bound of the Font Size setting, in px.
pub const FONT_SIZE_MAX: f64 = 18.0;

/// Clamp a requested UI font size into the supported range.
pub fn clamp_font_size(size: f64) -> f64 {
    size.clamp(FONT_SIZE_MIN, FONT_SIZE_MAX)
}

/// Write the app's theme overrides onto `theme`.
///
/// - `font_size` — the Font Size setting, clamped to 12–18.
/// - `focus_ring` — off. The framework's ring is painted *outside* a focused
///   control's border and reads as too heavy here; with the flag off,
///   `ThemeStyled::focus_ring_style` keeps only the 1px border tinted with
///   `theme.ring`, which costs no space and cannot be clipped.
fn write_overrides(theme: &mut Theme, font_size: f64) {
    theme.font_size = px(clamp_font_size(font_size) as f32);
    theme.focus_ring = false;
}

/// Apply the app's theme overrides to the live theme.
///
/// `Theme::change` rebuilds the theme from the config, so this must run after
/// every mode switch (and once at startup, from [`install`]). `sync_base`
/// rebuilds the Base tokens the components read, and `Root` re-derives the
/// window rem size from `theme.font_size` on the next paint.
pub fn apply_overrides(font_size: f64, cx: &mut App) {
    write_overrides(Theme::global_mut(cx), font_size);
    Theme::sync_base(cx);
}

/// Loads the embedded `mono-light` / `mono-dark` theme set, makes it the
/// active light/dark pair, and applies the app's overrides for `font_size`.
/// Call once in `main()` after `gpui_kit::init(cx)`.
pub fn install(cx: &mut App, font_size: f64) {
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
    apply_overrides(font_size, cx);
}

#[cfg(test)]
mod tests {
    use super::MONO_THEMES;
    use super::{FONT_SIZE_MAX, FONT_SIZE_MIN, clamp_font_size, write_overrides};
    use gpui_kit::component::{Theme, ThemeSet};
    use gpui_kit::px;

    /// The app turns the framework's outer focus ring off and clamps the font
    /// size; both are written by the same override pass that runs after every
    /// `Theme::change`.
    #[test]
    fn overrides_disable_focus_ring_and_clamp_font_size() {
        let mut theme = Theme::default();
        assert!(theme.focus_ring, "framework default keeps the ring on");

        write_overrides(&mut theme, 99.0);
        assert!(!theme.focus_ring, "the app turns the outer ring off");
        assert_eq!(theme.font_size, px(FONT_SIZE_MAX as f32));

        write_overrides(&mut theme, 1.0);
        assert!(!theme.focus_ring);
        assert_eq!(theme.font_size, px(FONT_SIZE_MIN as f32));
    }

    #[test]
    fn font_size_clamps_to_supported_range() {
        assert_eq!(clamp_font_size(0.0), FONT_SIZE_MIN);
        assert_eq!(clamp_font_size(11.9), FONT_SIZE_MIN);
        assert_eq!(clamp_font_size(FONT_SIZE_MIN), FONT_SIZE_MIN);
        assert_eq!(clamp_font_size(15.0), 15.0);
        assert_eq!(clamp_font_size(FONT_SIZE_MAX), FONT_SIZE_MAX);
        assert_eq!(clamp_font_size(19.0), FONT_SIZE_MAX);
        assert_eq!(clamp_font_size(999.0), FONT_SIZE_MAX);
    }

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
