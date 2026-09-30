use std::borrow::Cow;

use gpui_kit::component::{Theme, ThemeRegistry};
use gpui_kit::{App, px};

const MONO_THEMES: &str = include_str!("../assets/themes/mono.json");

/// Emboldened Xanh Mono italic (weight 600), derived from
/// `XanhMono-Italic.ttf` by `scripts/embolden-xanh-mono.py`.
///
/// The wordmarks request `Xanh Mono` at `FontWeight::SEMIBOLD`, but the family
/// ships only 400 faces and GPUI never synthesizes bold — without this face the
/// request silently resolved to the 400 italic and the 17px wordmark read thin
/// and blurry. Same family name, distinct `SemiBold Italic` subfamily, so the
/// weight request is what selects it.
const XANH_MONO_SEMIBOLD_ITALIC: &[u8] =
    include_bytes!("../assets/fonts/XanhMono-SemiBoldItalic.ttf");

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
            Cow::Borrowed(XANH_MONO_SEMIBOLD_ITALIC),
        ])
        .expect("failed to register embedded fonts");

    let mode = Theme::global(cx).mode;
    Theme::change(mode, None, cx);
    apply_overrides(font_size, cx);
}

#[cfg(test)]
mod tests {
    use super::MONO_THEMES;
    use super::{
        FONT_SIZE_MAX, FONT_SIZE_MIN, XANH_MONO_SEMIBOLD_ITALIC, clamp_font_size, write_overrides,
    };
    use gpui_kit::component::{Theme, ThemeSet};
    use gpui_kit::px;

    /// The wordmarks ask for `Xanh Mono` at `FontWeight::SEMIBOLD`, and GPUI
    /// never synthesizes bold — so the request only reads as SemiBold if the
    /// embedded face really carries that weight. Parsing the registered bytes
    /// with the same parser the font stack uses (`fontdb` → `ttf-parser`)
    /// catches a missing, stale, or mis-tagged face.
    #[test]
    fn embedded_wordmark_face_is_semibold_italic() {
        let face = ttf_parser::Face::parse(XANH_MONO_SEMIBOLD_ITALIC, 0)
            .expect("embedded XanhMono-SemiBoldItalic.ttf must parse");
        assert!(
            face.weight().to_number() >= 600,
            "wordmark face is weight {}, need >= 600",
            face.weight().to_number()
        );
        assert!(face.is_italic(), "wordmark face must stay italic");

        // fontdb matches on the family name, so it has to stay `Xanh Mono` —
        // that is what `sidebar_brand` and the About page request.
        let families: Vec<String> = face
            .names()
            .into_iter()
            .filter(|name| name.name_id == ttf_parser::name_id::FAMILY)
            .filter_map(|name| name.to_string())
            .collect();
        assert!(
            families.iter().any(|family| family == "Xanh Mono"),
            "expected family `Xanh Mono`, got {families:?}"
        );

        // The 400 italic must stay lighter, or the weight request would have
        // nothing to select between.
        let regular =
            ttf_parser::Face::parse(include_bytes!("../assets/fonts/XanhMono-Italic.ttf"), 0)
                .expect("embedded XanhMono-Italic.ttf must parse");
        assert!(
            regular.weight().to_number() < face.weight().to_number(),
            "the 400 italic and the wordmark face must differ in weight"
        );
    }

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

    /// sRGB channel (0–255) → linear-light value, per WCAG 2.x.
    fn srgb_to_linear(channel: u8) -> f64 {
        let c = f64::from(channel) / 255.0;
        if c <= 0.03928 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    }

    /// WCAG 2.x relative luminance of a `#rrggbb` string.
    fn relative_luminance(hex: &str) -> f64 {
        let hex = hex.trim_start_matches('#');
        let channel = |i: usize| {
            let byte = u8::from_str_radix(&hex[i..i + 2], 16)
                .unwrap_or_else(|_| panic!("`{hex}` is not a #rrggbb colour"));
            srgb_to_linear(byte)
        };
        0.2126 * channel(0) + 0.7152 * channel(2) + 0.0722 * channel(4)
    }

    /// WCAG 2.x contrast ratio between two `#rrggbb` strings.
    fn contrast_ratio(a: &str, b: &str) -> f64 {
        let (la, lb) = (relative_luminance(a), relative_luminance(b));
        let (hi, lo) = if la > lb { (la, lb) } else { (lb, la) };
        (hi + 0.05) / (lo + 0.05)
    }

    /// Every token pair the UI actually renders must clear WCAG AA: 4.5:1 for
    /// text, 3:1 for the boundaries of interactive components (1.4.11).
    ///
    /// The zdesk palette resolves several of these below the thresholds; the
    /// values in `mono.json` are the WCAG-adjusted ones (see the port spec).
    #[test]
    fn theme_tokens_meet_wcag_contrast() {
        let set: serde_json::Value =
            serde_json::from_str(MONO_THEMES).expect("embedded mono.json must deserialize");

        // (foreground, background) pairs rendered as text.
        const TEXT: &[(&str, &str)] = &[
            ("foreground", "background"),
            ("foreground", "group_box.background"),
            ("muted.foreground", "background"),
            ("muted.foreground", "group_box.background"),
            ("sidebar.foreground", "sidebar.background"),
            ("group_box.title.foreground", "group_box.background"),
            ("description_list.label.foreground", "group_box.background"),
            ("tab.foreground", "tab.background"),
            ("table.head.foreground", "table.head.background"),
            ("button.foreground", "button.background"),
            ("secondary.foreground", "secondary.background"),
            ("sidebar.accent.foreground", "sidebar.accent.background"),
            ("accent.foreground", "background"),
            ("primary.foreground", "primary.background"),
        ];
        // Component boundaries / state indicators.
        const UI: &[(&str, &str)] = &[
            ("input.border", "group_box.background"),
            ("input.border", "background"),
            ("switch.background", "group_box.background"),
            ("switch.thumb.background", "switch.background"),
            ("scrollbar.thumb.background", "group_box.background"),
            ("scrollbar.thumb.background", "background"),
            ("scrollbar.thumb.hover.background", "group_box.background"),
            ("scrollbar.thumb.hover.background", "background"),
            ("ring", "group_box.background"),
        ];

        for theme in set["themes"].as_array().expect("themes array") {
            let name = theme["name"].as_str().expect("theme name");
            let colors = &theme["colors"];
            let hex = |key: &str| -> &str {
                colors
                    .get(key)
                    .and_then(|value| value.as_str())
                    .unwrap_or_else(|| panic!("{name}: missing colour token `{key}`"))
            };

            for (fg, bg) in TEXT {
                let ratio = contrast_ratio(hex(fg), hex(bg));
                assert!(
                    ratio >= 4.5,
                    "{name}: {fg} on {bg} is {ratio:.2}:1, need 4.5:1"
                );
            }
            for (fg, bg) in UI {
                let ratio = contrast_ratio(hex(fg), hex(bg));
                assert!(
                    ratio >= 3.0,
                    "{name}: {fg} on {bg} is {ratio:.2}:1, need 3:1"
                );
            }
        }
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

    /// The startup default must equal the theme's own `font.size`: the app
    /// applies `AppSettings::default().font_size` as an override at boot, so a
    /// drift between the two silently changes the baseline the theme (and the
    /// window rem size every style helper derives from) was tuned for.
    #[test]
    fn default_font_size_matches_theme_config() {
        let set: serde_json::Value =
            serde_json::from_str(MONO_THEMES).expect("embedded mono.json must deserialize");
        let default = crate::AppSettings::default().font_size;

        for theme in set["themes"].as_array().expect("themes array") {
            let name = theme["name"].as_str().expect("theme name");
            let size = theme["font.size"].as_f64().expect("font.size");
            assert_eq!(
                size, default,
                "{name}: font.size {size} does not match the app default {default}"
            );
        }
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
