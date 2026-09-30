//! Cadence controls — the reference's two tab-like navigators: underline
//! tabs with mono counts ("All 24 · Draft 3 · …") and the pill segmented
//! control ("Your own · To review 2 · To pay 3").

use std::rc::Rc;

use gpui_kit::component::{ActiveTheme, StyledExt as _, h_flex, v_flex};
use gpui_kit::{
    App, ElementId, InteractiveElement as _, IntoElement, ParentElement as _, RenderOnce,
    SharedString, StatefulInteractiveElement as _, Styled as _, Window, div,
    prelude::FluentBuilder as _, px,
};

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
/// ink with an accent count and a 2px accent underline over the hairline.
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
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let base = self.id.clone();
        let active = self.active.min(self.tabs.len().saturating_sub(1));
        let mut row = h_flex()
            .gap_5()
            .px_4()
            .border_b_1()
            .border_color(cx.theme().border);
        for (ix, tab) in self.tabs.into_iter().enumerate() {
            let is_active = ix == active;
            let id: ElementId = SharedString::from(format!("{base}-{ix}")).into();
            let on_pick = self.on_pick.clone();
            row = row.child(
                v_flex()
                    .id(id)
                    .when_some(on_pick, |this, on_pick| {
                        this.cursor_pointer().on_click(move |_, window, cx| {
                            on_pick(ix, window, cx);
                        })
                    })
                    .gap_1()
                    .child(
                        h_flex()
                            .gap_1p5()
                            .items_center()
                            .child(
                                div()
                                    .text_base()
                                    .when(is_active, |this| this.font_medium())
                                    .text_color(if is_active {
                                        cx.theme().foreground
                                    } else {
                                        cx.theme().muted_foreground
                                    })
                                    .child(tab.label),
                            )
                            .children(tab.count.map(|count| {
                                super::blocks::mono(count.to_string(), cx)
                                    .text_sm()
                                    .text_color(if is_active {
                                        super::Accent::ink(cx)
                                    } else {
                                        cx.theme().muted_foreground
                                    })
                            })),
                    )
                    .child(
                        div()
                            .h(px(2.))
                            .w_full()
                            .rounded_full()
                            .mt_1()
                            .when(is_active, |this| this.bg(super::Accent::solid(cx))),
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

/// Pill segmented control: gray container, the active segment raised as a
/// white pill with a shadow and an accent count.
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
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let base = self.id.clone();
        let active = self.active.min(self.items.len().saturating_sub(1));
        let mut row = h_flex()
            .gap(px(2.))
            .p(px(3.))
            .rounded(px(10.))
            .bg(cx.theme().accent);
        for (ix, item) in self.items.into_iter().enumerate() {
            let is_active = ix == active;
            let id: ElementId = SharedString::from(format!("{base}-{ix}")).into();
            let on_pick = self.on_pick.clone();
            row = row.child(
                h_flex()
                    .id(id)
                    .when_some(on_pick, |this, on_pick| {
                        this.cursor_pointer().on_click(move |_, window, cx| {
                            on_pick(ix, window, cx);
                        })
                    })
                    .gap_1p5()
                    .items_center()
                    .px_3()
                    .h(px(30.))
                    .rounded(px(8.))
                    .when(is_active, |this| this.bg(cx.theme().popover).shadow_xs())
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
        row
    }
}
