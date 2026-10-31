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
//! `appearance-parity` R7 extends the same state with the Appearance page's
//! theme/font/math/motion/background/wallpaper settings (`state-persistence`
//! T1): they stay plain words and numbers in the one `settings.conf`, with the
//! typed mirrors (`UiFontFamily`, `MathFont`, `ReduceMotion`, …) here.
//!
//! Nothing here applies the theme: the settings pages and the Appearance
//! wiring read this state and call into `zeron_ui::theme` themselves.

use std::path::PathBuf;

use gpui::{App, Global, Hsla, Rgba, SharedString};
use zeron_theme::Color;
use zeron_ui::motion::ReduceMotion;
use zeron_ui::typography::UiFontFamily;

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

/// Interface font bounds shared by the terminal/code size settings, in px
/// (zeron `typography::FONT_SIZE_MIN`/`MAX`).
pub(crate) const FONT_SIZE_MIN: f64 = 8.0;
pub(crate) const FONT_SIZE_MAX: f64 = 32.0;
/// Terminal font size default, in px (zeron `TERMINAL_FONT_SIZE_DEFAULT`).
pub(crate) const TERMINAL_FONT_SIZE_DEFAULT: f64 = 13.0;
/// Code/diff font size default, in px (zeron `CODE_FONT_SIZE_DEFAULT`).
pub(crate) const CODE_FONT_SIZE_DEFAULT: f64 = 12.5;

/// Transcript width bounds, in px (zeron `TRANSCRIPT_WIDTH_*`).
pub(crate) const TRANSCRIPT_WIDTH_MIN: f64 = 560.0;
pub(crate) const TRANSCRIPT_WIDTH_MAX: f64 = 1200.0;
pub(crate) const TRANSCRIPT_WIDTH_DEFAULT: f64 = 736.0;
pub(crate) const TRANSCRIPT_WIDTH_STEP: f64 = 16.0;

/// Markdown math size, as a percent of the surrounding text. The Appearance
/// page's `MATH_SIZES` ladder bounds the stored value.
pub(crate) const MATH_SIZE_DEFAULT: u16 = 100;
pub(crate) const MATH_SIZE_MIN: u16 = 90;
pub(crate) const MATH_SIZE_MAX: u16 = 150;

/// Built-in theme variant ids every install starts on (zeron-theme builtins).
pub(crate) const THEME_LIGHT_DEFAULT: &str = "zeron-light";
pub(crate) const THEME_DARK_DEFAULT: &str = "zeron-dark";

/// Default framing of an overflowing new-thread background image.
pub(crate) const FOCAL_DEFAULT: f32 = 0.5;
pub(crate) const ZOOM_DEFAULT: f32 = 1.0;

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

/// Clamp a terminal/code font size into zeron's 8–32 px range.
///
/// The two size settings carry different defaults, so the caller supplies the
/// fallback used for non-finite input (never a NaN in `px()`).
pub(crate) fn clamp_ui_font_size(size: f64, default: f64) -> f64 {
    if !size.is_finite() {
        return default;
    }
    size.clamp(FONT_SIZE_MIN, FONT_SIZE_MAX)
}

/// Clamp a transcript width into range and snap it to the 16 px grid
/// (zeron `normalize_transcript_width`). Non-finite input falls back to the
/// default width, then snaps.
pub(crate) fn normalize_transcript_width(width: f64) -> f64 {
    let width = if width.is_finite() {
        width.clamp(TRANSCRIPT_WIDTH_MIN, TRANSCRIPT_WIDTH_MAX)
    } else {
        TRANSCRIPT_WIDTH_DEFAULT
    };
    TRANSCRIPT_WIDTH_MIN
        + ((width - TRANSCRIPT_WIDTH_MIN) / TRANSCRIPT_WIDTH_STEP).round() * TRANSCRIPT_WIDTH_STEP
}

/// Clamp a stored background-image focal point into `0.0..=1.0`; non-finite
/// input falls back to the centred default.
pub(crate) fn clamp_focal(value: f64) -> f64 {
    if !value.is_finite() {
        return f64::from(FOCAL_DEFAULT);
    }
    value.clamp(0.0, 1.0)
}

/// Clamp a stored background-image zoom into the adjustment's range;
/// non-finite input falls back to the unzoomed default.
pub(crate) fn clamp_zoom(value: f64) -> f64 {
    if !value.is_finite() {
        return f64::from(ZOOM_DEFAULT);
    }
    value.clamp(
        f64::from(NewThreadBackgroundAdjustment::MIN_ZOOM),
        f64::from(NewThreadBackgroundAdjustment::MAX_ZOOM),
    )
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

/// Parse a stored wallpaper color — zeron's `Color` accepts `#rgb`, `#rgba`,
/// `#rrggbb` and `#rrggbbaa` — or `None` when the file has no sample.
pub(crate) fn wallpaper_color_from_str(s: &str) -> Option<Color> {
    if s.is_empty() {
        return None;
    }
    s.parse::<Color>().ok()
}

/// The persisted wallpaper color word: uppercase `#RRGGBB`, the shape the
/// Appearance page shows and the file stores.
pub(crate) fn wallpaper_color_to_str(color: Color) -> String {
    format!("#{:02X}{:02X}{:02X}", color.r, color.g, color.b)
}

/// The persisted word for a font-family choice. Stable — written to disk.
pub(crate) fn font_family_word(family: &UiFontFamily) -> String {
    match family {
        UiFontFamily::MomoTrustSans => "momo-trust-sans".to_string(),
        UiFontFamily::HankenGrotesk => "hanken-grotesk".to_string(),
        UiFontFamily::GoogleSansCode => "google-sans-code".to_string(),
        UiFontFamily::XanhMono => "xanh-mono".to_string(),
        UiFontFamily::System => "system".to_string(),
        UiFontFamily::Installed(name) => format!("installed:{name}"),
    }
}

/// Parse a persisted font-family word. Unknown words, the empty word and
/// `installed:` with no name are `None`, so a hand-edited file falls back to
/// the key's default family rather than to a missing one.
pub(crate) fn font_family_from_word(word: &str) -> Option<UiFontFamily> {
    Some(match word {
        "momo-trust-sans" => UiFontFamily::MomoTrustSans,
        "hanken-grotesk" => UiFontFamily::HankenGrotesk,
        "google-sans-code" => UiFontFamily::GoogleSansCode,
        "xanh-mono" => UiFontFamily::XanhMono,
        "system" => UiFontFamily::System,
        _ => {
            let name = word.strip_prefix("installed:")?;
            if name.is_empty() {
                return None;
            }
            UiFontFamily::Installed(name.to_string())
        }
    })
}

/// The persisted word for the reduced-motion preference.
pub(crate) fn reduce_motion_word(preference: ReduceMotion) -> &'static str {
    match preference {
        ReduceMotion::System => "system",
        ReduceMotion::On => "on",
        ReduceMotion::Off => "off",
    }
}

/// Parse a persisted reduced-motion word; `None` for anything unknown, so the
/// caller falls back to following the OS.
pub(crate) fn reduce_motion_from_word(word: &str) -> Option<ReduceMotion> {
    Some(match word {
        "system" => ReduceMotion::System,
        "on" => ReduceMotion::On,
        "off" => ReduceMotion::Off,
        _ => return None,
    })
}

/// The face Markdown math is set in. Every face is bundled, so the choice
/// never depends on what the device has installed.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum MathFont {
    #[default]
    Fira,
    StixTwo,
    ComputerModern,
    Libertinus,
    Pagella,
    Euler,
}

impl MathFont {
    pub(crate) const ALL: [Self; 6] = [
        Self::Fira,
        Self::StixTwo,
        Self::ComputerModern,
        Self::Libertinus,
        Self::Pagella,
        Self::Euler,
    ];

    /// The name stored in the settings file.
    pub(crate) const fn key(self) -> &'static str {
        match self {
            Self::Fira => "fira",
            Self::StixTwo => "stixTwo",
            Self::ComputerModern => "computerModern",
            Self::Libertinus => "libertinus",
            Self::Pagella => "pagella",
            Self::Euler => "euler",
        }
    }

    /// Parse a persisted key; `None` for anything unknown, so the caller falls
    /// back to the default face.
    pub(crate) fn parse(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|font| font.key() == key)
    }

    #[allow(dead_code)] // wired by the appearance page (fonts section)
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Fira => "Fira Math",
            Self::StixTwo => "STIX Two",
            Self::ComputerModern => "Computer Modern",
            Self::Libertinus => "Libertinus",
            Self::Pagella => "Palatino",
            Self::Euler => "Euler",
        }
    }

    /// The short note beside the label in the face menu.
    #[allow(dead_code)] // wired by the appearance page (fonts section)
    pub(crate) const fn style(self) -> &'static str {
        match self {
            Self::Fira => "Sans",
            Self::StixTwo | Self::Libertinus | Self::Pagella => "Serif",
            Self::ComputerModern => "LaTeX",
            Self::Euler => "Upright",
        }
    }

    #[allow(dead_code)] // wired by the appearance page (fonts section)
    pub(crate) const fn description(self) -> &'static str {
        match self {
            Self::Fira => "A sans face that sits beside the interface font.",
            Self::StixTwo => "A Times-like serif face with the TeX look.",
            Self::ComputerModern => "A serif face with the classic LaTeX look.",
            Self::Libertinus => "A warm serif book face.",
            Self::Pagella => "A Palatino serif face, from TeX Gyre Pagella.",
            Self::Euler => "Zapf's upright, hand-drawn math.",
        }
    }
}

/// Non-destructive treatment composited inside the new-thread background's
/// fade mask.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum NewThreadBackgroundEffect {
    #[default]
    None,
    Dither,
    Ascii,
    Halftone,
    Scanlines,
}

impl NewThreadBackgroundEffect {
    pub(crate) const ALL: [Self; 5] = [
        Self::None,
        Self::Dither,
        Self::Ascii,
        Self::Halftone,
        Self::Scanlines,
    ];

    /// The persisted word for the effect; stable — written to disk.
    pub(crate) const fn word(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Dither => "dither",
            Self::Ascii => "ascii",
            Self::Halftone => "halftone",
            Self::Scanlines => "scanlines",
        }
    }

    /// Parse a persisted word; `None` for anything unknown, so the caller
    /// falls back to no treatment.
    pub(crate) fn parse(word: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|effect| effect.word() == word)
    }

    #[allow(dead_code)] // wired by the appearance page (background dialog)
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Dither => "Dither",
            Self::Ascii => "ASCII",
            Self::Halftone => "Halftone",
            Self::Scanlines => "Scanlines",
        }
    }

    #[allow(dead_code)] // wired by the appearance page (background dialog)
    pub(crate) const fn description(self) -> &'static str {
        match self {
            Self::None => "Shows the original artwork.",
            Self::Dither => "Rebuilds the artwork with a dithered color palette.",
            Self::Ascii => "Recreates the artwork with colored characters on black.",
            Self::Halftone => "Recreates the artwork with colored print dots on black.",
            Self::Scanlines => "Adds a pronounced horizontal display-line texture.",
        }
    }
}

/// Viewport-relative framing of the new-thread background image, kept with the
/// image it belongs to.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct NewThreadBackgroundAdjustment {
    /// Normalized horizontal alignment: 0 left, 1 right.
    pub(crate) focal_x: f32,
    /// Normalized vertical alignment: 0 top, 1 bottom.
    pub(crate) focal_y: f32,
    /// Multiplier applied after the image has been scaled to cover the pane.
    pub(crate) zoom: f32,
}

impl NewThreadBackgroundAdjustment {
    pub(crate) const MIN_ZOOM: f32 = 1.0;
    pub(crate) const MAX_ZOOM: f32 = 4.0;

    /// Clamp every field; a non-finite field falls back to its default rather
    /// than reaching the renderer.
    pub(crate) fn normalized(mut self) -> Self {
        let defaults = Self::default();
        self.focal_x = clamp_or(self.focal_x, 0.0, 1.0, defaults.focal_x);
        self.focal_y = clamp_or(self.focal_y, 0.0, 1.0, defaults.focal_y);
        self.zoom = clamp_or(self.zoom, Self::MIN_ZOOM, Self::MAX_ZOOM, defaults.zoom);
        self
    }
}

impl Default for NewThreadBackgroundAdjustment {
    fn default() -> Self {
        Self {
            focal_x: FOCAL_DEFAULT,
            focal_y: FOCAL_DEFAULT,
            zoom: ZOOM_DEFAULT,
        }
    }
}

/// Clamp a float into `min..=max`; non-finite input takes `fallback`.
fn clamp_or(value: f32, min: f32, max: f32, fallback: f32) -> f32 {
    if value.is_finite() {
        value.clamp(min, max)
    } else {
        fallback
    }
}

/// The device-local artwork behind the blank new-thread composer.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct NewThreadBackground {
    /// Managed copy inside the app's device-local data directory.
    pub(crate) path: PathBuf,
    /// Original file name shown in the Appearance settings row.
    pub(crate) name: String,
    /// Viewport-relative framing, kept with the image it belongs to.
    pub(crate) adjustment: NewThreadBackgroundAdjustment,
}

/// A persisted path word: empty means "None".
fn path_option(value: &str) -> Option<PathBuf> {
    (!value.is_empty()).then(|| PathBuf::from(value))
}

/// The persisted word for an optional path; a missing path writes empty.
fn path_word(path: Option<&std::path::Path>) -> String {
    path.map(|path| path.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// Rebuild the optional background image (path, original name, framing) from
/// its five flat settings keys. No path means no artwork; an empty name falls
/// back to the file name so the row always has something to show.
fn new_thread_background_from_stored(s: &StoredSettings) -> Option<NewThreadBackground> {
    if s.new_thread_background_path.is_empty() {
        return None;
    }
    let path = PathBuf::from(&s.new_thread_background_path);
    let name = if s.new_thread_background_name.is_empty() {
        path.file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default()
    } else {
        s.new_thread_background_name.clone()
    };
    let adjustment = NewThreadBackgroundAdjustment {
        focal_x: clamp_focal(s.new_thread_background_focal_x) as f32,
        focal_y: clamp_focal(s.new_thread_background_focal_y) as f32,
        zoom: clamp_zoom(s.new_thread_background_zoom) as f32,
    }
    .normalized();
    Some(NewThreadBackground {
        path,
        name,
        adjustment,
    })
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
    /// Registry variant id used in light appearance (unknown ids fall back at
    /// apply time).
    pub(crate) theme_light: String,
    /// Registry variant id used in dark appearance.
    pub(crate) theme_dark: String,
    /// Interface font family for the whole app.
    pub(crate) ui_font_family: UiFontFamily,
    /// Family used by the terminal surface.
    pub(crate) terminal_font_family: UiFontFamily,
    /// Family used by code/diff surfaces.
    pub(crate) code_font_family: UiFontFamily,
    /// Terminal font size in px, `8.0..=32.0`.
    pub(crate) terminal_font_size: f64,
    /// Code/diff font size in px, `8.0..=32.0`.
    pub(crate) code_font_size: f64,
    /// Transcript column width in px, clamped and snapped to the 16 px grid.
    pub(crate) transcript_width: f64,
    /// Whether Markdown math is rendered.
    pub(crate) render_math: bool,
    /// Face bundled Markdown math is set in.
    pub(crate) math_font: MathFont,
    /// Math size as a percent of the surrounding text, `90..=150`.
    pub(crate) math_size: u16,
    /// Whether mouse-wheel notches glide instead of jumping.
    pub(crate) smooth_scrolling: bool,
    /// User preference for reduced motion (the OS setting defines `System`).
    pub(crate) reduce_motion: ReduceMotion,
    /// Also snap animations while the main window is not focused.
    pub(crate) pause_in_background: bool,
    /// Whether the current wallpaper tints the theme's accent.
    pub(crate) wallpaper_theme_colors: bool,
    /// Sampled wallpaper color, when one has been extracted.
    pub(crate) wallpaper_color: Option<Color>,
    /// Device-local folder used by the random wallpaper shortcut.
    pub(crate) wallpaper_folder: Option<PathBuf>,
    /// Most recently displayed wallpaper source.
    pub(crate) wallpaper_source: Option<PathBuf>,
    /// Optional device-local artwork behind the blank new-thread composer.
    pub(crate) new_thread_background: Option<NewThreadBackground>,
    /// Non-destructive treatment composited over that artwork.
    pub(crate) new_thread_background_effect: NewThreadBackgroundEffect,
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
            theme_light: THEME_LIGHT_DEFAULT.to_string(),
            theme_dark: THEME_DARK_DEFAULT.to_string(),
            ui_font_family: UiFontFamily::MomoTrustSans,
            terminal_font_family: UiFontFamily::XanhMono,
            code_font_family: UiFontFamily::GoogleSansCode,
            terminal_font_size: TERMINAL_FONT_SIZE_DEFAULT,
            code_font_size: CODE_FONT_SIZE_DEFAULT,
            transcript_width: TRANSCRIPT_WIDTH_DEFAULT,
            render_math: true,
            math_font: MathFont::default(),
            math_size: MATH_SIZE_DEFAULT,
            smooth_scrolling: true,
            reduce_motion: ReduceMotion::System,
            pause_in_background: false,
            wallpaper_theme_colors: false,
            wallpaper_color: None,
            wallpaper_folder: None,
            wallpaper_source: None,
            new_thread_background: None,
            new_thread_background_effect: NewThreadBackgroundEffect::None,
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
        // Read the five flat background keys before their `String` siblings
        // are moved into the state.
        let new_thread_background = new_thread_background_from_stored(&s);
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
            theme_light: if s.theme_light.is_empty() {
                THEME_LIGHT_DEFAULT.to_string()
            } else {
                s.theme_light.clone()
            },
            theme_dark: if s.theme_dark.is_empty() {
                THEME_DARK_DEFAULT.to_string()
            } else {
                s.theme_dark.clone()
            },
            ui_font_family: font_family_from_word(&s.ui_font_family)
                .unwrap_or(UiFontFamily::MomoTrustSans),
            terminal_font_family: font_family_from_word(&s.terminal_font_family)
                .unwrap_or(UiFontFamily::XanhMono),
            code_font_family: font_family_from_word(&s.code_font_family)
                .unwrap_or(UiFontFamily::GoogleSansCode),
            terminal_font_size: clamp_ui_font_size(
                s.terminal_font_size,
                TERMINAL_FONT_SIZE_DEFAULT,
            ),
            code_font_size: clamp_ui_font_size(s.code_font_size, CODE_FONT_SIZE_DEFAULT),
            transcript_width: normalize_transcript_width(s.transcript_width),
            render_math: s.render_math,
            math_font: MathFont::parse(&s.math_font).unwrap_or_default(),
            math_size: s.math_size.clamp(MATH_SIZE_MIN, MATH_SIZE_MAX),
            smooth_scrolling: s.smooth_scrolling,
            reduce_motion: reduce_motion_from_word(&s.reduce_motion).unwrap_or_default(),
            pause_in_background: s.pause_in_background,
            wallpaper_theme_colors: s.wallpaper_theme_colors,
            wallpaper_color: wallpaper_color_from_str(&s.wallpaper_color),
            wallpaper_folder: path_option(&s.wallpaper_folder),
            wallpaper_source: path_option(&s.wallpaper_source),
            new_thread_background,
            new_thread_background_effect: NewThreadBackgroundEffect::parse(
                &s.new_thread_background_effect,
            )
            .unwrap_or_default(),
        }
    }

    pub(crate) fn to_stored(&self) -> StoredSettings {
        let background = self
            .new_thread_background
            .as_ref()
            .map(|background| background.adjustment.normalized());
        let (focal_x, focal_y, zoom) = match background {
            Some(adjustment) => (
                f64::from(adjustment.focal_x),
                f64::from(adjustment.focal_y),
                f64::from(adjustment.zoom),
            ),
            None => (
                f64::from(FOCAL_DEFAULT),
                f64::from(FOCAL_DEFAULT),
                f64::from(ZOOM_DEFAULT),
            ),
        };
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
            theme_light: self.theme_light.clone(),
            theme_dark: self.theme_dark.clone(),
            ui_font_family: font_family_word(&self.ui_font_family),
            terminal_font_family: font_family_word(&self.terminal_font_family),
            code_font_family: font_family_word(&self.code_font_family),
            terminal_font_size: self.terminal_font_size,
            code_font_size: self.code_font_size,
            transcript_width: self.transcript_width,
            render_math: self.render_math,
            math_font: self.math_font.key().to_string(),
            math_size: self.math_size,
            smooth_scrolling: self.smooth_scrolling,
            reduce_motion: reduce_motion_word(self.reduce_motion).to_string(),
            pause_in_background: self.pause_in_background,
            wallpaper_theme_colors: self.wallpaper_theme_colors,
            wallpaper_color: self
                .wallpaper_color
                .map(wallpaper_color_to_str)
                .unwrap_or_default(),
            wallpaper_folder: path_word(self.wallpaper_folder.as_deref()),
            wallpaper_source: path_word(self.wallpaper_source.as_deref()),
            new_thread_background_path: self
                .new_thread_background
                .as_ref()
                .map(|background| path_word(Some(&background.path)))
                .unwrap_or_default(),
            new_thread_background_name: self
                .new_thread_background
                .as_ref()
                .map(|background| background.name.clone())
                .unwrap_or_default(),
            new_thread_background_focal_x: focal_x,
            new_thread_background_focal_y: focal_y,
            new_thread_background_zoom: zoom,
            new_thread_background_effect: self.new_thread_background_effect.word().to_string(),
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
            terminal_font_size: f64::NAN,
            code_font_size: f64::INFINITY,
            transcript_width: f64::NAN,
            math_size: 999,
            new_thread_background_focal_x: f64::NAN,
            new_thread_background_focal_y: 5.0,
            new_thread_background_zoom: f64::NAN,
            ..StoredSettings::default()
        };
        let settings = AppSettings::from_stored(stored);
        assert_eq!(settings.font_size, 14.0);
        assert_eq!(settings.window_opacity, WINDOW_OPACITY_DEFAULT);
        assert_eq!(settings.sidebar_tone, SIDEBAR_TONE_DEFAULT);
        assert_eq!(settings.sidebar_width, SIDEBAR_WIDTH_DEFAULT);
        assert_eq!(settings.terminal_font_size, TERMINAL_FONT_SIZE_DEFAULT);
        assert_eq!(settings.code_font_size, CODE_FONT_SIZE_DEFAULT);
        assert_eq!(settings.transcript_width, TRANSCRIPT_WIDTH_DEFAULT);
        assert_eq!(settings.math_size, MATH_SIZE_MAX);
        // No background path means no artwork, so the bad framing never lands.
        assert_eq!(settings.new_thread_background, None);
    }

    /// The font-family words are the persisted whitelist, including the
    /// device-local `installed:` form; anything else is `None` so the caller
    /// keeps the key's default family.
    #[test]
    fn font_family_words_round_trip() {
        for family in [
            UiFontFamily::MomoTrustSans,
            UiFontFamily::HankenGrotesk,
            UiFontFamily::GoogleSansCode,
            UiFontFamily::XanhMono,
            UiFontFamily::System,
            UiFontFamily::Installed("Inter".into()),
            UiFontFamily::Installed("My Font".into()),
        ] {
            let word = font_family_word(&family);
            assert_eq!(font_family_from_word(&word), Some(family), "{word}");
        }
        assert_eq!(
            font_family_word(&UiFontFamily::Installed("Inter".into())),
            "installed:Inter"
        );
        for junk in ["", "momo", "installed:", "Momo Trust Sans", "installed"] {
            assert_eq!(font_family_from_word(junk), None, "{junk}");
        }
    }

    /// The reduced-motion words round-trip and unknown text is rejected.
    #[test]
    fn reduce_motion_words_round_trip() {
        for preference in [ReduceMotion::System, ReduceMotion::On, ReduceMotion::Off] {
            assert_eq!(
                reduce_motion_from_word(reduce_motion_word(preference)),
                Some(preference)
            );
        }
        assert_eq!(reduce_motion_from_word(""), None);
        assert_eq!(reduce_motion_from_word("sometimes"), None);
    }

    /// Every math face has a stable persisted key, a label, a style word and a
    /// description; unknown keys fall back to the caller's default.
    #[test]
    fn math_fonts_have_keys_labels_and_fall_back() {
        for font in MathFont::ALL {
            assert_eq!(MathFont::parse(font.key()), Some(font));
            assert!(!font.label().is_empty());
            assert!(!font.style().is_empty());
            assert!(!font.description().is_empty());
        }
        assert_eq!(
            MathFont::ALL.map(MathFont::key),
            [
                "fira",
                "stixTwo",
                "computerModern",
                "libertinus",
                "pagella",
                "euler",
            ]
        );
        assert_eq!(MathFont::parse("bogus"), None);
        assert_eq!(MathFont::parse(""), None);
        assert_eq!(MathFont::default(), MathFont::Fira);
    }

    /// Every background effect has a stable word, label and description; the
    /// unknown word is rejected so the caller keeps "none".
    #[test]
    fn background_effects_words_labels_and_fall_back() {
        for effect in NewThreadBackgroundEffect::ALL {
            assert_eq!(
                NewThreadBackgroundEffect::parse(effect.word()),
                Some(effect)
            );
            assert!(!effect.label().is_empty());
            assert!(!effect.description().is_empty());
        }
        assert_eq!(
            NewThreadBackgroundEffect::ALL.map(NewThreadBackgroundEffect::word),
            ["none", "dither", "ascii", "halftone", "scanlines"]
        );
        assert_eq!(NewThreadBackgroundEffect::parse("bogus"), None);
        assert_eq!(
            NewThreadBackgroundEffect::default(),
            NewThreadBackgroundEffect::None
        );
    }

    /// The wallpaper color codec accepts zeron's `Color` formats and writes the
    /// row's uppercase `#RRGGBB` word; junk is "no sample".
    #[test]
    fn wallpaper_color_words_round_trip() {
        assert_eq!(
            wallpaper_color_from_str("#aabbcc"),
            Some(Color::rgb(0xAA, 0xBB, 0xCC))
        );
        assert_eq!(
            wallpaper_color_to_str(Color::rgb(0xAA, 0xBB, 0xCC)),
            "#AABBCC"
        );
        assert_eq!(wallpaper_color_to_str(Color::rgb(0, 0x2e, 0x6f)), "#002E6F");
        for junk in ["", "not-a-color", "#12", "0x123456"] {
            assert_eq!(wallpaper_color_from_str(junk), None, "{junk}");
        }
    }

    /// The transcript width clamps to zeron's range and snaps to the 16 px
    /// grid; non-finite input first falls back to the default, which is on the
    /// grid.
    #[test]
    fn transcript_width_snaps_to_the_grid() {
        assert_eq!(normalize_transcript_width(0.0), TRANSCRIPT_WIDTH_MIN);
        assert_eq!(normalize_transcript_width(10_000.0), TRANSCRIPT_WIDTH_MAX);
        assert_eq!(normalize_transcript_width(736.0), 736.0);
        // The grid is 560 + n*16; 750 sits between 736 (n=11) and 752 (n=12)
        // and rounds to the nearer step.
        assert_eq!(normalize_transcript_width(750.0), 752.0);
        assert_eq!(normalize_transcript_width(743.0), 736.0);
        assert_eq!(
            normalize_transcript_width(f64::NAN),
            TRANSCRIPT_WIDTH_DEFAULT
        );
        assert_eq!(
            normalize_transcript_width(f64::INFINITY),
            TRANSCRIPT_WIDTH_DEFAULT
        );
    }

    /// The background framing clamps each field and replaces non-finite input
    /// with the default so the renderer never sees a NaN.
    #[test]
    fn background_adjustment_normalizes() {
        let normalized = NewThreadBackgroundAdjustment {
            focal_x: -1.0,
            focal_y: 2.0,
            zoom: 100.0,
        }
        .normalized();
        assert_eq!(normalized.focal_x, 0.0);
        assert_eq!(normalized.focal_y, 1.0);
        assert_eq!(normalized.zoom, NewThreadBackgroundAdjustment::MAX_ZOOM);

        let non_finite = NewThreadBackgroundAdjustment {
            focal_x: f32::NAN,
            focal_y: f32::INFINITY,
            zoom: f32::NEG_INFINITY,
        }
        .normalized();
        assert_eq!(non_finite, NewThreadBackgroundAdjustment::default());
        // Below MIN_ZOOM zooms in to no less than 1.0.
        assert_eq!(
            NewThreadBackgroundAdjustment {
                zoom: 0.5,
                ..Default::default()
            }
            .normalized()
            .zoom,
            NewThreadBackgroundAdjustment::MIN_ZOOM
        );
    }

    /// The five flat background keys rebuild the typed background and write
    /// back unchanged; a missing path means no artwork.
    #[test]
    fn new_thread_background_round_trips_through_the_store() {
        let stored = StoredSettings {
            new_thread_background_path: "/tmp/app data/bg image.png".into(),
            new_thread_background_name: "bg image.png".into(),
            new_thread_background_focal_x: 0.25,
            new_thread_background_focal_y: 0.75,
            new_thread_background_zoom: 2.0,
            ..StoredSettings::default()
        };
        let settings = AppSettings::from_stored(stored.clone());
        let background = settings
            .new_thread_background
            .as_ref()
            .expect("path means artwork");
        assert_eq!(background.path, PathBuf::from("/tmp/app data/bg image.png"));
        assert_eq!(background.name, "bg image.png");
        assert_eq!(settings.to_stored(), stored);

        // Empty path → no artwork, framing written back as the defaults.
        let none = AppSettings::from_stored(StoredSettings::default());
        assert_eq!(none.new_thread_background, None);
        assert_eq!(none.to_stored(), StoredSettings::default());

        // An empty name falls back to the file name so the row has a label.
        let unnamed = StoredSettings {
            new_thread_background_path: "/tmp/app data/bg image.png".into(),
            ..StoredSettings::default()
        };
        let settings = AppSettings::from_stored(unnamed);
        assert_eq!(settings.new_thread_background.unwrap().name, "bg image.png");
    }
}
