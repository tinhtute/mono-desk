//! The Appearance page's miniature theme previews: the mode cards' window
//! mock and the palette strip a theme-select row leads with.
//!
//! Ported from zeron's `crates/ui/src/settings/appearance.rs` (MIT, rev
//! 8b7afb3): `bar` / `miniature` / `miniature_split` / `preview` at
//! :1526-1532 and :1582-1683, `palette_preview` at :1692-1705.
//!
//! Adaptation: zeron's previews call `Theme::for_selection` with the live
//! `ThemeSelection` from its appearance service; mono-desk passes the persisted
//! `theme_light` / `theme_dark` ids through [`ThemeSelection`], so a preview
//! names the exact variant the install will use (the theme library is loaded
//! into the active registry in this phase set). No wallpaper tint is applied —
//! previews show the theme's own colors, like upstream's mode cards.

use gpui::prelude::*;
use gpui::{AnyElement, Hsla, div, px, relative};

use zeron_theme::{AccentSelection, SurfacePreference, ThemeSelection};
use zeron_ui::settings::widgets;
use zeron_ui::theme::{Appearance, Theme};

use crate::app_settings::Appearance as ModeChoice;

/// One text-bar line of a miniature (upstream :1526-1532).
fn bar(fraction: f32, tone: Hsla) -> gpui::Div {
    div()
        .h(px(5.0))
        .w(relative(fraction))
        .rounded(px(3.0))
        .bg(tone)
}

/// Which corners a miniature rounds: all four for a full card, the outer edge
/// only for one half of the system split (upstream :1582-1587).
#[derive(Clone, Copy, PartialEq, Eq)]
enum Corners {
    All,
    Left,
    Right,
}

/// zeron's miniature window mock: a sidebar column of text bars beside a
/// bordered content pane, painted from a resolved theme (upstream :1589-1636).
fn miniature(theme: &Theme, corners: Corners) -> AnyElement {
    let line = theme.text.opacity(0.22);
    let strong = theme.text.opacity(0.34);
    let r = px(widgets::OPTION_CARD_RADIUS);
    let root = div().size_full().flex().flex_row().bg(theme.surface);
    let root = match corners {
        Corners::All => root.rounded(r),
        Corners::Left => root.rounded_tl(r).rounded_bl(r),
        Corners::Right => root.rounded_tr(r).rounded_br(r),
    };
    root.child(
        div()
            .w(px(44.0))
            .h_full()
            .flex_none()
            .overflow_hidden()
            .flex()
            .flex_col()
            .gap(px(7.0))
            .px(px(8.0))
            .pt(px(14.0))
            .child(bar(0.70, strong))
            .child(bar(1.0, line))
            .child(bar(0.85, line))
            .child(bar(1.0, line)),
    )
    .child(
        div()
            .flex_1()
            .min_w_0()
            .my(px(8.0))
            .mr(px(8.0))
            .rounded(px(6.0))
            .border_1()
            .border_color(theme.border)
            .bg(theme.bg)
            .overflow_hidden()
            .flex()
            .flex_col()
            .gap(px(7.0))
            .p(px(10.0))
            .child(bar(0.62, strong))
            .child(bar(0.88, line))
            .child(bar(0.76, line))
            .child(bar(0.52, line)),
    )
    .into_any_element()
}

/// The split Light | Dark preview for the "System" card (upstream
/// :1638-1664), from the two persisted variant ids.
fn miniature_split(
    themes: &ThemeSelection,
    accent: AccentSelection,
    surface: SurfacePreference,
) -> AnyElement {
    let light = Theme::for_selection(
        Appearance::Light,
        themes.variant_id(zeron_theme::Appearance::Light),
        accent,
        surface,
    );
    let dark = Theme::for_selection(
        Appearance::Dark,
        themes.variant_id(zeron_theme::Appearance::Dark),
        accent,
        surface,
    );
    div()
        .size_full()
        .flex()
        .flex_row()
        .child(
            div()
                .w_1_2()
                .h_full()
                .overflow_hidden()
                .child(miniature(&light, Corners::Left)),
        )
        .child(
            div()
                .w_1_2()
                .h_full()
                .overflow_hidden()
                .child(miniature(&dark, Corners::Right)),
        )
        .into_any_element()
}

/// The preview for one mode card (upstream :1666-1683).
pub(super) fn preview(
    mode: ModeChoice,
    themes: &ThemeSelection,
    accent: AccentSelection,
    surface: SurfacePreference,
) -> AnyElement {
    match mode {
        ModeChoice::System => miniature_split(themes, accent, surface),
        ModeChoice::Light => miniature(
            &Theme::for_selection(
                Appearance::Light,
                themes.variant_id(zeron_theme::Appearance::Light),
                accent,
                surface,
            ),
            Corners::All,
        ),
        ModeChoice::Dark => miniature(
            &Theme::for_selection(
                Appearance::Dark,
                themes.variant_id(zeron_theme::Appearance::Dark),
                accent,
                surface,
            ),
            Corners::All,
        ),
    }
}

/// The three-band palette strip a theme-select option leads with (upstream
/// :1692-1705): shell surface, content plane, accent.
pub(super) fn palette_preview(theme: &Theme) -> gpui::Div {
    div()
        .flex_none()
        .w(px(30.0))
        .h(px(18.0))
        .rounded(px(5.0))
        .overflow_hidden()
        .border_1()
        .border_color(theme.border)
        .flex()
        .child(div().w_1_3().h_full().bg(theme.surface))
        .child(div().w_1_3().h_full().bg(theme.bg))
        .child(div().w_1_3().h_full().bg(theme.accent))
}
