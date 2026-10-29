//! Finance-showcase surface primitives — the card, its header, and the small
//! building blocks every reference card is assembled from. Ported from the
//! pre-port `src/cadence/blocks.rs` (plain divs on the old stack too); the
//! widgets that were real controls now come from zeron's settings kit and the
//! colours from the installed [`Theme`](zeron_ui::theme::Theme).

use gpui::prelude::*;
use gpui::{
    AnyElement, App, Div, Hsla, IntoElement, Pixels, RenderOnce, Role, SharedString, Stateful,
    Toggled, Window, div, px, relative,
};

use zeron_ui::icons::{self, icon};
use zeron_ui::settings::widgets::{self, ActionTone};
use zeron_ui::theme::Theme;

use super::{Accent, WindowCallback, h_flex, v_flex};

/// The reference's card: white (light) raised surface, `rounded` ~16px,
/// hairline border, faint shadow, on the gray canvas.
pub(crate) fn card(theme: &Theme) -> Div {
    div()
        .w_full()
        .rounded(px(16.))
        .border_1()
        .border_color(theme.border)
        .bg(theme.surface_overlay)
        .shadow_xs()
        .overflow_hidden()
}

/// The rounded-square line-icon tile in front of every card title.
pub(crate) fn icon_tile(icon_path: &'static str, theme: &Theme) -> Div {
    div()
        .flex()
        .items_center()
        .justify_center()
        .size(px(36.))
        .flex_shrink_0()
        .rounded(px(10.))
        .bg(theme.accent_wash)
        .child(icon(icon_path).size(px(18.)).text_color(theme.accent))
}

/// Text set in the theme's mono family — every numeral in the reference
/// (amounts, counts, badges, axis labels) goes through this.
pub(crate) fn mono(text: impl Into<SharedString>, theme: &Theme) -> Div {
    div()
        .font_family(theme.font_mono.clone())
        .child(text.into())
}

/// Card header: icon tile, title over subtitle, and an optional trailing slot
/// (badge, legend, close button).
#[derive(IntoElement)]
pub(crate) struct CardHeader {
    icon: &'static str,
    title: SharedString,
    subtitle: SharedString,
    trailing: Vec<AnyElement>,
}

impl CardHeader {
    pub fn new(
        icon: &'static str,
        title: impl Into<SharedString>,
        subtitle: impl Into<SharedString>,
    ) -> Self {
        Self {
            icon,
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
        let theme = Theme::of(cx).clone();
        h_flex()
            .justify_between()
            .items_start()
            .px_4()
            .pt_4()
            .pb_2()
            .child(
                h_flex()
                    .gap_3()
                    .items_center()
                    .child(icon_tile(self.icon, &theme))
                    .child(
                        v_flex()
                            .gap_0p5()
                            .min_w_0()
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(theme.text)
                                    .child(self.title),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.text_muted)
                                    .child(self.subtitle),
                            ),
                    ),
            )
            .children(self.trailing)
    }
}

/// Pill tone: accent tint (accent ink on the accent wash) or neutral (gray on
/// gray).
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum PillTone {
    Accent,
    Neutral,
}

/// The reference's mono pill — "2 need you", "0 to invoice", "3 claims",
/// "2 selected", the "Pending" tag. Mono text on a `rounded-full` tint.
#[derive(IntoElement)]
pub(crate) struct MonoPill {
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

    /// Solid accent fill with the on-solid ink — the "Today" marker.
    pub fn solid(mut self) -> Self {
        self.solid = true;
        self
    }
}

impl RenderOnce for MonoPill {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::of(cx).clone();
        let (bg, fg) = if self.solid {
            (Accent::solid(&theme), Accent::on_solid(&theme))
        } else {
            match self.tone {
                PillTone::Accent => (Accent::soft(&theme), Accent::ink(&theme)),
                PillTone::Neutral => (theme.wash(0.08), theme.text_muted),
            }
        };
        h_flex()
            .flex_none()
            .rounded_full()
            .bg(bg)
            .px_2()
            .py(px(2.))
            .text_color(fg)
            .child(mono(self.text, &theme).text_xs())
    }
}

/// A filter chip: icon + text, optionally a muted prefix before the value
/// ("Supplier" + "Any"), optionally a dropdown caret, optionally a remove ×.
/// The chip body is zeron's filled action button; the × is a nested button.
#[derive(IntoElement)]
pub(crate) struct FilterChip {
    id: SharedString,
    icon: Option<&'static str>,
    prefix: Option<SharedString>,
    text: SharedString,
    caret: bool,
    on_remove: Option<WindowCallback>,
}

impl FilterChip {
    pub fn new(id: impl Into<SharedString>, text: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            icon: None,
            prefix: None,
            text: text.into(),
            caret: false,
            on_remove: None,
        }
    }

    pub fn icon(mut self, icon_path: &'static str) -> Self {
        self.icon = Some(icon_path);
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

    /// Adds the trailing × and its handler.
    pub fn on_remove(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_remove = Some(std::rc::Rc::new(f));
        self
    }
}

impl RenderOnce for FilterChip {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::of(cx).clone();
        let muted = theme.text_muted;
        let hover = theme.wash(0.08);
        let label: SharedString = match &self.prefix {
            Some(prefix) => format!("{prefix} {}", self.text).into(),
            None => self.text.clone(),
        };
        let remove_id: SharedString = format!("{}-remove", self.id).into();

        widgets::action_button(&theme, ActionTone::Filled)
            .id(self.id)
            .tab_index(0)
            .role(Role::Button)
            .aria_label(label)
            .children(
                self.icon
                    .map(|icon_path| icon(icon_path).size(px(14.)).text_color(muted)),
            )
            .children(
                self.prefix
                    .map(|prefix| div().text_xs().text_color(muted).child(prefix)),
            )
            .child(div().text_xs().child(self.text))
            .when(self.caret, |this| {
                this.child(icon(icons::ALT_ARROW_DOWN).size(px(14.)).text_color(muted))
            })
            .children(self.on_remove.map(|on_remove| {
                div()
                    .id(remove_id)
                    .flex_none()
                    .p(px(2.))
                    .rounded(px(6.))
                    .cursor_pointer()
                    .role(Role::Button)
                    .aria_label("Remove filter")
                    .hover(move |this| this.bg(hover))
                    .on_click(move |_, window, cx| on_remove(window, cx))
                    .child(icon(icons::CLOSE).size(px(12.)).text_color(muted))
            }))
    }
}

/// One stat: muted label, mono value, optional muted sub-line. `accent`
/// switches the value to the accent ink (the reference's "Overdue" /
/// "To Be Paid" numbers).
#[derive(IntoElement)]
pub(crate) struct Stat {
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
        let theme = Theme::of(cx).clone();
        let value_color = if self.accent {
            Accent::ink(&theme)
        } else {
            theme.text
        };
        v_flex()
            .gap_1()
            .min_w_0()
            .child(
                div()
                    .text_xs()
                    .text_color(theme.text_muted)
                    .child(self.label),
            )
            .child(
                mono(self.value, &theme)
                    .text_base()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(value_color),
            )
            .children(
                self.sub
                    .map(|sub| div().text_xs().text_color(theme.text_muted).child(sub)),
            )
    }
}

/// A row of [`Stat`]s separated by hairline dividers, like the reference's
/// status strips.
pub(crate) fn stat_row(stats: Vec<Stat>, theme: &Theme) -> Div {
    let mut row = h_flex().gap_3();
    for (ix, stat) in stats.into_iter().enumerate() {
        if ix > 0 {
            row = row.child(div().w(px(1.)).self_center().h(px(36.)).bg(theme.border));
        }
        row = row.child(stat);
    }
    row
}

/// Two-tone meter: solid accent progress over the faint remainder,
/// `rounded-full`, 6px tall. `fraction` clamps to 0..1.
#[derive(IntoElement)]
pub(crate) struct Meter {
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
        let theme = Theme::of(cx).clone();
        div()
            .w_full()
            .h(px(6.))
            .rounded_full()
            .bg(Accent::faint(&theme))
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
                    .bg(Accent::solid(&theme)),
            )
    }
}

/// Legend entry: small round dot + muted label.
pub(crate) fn legend_dot(color: Hsla, label: impl Into<SharedString>, theme: &Theme) -> Div {
    h_flex()
        .gap_1p5()
        .items_center()
        .child(div().size(px(8.)).rounded_full().bg(color))
        .child(
            div()
                .text_xs()
                .text_color(theme.text_muted)
                .child(label.into()),
        )
}

/// Legend entry with a rounded-square swatch (the chart card headers).
pub(crate) fn legend_swatch(color: Hsla, label: impl Into<SharedString>, theme: &Theme) -> Div {
    h_flex()
        .gap_1p5()
        .items_center()
        .child(div().size(px(8.)).rounded(px(2.)).bg(color))
        .child(
            div()
                .text_xs()
                .text_color(theme.text_muted)
                .child(label.into()),
        )
}

/// The reference's toggle/checkbox, on zeron's switch. The kit's
/// `toggle_switch` is paint-only by design ("the caller owns activation and
/// accessibility"), so this helper adds the id, switch role, aria label and
/// toggled state, and turns a click into the next value.
pub(crate) fn switch(
    theme: &Theme,
    on: bool,
    id: impl Into<SharedString>,
    label: impl Into<SharedString>,
    on_click: impl Fn(bool, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let id: SharedString = id.into();
    let next = !on;
    widgets::toggle_switch(theme, on, id.clone())
        .id(id)
        .tab_index(0)
        .role(Role::Switch)
        .aria_label(label)
        .aria_toggled(if on { Toggled::True } else { Toggled::False })
        .focus_visible(|style| style.border_2().border_color(theme.accent))
        .cursor_pointer()
        .on_click(move |_, window, cx| on_click(next, window, cx))
}

/// Two-tone pie avatar: a `rounded-full` disc with a solid wedge fraction
/// and a faint remainder (the reference's supplier/project avatars).
pub(crate) fn pie_avatar(frac: f32, size: Pixels, theme: &Theme) -> Div {
    let frac = frac.clamp(0., 1.);
    div()
        .relative()
        .size(size)
        .rounded_full()
        .overflow_hidden()
        .flex_shrink_0()
        .child(div().absolute().inset_0().bg(Accent::faint(theme)))
        .child(
            div()
                .absolute()
                .left_0()
                .top_0()
                .bottom_0()
                .w(relative(frac))
                .bg(Accent::solid(theme)),
        )
}

/// A labelled action button on zeron's kit: `Solid` for the reference's black
/// primary, `Filled` for its secondary actions. The caller adds `.on_click`
/// (or leaves it inert like the old mock did).
pub(crate) fn button(
    theme: &Theme,
    id: impl Into<SharedString>,
    label: impl Into<SharedString>,
    tone: ActionTone,
) -> Stateful<Div> {
    let label: SharedString = label.into();
    widgets::action_button(theme, tone)
        .id(id.into())
        .tab_index(0)
        .role(Role::Button)
        .aria_label(label.clone())
        .child(label)
}

/// An icon-only action button (the cards' close affordances).
pub(crate) fn icon_button(
    theme: &Theme,
    id: impl Into<SharedString>,
    icon_path: &'static str,
    label: impl Into<SharedString>,
) -> Stateful<Div> {
    widgets::action_button(theme, ActionTone::Filled)
        .id(id.into())
        .tab_index(0)
        .role(Role::Button)
        .aria_label(label)
        .child(icon(icon_path).size(px(14.)).text_color(theme.text_muted))
}

/// Empty state: recessed illustration well, heading, muted body, action row
/// with an optional right-aligned aside ("Default markup 10%").
#[derive(IntoElement)]
pub(crate) struct EmptyState {
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
        let theme = Theme::of(cx).clone();
        v_flex()
            .gap_3()
            .px_4()
            .pb_4()
            .children(self.illustration)
            .child(
                div()
                    .text_base()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(theme.text)
                    .child(self.title),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(theme.text_muted)
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
