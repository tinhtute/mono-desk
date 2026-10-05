//! Cadence surface primitives — the card, its header, and the small
//! building blocks every reference card is assembled from. See
//! [`super`](crate::cadence) for the style analysis.

use std::rc::Rc;

use gpui_kit::assets::IconNamed;
use gpui_kit::base::spring;
use gpui_kit::component::{
    ActiveTheme, Icon, IconName, Sizable, Size, StyledExt as _, ThemeStyled as _,
    button::{Button, ButtonCustomVariant, ButtonVariants as _},
    h_flex, v_flex,
};
use gpui_kit::{
    AnyElement, App, Div, ElementId, Hsla, InteractiveElement as _, IntoElement,
    ParentElement as _, Pixels, RenderOnce, SharedString, StatefulInteractiveElement as _,
    Styled as _, Window, div, prelude::FluentBuilder as _, px, relative, rgb,
};

/// The reference's card: white, `rounded` ~16px, hairline border, faint
/// shadow, on the gray canvas. Colors come from the theme so the card
/// follows dark mode.
pub fn card(cx: &App) -> Div {
    div()
        .w_full()
        .rounded(px(16.))
        .border_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().popover)
        .shadow_xs()
        .overflow_hidden()
}

/// The rounded-square line-icon tile in front of every card title (36px
/// box, 10px radius, 18px glyph). Accepts `impl IconNamed` (both IconName
/// enums) or a ready-made `Icon`.
pub fn icon_tile(icon: impl Into<Icon>, cx: &App) -> Div {
    icon_tile_sized(icon, px(36.), px(18.), px(10.), cx)
}

/// [`icon_tile`] with explicit geometry — the claim rows' smaller variant
/// (32px box, 8px radius, 16px glyph) and any future sizes share this.
pub fn icon_tile_sized(
    icon: impl Into<Icon>,
    size: Pixels,
    glyph: Pixels,
    radius: Pixels,
    cx: &App,
) -> Div {
    div()
        .size(size)
        .flex_shrink_0()
        .rounded(radius)
        .bg(cx.theme().accent)
        .flex()
        .items_center()
        .justify_center()
        .text_color(cx.theme().foreground)
        .child(icon.into().with_size(Size::Size(glyph)))
}

/// Text set in the theme's mono family — every numeral in the reference
/// (amounts, counts, badges, axis labels) goes through this.
pub fn mono(text: impl Into<SharedString>, cx: &App) -> Div {
    div()
        .font_family(cx.theme().mono_font_family.clone())
        .child(text.into())
}

/// Card header: icon tile, title over subtitle, and an optional trailing
/// slot (badge, legend, close button).
#[derive(IntoElement)]
pub struct CardHeader {
    icon: Icon,
    title: SharedString,
    subtitle: SharedString,
    trailing: Vec<AnyElement>,
}

impl CardHeader {
    pub fn new(
        icon: impl IconNamed,
        title: impl Into<SharedString>,
        subtitle: impl Into<SharedString>,
    ) -> Self {
        Self {
            icon: Icon::new(icon),
            title: title.into(),
            subtitle: subtitle.into(),
            trailing: Vec::new(),
        }
    }

    /// Right-aligned slot: a [`MonoPill`], a legend, a close button…
    pub fn trailing(mut self, el: impl IntoElement) -> Self {
        self.trailing.push(el.into_any_element());
        self
    }
}

impl RenderOnce for CardHeader {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        h_flex()
            .justify_between()
            .px_4()
            .pt_4()
            .pb_2()
            .child(
                h_flex().gap_3().child(icon_tile(self.icon, cx)).child(
                    v_flex()
                        .gap_0p5()
                        .child(
                            div()
                                .text_lg()
                                .font_semibold()
                                .text_color(cx.theme().foreground)
                                .child(self.title),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child(self.subtitle),
                        ),
                ),
            )
            .children(self.trailing)
    }
}

/// Pill tone: accent tint (blue ink on blue tint) or neutral (gray on gray).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PillTone {
    Accent,
    Neutral,
}

/// The reference's mono pill — "2 need you", "0 to invoice", "3 claims",
/// "2 selected", the "Pending" tag. Mono text on a `rounded-full` tint.
#[derive(IntoElement)]
pub struct MonoPill {
    text: SharedString,
    tone: PillTone,
    solid: bool,
}

impl MonoPill {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            tone: PillTone::Accent,
            solid: false,
        }
    }

    pub fn tone(mut self, tone: PillTone) -> Self {
        self.tone = tone;
        self
    }

    /// Solid accent fill with white ink — the "Today" marker.
    pub fn solid(mut self) -> Self {
        self.solid = true;
        self
    }
}

impl RenderOnce for MonoPill {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let (bg, fg) = if self.solid {
            (super::Accent::solid(cx), super::Accent::on_solid(cx))
        } else {
            match self.tone {
                PillTone::Accent => (super::Accent::soft(cx), super::Accent::ink(cx)),
                PillTone::Neutral => (cx.theme().accent, cx.theme().muted_foreground),
            }
        };
        h_flex()
            .rounded_full()
            .bg(bg)
            .px_2()
            .when(self.solid, |this| this.px_2p5())
            .py(px(2.))
            .text_color(fg)
            .child(mono(self.text, cx).text_xs())
    }
}

/// A filter chip: icon + value, optionally a muted prefix before the value
/// ("Supplier" + "Any"), optionally a dropdown caret, optionally a remove
/// ×. The chip body and the × are separate hit areas (F15): the body owns
/// `on_open` (and the tab stop), the × owns `on_remove`. `.ghost()` renders
/// the "+ Add …" restore variant — muted ink, no fill.
#[derive(IntoElement)]
pub struct FilterChip {
    id: ElementId,
    icon: Option<Icon>,
    prefix: Option<SharedString>,
    text: SharedString,
    caret: bool,
    ghost: bool,
    on_open: Option<super::WindowCallback>,
    on_remove: Option<super::WindowCallback>,
}

impl FilterChip {
    pub fn new(id: impl Into<ElementId>, text: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            icon: None,
            prefix: None,
            text: text.into(),
            caret: false,
            ghost: false,
            on_open: None,
            on_remove: None,
        }
    }

    pub fn icon(mut self, icon: impl IconNamed) -> Self {
        self.icon = Some(Icon::new(icon));
        self
    }

    /// Muted label before the value ("Supplier" in "Supplier Any ▾").
    pub fn prefix(mut self, prefix: impl Into<SharedString>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    pub fn caret(mut self) -> Self {
        self.caret = true;
        self
    }

    /// Muted-outline "+ Add …" chip used to restore a removed filter.
    pub fn ghost(mut self) -> Self {
        self.ghost = true;
        self
    }

    /// Clicking the chip body (or Enter/Space while it is focused).
    pub fn on_open(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_open = Some(Rc::new(f));
        self
    }

    /// Adds the trailing × and its handler; Backspace/Delete while the chip
    /// is focused fires the same removal.
    pub fn on_remove(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_remove = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for FilterChip {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let Self {
            id,
            icon,
            prefix,
            text,
            caret,
            ghost,
            on_open,
            on_remove,
        } = self;
        let interactive = on_open.is_some() || on_remove.is_some() || caret;
        let openable = on_open.is_some() || caret;
        let focus = interactive.then(|| {
            window
                .use_keyed_state(id.clone(), cx, |_, cx| cx.focus_handle())
                .read(cx)
                .clone()
        });
        let focused = focus.as_ref().is_some_and(|f| f.is_focused(window));

        h_flex()
            .id(id.clone())
            .gap_1p5()
            .h(px(36.))
            .rounded(px(10.))
            .border_1()
            .border_color(if ghost {
                cx.theme().muted_foreground.alpha(0.4)
            } else {
                cx.theme().border
            })
            .when(!ghost, |this| this.bg(cx.theme().popover))
            .px_2p5()
            .items_center()
            .text_color(if ghost {
                cx.theme().muted_foreground
            } else {
                cx.theme().foreground
            })
            .when(interactive, |this| {
                this.tab_index(0)
                    .track_focus(focus.as_ref().expect("focus handle exists"))
                    .when(focused, |this| this.focus_ring_style(window, cx))
                    .when(openable, |this| {
                        this.cursor_pointer()
                            .hover(|style| style.bg(cx.theme().accent))
                    })
            })
            .when_some(on_open, |this, on_open| {
                this.on_click(move |_, window, cx| on_open(window, cx))
            })
            .when_some(on_remove.clone(), |this, on_remove| {
                this.on_key_down(move |event, _window, cx| {
                    if matches!(event.keystroke.key.as_str(), "backspace" | "delete") {
                        cx.stop_propagation();
                        on_remove(_window, cx);
                    }
                })
            })
            .children(icon.map(|icon| {
                Icon::new(icon)
                    .with_size(Size::Size(px(14.)))
                    .text_color(cx.theme().muted_foreground)
            }))
            .children(prefix.map(|prefix| {
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(prefix)
            }))
            .child(mono(text, cx).text_sm())
            .when(caret, |this| {
                this.child(
                    Icon::new(IconName::ChevronDown)
                        .with_size(Size::Size(px(14.)))
                        .text_color(cx.theme().muted_foreground),
                )
            })
            .children(on_remove.map(|on_remove| {
                div()
                    .id((id, "remove"))
                    .cursor_pointer()
                    .p(px(2.))
                    .rounded(px(6.))
                    .hover(|this| this.bg(cx.theme().accent))
                    .on_click(move |_, window, cx| {
                        cx.stop_propagation();
                        on_remove(window, cx);
                    })
                    .child(
                        Icon::new(IconName::Close)
                            .with_size(Size::Size(px(12.)))
                            .text_color(cx.theme().muted_foreground),
                    )
            }))
    }
}

/// One stat: muted label, mono value, optional muted sub-line. `accent`
/// switches the value to the accent ink (the reference's "Overdue" /
/// "To Be Paid" numbers).
#[derive(IntoElement)]
pub struct Stat {
    label: SharedString,
    value: SharedString,
    sub: Option<SharedString>,
    accent: bool,
}

impl Stat {
    pub fn new(label: impl Into<SharedString>, value: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            sub: None,
            accent: false,
        }
    }

    pub fn sub(mut self, sub: impl Into<SharedString>) -> Self {
        self.sub = Some(sub.into());
        self
    }

    pub fn accent(mut self) -> Self {
        self.accent = true;
        self
    }
}

impl RenderOnce for Stat {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let value_color = if self.accent {
            super::Accent::ink(cx)
        } else {
            cx.theme().foreground
        };
        v_flex()
            .gap_1()
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(self.label),
            )
            .child(
                mono(self.value, cx)
                    .text_xl()
                    .font_semibold()
                    .text_color(value_color),
            )
            .children(self.sub.map(|sub| {
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(sub)
            }))
    }
}

/// A row of [`Stat`]s separated by hairline dividers, like the reference's
/// status strips.
pub fn stat_row(stats: Vec<Stat>, cx: &App) -> Div {
    let mut row = h_flex().gap_4();
    for (ix, stat) in stats.into_iter().enumerate() {
        if ix > 0 {
            row = row.child(
                div()
                    .w(px(1.))
                    .self_center()
                    .h(px(36.))
                    .bg(cx.theme().border),
            );
        }
        row = row.child(stat);
    }
    row
}

/// Two-tone meter: solid accent progress over the faint remainder,
/// `rounded-full`, 6px tall. `fraction` clamps to 0..1.
#[derive(IntoElement)]
pub struct Meter {
    fraction: f32,
}

impl Meter {
    pub fn new(fraction: f32) -> Self {
        Self {
            fraction: fraction.clamp(0., 1.),
        }
    }
}

impl RenderOnce for Meter {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        div()
            .w_full()
            .h(px(6.))
            .rounded_full()
            .bg(super::Accent::faint(cx))
            .relative()
            .overflow_hidden()
            .child(
                div()
                    .absolute()
                    .left_0()
                    .top_0()
                    .h_full()
                    .w(relative(self.fraction))
                    .rounded_full()
                    .bg(super::Accent::solid(cx)),
            )
    }
}

/// Legend entry: small round dot + muted label.
pub fn legend_dot(color: Hsla, label: impl Into<SharedString>, cx: &App) -> Div {
    h_flex()
        .gap_1p5()
        .items_center()
        .child(div().size(px(8.)).rounded_full().bg(color))
        .child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(label.into()),
        )
}

/// Legend entry with a rounded-square swatch (the chart card headers).
pub fn legend_swatch(color: Hsla, label: impl Into<SharedString>, cx: &App) -> Div {
    h_flex()
        .gap_1p5()
        .items_center()
        .child(div().size(px(8.)).rounded(px(2.)).bg(color))
        .child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(label.into()),
        )
}

/// The reference's checkbox: 18px, `rounded` ~5px, accent fill with a white
/// check when on, hairline border when off.
#[derive(IntoElement)]
pub struct CadenceCheckbox {
    id: ElementId,
    checked: bool,
    on_click: super::BoolCallback,
}

impl CadenceCheckbox {
    pub fn new(id: impl Into<ElementId>, checked: bool) -> Self {
        Self {
            id: id.into(),
            checked,
            on_click: Rc::new(|_, _, _| {}),
        }
    }

    pub fn on_click(mut self, f: impl Fn(&bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Rc::new(f);
        self
    }
}

impl RenderOnce for CadenceCheckbox {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let checked = self.checked;
        let (bg, border, fg) = if checked {
            (
                super::Accent::solid(cx),
                super::Accent::solid(cx),
                super::Accent::on_solid(cx),
            )
        } else {
            (
                cx.theme().popover,
                cx.theme().border,
                cx.theme().muted_foreground,
            )
        };
        let focus_handle = window
            .use_keyed_state(self.id.clone(), cx, |_, cx| cx.focus_handle())
            .read(cx)
            .clone();
        let is_focused = focus_handle.is_focused(window);
        // The glyph fades through a spring so unchecking still shows it
        // briefly — mounting only on `checked` would cut it instantly.
        let mark_opacity = spring(
            (self.id.clone(), "mark"),
            if checked { 1. } else { 0. },
            cx.theme().motion_tokens().spring_control,
            window,
            cx,
        );
        let on_click = self.on_click.clone();
        let on_key = self.on_click.clone();
        div()
            .id(self.id)
            .cursor_pointer()
            .tab_index(0)
            .track_focus(&focus_handle)
            .size(px(18.))
            .flex_shrink_0()
            .rounded(px(5.))
            .border_1()
            .border_color(border)
            .bg(bg)
            .flex()
            .items_center()
            .justify_center()
            .hover(|this| {
                if checked {
                    this.bg(super::Accent::solid(cx).alpha(0.85))
                } else {
                    this.border_color(super::Accent::solid(cx))
                        .bg(super::Accent::wash(cx))
                }
            })
            .when(is_focused, |this| this.focus_ring_style(window, cx))
            .on_click(move |_, window, cx| {
                window.prevent_default();
                on_click(&!checked, window, cx);
                cx.stop_propagation();
            })
            .on_key_down(move |event, window, cx| {
                if matches!(event.keystroke.key.as_ref(), "space" | "enter") {
                    window.prevent_default();
                    on_key(&!checked, window, cx);
                    cx.stop_propagation();
                }
            })
            .when(mark_opacity > 0., |this| {
                this.child(
                    Icon::new(IconName::Check)
                        .with_size(Size::Size(px(12.)))
                        .text_color(fg)
                        .opacity(mark_opacity),
                )
            })
    }
}

/// The reference's toggle: 40×22 pill, accent fill with a white thumb when
/// on, hairline track when off.
#[derive(IntoElement)]
pub struct CadenceToggle {
    id: ElementId,
    checked: bool,
    on_click: super::BoolCallback,
}

impl CadenceToggle {
    pub fn new(id: impl Into<ElementId>, checked: bool) -> Self {
        Self {
            id: id.into(),
            checked,
            on_click: Rc::new(|_, _, _| {}),
        }
    }

    pub fn on_click(mut self, f: impl Fn(&bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Rc::new(f);
        self
    }
}

impl RenderOnce for CadenceToggle {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let checked = self.checked;
        let focus_handle = window
            .use_keyed_state(self.id.clone(), cx, |_, cx| cx.focus_handle())
            .read(cx)
            .clone();
        let is_focused = focus_handle.is_focused(window);
        // The thumb's travel is a spring, so toggling mid-flight reverses
        // from wherever it is instead of restarting or jumping.
        let thumb_x = spring(
            (self.id.clone(), "thumb"),
            if checked { px(20.) } else { px(2.) },
            cx.theme().motion_tokens().spring_move,
            window,
            cx,
        );
        let on_click = self.on_click.clone();
        let on_key = self.on_click.clone();
        div()
            .id(self.id)
            .cursor_pointer()
            .tab_index(0)
            .track_focus(&focus_handle)
            .w(px(40.))
            .h(px(22.))
            .rounded_full()
            .relative()
            .bg(if checked {
                super::Accent::solid(cx)
            } else {
                cx.theme().border
            })
            .hover(|this| {
                if checked {
                    this.bg(super::Accent::solid(cx).alpha(0.85))
                } else {
                    this.bg(cx.theme().border.opacity(0.8))
                }
            })
            .when(is_focused, |this| this.focus_ring_style(window, cx))
            .on_click(move |_, window, cx| {
                window.prevent_default();
                on_click(&!checked, window, cx);
                cx.stop_propagation();
            })
            .on_key_down(move |event, window, cx| {
                if matches!(event.keystroke.key.as_ref(), "space" | "enter") {
                    window.prevent_default();
                    on_key(&!checked, window, cx);
                    cx.stop_propagation();
                }
            })
            .child(
                div()
                    .absolute()
                    .top(px(2.))
                    .left(thumb_x)
                    .size(px(18.))
                    .rounded_full()
                    .bg(if cx.theme().mode.is_dark() {
                        rgb(0xffffff).into()
                    } else {
                        cx.theme().popover
                    })
                    .shadow_xs(),
            )
    }
}

/// Two-tone pie avatar: a `rounded-full` disc with a solid wedge fraction
/// and a faint remainder (the reference's supplier/project avatars).
pub fn pie_avatar(frac: f32, size: Pixels, cx: &App) -> Div {
    let frac = frac.clamp(0., 1.);
    div()
        .size(size)
        .rounded_full()
        .overflow_hidden()
        .flex_shrink_0()
        .child(div().absolute().inset_0().bg(super::Accent::faint(cx)))
        .child(
            div()
                .absolute()
                .left_0()
                .top_0()
                .bottom_0()
                .w(relative(frac))
                .bg(super::Accent::solid(cx)),
        )
}

/// Black primary button (white label in light mode; inverted in dark via
/// foreground/popover), `rounded` 10px — the reference's action buttons.
pub fn btn_primary(id: impl Into<ElementId>, label: impl Into<SharedString>, cx: &App) -> Button {
    Button::new(id).label(label).rounded(px(10.)).custom(
        ButtonCustomVariant::new(cx)
            .color(cx.theme().foreground)
            .foreground(cx.theme().popover)
            .hover(cx.theme().foreground.alpha(0.82))
            .active(cx.theme().foreground.alpha(0.75)),
    )
}

/// White bordered secondary button, `rounded` 10px.
pub fn btn_secondary(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Button {
    Button::new(id).label(label).outline().rounded(px(10.))
}

/// The 28px bordered × button in card-header trailing slots (Filter,
/// Assign to project). `on_close` owns the dismissal behavior; focusable
/// like the other binary controls.
#[derive(IntoElement)]
pub struct CloseButton {
    id: ElementId,
    on_close: super::WindowCallback,
}

impl CloseButton {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            on_close: Rc::new(|_, _| {}),
        }
    }

    pub fn on_close(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_close = Rc::new(f);
        self
    }
}

impl RenderOnce for CloseButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let focus_handle = window
            .use_keyed_state(self.id.clone(), cx, |_, cx| cx.focus_handle())
            .read(cx)
            .clone();
        let is_focused = focus_handle.is_focused(window);
        let on_close = self.on_close.clone();
        let on_key = self.on_close.clone();
        div()
            .id(self.id)
            .cursor_pointer()
            .tab_index(0)
            .track_focus(&focus_handle)
            .size(px(28.))
            .rounded(px(8.))
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().popover)
            .shadow_xs()
            .flex()
            .items_center()
            .justify_center()
            .hover(|this| this.bg(cx.theme().accent))
            .when(is_focused, |this| this.focus_ring_style(window, cx))
            .on_click(move |_, window, cx| {
                window.prevent_default();
                on_close(window, cx);
            })
            .on_key_down(move |event, window, cx| {
                if matches!(event.keystroke.key.as_ref(), "space" | "enter" | "escape") {
                    window.prevent_default();
                    on_key(window, cx);
                    cx.stop_propagation();
                }
            })
            .child(
                Icon::new(IconName::Close)
                    .with_size(Size::Size(px(14.)))
                    .text_color(cx.theme().muted_foreground),
            )
    }
}

/// Empty state: recessed illustration well, heading, muted body, action
/// row with an optional right-aligned aside ("Default markup 10%").
#[derive(IntoElement)]
pub struct EmptyState {
    title: SharedString,
    body: SharedString,
    illustration: Option<AnyElement>,
    actions: Vec<AnyElement>,
    aside: Option<AnyElement>,
}

impl EmptyState {
    pub fn new(title: impl Into<SharedString>, body: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            body: body.into(),
            illustration: None,
            actions: Vec::new(),
            aside: None,
        }
    }

    pub fn illustration(mut self, el: impl IntoElement) -> Self {
        self.illustration = Some(el.into_any_element());
        self
    }

    pub fn action(mut self, el: impl IntoElement) -> Self {
        self.actions.push(el.into_any_element());
        self
    }

    pub fn aside(mut self, el: impl IntoElement) -> Self {
        self.aside = Some(el.into_any_element());
        self
    }
}

impl RenderOnce for EmptyState {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        v_flex()
            .gap_3()
            .px_4()
            .pb_4()
            .children(self.illustration)
            .child(
                div()
                    .text_lg()
                    .font_semibold()
                    .text_color(cx.theme().foreground)
                    .child(self.title),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(self.body),
            )
            .child(
                h_flex()
                    .justify_between()
                    .items_center()
                    .pt_1()
                    .child(h_flex().gap_2().children(self.actions))
                    .children(self.aside),
            )
    }
}
