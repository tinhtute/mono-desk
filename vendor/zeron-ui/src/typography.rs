//! Device-local interface typography: bundled font registration, the catalog
//! choice, and the effective family installed into [`crate::theme::Theme`].
//!
//! Adapted from zeron's `crates/ui/src/typography.rs` (MIT). The parts that
//! depend on zeron's persisted settings store (`settings::update`) and its
//! `appearance` module are dropped: mono-desk bundles its own four families and
//! persists typography through `AppSettings`. The device-local font catalog is
//! kept — `fontdb`/`ttf-parser` inspect the installed faces so the Appearance
//! font pickers can list system families and classify fixed width. The shape of
//! the state global, the call sites `theme.rs` and `main.rs` use, and the
//! rem-based scaling rule are kept so a later upstream re-sync stays mechanical.
//!
//! Families registered here (all vendored under `assets/fonts/`):
//! - `Momo Trust Sans` — the interface sans (400/500/600/700);
//! - `Hanken Grotesk` — the pre-Momo interface sans, kept registered so a
//!   persisted selection still resolves;
//! - `Google Sans Code` — code/telemetry mono (400/500);
//! - `Xanh Mono` — the wordmark/display mono (400/italic, semibold italic).

use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
};

use gpui::{App, Global, Rems, SharedString, Window, px, rems};


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

/// A bundled, virtual, or device-local interface font choice.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum UiFontFamily {
    #[default]
    MomoTrustSans,
    HankenGrotesk,
    GoogleSansCode,
    XanhMono,
    System,
    /// A family supplied by the OS or the user, listed only when its installed
    /// faces passed the Latin-metrics check in
    /// `installed_families_with_latin_metrics`.
    Installed(String),
}

impl UiFontFamily {
    pub fn label(&self) -> &str {
        match self {
            Self::MomoTrustSans => "Momo Trust Sans",
            Self::HankenGrotesk => "Hanken Grotesk",
            Self::GoogleSansCode => "Google Sans Code",
            Self::XanhMono => "Xanh Mono",
            Self::System => "System UI",
            Self::Installed(name) => name,
        }
    }

    pub fn family_name(&self) -> &str {
        match self {
            Self::MomoTrustSans => "Momo Trust Sans",
            Self::HankenGrotesk => "Hanken Grotesk",
            Self::GoogleSansCode => "Google Sans Code",
            Self::XanhMono => "Xanh Mono",
            Self::System => ".SystemUIFont",
            Self::Installed(name) => name,
        }
    }
}

/// Which catalog families successfully registered during this process, and the
/// device-local families the Appearance font pickers may list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontAvailability {
    momo_trust_sans: bool,
    hanken_grotesk: bool,
    google_sans_code: bool,
    xanh_mono: bool,
    choices: Vec<UiFontFamily>,
    fixed_width: Vec<UiFontFamily>,
}

impl FontAvailability {
    pub fn choices(&self) -> &[UiFontFamily] {
        &self.choices
    }

    /// Families whose glyph advances match the terminal renderer's cell grid.
    pub fn fixed_width_choices(&self) -> &[UiFontFamily] {
        &self.fixed_width
    }

    pub fn is_available(&self, family: &UiFontFamily) -> bool {
        match family {
            UiFontFamily::MomoTrustSans => self.momo_trust_sans,
            UiFontFamily::HankenGrotesk => self.hanken_grotesk,
            UiFontFamily::GoogleSansCode => self.google_sans_code,
            UiFontFamily::XanhMono => self.xanh_mono,
            UiFontFamily::System => true,
            UiFontFamily::Installed(_) => self.choices.contains(family),
        }
    }

    pub fn is_fixed_width_available(&self, family: &UiFontFamily) -> bool {
        self.is_available(family) && self.fixed_width.contains(family)
    }

    fn fallback(&self) -> UiFontFamily {
        if self.momo_trust_sans {
            UiFontFamily::MomoTrustSans
        } else {
            UiFontFamily::System
        }
    }

    /// Code and terminal degrade to a bundled mono face instead of the
    /// interface sans: a device-local pick that later disappears should not
    /// silently turn fixed-width surfaces proportional.
    fn fallback_mono(&self) -> UiFontFamily {
        if self.google_sans_code {
            UiFontFamily::GoogleSansCode
        } else {
            self.fallback()
        }
    }
}

impl Default for FontAvailability {
    fn default() -> Self {
        Self {
            momo_trust_sans: false,
            hanken_grotesk: false,
            google_sans_code: false,
            xanh_mono: false,
            choices: vec![UiFontFamily::System],
            fixed_width: Vec::new(),
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

/// Family names of the bundled faces: the `Installed` section of the catalog is
/// captured before these register, but a system copy of a bundled family must
/// still not be offered twice.
const BUNDLED_FAMILY_NAMES: [&str; 4] = [
    "Momo Trust Sans",
    "Hanken Grotesk",
    "Google Sans Code",
    "Xanh Mono",
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

/// Relative spread the sampled advances may show and still count as fixed
/// width. Hinting and rounding leave sub-unit differences in real mono faces;
/// proportional faces differ by whole multiples, never a percent.
const FIXED_WIDTH_TOLERANCE: f32 = 0.01;

/// Whether a face satisfies the terminal renderer's assumption that every
/// Latin cell is one `m` wide.
///
/// The PANOSE bit and `fontdb`'s `monospaced` flag are metadata a font can set
/// wrongly; what the grid actually needs is equal advances, so measure them.
fn face_is_fixed_width(font: &ttf_parser::Face) -> bool {
    let Some(advances) = ['i', 'm', 'W', '0']
        .iter()
        .map(|ch| {
            font.glyph_index(*ch)
                .and_then(|id| font.glyph_hor_advance(id))
        })
        .collect::<Option<Vec<_>>>()
    else {
        return false;
    };
    let (min, max) = (
        advances.iter().copied().min().unwrap_or(0),
        advances.iter().copied().max().unwrap_or(0),
    );
    min > 0 && f32::from(max - min) <= FIXED_WIDTH_TOLERANCE * f32::from(min)
}

/// Families that contain the Latin glyph GPUI relies on for text metrics,
/// mapped to whether every one of their faces is fixed width.
///
/// `all_font_names` intentionally reports every system family, including
/// script-specific and symbol fonts. GPUI must reject a face without `m`, but
/// doing that only after it has been selected both falls back silently and
/// emits a warning. Inspect the installed files first so those families never
/// appear in an interface-font picker.
fn installed_families_with_latin_metrics() -> BTreeMap<String, bool> {
    let mut database = fontdb::Database::new();
    database.load_system_fonts();

    let mut families = BTreeMap::new();
    for face in database.faces() {
        let Some(Some(fixed_width)) = database.with_face_data(face.id, |data, index| {
            ttf_parser::Face::parse(data, index)
                .ok()
                .filter(|font| font.glyph_index('m').is_some())
                .map(|font| face_is_fixed_width(&font))
        }) else {
            continue;
        };
        for (name, _) in &face.families {
            // Selecting a family selects all its faces, so one proportional
            // italic is enough to break the grid.
            families
                .entry(name.clone())
                .and_modify(|all_fixed| *all_fixed &= fixed_width)
                .or_insert(fixed_width);
        }
    }
    families
}

/// Register each family independently so one bad optional asset cannot hide the
/// rest of the catalog.
pub fn register_fonts(cx: &App) -> FontAvailability {
    // Capture device fonts before adding ours so the Installed section only
    // contains families supplied by the OS/user, not our embedded assets.
    let families_with_latin_metrics = installed_families_with_latin_metrics();
    let system_names: BTreeSet<_> = cx
        .text_system()
        .all_font_names()
        .into_iter()
        .filter(|name| !name.starts_with('.'))
        .filter(|name| !BUNDLED_FAMILY_NAMES.contains(&name.as_str()))
        .filter(|name| families_with_latin_metrics.contains_key(name))
        .collect();

    let momo_trust_sans = register_family(cx, &UiFontFamily::MomoTrustSans, &MOMO_TRUST_SANS);
    let hanken_grotesk = register_family(cx, &UiFontFamily::HankenGrotesk, &HANKEN_GROTESK);
    let google_sans_code = register_family(cx, &UiFontFamily::GoogleSansCode, &GOOGLE_SANS_CODE);
    let xanh_mono = register_family(cx, &UiFontFamily::XanhMono, &XANH_MONO);

    let mut choices = Vec::new();
    for (family, registered) in [
        (UiFontFamily::MomoTrustSans, momo_trust_sans),
        (UiFontFamily::HankenGrotesk, hanken_grotesk),
        (UiFontFamily::GoogleSansCode, google_sans_code),
        (UiFontFamily::XanhMono, xanh_mono),
    ] {
        if registered {
            choices.push(family);
        }
    }
    choices.push(UiFontFamily::System);

    // Bundled mono faces stay the first fixed-width entries; the installed
    // families follow, sorted by the `BTreeSet` above.
    let mut fixed_width = Vec::new();
    for (family, registered) in [
        (UiFontFamily::GoogleSansCode, google_sans_code),
        (UiFontFamily::XanhMono, xanh_mono),
    ] {
        if registered {
            fixed_width.push(family);
        }
    }
    for name in system_names {
        if families_with_latin_metrics.get(&name) == Some(&true) {
            fixed_width.push(UiFontFamily::Installed(name.clone()));
        }
        choices.push(UiFontFamily::Installed(name));
    }

    FontAvailability {
        momo_trust_sans,
        hanken_grotesk,
        google_sans_code,
        xanh_mono,
        choices,
        fixed_width,
    }
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
        repaint_after_typography_change(cx);
    }
    effective_changed
}

/// Repaint after a typography change; the **app** owns the theme reinstall.
///
/// Upstream rebuilt the theme here with `Theme::install_preferences`, which
/// mono-desk cannot do: that helper resolves only the built-in light/dark
/// variant, so every font change clobbered the selected `variant_id`,
/// `accent_selection` and `surface_preference` (the `appearance-parity`
/// `state-persistence` T2 clobber). The vendored setters therefore only update
/// `TypographyState` (and its generation) and repaint; the app re-installs the
/// full selection through `settings::apply_appearance`, whose ordering applies
/// typography first and the theme last so the installed `Theme`'s font fields
/// never go stale.
fn repaint_after_typography_change(cx: &mut App) {
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
    repaint_after_typography_change(cx);
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
    repaint_after_typography_change(cx);
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
    repaint_after_typography_change(cx);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test fixture: every bundled family registered, `System` available, and
    /// two device-local families — one proportional, one fixed width (mirrors
    /// upstream's `FontAvailability::all()`).
    fn availability() -> FontAvailability {
        FontAvailability {
            momo_trust_sans: true,
            hanken_grotesk: true,
            google_sans_code: true,
            xanh_mono: true,
            choices: vec![
                UiFontFamily::MomoTrustSans,
                UiFontFamily::HankenGrotesk,
                UiFontFamily::GoogleSansCode,
                UiFontFamily::XanhMono,
                UiFontFamily::System,
                UiFontFamily::Installed("Arial".into()),
                UiFontFamily::Installed("Menlo".into()),
            ],
            fixed_width: vec![
                UiFontFamily::GoogleSansCode,
                UiFontFamily::XanhMono,
                UiFontFamily::Installed("Menlo".into()),
            ],
        }
    }

    /// Test fixture mirroring upstream's `FontAvailability::without`: a family
    /// that failed to register (or is no longer installed) disappears from the
    /// availability lists.
    fn without(mut availability: FontAvailability, family: &UiFontFamily) -> FontAvailability {
        match family {
            UiFontFamily::MomoTrustSans => availability.momo_trust_sans = false,
            UiFontFamily::HankenGrotesk => availability.hanken_grotesk = false,
            UiFontFamily::GoogleSansCode => availability.google_sans_code = false,
            UiFontFamily::XanhMono => availability.xanh_mono = false,
            UiFontFamily::System => {}
            UiFontFamily::Installed(_) => {
                availability.choices.retain(|choice| choice != family);
            }
        }
        availability.fixed_width.retain(|choice| choice != family);
        availability
    }

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
    fn ui_font_sizes_have_stable_labels_and_normalize() {
        assert_eq!(UiFontSize::default().label().as_ref(), "16 px");
        assert_eq!(UiFontSize(250).normalized(), UiFontSize(20));
        assert_eq!(ui_rems(14.0).0, 0.875);
    }

    /// Bundled and installed names label the picker; `System` keeps GPUI's
    /// private UI family name as the family it installs.
    #[test]
    fn family_labels_cover_bundled_and_installed() {
        for (family, label, family_name) in [
            (UiFontFamily::MomoTrustSans, "Momo Trust Sans", "Momo Trust Sans"),
            (
                UiFontFamily::HankenGrotesk,
                "Hanken Grotesk",
                "Hanken Grotesk",
            ),
            (
                UiFontFamily::GoogleSansCode,
                "Google Sans Code",
                "Google Sans Code",
            ),
            (UiFontFamily::XanhMono, "Xanh Mono", "Xanh Mono"),
            (UiFontFamily::System, "System UI", ".SystemUIFont"),
            (
                UiFontFamily::Installed("Menlo".into()),
                "Menlo",
                "Menlo",
            ),
        ] {
            assert_eq!(family.label(), label, "{family:?}");
            assert_eq!(family.family_name(), family_name, "{family:?}");
        }
    }

    #[test]
    fn unavailable_family_resolves_to_the_interface_default() {
        let unavailable = UiFontFamily::Installed("Inter".into());
        let availability = without(availability(), &unavailable);
        assert_eq!(
            resolve_effective(&unavailable, &availability),
            UiFontFamily::MomoTrustSans
        );
        assert!(availability.is_available(&UiFontFamily::System));
    }

    #[test]
    fn unavailable_code_and_terminal_families_stay_monospaced() {
        let unavailable = UiFontFamily::Installed("MesloLGS NF".into());
        let availability = availability();
        assert_eq!(
            resolve_effective_mono(&unavailable, &availability, false),
            UiFontFamily::GoogleSansCode
        );
        assert_eq!(
            resolve_effective_mono(
                &unavailable,
                &without(availability.clone(), &unavailable),
                true
            ),
            UiFontFamily::GoogleSansCode
        );
        // Only when the bundled mono face itself failed to register does a
        // fixed-width surface fall back to the interface family.
        let without_mono = without(availability, &UiFontFamily::GoogleSansCode);
        assert_eq!(
            resolve_effective_mono(&unavailable, &without_mono, false),
            UiFontFamily::MomoTrustSans
        );
    }

    #[test]
    fn advance_check_separates_bundled_mono_from_bundled_sans() {
        for bytes in GOOGLE_SANS_CODE {
            assert!(face_is_fixed_width(
                &ttf_parser::Face::parse(bytes, 0).unwrap()
            ));
        }
        for bytes in XANH_MONO {
            assert!(face_is_fixed_width(
                &ttf_parser::Face::parse(bytes, 0).unwrap()
            ));
        }
        for bytes in MOMO_TRUST_SANS {
            assert!(!face_is_fixed_width(
                &ttf_parser::Face::parse(bytes, 0).unwrap()
            ));
        }
        for bytes in HANKEN_GROTESK {
            assert!(!face_is_fixed_width(
                &ttf_parser::Face::parse(bytes, 0).unwrap()
            ));
        }
    }

    #[test]
    fn terminal_catalog_drops_proportional_families() {
        let availability = availability();
        for proportional in [
            UiFontFamily::MomoTrustSans,
            UiFontFamily::HankenGrotesk,
            UiFontFamily::System,
            UiFontFamily::Installed("Arial".into()),
        ] {
            assert!(availability.is_available(&proportional));
            assert!(!availability.is_fixed_width_available(&proportional));
            assert!(!availability.fixed_width_choices().contains(&proportional));
        }
        for fixed in [
            UiFontFamily::GoogleSansCode,
            UiFontFamily::XanhMono,
            UiFontFamily::Installed("Menlo".into()),
        ] {
            assert!(availability.is_fixed_width_available(&fixed));
            assert!(availability.fixed_width_choices().contains(&fixed));
        }
    }

    #[test]
    fn persisted_proportional_terminal_family_falls_back() {
        let availability = availability();
        // Written by a build before the terminal picker was constrained.
        let persisted = UiFontFamily::Installed("Arial".into());
        assert_eq!(
            resolve_effective_mono(&persisted, &availability, true),
            UiFontFamily::GoogleSansCode
        );
        assert_eq!(
            resolve_effective_mono(&UiFontFamily::System, &availability, true),
            UiFontFamily::GoogleSansCode
        );
        // The same value is still legal for code and diffs.
        assert_eq!(
            resolve_effective_mono(&persisted, &availability, false),
            persisted
        );
    }

    #[test]
    fn catalog_keeps_bundled_and_virtual_choices_first() {
        let availability = availability();
        assert_eq!(
            availability.choices()[..5],
            [
                UiFontFamily::MomoTrustSans,
                UiFontFamily::HankenGrotesk,
                UiFontFamily::GoogleSansCode,
                UiFontFamily::XanhMono,
                UiFontFamily::System,
            ]
        );
        // Device-local families come after the bundled set and stay sorted.
        assert_eq!(
            availability.choices()[5..],
            [
                UiFontFamily::Installed("Arial".into()),
                UiFontFamily::Installed("Menlo".into()),
            ]
        );
    }

    /// Before `register_fonts` runs (or when no bundled face registered) the
    /// catalog offers only the always-available system family, and both
    /// fallbacks land there instead of on a missing bundled face.
    #[test]
    fn default_availability_only_offers_system() {
        let availability = FontAvailability::default();
        assert_eq!(availability.choices(), [UiFontFamily::System]);
        assert!(availability.fixed_width_choices().is_empty());
        assert!(!availability.is_available(&UiFontFamily::MomoTrustSans));
        assert!(!availability.is_available(&UiFontFamily::Installed("Menlo".into())));
        assert!(availability.is_available(&UiFontFamily::System));
        assert_eq!(availability.fallback(), UiFontFamily::System);
        assert_eq!(availability.fallback_mono(), UiFontFamily::System);
    }

    #[test]
    fn latin_metric_filter_requires_the_glyph_gpui_measures() {
        let latin = ttf_parser::Face::parse(MOMO_TRUST_SANS[0], 0).unwrap();
        assert!(latin.glyph_index('m').is_some());
    }
}
