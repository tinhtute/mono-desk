//! Device-local interface typography: bundled font registration, the catalog
//! choice, and the effective family installed into [`crate::theme::Theme`].
//!
//! Adapted from zeron's `crates/ui/src/typography.rs` (MIT). The parts that
//! depend on zeron's persisted settings store (`settings::update`), its
//! `appearance` module and font inspection via `fontdb`/`ttf-parser` are
//! dropped: mono-desk bundles its own four families, persists typography
//! through `AppSettings`, and only offers the bundled faces plus the system UI
//! font. The shape of the state global, the call sites `theme.rs` and
//! `main.rs` use, and the rem-based scaling rule are kept so a later upstream
//! re-sync stays mechanical.
//!
//! Families registered here (all vendored under `assets/fonts/`):
//! - `Momo Trust Sans` — the interface sans (400/500/600/700);
//! - `Hanken Grotesk` — the pre-Momo interface sans, kept registered so a
//!   persisted selection still resolves;
//! - `Google Sans Code` — code/telemetry mono (400/500);
//! - `Xanh Mono` — the wordmark/display mono (400/italic, semibold italic).

use std::borrow::Cow;

use gpui::{App, Global, Rems, SharedString, Window, px, rems};

use crate::theme::{AccentColor, Appearance, Theme};

/// Base size used by rem-based interface text. Persisted as a number so future
/// builds can add choices without changing the settings format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct UiFontSize(u8);

impl UiFontSize {
    pub const ALL: [Self; 7] = [
        Self(12),
        Self(13),
        Self(14),
        Self(15),
        Self(16),
        Self(18),
        Self(20),
    ];

    pub const fn pixels(self) -> f32 {
        self.0 as f32
    }

    pub fn label(self) -> SharedString {
        format!("{} px", self.0).into()
    }

    pub fn normalized(self) -> Self {
        Self::ALL
            .into_iter()
            .min_by_key(|candidate| candidate.0.abs_diff(self.0))
            .unwrap_or_default()
    }
}

impl Default for UiFontSize {
    fn default() -> Self {
        Self(16)
    }
}

/// Convert a size designed at the 16px baseline into a scalable interface rem.
/// Code, diffs and terminal text deliberately keep absolute pixel sizes.
pub const fn ui_rems(pixels_at_default: f32) -> Rems {
    rems(pixels_at_default / 16.0)
}

pub const CODE_FONT_SIZE_DEFAULT: f32 = 12.5;
pub const TERMINAL_FONT_SIZE_DEFAULT: f32 = 13.0;
pub const FONT_SIZE_MIN: f32 = 8.0;
pub const FONT_SIZE_MAX: f32 = 32.0;

/// Clamp an absolute code/terminal pixel size into the supported range.
pub fn clamp_font_size(size: f32) -> f32 {
    size.clamp(FONT_SIZE_MIN, FONT_SIZE_MAX)
}

/// A bundled interface font choice. `System` is always available and is the
/// fallback when a bundled family fails to register.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum UiFontFamily {
    #[default]
    MomoTrustSans,
    HankenGrotesk,
    GoogleSansCode,
    XanhMono,
    System,
}

impl UiFontFamily {
    pub const ALL: [Self; 5] = [
        Self::MomoTrustSans,
        Self::HankenGrotesk,
        Self::GoogleSansCode,
        Self::XanhMono,
        Self::System,
    ];

    pub fn label(&self) -> &str {
        match self {
            Self::MomoTrustSans => "Momo Trust Sans",
            Self::HankenGrotesk => "Hanken Grotesk",
            Self::GoogleSansCode => "Google Sans Code",
            Self::XanhMono => "Xanh Mono",
            Self::System => "System UI",
        }
    }

    pub fn family_name(&self) -> &str {
        match self {
            Self::MomoTrustSans => "Momo Trust Sans",
            Self::HankenGrotesk => "Hanken Grotesk",
            Self::GoogleSansCode => "Google Sans Code",
            Self::XanhMono => "Xanh Mono",
            Self::System => ".SystemUIFont",
        }
    }

    /// Whether the family's glyphs share one advance width, which the terminal
    /// grid assumes. Bundled mono faces only; `System` is never grid-safe.
    fn is_fixed_width(&self) -> bool {
        matches!(self, Self::GoogleSansCode | Self::XanhMono)
    }
}

/// Which catalog families successfully registered during this process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontAvailability {
    registered: Vec<UiFontFamily>,
}

impl FontAvailability {
    pub fn choices(&self) -> &[UiFontFamily] {
        &self.registered
    }

    /// Families whose glyph advances match the terminal renderer's cell grid.
    pub fn fixed_width_choices(&self) -> impl Iterator<Item = &UiFontFamily> {
        self.registered.iter().filter(|family| family.is_fixed_width())
    }

    pub fn is_available(&self, family: &UiFontFamily) -> bool {
        matches!(family, UiFontFamily::System) || self.registered.contains(family)
    }

    pub fn is_fixed_width_available(&self, family: &UiFontFamily) -> bool {
        family.is_fixed_width() && self.is_available(family)
    }

    fn fallback(&self) -> UiFontFamily {
        if self.is_available(&UiFontFamily::MomoTrustSans) {
            UiFontFamily::MomoTrustSans
        } else {
            UiFontFamily::System
        }
    }

    /// Code and terminal degrade to a bundled mono face instead of the
    /// interface sans: a missing face should not silently turn fixed-width
    /// surfaces proportional.
    fn fallback_mono(&self) -> UiFontFamily {
        if self.is_available(&UiFontFamily::GoogleSansCode) {
            UiFontFamily::GoogleSansCode
        } else {
            self.fallback()
        }
    }
}

impl Default for FontAvailability {
    fn default() -> Self {
        Self {
            registered: vec![UiFontFamily::System],
        }
    }
}

/// Requested and effective typography for the process.
///
/// Interface and code/diff pick from the whole catalog — they lay text out
/// naturally, so proportional faces are legal. The terminal paints into a cell
/// grid sized by the `m` advance, so it picks from the fixed-width subset only.
pub struct TypographyState {
    pub requested: UiFontFamily,
    pub effective: UiFontFamily,
    pub size: UiFontSize,
    pub terminal_requested: UiFontFamily,
    pub terminal_effective: UiFontFamily,
    pub terminal_font_size: f32,
    pub code_requested: UiFontFamily,
    pub code_effective: UiFontFamily,
    pub code_font_size: f32,
    pub availability: FontAvailability,
    /// Monotonic signal for layout caches whose measurements depend on
    /// typography. Kept separate from the theme style generation so palette
    /// changes do not force expensive list remeasurement.
    generation: u32,
}

impl Global for TypographyState {}

const MOMO_TRUST_SANS: [&[u8]; 4] = [
    include_bytes!("../../../assets/fonts/MomoTrustSans-Regular.ttf"),
    include_bytes!("../../../assets/fonts/MomoTrustSans-Medium.ttf"),
    include_bytes!("../../../assets/fonts/MomoTrustSans-SemiBold.ttf"),
    include_bytes!("../../../assets/fonts/MomoTrustSans-Bold.ttf"),
];

const HANKEN_GROTESK: [&[u8]; 4] = [
    include_bytes!("../../../assets/fonts/HankenGrotesk-Regular.ttf"),
    include_bytes!("../../../assets/fonts/HankenGrotesk-Medium.ttf"),
    include_bytes!("../../../assets/fonts/HankenGrotesk-SemiBold.ttf"),
    include_bytes!("../../../assets/fonts/HankenGrotesk-Bold.ttf"),
];

const GOOGLE_SANS_CODE: [&[u8]; 2] = [
    include_bytes!("../../../assets/fonts/GoogleSansCode-Regular.ttf"),
    include_bytes!("../../../assets/fonts/GoogleSansCode-Medium.ttf"),
];

const XANH_MONO: [&[u8]; 3] = [
    include_bytes!("../../../assets/fonts/XanhMono-Regular.ttf"),
    include_bytes!("../../../assets/fonts/XanhMono-Italic.ttf"),
    include_bytes!("../../../assets/fonts/XanhMono-SemiBoldItalic.ttf"),
];

/// Every bundled face, for renderers that install the family outside gpui's
/// text system (SVG text-to-path, offscreen capture).
pub(crate) fn bundled_font_faces() -> impl Iterator<Item = &'static [u8]> {
    MOMO_TRUST_SANS
        .iter()
        .chain(HANKEN_GROTESK.iter())
        .chain(GOOGLE_SANS_CODE.iter())
        .chain(XANH_MONO.iter())
        .copied()
}

fn register_family(cx: &App, family: &UiFontFamily, faces: &'static [&'static [u8]]) -> bool {
    let fonts = faces.iter().map(|face| Cow::Borrowed(*face)).collect();
    match cx.text_system().add_fonts(fonts) {
        Ok(()) => true,
        Err(err) => {
            tracing::warn!(font_family = family.label(), error = %err, "failed to register bundled font family");
            false
        }
    }
}

/// Register each family independently so one bad optional asset cannot hide the
/// rest of the catalog.
pub fn register_fonts(cx: &App) -> FontAvailability {
    let mut registered = Vec::new();
    for (family, faces) in [
        (UiFontFamily::MomoTrustSans, &MOMO_TRUST_SANS[..]),
        (UiFontFamily::HankenGrotesk, &HANKEN_GROTESK[..]),
        (UiFontFamily::GoogleSansCode, &GOOGLE_SANS_CODE[..]),
        (UiFontFamily::XanhMono, &XANH_MONO[..]),
    ] {
        if register_family(cx, &family, faces) {
            registered.push(family);
        }
    }
    registered.push(UiFontFamily::System);
    FontAvailability { registered }
}

fn resolve_effective(requested: &UiFontFamily, availability: &FontAvailability) -> UiFontFamily {
    if availability.is_available(requested) {
        requested.clone()
    } else {
        availability.fallback()
    }
}

/// `fixed_width_only` guards the terminal grid: a settings file may already
/// name a proportional family, which must fall back rather than render cells
/// the painted glyphs do not fill.
fn resolve_effective_mono(
    requested: &UiFontFamily,
    availability: &FontAvailability,
    fixed_width_only: bool,
) -> UiFontFamily {
    let usable = if fixed_width_only {
        availability.is_fixed_width_available(requested)
    } else {
        availability.is_available(requested)
    };
    if usable {
        requested.clone()
    } else {
        availability.fallback_mono()
    }
}

/// Install typography state before appearance builds the first [`Theme`].
#[allow(clippy::too_many_arguments)]
pub fn init(
    requested: UiFontFamily,
    size: UiFontSize,
    terminal_requested: UiFontFamily,
    terminal_font_size: f32,
    code_requested: UiFontFamily,
    code_font_size: f32,
    availability: FontAvailability,
    cx: &mut App,
) {
    let effective = resolve_effective(&requested, &availability);
    let terminal_effective = resolve_effective_mono(&terminal_requested, &availability, true);
    let code_effective = resolve_effective_mono(&code_requested, &availability, false);
    cx.set_global(TypographyState {
        requested,
        effective,
        size: size.normalized(),
        terminal_requested,
        terminal_effective,
        terminal_font_size: clamp_font_size(terminal_font_size),
        code_requested,
        code_effective,
        code_font_size: clamp_font_size(code_font_size),
        availability,
        generation: 0,
    });
}

pub fn requested(cx: &App) -> UiFontFamily {
    cx.try_global::<TypographyState>()
        .map(|state| state.requested.clone())
        .unwrap_or_default()
}

pub fn effective(cx: &App) -> UiFontFamily {
    cx.try_global::<TypographyState>()
        .map(|state| state.effective.clone())
        .unwrap_or_default()
}

pub fn effective_family_name(cx: &App) -> SharedString {
    effective(cx).family_name().into()
}

pub fn terminal_requested(cx: &App) -> UiFontFamily {
    cx.try_global::<TypographyState>()
        .map(|state| state.terminal_requested.clone())
        .unwrap_or(UiFontFamily::GoogleSansCode)
}

pub fn terminal_effective(cx: &App) -> UiFontFamily {
    cx.try_global::<TypographyState>()
        .map(|state| state.terminal_effective.clone())
        .unwrap_or(UiFontFamily::GoogleSansCode)
}

pub fn terminal_effective_family_name(cx: &App) -> SharedString {
    terminal_effective(cx).family_name().into()
}

pub fn terminal_font_size(cx: &App) -> f32 {
    cx.try_global::<TypographyState>()
        .map(|state| state.terminal_font_size)
        .unwrap_or(TERMINAL_FONT_SIZE_DEFAULT)
}

pub fn code_requested(cx: &App) -> UiFontFamily {
    cx.try_global::<TypographyState>()
        .map(|state| state.code_requested.clone())
        .unwrap_or(UiFontFamily::GoogleSansCode)
}

pub fn code_effective(cx: &App) -> UiFontFamily {
    cx.try_global::<TypographyState>()
        .map(|state| state.code_effective.clone())
        .unwrap_or(UiFontFamily::GoogleSansCode)
}

pub fn code_effective_family_name(cx: &App) -> SharedString {
    code_effective(cx).family_name().into()
}

pub fn code_font_size(cx: &App) -> f32 {
    cx.try_global::<TypographyState>()
        .map(|state| state.code_font_size)
        .unwrap_or(CODE_FONT_SIZE_DEFAULT)
}

pub fn font_size(cx: &App) -> UiFontSize {
    cx.try_global::<TypographyState>()
        .map(|state| state.size)
        .unwrap_or_default()
}

pub fn availability(cx: &App) -> FontAvailability {
    cx.try_global::<TypographyState>()
        .map(|state| state.availability.clone())
        .unwrap_or_default()
}

/// Monotonic id of the current effective UI typography (family + size).
/// Long-lived layout caches compare this value to invalidate measurements that
/// can change when prose wraps differently.
pub fn generation(cx: &App) -> u32 {
    cx.try_global::<TypographyState>()
        .map(|state| state.generation)
        .unwrap_or_default()
}

/// Validate and apply one confirmed interface family. Returns whether the
/// effective family changed (re-selecting the current family is a no-op).
///
/// Persistence is the caller's: the settings page in `settings-surface` writes
/// the choice through `AppSettings` after this returns.
pub fn set_family(family: UiFontFamily, cx: &mut App) -> bool {
    let Some(state) = cx.try_global::<TypographyState>() else {
        return false;
    };
    if !state.availability.is_available(&family) {
        return false;
    }
    let effective = resolve_effective(&family, &state.availability);
    if state.requested == family && state.effective == effective {
        return false;
    }

    let effective_changed = state.effective != effective;
    let state = cx.global_mut::<TypographyState>();
    state.requested = family;
    state.effective = effective;

    if effective_changed {
        state.generation = state.generation.wrapping_add(1);
        reinstall_theme(cx);
    }
    effective_changed
}

/// Rebuild the [`Theme`] so it picks up the new families and sizes it carries,
/// then repaint.
fn reinstall_theme(cx: &mut App) {
    let (appearance, accent) = cx
        .try_global::<Theme>()
        .map(|theme| (theme.appearance, theme.accent_color))
        .unwrap_or((Appearance::Dark, AccentColor::default()));
    Theme::install_preferences(appearance, accent, cx);
    cx.refresh_windows();
}

/// Apply and persist the terminal family. Returns whether anything changed.
pub fn set_terminal_family(family: UiFontFamily, cx: &mut App) -> bool {
    set_category_family(family, Category::Terminal, cx)
}

/// Apply and persist the code/diff family. Returns whether anything changed.
pub fn set_code_family(family: UiFontFamily, cx: &mut App) -> bool {
    set_category_family(family, Category::Code, cx)
}

#[derive(Clone, Copy)]
enum Category {
    Terminal,
    Code,
}

impl Category {
    fn fixed_width_only(self) -> bool {
        matches!(self, Self::Terminal)
    }
}

fn set_category_family(family: UiFontFamily, category: Category, cx: &mut App) -> bool {
    let Some(state) = cx.try_global::<TypographyState>() else {
        return false;
    };
    let fixed_width_only = category.fixed_width_only();
    let selectable = if fixed_width_only {
        state.availability.is_fixed_width_available(&family)
    } else {
        state.availability.is_available(&family)
    };
    if !selectable {
        return false;
    }
    let effective = resolve_effective_mono(&family, &state.availability, fixed_width_only);
    let (current_requested, current_effective) = match category {
        Category::Terminal => (&state.terminal_requested, &state.terminal_effective),
        Category::Code => (&state.code_requested, &state.code_effective),
    };
    if *current_requested == family && *current_effective == effective {
        return false;
    }

    let state = cx.global_mut::<TypographyState>();
    state.generation = state.generation.wrapping_add(1);
    match category {
        Category::Terminal => {
            state.terminal_requested = family;
            state.terminal_effective = effective;
        }
        Category::Code => {
            state.code_requested = family;
            state.code_effective = effective;
        }
    }
    reinstall_theme(cx);
    true
}

/// Apply the terminal pixel size. Returns whether it changed.
pub fn set_terminal_font_size(size: f32, cx: &mut App) -> bool {
    set_category_font_size(size, Category::Terminal, cx)
}

/// Apply the code/diff pixel size. Returns whether it changed.
pub fn set_code_font_size(size: f32, cx: &mut App) -> bool {
    set_category_font_size(size, Category::Code, cx)
}

fn set_category_font_size(size: f32, category: Category, cx: &mut App) -> bool {
    let size = clamp_font_size(size);
    let Some(state) = cx.try_global::<TypographyState>() else {
        return false;
    };
    let current = match category {
        Category::Terminal => state.terminal_font_size,
        Category::Code => state.code_font_size,
    };
    if current == size {
        return false;
    }

    let state = cx.global_mut::<TypographyState>();
    state.generation = state.generation.wrapping_add(1);
    match category {
        Category::Terminal => state.terminal_font_size = size,
        Category::Code => state.code_font_size = size,
    }
    reinstall_theme(cx);
    true
}

/// Apply a supported UI size to the current window. Returns whether it changed.
pub fn set_font_size(size: UiFontSize, window: &mut Window, cx: &mut App) -> bool {
    let size = size.normalized();
    let Some(state) = cx.try_global::<TypographyState>() else {
        return false;
    };
    if state.size == size {
        return false;
    }

    let state = cx.global_mut::<TypographyState>();
    state.size = size;
    state.generation = state.generation.wrapping_add(1);
    window.set_rem_size(px(size.pixels()));
    reinstall_theme(cx);
    true
}

#[cfg(test)]
mod tests {
    use super::{UiFontFamily, UiFontSize};

    #[test]
    fn sizes_snap_to_the_catalog() {
        assert_eq!(UiFontSize(11).normalized(), UiFontSize(12));
        // A tie (17 is one away from both 16 and 18) snaps down, matching the
        // upstream `min_by_key` order.
        assert_eq!(UiFontSize(17).normalized(), UiFontSize(16));
        assert_eq!(UiFontSize(19).normalized(), UiFontSize(18));
        assert_eq!(UiFontSize(14).normalized(), UiFontSize(14));
    }

    #[test]
    fn mono_families_are_the_fixed_width_ones() {
        assert!(UiFontFamily::GoogleSansCode.is_fixed_width());
        assert!(UiFontFamily::XanhMono.is_fixed_width());
        assert!(!UiFontFamily::MomoTrustSans.is_fixed_width());
        assert!(!UiFontFamily::System.is_fixed_width());
    }
}
