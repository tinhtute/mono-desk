//! Cadence — components cloned from the finance-UI reference
//! (`x.com/yurygok/status/2105191658337841531`; the four renders are the
//! tweet's images, mirrored under `~/quaqua/runs/ref-cadence/`).
//!
//! ## Style analysis (what the reference is made of)
//!
//! Surfaces — a light-gray canvas; white cards (`rounded` ~16px, hairline
//! border, faint shadow); recessed "wells" inside cards (chart areas, filter
//! sections, the empty-state illustration); a light-blue wash for selection
//! and highlighted ranges. Mapped onto the mono theme: canvas =
//! `theme.background`, card = `theme.popover`, well = `theme.muted`,
//! hairline = `theme.border`, wash = [`Accent::wash`].
//!
//! Ink — near-black text, mid-gray secondary text, and one vivid indigo
//! accent used for every active/selected/positive state. The mono theme's
//! primary (`#124b68`) is a different hue, so the accent family is local to
//! this module (same policy as `status_chip`): [`Accent`] with a WCAG test.
//!
//! Typography — the reference's signature trait: a grotesk for UI copy and a
//! **monospace for every numeral** — amounts, counts, badges, axis labels,
//! even "1 col = 1 week". Here that is the theme's `mono_font_family`
//! (Google Sans Code); [`blocks::mono`] is the one-liner every numeric text
//! goes through.
//!
//! Controls — black primary buttons with white labels (light mode; inverted
//! in dark mode via foreground/popover), white bordered secondary buttons,
//! blue-filled checkboxes and switch, underline tabs and a pill segmented
//! control, both carrying mono counts.
//!
//! Data marks — micro-charts drawn from plain divs: histograms, stacked
//! bars over dashed gridlines, a waffle grid, horizontal aging bars, a
//! dual-handle range slider, and gantt-style row bars. GPUI cannot dash a
//! border, so dashed strokes (gridlines, pending gantt segments, empty
//! waffle cells) render as hairline solids at reduced alpha.
//!
//! ## Component inventory
//!
//! [`blocks`]: [`blocks::card`], [`blocks::CardHeader`],
//!   [`blocks::icon_tile`], [`blocks::mono`], [`blocks::MonoPill`],
//!   [`blocks::FilterChip`], [`blocks::Stat`], [`blocks::Meter`],
//!   [`blocks::legend_dot`].
//! [`controls`]: [`controls::Tabs`], [`controls::Segmented`].
//! [`charts`]: [`charts::MiniBars`], [`charts::BarChartMini`],
//!   [`charts::Waffle`], [`charts::AgingBars`], [`charts::RangeSlider`],
//!   [`charts::Gantt`].
//! [`showcase`]: the eight reference cards assembled on one canvas, mounted
//!   as the "Finance UI" settings page; interactions write to [`DemoState`].
//!
//! Showcase copy is the reference's own English sample data, kept verbatim
//! for fidelity (same policy as the mock's numbers); page chrome is `tr()`d.

pub mod blocks;
pub mod charts;
pub mod controls;
pub mod showcase;

use std::rc::Rc;

use gpui_kit::component::{ActiveTheme, ThemeMode};
use gpui_kit::{App, Global, Hsla, Window, rgb};

/// Callback shapes shared by the interactive cadence components; the
/// aliases keep the builder fields under clippy's type-complexity limit.
pub type WindowCallback = Rc<dyn Fn(&mut Window, &mut App)>;
pub type BoolCallback = Rc<dyn Fn(&bool, &mut Window, &mut App)>;
pub type PickCallback = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// The reference's indigo accent family, resolved per theme mode.
///
/// Light values are sampled from the reference; dark values are its pastel
/// counterparts (desaturated fills, lightened ink). `ink` clears WCAG AA on
/// both the card surface and the tint backgrounds it appears on —
/// [`tests::accent_ink_clears_wcag_aa`] holds every pair to 4.5:1.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Accent;

impl Accent {
    /// Solid fills: bars, underline, checkbox fill, meter progress.
    pub fn solid(cx: &App) -> Hsla {
        set(mode(cx)).solid
    }

    /// Mid step of the accent ramp (waffle cells).
    pub fn mid(cx: &App) -> Hsla {
        set(mode(cx)).mid
    }

    /// Faint step: unscheduled bars, light waffle cells, meter remainder.
    pub fn faint(cx: &App) -> Hsla {
        set(mode(cx)).faint
    }

    /// Tint background: pills, the chart tooltip.
    pub fn soft(cx: &App) -> Hsla {
        set(mode(cx)).soft
    }

    /// Lighter wash: selected rows, highlighted sparkline ranges.
    pub fn wash(cx: &App) -> Hsla {
        set(mode(cx)).wash
    }

    /// Accent text on card surfaces and tints.
    pub fn ink(cx: &App) -> Hsla {
        set(mode(cx)).ink
    }

    /// Ink drawn ON the solid accent (the "Today" pill, the checkbox
    /// check): white in light mode; in dark mode the solid is a light
    /// pastel, so the ink flips dark to hold WCAG AA.
    pub fn on_solid(cx: &App) -> Hsla {
        set(mode(cx)).on_solid
    }
}

fn mode(cx: &App) -> ThemeMode {
    cx.theme().mode
}

/// One mode's accent ramp as opaque `0xRRGGBB` hex — the single source of
/// truth the renderer resolves and the WCAG test re-checks.
struct AccentHex {
    solid: u32,
    mid: u32,
    faint: u32,
    soft: u32,
    wash: u32,
    ink: u32,
    on_solid: u32,
}

const LIGHT: AccentHex = AccentHex {
    solid: 0x2b4bf2,
    mid: 0x7c8cf8,
    faint: 0xc9d2fc,
    soft: 0xe4e8fe,
    wash: 0xeef1fe,
    ink: 0x2b4bf2,
    on_solid: 0xffffff,
};

const DARK: AccentHex = AccentHex {
    solid: 0x6e86ff,
    mid: 0x4c5e9e,
    faint: 0x2e3b66,
    soft: 0x26325c,
    wash: 0x1e2745,
    ink: 0xa9b8ff,
    on_solid: 0x1a2244,
};

/// One mode's resolved accent ramp.
#[derive(Clone, Copy)]
struct AccentSet {
    solid: Hsla,
    mid: Hsla,
    faint: Hsla,
    soft: Hsla,
    wash: Hsla,
    ink: Hsla,
    on_solid: Hsla,
}

fn set(mode: ThemeMode) -> AccentSet {
    let h = if mode.is_dark() { DARK } else { LIGHT };
    AccentSet {
        solid: rgb(h.solid).into(),
        mid: rgb(h.mid).into(),
        faint: rgb(h.faint).into(),
        soft: rgb(h.soft).into(),
        wash: rgb(h.wash).into(),
        ink: rgb(h.ink).into(),
        on_solid: rgb(h.on_solid).into(),
    }
}

/// Interactive state of the showcase page, stored as a GPUI global so the
/// `SettingItem::render` closures (which only receive `&App` / `&mut App`)
/// can read and write it. The default mirrors the reference render's
/// initial state: date chip on, toggle on, both doc types checked, two
/// purchase orders picked, Atlas assigned, all expenses checked.
#[derive(Default)]
pub struct DemoState {
    /// Purchase orders: active tab index.
    pub po_tab: usize,
    /// Purchase orders: per-row selection (Northwind, Haldon, Lumen).
    pub po_rows: [bool; 3],
    /// Filter panel: the date-range chip is present (its × removes it).
    pub date_filter: bool,
    /// Filter panel: "Overdue only" switch.
    pub overdue_only: bool,
    /// Filter panel: Document Types checkboxes.
    pub doc_bills: bool,
    pub doc_credit_notes: bool,
    /// Filter panel: dual-handle range slider, 0..1 track positions.
    pub slider: (f32, f32),
    /// Expense claims: active segment index.
    pub claims_tab: usize,
    /// Assign panel: selected project row (None = nothing picked).
    pub assigned: Option<usize>,
    /// Assign panel: per-expense checkboxes.
    pub expenses: [bool; 3],
}

impl Global for DemoState {}

impl DemoState {
    /// The reference render's initial state.
    pub fn demo() -> Self {
        Self {
            po_tab: 0,
            po_rows: [true, true, false],
            date_filter: true,
            overdue_only: true,
            doc_bills: true,
            doc_credit_notes: true,
            slider: (0.25, 0.78),
            claims_tab: 2,
            assigned: Some(0),
            expenses: [true, true, true],
        }
    }
}

impl DemoState {
    pub fn global(cx: &App) -> &Self {
        cx.global::<Self>()
    }

    /// Mutate through `f` and repaint — the one-liner every showcase
    /// interaction handler reduces to.
    pub fn update(cx: &mut App, f: impl FnOnce(&mut Self)) {
        f(cx.global_mut::<Self>());
        cx.refresh_windows();
    }
}

#[cfg(test)]
mod tests {
    use super::{DARK, LIGHT};
    use gpui_kit::component::ThemeMode;

    /// WCAG 2.1 relative luminance of an opaque sRGB color.
    fn relative_luminance(hex: u32) -> f32 {
        let channel = |shift: u32| {
            let c = ((hex >> shift) & 0xff) as f32 / 255.;
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * channel(16) + 0.7152 * channel(8) + 0.0722 * channel(0)
    }

    /// WCAG 2.1 contrast ratio between two opaque sRGB colors.
    fn contrast(a: u32, b: u32) -> f32 {
        let (la, lb) = (relative_luminance(a), relative_luminance(b));
        let (hi, lo) = if la > lb { (la, lb) } else { (lb, la) };
        (hi + 0.05) / (lo + 0.05)
    }

    /// Accent ink must clear WCAG AA (4.5:1) on every surface it is drawn
    /// over: the card (popover), the canvas (background), and both tint
    /// backgrounds (`soft`, `wash`), in both modes.
    ///
    /// The theme surfaces are pinned to the published mono.json values here:
    /// the test needs concrete hex, and a drift in those tokens must fail
    /// loudly rather than silently drop the accent below AA.
    #[test]
    fn accent_ink_clears_wcag_aa() {
        const POPOVER: [u32; 2] = [0xffffff, 0x212121];
        const BACKGROUND: [u32; 2] = [0xf7f7f7, 0x171717];
        for (mode, ix) in [(ThemeMode::Light, 0), (ThemeMode::Dark, 1)] {
            let set = if mode.is_dark() { DARK } else { LIGHT };
            for (name, surface) in [
                ("popover", POPOVER[ix]),
                ("background", BACKGROUND[ix]),
                ("soft", set.soft),
                ("wash", set.wash),
            ] {
                let ratio = contrast(set.ink, surface);
                assert!(
                    ratio >= 4.5,
                    "{mode:?}: ink on {name} is {ratio:.2}:1, need 4.5:1"
                );
            }
        }
    }

    /// The "Today" pill and the checkbox draw `on_solid` ink over the
    /// solid accent; that pair must clear AA in both modes.
    #[test]
    fn on_solid_ink_clears_aa_on_solid_accent() {
        for set in [LIGHT, DARK] {
            let ratio = contrast(set.on_solid, set.solid);
            assert!(
                ratio >= 4.5,
                "on_solid ink on solid accent is {ratio:.2}:1, need 4.5:1"
            );
        }
    }
}
