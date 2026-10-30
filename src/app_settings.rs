//! The app's settings state and the settings-domain defaults/clamps it owns.
//!
//! Ported unchanged (behaviour and persisted words, plan NG4) from the
//! pre-port gpui-kit implementation on `wip/gpui-kit-shell`:
//!
//! - [`AppSettings`], [`Appearance`], [`DigestCadence`] and the
//!   `from_stored`/`to_stored` codec: `src/main.rs` (fields/defaults
//!   ~236–305, helpers ~138–235, conversions ~314–402).
//! - The default/clamp/accent layer: `src/theme.rs` (~25–165, ~180–186). In
//!   the pre-port app that module was the app's own theme layer; on the zeron
//!   stack the theme is `zeron_ui::theme`, so these are **app-owned
//!   settings-domain values and defaults**, not paint tokens. The old
//!   `Appearance::resolve`/theme application stayed behind (wired by
//!   `settings-surface` T2/T3).
//!
//! Nothing here applies the theme: the settings pages and the Appearance
//! wiring read this state and call into `zeron_ui::theme` themselves.

use gpui::{App, Global, Hsla, Rgba, SharedString};

use crate::persist::StoredSettings;

/// Lower bound of the Window opacity setting, in percent.
///
/// Floored well above zero: below ~40% the frost layer stops carrying
/// enough contrast over an arbitrary desktop backdrop for this app's text,
/// which is a readability cliff rather than a taste choice.
pub(crate) const WINDOW_OPACITY_MIN: f64 = 40.0;
/// Upper bound of the Window opacity setting, in percent (opaque).
pub(crate) const WINDOW_OPACITY_MAX: f64 = 100.0;
/// Default Window opacity, in percent, when translucency is first enabled.
///
/// Matches monocode's 0.85 surface opacity: low enough to see the frosted
/// backdrop immediately, high enough for the settings text to stay readable.
pub(crate) const WINDOW_OPACITY_DEFAULT: f64 = 85.0;

/// Lower bound of the Sidebar tone setting, in percent of the max wash.
pub(crate) const SIDEBAR_TONE_MIN: f64 = 0.0;
/// Upper bound of the Sidebar tone setting, in percent of the max wash.
pub(crate) const SIDEBAR_TONE_MAX: f64 = 100.0;
/// Default Sidebar tone, in percent: 50% of the max wash, i.e. an ink tint of
/// 0.05 — the same weight zeron's glass sidebar paints (`wash(0.05)`).
pub(crate) const SIDEBAR_TONE_DEFAULT: f64 = 50.0;
/// Sidebar rail width bounds, in logical pixels.
///
/// Matches zeron's `SIDEBAR_MIN`/`SIDEBAR_MAX`/`SIDEBAR_DEFAULT`: the rail is
/// resizable between those limits and its last width persists across launches.
pub(crate) const SIDEBAR_WIDTH_MIN: f64 = 242.0;
pub(crate) const SIDEBAR_WIDTH_MAX: f64 = 400.0;
pub(crate) const SIDEBAR_WIDTH_DEFAULT: f64 = 256.0;

/// Clamp a requested sidebar width into the supported pixel range.
///
/// A non-finite value falls back to the default width, so a malformed file
/// cannot produce a zero-width or runaway rail.
pub(crate) fn clamp_sidebar_width(width: f64) -> f64 {
    if !width.is_finite() {
        return SIDEBAR_WIDTH_DEFAULT;
    }
    width.clamp(SIDEBAR_WIDTH_MIN, SIDEBAR_WIDTH_MAX)
}

/// Clamp a requested window opacity into the supported percentage range.
///
/// Same NaN guard as the other clamps, but the fallback is the *opaque*
/// end: a malformed value must never make the window unexpectedly
/// transparent.
pub(crate) fn clamp_window_opacity(opacity: f64) -> f64 {
    if !opacity.is_finite() {
        return WINDOW_OPACITY_MAX;
    }
    opacity.clamp(WINDOW_OPACITY_MIN, WINDOW_OPACITY_MAX)
}

/// Clamp a requested sidebar tone into its percentage range.
///
/// The fallback is the default tone, never "more ink than asked for": a
/// malformed file must not darken the rail unexpectedly.
pub(crate) fn clamp_sidebar_tone(tone: f64) -> f64 {
    if !tone.is_finite() {
        return SIDEBAR_TONE_DEFAULT;
    }
    tone.clamp(SIDEBAR_TONE_MIN, SIDEBAR_TONE_MAX)
}

/// Parse a stored accent — `#RRGGBB` (or `#RRGGBBAA`) — or `None` for the
/// theme's own.
///
/// Anything unparseable is `None`, so a hand-edited file falls back to the
/// shipped palette rather than to an arbitrary color. The pre-port layer used
/// the pre-port UI kit's `Colorize::parse_hex`; the zeron stack has no such
/// extension trait, so the same byte-level codec is implemented here over
/// gpui's `Rgba` (same accepted input set, same `Hsla` result).
pub(crate) fn accent_from_str(s: &str) -> Option<Hsla> {
    let hex = s.trim_start_matches('#');
    // Byte-sliced below: non-ASCII input is rejected up front instead of
    // panicking on a non-char-boundary (the old codec sliced blindly).
    if !matches!(hex.len(), 6 | 8) || !hex.is_ascii() {
        return None;
    }
    let channel = |range: std::ops::Range<usize>| u8::from_str_radix(&hex[range], 16).ok();
    let r = channel(0..2)? as f32 / 255.;
    let g = channel(2..4)? as f32 / 255.;
    let b = channel(4..6)? as f32 / 255.;
    let a = match hex.len() {
        8 => channel(6..8)? as f32 / 255.,
        _ => 1.,
    };
    Some(Hsla::from(Rgba { r, g, b, a }))
}

/// The persisted `#RRGGBB` word for an accent, uppercase like the pre-port kit's
/// `Colorize::to_hex` (the pre-port `to_stored` used it verbatim).
pub(crate) fn accent_to_str(accent: Hsla) -> String {
    let rgb = accent.to_rgb();
    let channel = |c: f32| (c * 255.) as u32;
    if accent.a < 1. {
        format!(
            "#{:02X}{:02X}{:02X}{:02X}",
            channel(rgb.r),
            channel(rgb.g),
            channel(rgb.b),
            channel(accent.a)
        )
    } else {
        format!(
            "#{:02X}{:02X}{:02X}",
            channel(rgb.r),
            channel(rgb.g),
            channel(rgb.b)
        )
    }
}

/// How the app picks its theme: one mode always, or as the system does.
///
/// `zeron_ui::theme::Appearance` is the *resolved* light/dark mode; this is
/// the app's *choice*, which can also be "follow the OS", so the two cannot
/// be the same type.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Appearance {
    #[default]
    Light,
    Dark,
    System,
}

impl Appearance {
    /// The persisted `key=value` word. Stable — it is written to disk.
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Appearance::Light => "light",
            Appearance::Dark => "dark",
            Appearance::System => "system",
        }
    }

    /// Parse a persisted word. `None` for anything else, so a hand-edited
    /// file falls back to the default rather than becoming unselectable.
    pub(crate) fn parse(s: &str) -> Option<Self> {
        match s {
            "light" => Some(Appearance::Light),
            "dark" => Some(Appearance::Dark),
            "system" => Some(Appearance::System),
            _ => None,
        }
    }

    /// The i18n key for the mode's row/segment label; the old pages paired
    /// the choice with `appearance.light|dark|system` in all four locales.
    pub(crate) fn label_key(self) -> &'static str {
        match self {
            Appearance::Light => "appearance.light",
            Appearance::Dark => "appearance.dark",
            Appearance::System => "appearance.system",
        }
    }
}

/// How often digest notifications arrive (the Notifications → Digest radio).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DigestCadence {
    Immediate,
    Daily,
    Weekly,
}

impl DigestCadence {
    /// The value persisted and used as each radio's key.
    pub(crate) fn value(self) -> &'static str {
        match self {
            Self::Immediate => "immediate",
            Self::Daily => "daily",
            Self::Weekly => "weekly",
        }
    }

    /// The i18n key for the radio's label.
    pub(crate) fn label_key(self) -> &'static str {
        match self {
            Self::Immediate => "digest.immediate",
            Self::Daily => "digest.daily",
            Self::Weekly => "digest.weekly",
        }
    }

    /// Parse a stored value; unknown text falls back to `Immediate` (the same
    /// whitelist `persist::parse` applies).
    pub(crate) fn parse(value: &str) -> Self {
        match value {
            "daily" => Self::Daily,
            "weekly" => Self::Weekly,
            _ => Self::Immediate,
        }
    }

    pub(crate) const ALL: [Self; 3] = [Self::Immediate, Self::Daily, Self::Weekly];
}

/// Application state, stored as a GPUI global so the settings pages can reach
/// it through `&App` / `&mut App` (old `main.rs` field docs kept).
///
/// Fields are `pub(crate)` — the pre-port state lived in `main.rs` and was
/// private by virtue of the module boundary; the pages/tests now live in
/// sibling modules.
pub(crate) struct AppSettings {
    pub(crate) appearance: Appearance,
    /// Brand accent, or `None` for the theme's own. Persisted as `#RRGGBB`.
    pub(crate) accent: Option<Hsla>,
    pub(crate) language: SharedString,
    pub(crate) font_size: f64,
    /// Whether the window background is translucent (the glass master
    /// switch). `false` renders the shipped opaque palette.
    pub(crate) background_translucent: bool,
    /// The single frost-shell opacity in percent while translucent; clamped
    /// to 40–100. One layer for the whole window, zeron-style.
    pub(crate) window_opacity: f64,
    /// Sidebar ink tone in percent of the max wash, 0–100. Painted *over* the
    /// frost (never its own glass base), so the rail separates from the
    /// content without a second alpha surface.
    pub(crate) sidebar_tone: f64,
    /// Persisted rail width in logical pixels (242–400). The drag seam writes
    /// it live and persists on release, like the percentage sliders.
    pub(crate) sidebar_width: f64,
    /// Whether the window asks the platform to blur what is behind it. Only
    /// meaningful while `background_translucent` is on.
    pub(crate) background_blur: bool,
    pub(crate) launch_at_login: bool,
    pub(crate) notifications_enabled: bool,
    pub(crate) username: SharedString,
    pub(crate) display_name: SharedString,
    pub(crate) channel_email: bool,
    pub(crate) channel_desktop: bool,
    pub(crate) channel_digest: bool,
    pub(crate) digest: DigestCadence,
    pub(crate) timezone: SharedString,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            appearance: Appearance::default(),
            accent: None,
            language: "en".into(),
            // Matches the theme's `font.size` (14px) so the applied default
            // preserves the approved visual baseline.
            font_size: 14.0,
            // Glass is opt-in; when it is switched on, the default frost
            // opacity matches monocode's 0.85 surface (see
            // `WINDOW_OPACITY_DEFAULT`), and the sidebar keeps the subtle 5%
            // ink tone zeron's glass sidebar uses.
            background_translucent: false,
            window_opacity: WINDOW_OPACITY_DEFAULT,
            sidebar_tone: SIDEBAR_TONE_DEFAULT,
            sidebar_width: SIDEBAR_WIDTH_DEFAULT,
            background_blur: false,
            launch_at_login: false,
            notifications_enabled: true,
            username: "ninja".into(),
            display_name: "Ninja".into(),
            channel_email: true,
            channel_desktop: true,
            // One child off: the Channels parent opens in its mixed state.
            channel_digest: false,
            digest: DigestCadence::Immediate,
            timezone: "system".into(),
        }
    }
}

impl Global for AppSettings {}

impl AppSettings {
    pub(crate) fn global(cx: &App) -> &Self {
        cx.global::<AppSettings>()
    }

    pub(crate) fn global_mut(cx: &mut App) -> &mut Self {
        cx.global_mut::<AppSettings>()
    }
}

impl AppSettings {
    pub(crate) fn from_stored(s: StoredSettings) -> Self {
        Self {
            appearance: Appearance::parse(&s.appearance).unwrap_or_default(),
            accent: accent_from_str(&s.accent),
            language: s.language.into(),
            font_size: if s.font_size.is_finite() {
                s.font_size
            } else {
                14.0
            },
            background_translucent: s.background_translucent,
            // The codec already clamps; this is the same guard as
            // `font_size` for a settings struct built in memory.
            window_opacity: if s.window_opacity.is_finite() {
                clamp_window_opacity(s.window_opacity)
            } else {
                AppSettings::default().window_opacity
            },
            sidebar_tone: if s.sidebar_tone.is_finite() {
                clamp_sidebar_tone(s.sidebar_tone)
            } else {
                AppSettings::default().sidebar_tone
            },
            sidebar_width: if s.sidebar_width.is_finite() {
                clamp_sidebar_width(s.sidebar_width)
            } else {
                AppSettings::default().sidebar_width
            },
            background_blur: s.background_blur,
            launch_at_login: s.launch_at_login,
            notifications_enabled: s.notifications_enabled,
            username: s.username.into(),
            display_name: s.display_name.into(),
            channel_email: s.channel_email,
            channel_desktop: s.channel_desktop,
            channel_digest: s.channel_digest,
            digest: DigestCadence::parse(&s.digest),
            timezone: s.timezone.into(),
        }
    }

    pub(crate) fn to_stored(&self) -> StoredSettings {
        StoredSettings {
            appearance: self.appearance.as_str().to_string(),
            accent: self.accent.map(accent_to_str).unwrap_or_default(),
            language: self.language.to_string(),
            font_size: self.font_size,
            background_translucent: self.background_translucent,
            window_opacity: self.window_opacity,
            sidebar_tone: self.sidebar_tone,
            sidebar_width: self.sidebar_width,
            background_blur: self.background_blur,
            launch_at_login: self.launch_at_login,
            notifications_enabled: self.notifications_enabled,
            username: self.username.to_string(),
            display_name: self.display_name.to_string(),
            channel_email: self.channel_email,
            channel_desktop: self.channel_desktop,
            channel_digest: self.channel_digest,
            digest: self.digest.value().to_string(),
            timezone: self.timezone.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ported from old `src/theme.rs`
    /// `glass_opacity_clamps_to_supported_range`.
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

        // The sidebar tone is a wash percentage clamped to 0–100, and a bad
        // value falls back to the default rather than to the strongest tone.
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

    /// Ported from old `src/theme.rs` `stored_accent_parses_or_falls_back`
    /// (the old preset list was theme palette data and stayed behind; the
    /// words it asserted parse are inlined here).
    #[test]
    fn stored_accent_parses_or_falls_back() {
        for hex in [
            "#124b68", "#2e6fd0", "#1f7a45", "#8d6be7", "#935f06", "#bb1b1b",
        ] {
            assert!(accent_from_str(hex).is_some(), "{hex} must parse");
        }
        assert!(accent_from_str("").is_none());
        assert!(accent_from_str("not-a-color").is_none());
        assert!(accent_from_str("#GGGGGG").is_none());
    }

    /// The persisted accent word is `#RRGGBB`, uppercase (the old
    /// `Colorize::to_hex` output), and it always re-parses.
    ///
    /// The codec round-trips through HSL, so one encode can move a channel by
    /// a unit (`#124b68` writes `#124B67`). That is the pre-port
    /// pre-port path's behaviour; plan NG4 freezes it rather than
    /// "fixing" it, so this test asserts the shape, not a byte-exact hop.
    #[test]
    fn accent_words_round_trip() {
        for hex in ["#124b68", "#2e6fd0", "#bb1b1b"] {
            let accent = accent_from_str(hex).expect("valid hex");
            let word = accent_to_str(accent);
            assert_eq!(word.len(), 7, "{hex}: {word}");
            assert_eq!(word, word.to_uppercase(), "{hex}: {word}");
            assert!(word.starts_with('#'), "{hex}: {word}");
            assert!(accent_from_str(&word).is_some(), "{hex}: {word}");
        }
        // Junk (including a value that is not 6/8 hex digits) never escapes.
        assert!(accent_from_str("#12").is_none());
        assert!(accent_from_str("ééé").is_none());
    }

    /// Ported from old `src/theme.rs`
    /// `appearance_words_round_trip_and_reject_junk`.
    #[test]
    fn appearance_words_round_trip_and_reject_junk() {
        for appearance in [Appearance::Light, Appearance::Dark, Appearance::System] {
            assert_eq!(Appearance::parse(appearance.as_str()), Some(appearance));
        }
        assert_eq!(Appearance::parse("Light"), None);
        assert_eq!(Appearance::parse("auto"), None);
        assert_eq!(Appearance::parse(""), None);
    }

    /// Ported from old `src/theme.rs`
    /// `appearance_default_matches_the_app_default` (the theme-resolution half
    /// is T2/T3 work and stayed behind).
    #[test]
    fn appearance_default_matches_the_app_default() {
        assert_eq!(Appearance::default(), Appearance::Light);
        assert_eq!(AppSettings::default().appearance, Appearance::Light);
    }

    /// The digest cadence words are the persisted whitelist and its labels are
    /// real i18n keys.
    #[test]
    fn digest_cadence_words_and_labels_round_trip() {
        for cadence in DigestCadence::ALL {
            assert_eq!(DigestCadence::parse(cadence.value()), cadence);
            assert_eq!(
                crate::i18n::tr(cadence.label_key(), "en"),
                match cadence {
                    DigestCadence::Immediate => "Immediate",
                    DigestCadence::Daily => "Daily",
                    DigestCadence::Weekly => "Weekly",
                }
            );
        }
        assert_eq!(DigestCadence::parse("bogus"), DigestCadence::Immediate);
        for appearance in [Appearance::Light, Appearance::Dark, Appearance::System] {
            assert!(!crate::i18n::tr(appearance.label_key(), "en").is_empty());
        }
    }

    /// Plan NG4: the in-memory default must mirror the persisted default —
    /// otherwise a missing config file would silently change the app's start
    /// state.
    #[test]
    fn app_default_mirrors_stored_default() {
        assert_eq!(
            AppSettings::default().to_stored(),
            StoredSettings::default()
        );
        assert_eq!(
            AppSettings::from_stored(StoredSettings::default()).to_stored(),
            StoredSettings::default()
        );
    }

    /// `from_stored` applies the same non-finite guards as the codec: a
    /// settings struct built in memory (or a hostile file) can never hand the
    /// UI a NaN/∞ number.
    #[test]
    fn from_stored_guards_non_finite_values() {
        let stored = StoredSettings {
            font_size: f64::NAN,
            window_opacity: f64::INFINITY,
            sidebar_tone: f64::NAN,
            sidebar_width: f64::NEG_INFINITY,
            ..StoredSettings::default()
        };
        let settings = AppSettings::from_stored(stored);
        assert_eq!(settings.font_size, 14.0);
        assert_eq!(settings.window_opacity, WINDOW_OPACITY_DEFAULT);
        assert_eq!(settings.sidebar_tone, SIDEBAR_TONE_DEFAULT);
        assert_eq!(settings.sidebar_width, SIDEBAR_WIDTH_DEFAULT);
    }
}
