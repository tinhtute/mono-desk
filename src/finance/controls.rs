//! Finance-showcase controls — the reference's two tab-like navigators
//! (underline tabs with mono counts and the pill segmented control), rebuilt
//! on zeron's widgets. zeron ships no segmented control, so both are a row of
//! `section_tab`s with zeron's `tab_selection_t` selected-state fade; the
//! segmented variant sits in a `block_fill` well, the reference's gray pill
//! container.

use std::rc::Rc;

use gpui::prelude::*;
use gpui::{App, Div, Role, SharedString, Stateful, Window, px};

use zeron_ui::motion;
use zeron_ui::settings::widgets;
use zeron_ui::theme::Theme;

use super::Accent;
use super::blocks::mono;
use super::h_flex;

/// One tab: label + optional mono count.
#[derive(Clone)]
pub(crate) struct Tab {
    pub label: SharedString,
    pub count: Option<u64>,
}

impl Tab {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            count: None,
        }
    }

    pub fn count(mut self, count: u64) -> Self {
        self.count = Some(count);
        self
    }
}

/// One segment: label + optional mono count.
#[derive(Clone)]
pub(crate) struct Segment {
    pub label: SharedString,
    pub count: Option<u64>,
}

impl Segment {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            count: None,
        }
    }

    pub fn count(mut self, count: u64) -> Self {
        self.count = Some(count);
        self
    }
}

type PickCallback = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// The per-item data [`tab_item`] renders, bundled so the helper stays under
/// clippy's argument-count limit.
struct TabSpec<'a> {
    key_base: &'a str,
    ix: usize,
    selected: bool,
    label: SharedString,
    count: Option<u64>,
}

/// One zeron `section_tab` with the reference's mono count, id, tab role and
/// roving tab stop.
fn tab_item(
    theme: &Theme,
    window: &mut Window,
    cx: &mut App,
    spec: TabSpec<'_>,
    on_pick: PickCallback,
) -> Stateful<Div> {
    let TabSpec {
        key_base,
        ix,
        selected,
        label,
        count,
    } = spec;
    let selection_key: SharedString = format!("{key_base}-{ix}-selection").into();
    let selection_t =
        widgets::tab_selection_t(window, selection_key, selected, motion::reduced_motion(cx));
    let count_color = if selected {
        Accent::ink(theme)
    } else {
        theme.text_muted
    };
    let key: SharedString = format!("{key_base}-{ix}").into();
    let hover_key: SharedString = format!("{key_base}-{ix}-hover").into();
    widgets::section_tab(theme, selected, selection_t, key, hover_key)
        .role(Role::Tab)
        .tab_index(0)
        .aria_label(label)
        .aria_selected(selected)
        .on_click(move |_, window, cx| on_pick(ix, window, cx))
        .children(count.map(|count| {
            mono(count.to_string(), theme)
                .text_xs()
                .text_color(count_color)
        }))
}

/// Underline-tab replacement: a horizontal strip of zeron tabs, the active one
/// carrying the kit's selected wash and the accent count.
pub(crate) fn tabs(
    theme: &Theme,
    window: &mut Window,
    cx: &mut App,
    id: &str,
    tabs: Vec<Tab>,
    active: usize,
    on_pick: impl Fn(usize, &mut Window, &mut App) + 'static,
) -> Div {
    let active = active.min(tabs.len().saturating_sub(1));
    let on_pick: PickCallback = Rc::new(on_pick);
    let mut row = h_flex()
        .gap(px(2.))
        .px_4()
        .py_1()
        // The card is narrower than the reference's, but the labels and mono
        // counts scale with the Font Size setting; wrapping keeps every tab
        // reachable instead of clipping the tail off the row.
        .flex_wrap()
        .border_b_1()
        .border_color(theme.border);
    for (ix, tab) in tabs.into_iter().enumerate() {
        row = row.child(tab_item(
            theme,
            window,
            cx,
            TabSpec {
                key_base: id,
                ix,
                selected: ix == active,
                label: tab.label,
                count: tab.count,
            },
            on_pick.clone(),
        ));
    }
    row
}

/// Pill-segmented replacement: zeron tabs in the reference's gray pill well.
pub(crate) fn segmented(
    theme: &Theme,
    window: &mut Window,
    cx: &mut App,
    id: &str,
    items: Vec<Segment>,
    active: usize,
    on_pick: impl Fn(usize, &mut Window, &mut App) + 'static,
) -> Div {
    let active = active.min(items.len().saturating_sub(1));
    let on_pick: PickCallback = Rc::new(on_pick);
    let mut row = h_flex()
        .gap(px(2.))
        .p(px(3.))
        .rounded(px(10.))
        .flex_wrap()
        .bg(widgets::block_fill(theme));
    for (ix, item) in items.into_iter().enumerate() {
        row = row.child(tab_item(
            theme,
            window,
            cx,
            TabSpec {
                key_base: id,
                ix,
                selected: ix == active,
                label: item.label,
                count: item.count,
            },
            on_pick.clone(),
        ));
    }
    row
}
