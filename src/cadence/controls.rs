//! Cadence controls — the reference's two tab-like navigators: underline
//! tabs with mono counts ("All 24 · Draft 3 · …") and the pill segmented
//! control ("Your own · To review 2 · To pay 3").
//!
//! Both are W3C-tablist navigators: the container holds focus, ArrowLeft /
//! ArrowRight move the active item with wraparound, Home / End jump, and
//! Space / Enter re-confirm. The active item's marker slides to it under a
//! `spring_move` spring; `Tabs` falls back to a per-tab baseline when the
//! row wraps, where a slider couldn't travel across lines.

use std::cell::RefCell;
use std::rc::Rc;

use gpui_kit::base::spring;
use gpui_kit::component::{
    ActiveTheme, ElementExt as _, StyledExt as _, ThemeStyled as _, h_flex, v_flex,
};
use gpui_kit::{
    App, Bounds, ElementId, InteractiveElement as _, IntoElement, KeyDownEvent, ParentElement as _,
    Pixels, RenderOnce, SharedString, StatefulInteractiveElement as _, Styled as _, Window, div,
    prelude::FluentBuilder as _, px,
};

/// Per-item bounds plus the container's, captured during prepaint so the
/// sliding indicator can be positioned next frame. Writes go through a
/// `RefCell` so measuring never schedules a repaint on its own — the same
/// trick `vendor/gpui-component/src/tab/tab_bar.rs` uses.
struct NavBounds {
    container: Bounds<Pixels>,
    items: Vec<Bounds<Pixels>>,
}

impl NavBounds {
    fn new(len: usize) -> Self {
        Self {
            container: Bounds::default(),
            items: vec![Bounds::default(); len],
        }
    }

    /// Indicator geometry for item `ix`, or `None` while measurement is
    /// pending (first frame) or while the items wrap across rows — a slider
    /// can't travel between lines. `pad_left` is the container's left
    /// padding: `on_prepaint` bounds are recorded from the padding box while
    /// absolute insets resolve against the border box, so the padding has to
    /// be added back.
    fn item(&self, ix: usize, pad_left: Pixels) -> Option<(Pixels, Pixels)> {
        let item = *self.items.get(ix)?;
        if item.size.width <= px(0.) || self.container.size.width <= px(0.) {
            return None;
        }
        let first_y = self.items.first()?.origin.y;
        if self.items.iter().any(|i| i.origin.y != first_y) {
            return None;
        }
        Some((
            item.origin.x - self.container.origin.x + pad_left,
            item.size.width,
        ))
    }
}

/// Fetch the shared `NavBounds` for a navigator, resizing when the item
/// count changes.
fn nav_bounds(
    window: &mut Window,
    cx: &mut App,
    key: String,
    len: usize,
) -> Rc<RefCell<NavBounds>> {
    let bounds = window
        .use_keyed_state(key, cx, |_, _| Rc::new(RefCell::new(NavBounds::new(len))))
        .read(cx)
        .clone();
    bounds.borrow_mut().items.resize(len, Bounds::default());
    bounds
}

/// One-frame warm-up so `on_prepaint` gets a chance to record bounds before
/// the indicator renders — without it the slider would sit at 0 for a frame.
/// Mirrors `tab_bar.rs`'s `init` keyed state.
fn nav_warmup(window: &mut Window, cx: &mut App, key: String) {
    let initialized = window.use_keyed_state(key, cx, |_, _| false);
    if !*initialized.read(cx) {
        initialized.update(cx, |v, _| *v = true);
    }
}

/// Arrow-key nav shared by `Tabs` and `Segmented`: Left / Right step the
/// active item with wraparound, Home / End jump to the ends, Space / Enter
/// re-confirm it.
fn nav_key_down(
    event: &KeyDownEvent,
    len: usize,
    active: usize,
    on_pick: &super::PickCallback,
    window: &mut Window,
    cx: &mut App,
) {
    if len == 0 || event.keystroke.modifiers.modified() {
        return;
    }
    let next = match event.keystroke.key.as_str() {
        "left" => Some((active + len - 1) % len),
        "right" => Some((active + 1) % len),
        "home" => Some(0),
        "end" => Some(len - 1),
        "space" | "enter" => Some(active),
        _ => None,
    };
    if let Some(ix) = next {
        window.prevent_default();
        on_pick(ix, window, cx);
    }
}

/// One tab: label + optional mono count.
#[derive(Clone)]
pub struct Tab {
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

/// Underline tabs: muted labels with mono counts, the active one in full
/// ink with an accent count. A 2px accent bar slides under the active tab
/// while the row stays on one line; wrapped rows mark the active tab with
/// a per-tab baseline that overlaps the hairline (no double line).
#[derive(IntoElement)]
pub struct Tabs {
    id: SharedString,
    tabs: Vec<Tab>,
    active: usize,
    on_pick: Option<super::PickCallback>,
}

impl Tabs {
    pub fn new(id: impl Into<SharedString>, tabs: Vec<Tab>) -> Self {
        Self {
            id: id.into(),
            tabs,
            active: 0,
            on_pick: None,
        }
    }

    pub fn active(mut self, active: usize) -> Self {
        self.active = active;
        self
    }

    pub fn on_pick(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_pick = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for Tabs {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let base = self.id.clone();
        let len = self.tabs.len();
        let active = self.active.min(len.saturating_sub(1));

        let focus_handle = window
            .use_keyed_state(format!("{base}-focus"), cx, |_, cx| cx.focus_handle())
            .read(cx)
            .clone();
        let is_focused = focus_handle.is_focused(window);
        let bounds = nav_bounds(window, cx, format!("{base}-bounds"), len);
        nav_warmup(window, cx, format!("{base}-init"));

        let ink = cx.theme().foreground;
        let muted = cx.theme().muted_foreground;
        let accent = super::Accent::solid(cx);
        let accent_ink = super::Accent::ink(cx);
        let motion = cx.theme().motion_tokens().spring_move;

        // The container pads x by 16, so insets sit 16px right of the
        // recorded padding-box offsets (see `NavBounds::item`). The bar dips
        // 1px into the hairline, landing where `mb(-1)` puts the fallback.
        let indicator = bounds.borrow().item(active, px(16.)).map(|(left, width)| {
            let left = spring(
                (format!("{base}-indicator"), "left"),
                left,
                motion,
                window,
                cx,
            );
            let width = spring(
                (format!("{base}-indicator"), "width"),
                width,
                motion,
                window,
                cx,
            );
            div()
                .absolute()
                .left(left)
                .bottom(px(-1.))
                .h(px(2.))
                .w(width)
                .rounded_full()
                .bg(accent)
                .into_any_element()
        });
        let sliding = indicator.is_some();

        let mut row = h_flex()
            .id(base.clone())
            .track_focus(&focus_handle)
            .tab_index(0)
            .relative()
            .gap_5()
            .px_4()
            // The card is a fixed 500px, but the labels and mono counts scale
            // with the Font Size setting: at 32px the five tabs need ~750px.
            // Wrapping keeps every tab reachable instead of clipping the tail
            // off the row.
            .flex_wrap()
            .border_b_1()
            .border_color(cx.theme().border)
            .when_some(self.on_pick.clone(), |this, on_pick| {
                this.on_key_down(move |event, window, cx| {
                    nav_key_down(event, len, active, &on_pick, window, cx);
                })
            });
        {
            let bounds = bounds.clone();
            row = row.on_prepaint(move |b, _, _| bounds.borrow_mut().container = b);
        }
        row = row.children(indicator);

        for (ix, tab) in self.tabs.into_iter().enumerate() {
            let is_active = ix == active;
            let id: ElementId = SharedString::from(format!("{base}-{ix}")).into();
            let hover_name: SharedString = format!("{base}-{ix}-hover").into();
            let on_pick = self.on_pick.clone();
            let bounds = bounds.clone();
            row = row.child(
                v_flex()
                    .id(id)
                    .group(hover_name.clone())
                    .when_some(on_pick, |this, on_pick| {
                        this.cursor_pointer().on_click(move |_, window, cx| {
                            on_pick(ix, window, cx);
                        })
                    })
                    .on_prepaint(move |b, _, _| {
                        if let Some(slot) = bounds.borrow_mut().items.get_mut(ix) {
                            *slot = b;
                        }
                    })
                    .rounded(px(4.))
                    .when(is_focused && is_active, |this| {
                        this.focus_ring_style(window, cx)
                    })
                    .child(
                        h_flex()
                            .gap_1p5()
                            // Baseline keeps the smaller mono count on the
                            // label's text line instead of centered on it.
                            .items_baseline()
                            .child(
                                div()
                                    .text_base()
                                    .when(is_active, |this| this.font_medium())
                                    .text_color(if is_active { ink } else { muted })
                                    .when(!is_active, |this| {
                                        this.group_hover(hover_name.clone(), |s| {
                                            s.text_color(ink.alpha(0.85))
                                        })
                                    })
                                    .child(tab.label),
                            )
                            .children(tab.count.map(|count| {
                                super::blocks::mono(count.to_string(), cx)
                                    .text_sm()
                                    .text_color(if is_active { accent_ink } else { muted })
                            })),
                    )
                    .child(
                        // Per-tab baseline: the active mark when the slider is
                        // out (first frame, wrapped rows), a faint hint under
                        // hovered inactive tabs. `mb(-1)` pulls it onto the
                        // container's hairline, matching the slider.
                        div()
                            .h(px(2.))
                            .w_full()
                            .rounded_full()
                            .mt(px(4.))
                            .mb(px(-1.))
                            .when(is_active && !sliding, |this| this.bg(accent))
                            .when(!is_active, |this| {
                                this.group_hover(hover_name.clone(), |s| s.bg(ink.alpha(0.15)))
                            }),
                    ),
            );
        }
        row
    }
}

/// One segment: label + optional mono count.
#[derive(Clone)]
pub struct Segment {
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

/// Pill segmented control: a 36px gray capsule, the active segment raised
/// as a white pill with a shadow that slides to the clicked segment.
#[derive(IntoElement)]
pub struct Segmented {
    id: SharedString,
    items: Vec<Segment>,
    active: usize,
    on_pick: Option<super::PickCallback>,
}

impl Segmented {
    pub fn new(id: impl Into<SharedString>, items: Vec<Segment>) -> Self {
        Self {
            id: id.into(),
            items,
            active: 0,
            on_pick: None,
        }
    }

    pub fn active(mut self, active: usize) -> Self {
        self.active = active;
        self
    }

    pub fn on_pick(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_pick = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for Segmented {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let base = self.id.clone();
        let len = self.items.len();
        let active = self.active.min(len.saturating_sub(1));

        let focus_handle = window
            .use_keyed_state(format!("{base}-focus"), cx, |_, cx| cx.focus_handle())
            .read(cx)
            .clone();
        let is_focused = focus_handle.is_focused(window);
        let bounds = nav_bounds(window, cx, format!("{base}-bounds"), len);
        nav_warmup(window, cx, format!("{base}-init"));

        let motion = cx.theme().motion_tokens().spring_move;
        let surface = cx.theme().popover;
        let hover_bg = cx.theme().background.alpha(0.5);

        // The track pads by 3, so insets sit 3px right of the recorded
        // padding-box offsets (see `NavBounds::item`). The pill fills the
        // 30px content height between the 3px insets.
        let pill = bounds.borrow().item(active, px(3.)).map(|(left, width)| {
            let left = spring((format!("{base}-pill"), "left"), left, motion, window, cx);
            let width = spring((format!("{base}-pill"), "width"), width, motion, window, cx);
            div()
                .absolute()
                .left(left)
                .top(px(3.))
                .bottom(px(3.))
                .w(width)
                .rounded_full()
                .bg(surface)
                .shadow_xs()
                .into_any_element()
        });
        let sliding = pill.is_some();

        let mut track = h_flex()
            .id(base.clone())
            .track_focus(&focus_handle)
            .tab_index(0)
            .relative()
            .gap(px(2.))
            .p(px(3.))
            // 36px control height to match the filter chips in the same row.
            .h(px(36.))
            .rounded_full()
            .bg(cx.theme().accent)
            .when(is_focused, |this| this.focus_ring_style(window, cx))
            .when_some(self.on_pick.clone(), |this, on_pick| {
                this.on_key_down(move |event, window, cx| {
                    nav_key_down(event, len, active, &on_pick, window, cx);
                })
            });
        {
            let bounds = bounds.clone();
            track = track.on_prepaint(move |b, _, _| bounds.borrow_mut().container = b);
        }
        track = track.children(pill);

        for (ix, item) in self.items.into_iter().enumerate() {
            let is_active = ix == active;
            let id: ElementId = SharedString::from(format!("{base}-{ix}")).into();
            let on_pick = self.on_pick.clone();
            let bounds = bounds.clone();
            track = track.child(
                h_flex()
                    .id(id)
                    .when_some(on_pick, |this, on_pick| {
                        this.cursor_pointer().on_click(move |_, window, cx| {
                            on_pick(ix, window, cx);
                        })
                    })
                    .on_prepaint(move |b, _, _| {
                        if let Some(slot) = bounds.borrow_mut().items.get_mut(ix) {
                            *slot = b;
                        }
                    })
                    .gap_1p5()
                    .items_center()
                    .px_3()
                    // 30px inside the 3px-padded 36px track.
                    .h_full()
                    .rounded_full()
                    // Fallback surface while the slider is out (first frame).
                    .when(is_active && !sliding, |this| this.bg(surface).shadow_xs())
                    .when(!is_active, |this| this.hover(move |s| s.bg(hover_bg)))
                    .child(
                        div()
                            .text_sm()
                            .when(is_active, |this| this.font_medium())
                            .text_color(if is_active {
                                cx.theme().foreground
                            } else {
                                cx.theme().muted_foreground
                            })
                            .child(item.label),
                    )
                    .children(item.count.map(|count| {
                        super::blocks::mono(count.to_string(), cx)
                            .text_sm()
                            .text_color(if is_active {
                                super::Accent::ink(cx)
                            } else {
                                cx.theme().muted_foreground
                            })
                    })),
            );
        }
        track
    }
}
