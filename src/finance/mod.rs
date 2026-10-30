//! The finance showcase the "Finance UI" settings page hosts, adapted from
//! the pre-port app's `src/cadence/` modules (preserved on
//! `wip/gpui-kit-shell`), which were built on the pre-port UI kit.
//!
//! ## What this is
//!
//! The reference (`x.com/yurygok/status/2105191658337841531`; the four renders
//! are the tweet's images) is a finance dashboard art board: a light-gray
//! canvas carrying eight white cards in two columns — purchase orders, a filter
//! panel, upcoming bills, aging, expense claims, a category waffle, a billable
//! expenses empty state and an assign-to-project panel.
//!
//! The port keeps that canvas, its own sample data and its interaction state;
//! only the substrate changed. Cards, charts and the reference's small paint
//! primitives are plain gpui divs (they were already layout/paint on the old
//! stack), painted from zeron's `Theme` tokens; the controls that are real
//! controls — tab strips, toggles and text actions — come from zeron's
//! vendored settings kit (`zeron_ui::settings::widgets`). zeron ships no
//! checkbox, slider or segmented control, so:
//!
//! - checkboxes are [`blocks::switch`] (zeron's `toggle_switch`, with the
//!   caller owning activation and accessibility, as the kit documents);
//! - the two tab-like navigators are [`controls::tabs`] / [`controls::segmented`]
//!   built from zeron's `section_tab` + `tab_selection_t` pair;
//! - the reference's dual-handle range slider is static paint here because it
//!   was never wired to interaction on the old stack either.
//!
//! ## Color
//!
//! The old module carried a local indigo accent family sampled from the
//! reference. That family is now resolved from the installed theme's accent
//! ramp ([`Accent`]) so the showcase follows dark mode and the user's accent
//! choice; the mix fractions reproduce the reference's five tint steps from
//! the card surface. Neutral surfaces map onto zeron's tokens: the canvas is
//! `theme.surface`, cards are `theme.surface_overlay`, recessed wells are
//! `theme.band`, hairlines are `theme.border`, ink is `theme.text` /
//! `theme.text_muted`.
//!
//! ## Typography
//!
//! The reference sets every numeral — amounts, counts, badges, axis labels —
//! in a monospace. [`blocks::mono`] is that one-liner, using the theme's
//! `font_mono` family. Copy is the reference's own English sample data, kept
//! verbatim (same policy as the mock's numbers); page chrome (title,
//! subtitle) is `tr()`d by the host page.
//!
//! ## Inventory
//!
//! [`blocks`]: card/header/icon tile/mono/pill/filter chip/stat/meter/legend/
//! switch/avatar/empty state/buttons.
//! [`charts`]: mini bars, stacked bar chart, waffle, aging bars, range slider,
//! gantt row.
//! [`controls`]: tab strip and segmented strip.
//! [`showcase`]: the eight cards assembled on the canvas, mounted as the
//! "Finance UI" settings page; interactions write to [`DemoState`].

use std::rc::Rc;

use gpui::{AnyElement, App, Div, Global, Hsla, Styled as _, Window, div};

use zeron_ui::theme::{self, Theme};

mod blocks;
mod charts;
mod controls;
mod showcase;

/// Callback shape shared by the paint primitives that take one (the filter
/// chip's remove action); the alias keeps builder fields under clippy's
/// type-complexity limit.
pub(crate) type WindowCallback = Rc<dyn Fn(&mut Window, &mut App)>;

/// A row flex container — the old stack's `h_flex` one-liner, recreated on
/// plain gpui so every card keeps its original call shape.
pub(crate) fn h_flex() -> Div {
    div().flex().flex_row()
}

/// A column flex container — the old stack's `v_flex`.
pub(crate) fn v_flex() -> Div {
    div().flex().flex_col()
}

/// The reference's accent family, resolved per installed theme.
///
/// The old module pinned the sampled indigo hexes for both modes; the port
/// resolves them from the theme accent instead, mixing toward the card surface
/// with the same fractions the sampled ramp implies (~0.42, ~0.76, ~0.88,
/// ~0.94), so light/dark and the user's accent choice both flow through. `ink`
/// is the accent itself (it clears WCAG AA on the card and both tints in both
/// appearances — `accent_ink_clears_wcag_aa`); `on_solid` is the ink drawn on
/// the solid fill and flips in dark mode the way the reference's dark pastel
/// needs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Accent;

impl Accent {
    /// Solid fills: bars, underline, switch fill, meter progress.
    pub fn solid(theme: &Theme) -> Hsla {
        theme.accent
    }

    /// Mid step of the accent ramp (waffle cells).
    pub fn mid(theme: &Theme) -> Hsla {
        mix(theme, 0.42)
    }

    /// Faint step: unscheduled bars, light waffle cells, meter remainder.
    pub fn faint(theme: &Theme) -> Hsla {
        mix(theme, 0.76)
    }

    /// Tint background: pills, the chart tooltip.
    pub fn soft(theme: &Theme) -> Hsla {
        mix(theme, 0.88)
    }

    /// Lighter wash: selected rows, highlighted sparkline ranges.
    pub fn wash(theme: &Theme) -> Hsla {
        mix(theme, 0.94)
    }

    /// Accent text on card surfaces and tints.
    pub fn ink(theme: &Theme) -> Hsla {
        theme.accent
    }

    /// Ink drawn ON the solid accent (the "Today" pill, the switch's on-state
    /// rim). White in light mode; in dark mode the solid is a light pastel, so
    /// the ink flips dark to hold the contrast.
    pub fn on_solid(theme: &Theme) -> Hsla {
        if theme.appearance.is_dark() {
            theme.bg
        } else {
            theme.on_accent
        }
    }
}

fn mix(theme: &Theme, toward_surface: f32) -> Hsla {
    theme::mix(theme.accent, theme.surface_card, toward_surface)
}

/// Interaction/demo state for the showcase cards, stored as a gpui global so
/// the card renderers (which only receive `&App` / `&mut App` from their click
/// handlers) can read and write it. The default mirrors the reference render's
/// initial state: date chip on, overdue switch on, both doc types on, two
/// purchase orders picked, Atlas assigned, all expenses picked.
#[derive(Clone, Debug, Default)]
pub(crate) struct DemoState {
    /// Purchase orders: active tab index.
    pub po_tab: usize,
    /// Purchase orders: per-row selection (Northwind, Haldon, Lumen).
    pub po_rows: [bool; 3],
    /// Filter panel: the date-range chip is present (its × removes it).
    pub date_filter: bool,
    /// Filter panel: "Overdue only" switch.
    pub overdue_only: bool,
    /// Filter panel: Document Types switches.
    pub doc_bills: bool,
    pub doc_credit_notes: bool,
    /// Filter panel: the static dual-handle range, 0..1 track positions (the
    /// reference render's handles; never interactive on the old stack either).
    pub slider: (f32, f32),
    /// Expense claims: active segment index.
    pub claims_tab: usize,
    /// Assign panel: selected project row (None = nothing picked).
    pub assigned: Option<usize>,
    /// Assign panel: per-expense switches.
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

    /// The installed demo state.
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

/// The showcase canvas, mounted in the Finance UI page's `page_column`.
pub(crate) fn page(lang: &str) -> AnyElement {
    showcase::page(lang)
}

/// Install the demo state. Called once from `main.rs` before the window opens.
pub(crate) fn init(cx: &mut App) {
    cx.set_global(DemoState::demo());
}

#[cfg(test)]
mod tests {
    use super::Accent;
    use zeron_ui::theme::{self, Theme};

    /// Accent ink must clear WCAG AA (4.5:1) on every surface the showcase
    /// draws it over: the card, the canvas, and both tint backgrounds, in both
    /// appearances. The ramp is theme-derived now, so the test resolves it from
    /// the built-in palettes instead of pinning hexes.
    #[test]
    fn accent_ink_clears_wcag_aa() {
        for theme in [Theme::light(), Theme::dark()] {
            let ink = Accent::ink(&theme);
            for (name, surface) in [
                ("card", theme.surface_overlay),
                ("canvas", theme.surface),
                ("soft", Accent::soft(&theme)),
                ("wash", Accent::wash(&theme)),
            ] {
                let ratio = theme::contrast_ratio(ink, surface);
                assert!(
                    ratio >= 4.5,
                    "{:?}: ink on {name} is {ratio:.2}:1, need 4.5:1",
                    theme.appearance
                );
            }
        }
    }

    /// The "Today" pill and the switch's on-state draw `on_solid` ink over the
    /// solid accent; that pair must clear AA in both modes.
    #[test]
    fn on_solid_ink_clears_aa_on_solid_accent() {
        for theme in [Theme::light(), Theme::dark()] {
            let ratio = theme::contrast_ratio(Accent::on_solid(&theme), Accent::solid(&theme));
            assert!(
                ratio >= 4.5,
                "{:?}: on_solid ink on solid accent is {ratio:.2}:1, need 4.5:1",
                theme.appearance
            );
        }
    }

    /// The demo's initial state is the reference render's: two rows picked, the
    /// date chip present, the overdue switch on, Atlas assigned.
    #[test]
    fn demo_state_matches_the_reference_render() {
        let state = super::DemoState::demo();
        assert_eq!(state.po_rows, [true, true, false]);
        assert!(state.date_filter);
        assert!(state.overdue_only);
        assert!(state.doc_bills && state.doc_credit_notes);
        assert_eq!(state.slider, (0.25, 0.78));
        assert_eq!(state.claims_tab, 2);
        assert_eq!(state.assigned, Some(0));
        assert_eq!(state.expenses, [true, true, true]);
    }
}
