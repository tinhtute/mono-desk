use std::borrow::Cow;

use gpui_kit::component::{Colorize as _, Theme, ThemeMode, ThemeRegistry};
use gpui_kit::{App, Hsla, WindowAppearance, px};

use crate::tokens::UiTokens;

/// The app's embedded theme set (`mono-light` / `mono-dark`).
///
/// Crate-visible so tests elsewhere can resolve the app's own palette instead
/// of the framework defaults.
pub(crate) const MONO_THEMES: &str = include_str!("../assets/themes/mono.json");

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
pub const FONT_SIZE_MIN: f64 = 8.0;
/// Upper bound of the Font Size setting, in px.
pub const FONT_SIZE_MAX: f64 = 32.0;

/// Lower bound of the Window opacity setting, in percent.
///
/// Floored well above zero: below ~40% the frost layer stops carrying
/// enough contrast over an arbitrary desktop backdrop for this app's text,
/// which is a readability cliff rather than a taste choice.
pub const WINDOW_OPACITY_MIN: f64 = 40.0;
/// Upper bound of the Window opacity setting, in percent (opaque).
pub const WINDOW_OPACITY_MAX: f64 = 100.0;
/// Default Window opacity, in percent, when translucency is first enabled.
///
/// Matches monocode's 0.85 surface opacity: low enough to see the frosted
/// backdrop immediately, high enough for the settings text to stay readable.
pub const WINDOW_OPACITY_DEFAULT: f64 = 85.0;

/// Lower bound of the Sidebar tone setting, in percent of the max wash.
pub const SIDEBAR_TONE_MIN: f64 = 0.0;
/// Upper bound of the Sidebar tone setting, in percent of the max wash.
pub const SIDEBAR_TONE_MAX: f64 = 100.0;
/// Default Sidebar tone, in percent: 50% of the max wash, i.e. an ink tint of
/// 0.05 — the same weight zeron's glass sidebar paints (`wash(0.05)`).
pub const SIDEBAR_TONE_DEFAULT: f64 = 50.0;
/// The strongest sidebar tone, as an alpha: 100% puts 10% ink over the frost.
///
/// Deliberately small: the tone separates the navigation column from the
/// content without becoming a second surface, which is what the previous
/// pane-alpha model did (and why it read as two stacked panels).
pub const SIDEBAR_TONE_MAX_ALPHA: f32 = 0.10;

/// Sidebar rail width bounds, in logical pixels.
///
/// Matches zeron's `SIDEBAR_MIN`/`SIDEBAR_MAX`/`SIDEBAR_DEFAULT`: the rail is
/// resizable between those limits and its last width persists across launches.
pub const SIDEBAR_WIDTH_MIN: f64 = 242.0;
pub const SIDEBAR_WIDTH_MAX: f64 = 400.0;
pub const SIDEBAR_WIDTH_DEFAULT: f64 = 256.0;

/// Clamp a requested sidebar width into the supported pixel range.
///
/// A non-finite value falls back to the default width, so a malformed file
/// cannot produce a zero-width or runaway rail.
pub fn clamp_sidebar_width(width: f64) -> f64 {
    if !width.is_finite() {
        return SIDEBAR_WIDTH_DEFAULT;
    }
    width.clamp(SIDEBAR_WIDTH_MIN, SIDEBAR_WIDTH_MAX)
}

/// Clamp a requested window opacity into the supported percentage range.
///
/// Same NaN guard as [`clamp_font_size`], but the fallback is the *opaque*
/// end: a malformed value must never make the window unexpectedly
/// transparent.
pub fn clamp_window_opacity(opacity: f64) -> f64 {
    if !opacity.is_finite() {
        return WINDOW_OPACITY_MAX;
    }
    opacity.clamp(WINDOW_OPACITY_MIN, WINDOW_OPACITY_MAX)
}

/// Clamp a requested sidebar tone into its percentage range.
///
/// The fallback is the default tone, never "more ink than asked for": a
/// malformed file must not darken the rail unexpectedly.
pub fn clamp_sidebar_tone(tone: f64) -> f64 {
    if !tone.is_finite() {
        return SIDEBAR_TONE_DEFAULT;
    }
    tone.clamp(SIDEBAR_TONE_MIN, SIDEBAR_TONE_MAX)
}

/// The ink alpha the sidebar paints for a tone percentage.
///
/// `0` returns fully transparent (the column is then separated only by its
/// hairline), `100` returns [`SIDEBAR_TONE_MAX_ALPHA`]; a non-finite value
/// falls back to the default tone.
pub fn sidebar_tone_alpha(percent: f64) -> f32 {
    (clamp_sidebar_tone(percent) / 100.0) as f32 * SIDEBAR_TONE_MAX_ALPHA
}

/// Clamp a requested UI font size into the supported range.
///
/// `f64::clamp` is NaN-transparent — `NaN.clamp(8.0, 32.0)` returns `NaN`
/// — so a NaN would flow straight into `px()` and poison the rem-based
/// layout. NaN falls back to the default size instead. Infinities need no
/// special case: `clamp` already saturates them to the bounds. Callers
/// gate on `is_finite` before this, so the NaN arm is a backstop.
pub fn clamp_font_size(size: f64) -> f64 {
    if size.is_nan() {
        return 14.0;
    }
    size.clamp(FONT_SIZE_MIN, FONT_SIZE_MAX)
}

/// How the app picks its theme: one mode always, or as the system does.
///
/// `ThemeMode` itself has only `Light`/`Dark` — it is the framework's
/// resolved state. This is the app's *choice*, which can also be "follow
/// the OS", so the two cannot be the same type.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Appearance {
    #[default]
    Light,
    Dark,
    System,
}

impl Appearance {
    /// The persisted `key=value` word. Stable — it is written to disk.
    pub fn as_str(self) -> &'static str {
        match self {
            Appearance::Light => "light",
            Appearance::Dark => "dark",
            Appearance::System => "system",
        }
    }

    /// Parse a persisted word. `None` for anything else, so a hand-edited
    /// file falls back to the default rather than becoming unselectable.
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "light" => Some(Appearance::Light),
            "dark" => Some(Appearance::Dark),
            "system" => Some(Appearance::System),
            _ => None,
        }
    }

    /// The mode to show: the chosen one, or the system's when `System`.
    pub fn resolve(self, system: WindowAppearance) -> ThemeMode {
        match self {
            Appearance::Light => ThemeMode::Light,
            Appearance::Dark => ThemeMode::Dark,
            Appearance::System => ThemeMode::from(system),
        }
    }
}

/// Named accent presets, each drawn from a base hue the palette already
/// carries, so a chosen accent still sits inside the graphite scheme.
///
/// The words are persisted, so they are stable; the colors are not.
pub const ACCENT_PRESETS: [(&str, &str); 6] = [
    ("teal", "#124b68"),
    ("blue", "#2e6fd0"),
    ("green", "#1f7a45"),
    ("violet", "#8d6be7"),
    ("amber", "#935f06"),
    ("red", "#bb1b1b"),
];

/// Parse a stored accent — `#RRGGBB` — or `None` for the theme's own.
///
/// Anything unparseable is `None`, so a hand-edited file falls back to the
/// shipped palette rather than to an arbitrary color.
pub fn accent_from_str(s: &str) -> Option<Hsla> {
    Hsla::parse_hex(s).ok()
}

/// Write `accent` onto the theme's brand tokens.
///
/// The picked color drives the **primary** family and the focus ring — the
/// tokens that read as "the accent" (primary buttons, selection, focus).
/// It deliberately does not touch `colors.accent`, which in this framework
/// is the neutral hover wash behind list and menu rows, not a brand color.
///
/// The shipped palette lightens hover and active in light mode and darkens
/// them in dark mode; the derived shades keep that direction.
fn write_accent(theme: &mut Theme, accent: Hsla) {
    let shade = |amount: f32| {
        if theme.mode.is_dark() {
            accent.darken(amount)
        } else {
            accent.lighten(amount)
        }
    };
    theme.colors.primary = accent;
    theme.colors.primary_hover = shade(0.08);
    theme.colors.primary_active = shade(0.20);
    theme.colors.ring = accent;
    // Readable on the accent: near-black on a light color, white otherwise.
    theme.colors.primary_foreground = Hsla {
        h: 0.0,
        s: 0.0,
        l: if accent.l > 0.6 { 0.15 } else { 1.0 },
        a: 1.0,
    };
}

/// The colour a window-base surface paints while the glass setting is on.
///
/// The palette colour keeps its hue/saturation/lightness and takes the chosen
/// alpha. At the opaque end — and for a NaN setting — the colour comes back
/// untouched, so the default rendering cannot drift from `mono.json`.
///
/// The alpha is applied at *paint* time rather than written onto the theme
/// palette on purpose: the vendored `Root`, dialogs and sheets all read the
/// shared `tokens.background` snapshot, and dialog/sheet content must stay
/// opaque for readability. The alpha is instead painted as the single
/// full-window frost layer (see `main::shell_frost_bg` and the base paint in
/// `shell::AppShell`); chrome above it stays transparent.
pub(crate) fn glass_surface(color: Hsla, opacity_percent: f64) -> Hsla {
    let alpha = (clamp_window_opacity(opacity_percent) / 100.0) as f32;
    if alpha >= 1.0 {
        return color;
    }
    Hsla { a: alpha, ..color }
}

/// Write the app's theme overrides onto `theme`.
///
/// Crate-visible so the token tests can exercise the override path on a bare
/// `Theme`, without an `App`.
///
/// - `font_size` — the Font Size setting, clamped to 8–32.
/// - `focus_ring` — off. The framework's ring is painted *outside* a focused
///   control's border and reads as too heavy here; with the flag off,
///   `ThemeStyled::focus_ring_style` keeps only the 1px border tinted with
///   `theme.ring`, which costs no space and cannot be clipped.
///
/// The accent is applied by [`apply_overrides`] instead, so this stays a
/// font-size/ring concern and its existing callers do not move.
pub(crate) fn write_overrides(theme: &mut Theme, font_size: f64) {
    theme.font_size = px(clamp_font_size(font_size) as f32);
    theme.focus_ring = false;
}

/// Apply the app's theme overrides to the live theme.
///
/// `Theme::change` rebuilds the theme from the config, so this must run after
/// every mode switch (and once at startup, from [`install`]) — otherwise both
/// the font size and the accent silently revert. `sync_base` rebuilds the
/// Base tokens the components read, `Root` re-derives the window rem size
/// from `theme.font_size` on the next paint, and [`UiTokens::sync`]
/// re-resolves the app's elevation snapshot from the rebuilt theme.
pub fn apply_overrides(font_size: f64, accent: Option<Hsla>, cx: &mut App) {
    let theme = Theme::global_mut(cx);
    write_overrides(theme, font_size);
    if let Some(accent) = accent {
        write_accent(theme, accent);
    }
    Theme::sync_base(cx);
    UiTokens::sync(cx);
}

/// Loads the embedded `mono-light` / `mono-dark` theme set, makes it the
/// active light/dark pair, and applies the app's overrides for `font_size`
/// and `accent`. Call once in `main()` after `gpui_kit::init(cx)`.
pub fn install(cx: &mut App, font_size: f64, accent: Option<Hsla>) {
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
            Cow::Borrowed(&include_bytes!("../assets/fonts/MomoTrustSans-Regular.ttf")[..]),
            Cow::Borrowed(&include_bytes!("../assets/fonts/MomoTrustSans-Medium.ttf")[..]),
            Cow::Borrowed(&include_bytes!("../assets/fonts/MomoTrustSans-SemiBold.ttf")[..]),
            Cow::Borrowed(&include_bytes!("../assets/fonts/MomoTrustSans-Bold.ttf")[..]),
            Cow::Borrowed(&include_bytes!("../assets/fonts/GoogleSansCode-Regular.ttf")[..]),
            Cow::Borrowed(&include_bytes!("../assets/fonts/GoogleSansCode-Medium.ttf")[..]),
            Cow::Borrowed(&include_bytes!("../assets/fonts/XanhMono-Regular.ttf")[..]),
            Cow::Borrowed(&include_bytes!("../assets/fonts/XanhMono-Italic.ttf")[..]),
            Cow::Borrowed(XANH_MONO_SEMIBOLD_ITALIC),
        ])
        .expect("failed to register embedded fonts");

    let mode = Theme::global(cx).mode;
    Theme::change(mode, None, cx);
    apply_overrides(font_size, accent, cx);
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::rc::Rc;

    use super::MONO_THEMES;
    use super::{
        ACCENT_PRESETS, Appearance, FONT_SIZE_MAX, FONT_SIZE_MIN, SIDEBAR_TONE_DEFAULT,
        SIDEBAR_TONE_MAX_ALPHA, WINDOW_OPACITY_MAX, WINDOW_OPACITY_MIN, XANH_MONO_SEMIBOLD_ITALIC,
        accent_from_str, clamp_font_size, clamp_sidebar_tone, clamp_sidebar_width,
        clamp_window_opacity, glass_surface, sidebar_tone_alpha, write_accent, write_overrides,
    };
    use gpui_kit::component::{Colorize as _, Theme, ThemeMode, ThemeSet};
    use gpui_kit::{Hsla, WindowAppearance, px, rems};

    /// The wordmarks ask for `Xanh Mono` at `FontWeight::SEMIBOLD`, and GPUI
    /// never synthesizes bold — so the request only reads as SemiBold if the
    /// embedded face really carries that weight.
    ///
    /// `ttf-parser` is not literally what macOS renders through (`zed-font-kit`
    /// → CoreText); it is what the Linux backend uses (`cosmic_text` →
    /// `fontdb` → `ttf-parser`), and both backends read the same `name`/`OS/2`
    /// tables that this parses. Reading the registered bytes directly catches
    /// a missing, stale, or mis-tagged face on either platform.
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

    /// Every embedded UI sans face must belong to the `Momo Trust Sans`
    /// family and carry a distinct weight.
    ///
    /// This is the one property that makes the swap work on *both* font
    /// backends, and neither is forgiving when it is violated:
    ///
    /// - macOS (`zed-font-kit` → CoreText) asks CoreText for the family, then
    ///   picks the closest weight with `find_best_match`.
    /// - Linux (`cosmic_text` → `fontdb`) looks a family up by
    ///   `TYPOGRAPHIC_FAMILY` (name 16) and only falls back to `FAMILY`
    ///   (name 1) when 16 is absent.
    ///
    /// GPUI never synthesizes bold, so a face that lands in the wrong family
    /// silently degrades to whatever weight the family *does* have — the bug
    /// `XANH_MONO_SEMIBOLD_ITALIC` exists to avoid. Resolving the name the way
    /// `fontdb` does keeps that from recurring when the fonts are re-vendored
    /// or upgraded.
    #[test]
    fn embedded_ui_faces_share_one_family_and_distinct_weights() {
        const FACES: [(&[u8], u16); 4] = [
            (
                include_bytes!("../assets/fonts/MomoTrustSans-Regular.ttf"),
                400,
            ),
            (
                include_bytes!("../assets/fonts/MomoTrustSans-Medium.ttf"),
                500,
            ),
            (
                include_bytes!("../assets/fonts/MomoTrustSans-SemiBold.ttf"),
                600,
            ),
            (
                include_bytes!("../assets/fonts/MomoTrustSans-Bold.ttf"),
                700,
            ),
        ];

        for (bytes, expected_weight) in FACES {
            let face = ttf_parser::Face::parse(bytes, 0).expect("embedded face must parse");

            assert_eq!(
                face.weight().to_number(),
                expected_weight,
                "embedded face should be weight {expected_weight}",
            );

            // Mirrors `fontdb`'s rule: typographic family wins when present.
            let names: Vec<_> = face.names().into_iter().collect();
            let typed_family = names
                .iter()
                .filter(|name| name.name_id == ttf_parser::name_id::TYPOGRAPHIC_FAMILY)
                .filter_map(|name| name.to_string())
                .next();
            let family = typed_family.or_else(|| {
                names
                    .iter()
                    .filter(|name| name.name_id == ttf_parser::name_id::FAMILY)
                    .filter_map(|name| name.to_string())
                    .next()
            });
            assert_eq!(
                family.as_deref(),
                Some("Momo Trust Sans"),
                "weight {expected_weight} must resolve under `Momo Trust Sans`; \
                 a per-weight family name makes the face unreachable",
            );
        }
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

    /// Glass paint keeps the palette colour and only ever moves the alpha:
    /// the opaque end returns the exact shipped colour, so the default
    /// rendering cannot drift from `mono.json`.
    #[test]
    fn glass_paint_moves_only_alpha() {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let palette = mono_theme(mode);
            for color in [
                palette.colors.background,
                palette.colors.sidebar,
                palette.colors.title_bar,
                palette.colors.group_box,
            ] {
                // 100%, and anything above it, is today's look.
                assert_eq!(glass_surface(color, WINDOW_OPACITY_MAX), color);
                assert_eq!(glass_surface(color, 250.0), color);

                let glass = glass_surface(color, 60.0);
                assert!((glass.a - 0.6).abs() < 1e-6, "{mode:?}: alpha {}", glass.a);
                // Only alpha moves: hue/saturation/lightness are the palette's.
                assert_eq!(glass.h, color.h);
                assert_eq!(glass.s, color.s);
                assert_eq!(glass.l, color.l);

                // The floor is enforced, not just documented.
                assert!((glass_surface(color, 1.0).a - 0.4).abs() < 1e-6);
                // A malformed value falls back to opaque, never to invisible.
                assert_eq!(glass_surface(color, f64::NAN), color);
            }
        }
    }

    /// The opacity clamp mirrors the font-size one: finite values saturate,
    /// NaN falls back to the opaque end.
    #[test]
    fn glass_opacity_clamps_to_supported_range() {
        assert_eq!(clamp_window_opacity(0.0), WINDOW_OPACITY_MIN);
        assert_eq!(clamp_window_opacity(39.9), WINDOW_OPACITY_MIN);
        assert_eq!(clamp_window_opacity(WINDOW_OPACITY_MIN), WINDOW_OPACITY_MIN);
        assert_eq!(clamp_window_opacity(72.0), 72.0);
        assert_eq!(clamp_window_opacity(WINDOW_OPACITY_MAX), WINDOW_OPACITY_MAX);
        assert_eq!(clamp_window_opacity(100.1), WINDOW_OPACITY_MAX);
        assert_eq!(clamp_window_opacity(f64::NAN), WINDOW_OPACITY_MAX);
        assert_eq!(clamp_window_opacity(f64::INFINITY), WINDOW_OPACITY_MAX);
        assert_eq!(clamp_window_opacity(f64::NEG_INFINITY), WINDOW_OPACITY_MAX);

        // The sidebar tone is a wash strength, not an opacity: zero must be
        // invisible, the default must land on zeron's 5% ink, and a bad value
        // must fall back to the default rather than to the strongest tone.
        assert_eq!(sidebar_tone_alpha(0.0), 0.0);
        assert!((sidebar_tone_alpha(SIDEBAR_TONE_DEFAULT) - 0.05).abs() < f32::EPSILON);
        assert_eq!(sidebar_tone_alpha(100.0), SIDEBAR_TONE_MAX_ALPHA);
        assert_eq!(clamp_sidebar_tone(f64::NAN), SIDEBAR_TONE_DEFAULT);
        assert_eq!(clamp_sidebar_tone(f64::INFINITY), SIDEBAR_TONE_DEFAULT);
        assert_eq!(clamp_sidebar_tone(-10.0), 0.0);
        assert_eq!(clamp_sidebar_tone(1000.0), 100.0);

        // The rail width clamps to zeron's 242–400 range and falls back to the
        // default on junk (never to a zero-width rail).
        assert_eq!(clamp_sidebar_width(100.0), 242.0);
        assert_eq!(clamp_sidebar_width(256.0), 256.0);
        assert_eq!(clamp_sidebar_width(10_000.0), 400.0);
        assert_eq!(clamp_sidebar_width(f64::NAN), 256.0);
    }

    /// An accent drives the brand tokens — primary, its derived shades, and
    /// the focus ring — and leaves the framework's neutral list-hover wash
    /// (`colors.accent`) alone, which is not what "accent" means to a user.
    #[test]
    fn accent_drives_the_primary_family_and_the_ring() {
        let accent = Hsla::parse_hex("#2e6fd0").unwrap();
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let mut theme = mono_theme(mode);
            let wash_before = theme.colors.accent;

            write_accent(&mut theme, accent);

            assert_eq!(theme.colors.primary, accent, "{mode:?}: primary");
            assert_eq!(theme.colors.ring, accent, "{mode:?}: ring");
            assert_eq!(
                theme.colors.accent, wash_before,
                "{mode:?}: hover wash untouched"
            );
            // The derived shades move away from the accent, in the direction
            // the shipped palette uses for this mode.
            let hover = theme.colors.primary_hover;
            let active = theme.colors.primary_active;
            if mode.is_dark() {
                assert!(hover.l < accent.l, "{mode:?}: hover darkens");
                assert!(active.l < hover.l, "{mode:?}: active darkens further");
            } else {
                assert!(hover.l > accent.l, "{mode:?}: hover lightens");
                assert!(active.l > hover.l, "{mode:?}: active lightens further");
            }
        }
    }

    /// The label on the accent stays readable: near-black on a light color,
    /// white on a dark one.
    #[test]
    fn accent_foreground_stays_readable() {
        let light_accent = Hsla::parse_hex("#f6d55c").unwrap();
        let dark_accent = Hsla::parse_hex("#124b68").unwrap();
        for (accent, expected_l) in [(light_accent, 0.15), (dark_accent, 1.0)] {
            let mut theme = mono_theme(ThemeMode::Light);
            write_accent(&mut theme, accent);
            let fg = theme.colors.primary_foreground;
            assert_eq!(fg.l, expected_l, "foreground lightness for {accent:?}");
            assert_eq!(fg.a, 1.0, "the foreground is opaque");
        }
    }

    /// A stored accent round-trips, and junk falls back to the theme's own
    /// color rather than to something arbitrary.
    #[test]
    fn stored_accent_parses_or_falls_back() {
        for (_, hex) in ACCENT_PRESETS {
            assert!(accent_from_str(hex).is_some(), "{hex} must parse");
        }
        assert!(accent_from_str("").is_none());
        assert!(accent_from_str("not-a-color").is_none());
        assert!(accent_from_str("#GGGGGG").is_none());
    }

    /// The appearance choice resolves to a mode: Light and Dark ignore the
    /// system, System follows it — including the two `Vibrant` appearances
    /// macOS reports for translucent windows.
    #[test]
    fn appearance_resolves_against_the_system() {
        for (system, expected) in [
            (WindowAppearance::Light, ThemeMode::Light),
            (WindowAppearance::Dark, ThemeMode::Dark),
            (WindowAppearance::VibrantLight, ThemeMode::Light),
            (WindowAppearance::VibrantDark, ThemeMode::Dark),
        ] {
            assert_eq!(Appearance::Light.resolve(system), ThemeMode::Light);
            assert_eq!(Appearance::Dark.resolve(system), ThemeMode::Dark);
            assert_eq!(Appearance::System.resolve(system), expected);
        }
    }

    /// The persisted word round-trips, and anything else is rejected so a
    /// hand-edited file cannot produce an unselectable value.
    #[test]
    fn appearance_words_round_trip_and_reject_junk() {
        for appearance in [Appearance::Light, Appearance::Dark, Appearance::System] {
            assert_eq!(Appearance::parse(appearance.as_str()), Some(appearance));
        }
        assert_eq!(Appearance::parse("Light"), None);
        assert_eq!(Appearance::parse("auto"), None);
        assert_eq!(Appearance::parse(""), None);
    }

    /// The default must mirror `AppSettings::default()`, which starts in
    /// light mode — a stored-settings default that disagreed would silently
    /// flip the theme on a missing config file.
    #[test]
    fn appearance_default_matches_the_app_default() {
        assert_eq!(Appearance::default(), Appearance::Light);
        assert_eq!(
            Appearance::default().resolve(WindowAppearance::Dark),
            ThemeMode::Light
        );
    }

    /// The app's own palette, resolved the way `Theme::change` resolves it.
    fn mono_theme(mode: ThemeMode) -> Theme {
        let set: ThemeSet =
            serde_json::from_str(MONO_THEMES).expect("embedded mono.json must deserialize");
        let config = set
            .themes
            .into_iter()
            .find(|theme| theme.mode == mode)
            .expect("both modes are present in the embedded theme set");
        let mut theme = Theme::default();
        theme.apply_config(&Rc::new(config));
        theme
    }

    /// The agreed token layer: the config values both modes carry and the
    /// tokens the vendored components derive from them. Any drift here moves
    /// the app's density or shape, so every value is pinned.
    #[test]
    fn theme_token_values_are_pinned() {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let theme = mono_theme(mode);
            let mode = mode.name();

            // Config values (the palette digest test pins the colours).
            assert_eq!(theme.font_size, px(14.), "{mode}: font.size");
            assert_eq!(theme.mono_font_size, px(12.), "{mode}: mono_font.size");
            assert_eq!(theme.radius, px(8.), "{mode}: radius");
            assert_eq!(theme.radius_lg, px(16.), "{mode}: radius.lg");
            assert!(!theme.shadow, "{mode}: shadow");

            // Derived radius tokens: sm = radius/2, md = radius, lg =
            // radius.lg, xl = radius*2 — the lg/xl collision (both 16) is
            // accepted (O3).
            let radius = theme.radius_tokens();
            assert_eq!(radius.none, px(0.), "{mode}: radius.none");
            assert_eq!(radius.sm, px(4.), "{mode}: radius.sm");
            assert_eq!(radius.md, px(8.), "{mode}: radius.md");
            assert_eq!(radius.lg, px(16.), "{mode}: radius.lg");
            assert_eq!(radius.xl, px(16.), "{mode}: radius.xl");
            assert_eq!(radius.full, px(9999.), "{mode}: radius.full");

            // Spacing is the framework default scale — no app override.
            let spacing = theme.spacing_tokens();
            assert_eq!(spacing.xxs, px(2.), "{mode}: spacing.xxs");
            assert_eq!(spacing.xs, px(4.), "{mode}: spacing.xs");
            assert_eq!(spacing.sm, px(8.), "{mode}: spacing.sm");
            assert_eq!(spacing.md, px(12.), "{mode}: spacing.md");
            assert_eq!(spacing.lg, px(16.), "{mode}: spacing.lg");
            assert_eq!(spacing.xl, px(24.), "{mode}: spacing.xl");
            assert_eq!(spacing.xxl, px(32.), "{mode}: spacing.xxl");

            // Typography: framework defaults with the sizes taken from the
            // config (`md` = font.size, `mono_md` = mono_font.size).
            let typography = theme.typography_tokens();
            assert_eq!(typography.xs.size, px(12.), "{mode}: typography.xs.size");
            assert_eq!(
                typography.xs.line_height,
                px(16.),
                "{mode}: typography.xs.line_height"
            );
            assert_eq!(typography.sm.size, px(14.), "{mode}: typography.sm.size");
            assert_eq!(
                typography.sm.line_height,
                px(20.),
                "{mode}: typography.sm.line_height"
            );
            assert_eq!(typography.md.size, px(14.), "{mode}: typography.md.size");
            assert_eq!(
                typography.md.line_height,
                px(24.),
                "{mode}: typography.md.line_height"
            );
            assert_eq!(typography.lg.size, px(18.), "{mode}: typography.lg.size");
            assert_eq!(
                typography.lg.line_height,
                px(28.),
                "{mode}: typography.lg.line_height"
            );
            assert_eq!(typography.xl.size, px(20.), "{mode}: typography.xl.size");
            assert_eq!(
                typography.xl.line_height,
                px(28.),
                "{mode}: typography.xl.line_height"
            );
            assert_eq!(
                typography.mono_md.size,
                px(12.),
                "{mode}: typography.mono_md.size"
            );
            assert_eq!(
                typography.mono_md.line_height,
                px(20.),
                "{mode}: typography.mono_md.line_height"
            );

            // `shadow: false` keeps the vendored elevation snapshot empty.
            let shadow = theme.shadow_tokens();
            assert!(shadow.sm.is_empty(), "{mode}: shadow.sm");
            assert!(shadow.md.is_empty(), "{mode}: shadow.md");
            assert!(shadow.lg.is_empty(), "{mode}: shadow.lg");
        }
    }

    /// `Root::render` sets the window rem to `theme.font_size`, so every rem
    /// helper renders at 14/16 of its Tailwind nominal. The card's
    /// `.rounded_lg()` (0.5rem), `text_sm` (0.875rem) and `h_8` (2rem) are
    /// pinned at the approved base.
    #[test]
    fn rem_derived_values_match_the_approved_base() {
        let mut theme = Theme::default();
        write_overrides(&mut theme, 14.0);
        assert_eq!(theme.font_size, px(14.));

        let rem = theme.font_size;
        assert_eq!(rems(0.5).to_pixels(rem), px(7.), "card radius");
        assert_eq!(rems(0.875).to_pixels(rem), px(12.25), "text_sm");
        assert_eq!(rems(2.).to_pixels(rem), px(28.), "h_8");
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

    /// Canonical (key-sorted, compact) JSON of a mode's `colors` object.
    fn canonical_colors(colors: &serde_json::Value) -> String {
        let canonical: BTreeMap<String, serde_json::Value> = colors
            .as_object()
            .expect("colors must be an object")
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();
        serde_json::to_string(&canonical).expect("canonical colors must serialize")
    }

    /// FNV-1a 64 digest of `bytes`.
    fn fnv1a64(bytes: &[u8]) -> u64 {
        let mut hash = 0xcbf2_9ce4_8422_2325_u64;
        for byte in bytes {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        hash
    }

    /// Every colour byte is frozen: the palette digest must not move without
    /// an owner decision. The canonical JSON is printed on mismatch so an
    /// intentional change can be re-pinned by inspection.
    #[test]
    fn mono_palettes_are_pinned() {
        let set: serde_json::Value =
            serde_json::from_str(MONO_THEMES).expect("embedded mono.json must deserialize");

        let themes = set["themes"].as_array().expect("themes array");
        // Without this, an empty or renamed `themes` array would make the
        // loop below iterate zero times and the test would pass while
        // pinning nothing at all.
        assert_eq!(
            themes.len(),
            2,
            "expected exactly mono-light and mono-dark, found {} theme(s)",
            themes.len()
        );
        for theme in themes {
            let name = theme["name"].as_str().expect("theme name");
            let canonical = canonical_colors(&theme["colors"]);
            let expected = match name {
                "mono-light" => 0x14ec_aa66_cfa4_390a_u64,
                "mono-dark" => 0xa2f9_93cc_b566_9fcd_u64,
                other => panic!("unexpected theme `{other}` in the embedded set"),
            };
            assert_eq!(
                fnv1a64(canonical.as_bytes()),
                expected,
                "{name}: palette changed; canonical colors:\n{canonical}"
            );
        }
    }

    #[test]
    fn font_size_clamps_to_supported_range() {
        assert_eq!(clamp_font_size(0.0), FONT_SIZE_MIN);
        assert_eq!(clamp_font_size(7.9), FONT_SIZE_MIN);
        assert_eq!(clamp_font_size(FONT_SIZE_MIN), FONT_SIZE_MIN);
        assert_eq!(clamp_font_size(15.0), 15.0);
        assert_eq!(clamp_font_size(FONT_SIZE_MAX), FONT_SIZE_MAX);
        assert_eq!(clamp_font_size(33.0), FONT_SIZE_MAX);
        assert_eq!(clamp_font_size(999.0), FONT_SIZE_MAX);
        // Non-finite input must not pass through: `f64::clamp` alone is
        // NaN-transparent and would hand `px()` a NaN rem size.
        assert_eq!(clamp_font_size(f64::NAN), 14.0);
        assert_eq!(clamp_font_size(f64::INFINITY), FONT_SIZE_MAX);
        assert_eq!(clamp_font_size(f64::NEG_INFINITY), FONT_SIZE_MIN);
    }

    /// The startup default and the page reset closures both use
    /// `AppSettings::default().font_size`: it must be exactly 14, equal both
    /// modes' `font.size`, and sit inside the supported range. A drift here
    /// silently changes the baseline the theme (and the window rem size every
    /// style helper derives from) was tuned for.
    #[test]
    fn default_and_reset_agree_with_the_config() {
        let default = crate::AppSettings::default().font_size;
        assert_eq!(default, 14.0, "the startup default is pinned to 14");
        assert!(
            (FONT_SIZE_MIN..=FONT_SIZE_MAX).contains(&default),
            "the default must sit inside the supported range"
        );

        let set: serde_json::Value =
            serde_json::from_str(MONO_THEMES).expect("embedded mono.json must deserialize");
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
            assert_eq!(
                config.font_family.as_deref(),
                Some("Momo Trust Sans"),
                "{name}: font.family must name the embedded UI sans, \
                 otherwise the theme asks for a family the app never registers",
            );
            assert!(
                config.mono_font_family.is_some(),
                "{name}: mono_font.family"
            );
        }
    }
}
