//! zeron's settings select widget, ported from
//! `crates/ui/src/settings/widgets.rs` (MIT, Copyright 2026 Wing, pinned rev
//! 8b7afb3) in the plan's `settings-surface` phase (R8): the settings pages use
//! zeron's real select (`select_trigger` opening a frosted `select_menu` of
//! `SelectOption` rows) instead of a local approximation.
//!
//! Ported verbatim from upstream's `widgets.rs` lines 1101-1537 — the
//! `SELECT_HEIGHT`/`SELECT_MENU_MIN_WIDTH` constants, `select_fill`,
//! `select_trigger`, `select_value`, `select_chevron`, `select_menu`,
//! `select_row`, `select_check`, `SelectState`, `close_select`, `SelectOption`,
//! `select`, `Select`, and the private `SelectReach`/`SelectHandler`,
//! `close_select_rc` and `commit_select` helpers. Upstream has no tests for
//! this block, so none are dropped.
//!
//! Adaptations from upstream (file split + imports only):
//! - `dropdown_rows` and `SettingsDropdown` now come from [`super::menu`] (upstream
//!   kept them in the same `widgets.rs` file); `SettingsDropdown` is
//!   `pub(crate)` there for this cross-module construction.
//! - imports are trimmed to what this file uses.

use std::rc::Rc;

use gpui::{AnyElement, Context, SharedString, Window, div, prelude::*, px};

use super::menu::{SettingsDropdown, dropdown_rows};
use crate::popover;
use crate::theme::Theme;

/// Height of every settings select trigger: centered in a 60px card row.
pub const SELECT_HEIGHT: f32 = 32.0;
/// Menus never shrink below this, so short values still open a readable list.
const SELECT_MENU_MIN_WIDTH: f32 = 160.0;

/// A select's fill on a settings block: a translucent wash one step above
/// [`block_fill`], lifting while hovered or open. Washes read through frost
/// and stay tonal on solid surfaces, in both appearances.
pub(crate) fn select_fill(theme: &Theme, lifted: bool) -> gpui::Hsla {
    theme.wash(if lifted { 0.10 } else { 0.06 })
}

/// The trigger of every settings dropdown: a borderless filled pill. The
/// caller adds the value ([`select_value`], after any leading glyph), then
/// [`select_chevron`], plus its own listeners. The transparent edge is held
/// for the focus ring, so focusing never shifts the label.
pub fn select_trigger(
    theme: &Theme,
    id: impl Into<SharedString>,
    open: bool,
) -> gpui::Stateful<gpui::Div> {
    let id: SharedString = id.into();
    let hover_key: SharedString = format!("{id}-hover").into();
    let lifted = select_fill(theme, true);
    let accent = theme.accent;
    div()
        .id(id)
        .relative()
        .flex_none()
        .max_w_full()
        .h(px(SELECT_HEIGHT))
        .pl(px(10.0))
        .pr(px(8.0))
        .rounded(px(8.0))
        .border_1()
        .border_color(gpui::transparent_black())
        .bg(if open {
            lifted
        } else {
            crate::motion::hover_blend(&hover_key, select_fill(theme, false), lifted)
        })
        .on_hover(crate::motion::hover_listener(hover_key))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(8.0))
        .cursor_pointer()
        .text_size(crate::typography::ui_rems(12.5))
        .text_color(theme.text)
        .tab_index(0)
        .role(gpui::Role::Button)
        .aria_expanded(open)
        .focus_visible(move |s| s.border_color(accent))
}

/// The trigger's current value, truncating before the chevron.
pub fn select_value(label: impl Into<SharedString>) -> gpui::Div {
    div().flex_1().min_w_0().truncate().child(label.into())
}

pub fn select_chevron(theme: &Theme, open: bool) -> gpui::Svg {
    crate::icons::icon(crate::icons::ALT_ARROW_DOWN)
        .size(px(14.0))
        .flex_none()
        .text_color(if open { theme.text } else { theme.text_muted })
}

/// The open menu: the shared popover card, which [`dropdown`] mounts inside
/// the frosted menu layer (an opaque card where frost is off).
pub fn select_menu(theme: &Theme) -> gpui::Div {
    popover::popover_card(theme).flex().flex_col()
}

/// One option row. `highlighted` is the keyboard cursor; the caller adds the
/// label, then [`select_check`].
pub fn select_row(
    theme: &Theme,
    selected: bool,
    highlighted: bool,
    fade_key: impl Into<SharedString>,
) -> gpui::Div {
    popover::menu_row_nav(theme, selected, highlighted, fade_key)
}

/// The trailing check slot, kept on every row so labels align.
pub fn select_check(theme: &Theme, selected: bool) -> gpui::Div {
    div()
        .w(px(18.0))
        .flex_none()
        .flex()
        .justify_end()
        .when(selected, |slot| {
            slot.child(
                crate::icons::icon(crate::icons::CHECK)
                    .size(px(14.0))
                    .text_color(theme.accent),
            )
        })
}

/// Open state for one [`select`], owned by its page: the popup with its exit
/// phase, and the keyboard highlight.
#[derive(Default)]
pub struct SelectState {
    menu: popover::Popup<()>,
    highlighted: usize,
}

impl SelectState {
    pub fn is_open(&self) -> bool {
        self.menu.is_open()
    }

    /// Open with the keyboard highlight on option `highlighted`.
    pub fn open(&mut self, highlighted: usize) {
        self.highlighted = highlighted;
        self.menu.open(());
    }
}

/// Close a [`select`] from outside it — another menu on the page opening.
/// `reach` as in [`select`].
pub fn close_select<V: 'static>(
    view: &mut V,
    reach: impl Fn(&mut V) -> &mut SelectState + 'static,
    cx: &mut Context<V>,
) {
    let reach: SelectReach<V> = Rc::new(reach);
    close_select_rc(view, &reach, cx);
}

/// One option in a [`select`] menu.
pub struct SelectOption {
    label: SharedString,
    leading: Option<Rc<dyn Fn() -> AnyElement>>,
    detail: Option<SharedString>,
}

impl SelectOption {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            leading: None,
            detail: None,
        }
    }

    /// A glyph or swatch before the label, in the row and (when selected)
    /// the trigger.
    pub fn leading(mut self, leading: impl Fn() -> AnyElement + 'static) -> Self {
        self.leading = Some(Rc::new(leading));
        self
    }

    /// A quiet note after the label, in the row only.
    pub fn detail(mut self, detail: impl Into<SharedString>) -> Self {
        self.detail = Some(detail.into());
        self
    }
}

type SelectReach<V> = Rc<dyn Fn(&mut V) -> &mut SelectState>;
type SelectHandler<V> = Rc<dyn Fn(&mut V, usize, &mut Window, &mut Context<V>)>;

/// The settings dropdown: a [`select_trigger`] opening a [`select_menu`] of
/// options with a check on the current one. Pointer: the trigger toggles and
/// an outside press dismisses. Keyboard: ↑/↓ open and move the highlight,
/// Home/End jump, Enter/Space open or commit, Escape closes. `reach`
/// re-borrows the page's [`SelectState`] inside listeners; `label` names the
/// control for assistive tech, which hears "label: value".
pub fn select<V: 'static>(
    id: impl Into<SharedString>,
    label: impl Into<SharedString>,
    theme: &Theme,
    reach: impl Fn(&mut V) -> &mut SelectState + 'static,
) -> Select<V> {
    Select {
        id: id.into(),
        label: label.into(),
        theme: theme.clone(),
        options: Vec::new(),
        selected: 0,
        width: None,
        menu_width: None,
        font: None,
        heading: None,
        align: popover::MenuAlign::End,
        reach: Rc::new(reach),
        on_select: None,
    }
}

pub struct Select<V: 'static> {
    id: SharedString,
    label: SharedString,
    theme: Theme,
    options: Vec<SelectOption>,
    selected: usize,
    width: Option<f32>,
    menu_width: Option<f32>,
    font: Option<SharedString>,
    heading: Option<&'static str>,
    align: popover::MenuAlign,
    reach: SelectReach<V>,
    on_select: Option<SelectHandler<V>>,
}

fn close_select_rc<V: 'static>(view: &mut V, reach: &SelectReach<V>, cx: &mut Context<V>) {
    if reach(view).menu.begin_close() {
        let reach = reach.clone();
        popover::reap_popup(cx, move |view: &mut V| &mut reach(view).menu);
    }
    cx.notify();
}

fn commit_select<V: 'static>(
    view: &mut V,
    ix: usize,
    reach: &SelectReach<V>,
    on_select: &Option<SelectHandler<V>>,
    window: &mut Window,
    cx: &mut Context<V>,
) {
    close_select_rc(view, reach, cx);
    if let Some(on_select) = on_select {
        on_select(view, ix, window, cx);
    }
}

impl<V: 'static> Select<V> {
    pub fn options(
        mut self,
        options: impl IntoIterator<Item = SelectOption>,
        selected: usize,
    ) -> Self {
        self.options = options.into_iter().collect();
        self.selected = selected;
        self
    }

    pub fn on_select(
        mut self,
        on_select: impl Fn(&mut V, usize, &mut Window, &mut Context<V>) + 'static,
    ) -> Self {
        self.on_select = Some(Rc::new(on_select));
        self
    }

    /// Fixed trigger width; the menu matches it unless [`Self::menu_width`]
    /// says otherwise. Without one the trigger fits its value.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn menu_width(mut self, width: f32) -> Self {
        self.menu_width = Some(width);
        self
    }

    /// Font for the value and the rows (key names read in mono).
    pub fn font_family(mut self, font: SharedString) -> Self {
        self.font = Some(font);
        self
    }

    /// A small heading over the rows.
    pub fn heading(mut self, heading: &'static str) -> Self {
        self.heading = Some(heading);
        self
    }

    /// Open the menu rightward from the trigger's left edge, for triggers
    /// that lead a row rather than end it.
    pub fn open_rightward(mut self) -> Self {
        self.align = popover::MenuAlign::Start;
        self
    }

    pub fn render(self, state: &SelectState, cx: &mut Context<V>) -> gpui::Stateful<gpui::Div> {
        let Self {
            id,
            label,
            theme,
            options,
            selected,
            width,
            menu_width,
            font,
            heading,
            align,
            reach,
            on_select,
        } = self;
        let count = options.len();
        let open = state.menu.is_open();
        let current = options.get(selected);
        let value = current
            .map(|option| option.label.clone())
            .unwrap_or_default();
        let trigger_leading = current.and_then(|option| option.leading.clone());

        let mut trigger = select_trigger(&theme, id.clone(), open)
            .debug_selector({
                let id = id.clone();
                move || id.to_string()
            })
            .aria_label(format!("{label}: {value}"))
            .when_some(width, |el, width| el.w(px(width)))
            .when_some(font.clone(), |el, font| el.font_family(font))
            .on_mouse_down(gpui::MouseButton::Left, {
                let reach = reach.clone();
                cx.listener(move |view, _, _, _| reach(view).menu.note_trigger_press())
            })
            .on_click({
                let reach = reach.clone();
                let on_select = on_select.clone();
                cx.listener(move |view, event: &gpui::ClickEvent, window, cx| {
                    let state = reach(view);
                    if event.is_keyboard() && state.menu.is_open() {
                        let ix = state.highlighted;
                        commit_select(view, ix, &reach, &on_select, window, cx);
                    } else if state.menu.take_press_was_open() || state.menu.is_open() {
                        close_select_rc(view, &reach, cx);
                    } else {
                        state.open(selected);
                        cx.notify();
                    }
                })
            })
            // Enter/Space arrive as keyboard clicks above; this handles
            // movement and dismissal.
            .on_key_down({
                let reach = reach.clone();
                cx.listener(move |view, event: &gpui::KeyDownEvent, _, cx| {
                    let key = event.keystroke.key.as_str();
                    let modifiers = event.keystroke.modifiers;
                    let menu_key =
                        popover::classify_key(key, modifiers.platform, modifiers.control);
                    let state = reach(view);
                    if !state.menu.is_open() {
                        if matches!(menu_key, popover::MenuKey::Up | popover::MenuKey::Down) {
                            state.open(selected);
                            cx.stop_propagation();
                            cx.notify();
                        }
                        return;
                    }
                    let current = Some(state.highlighted.min(count.saturating_sub(1)));
                    state.highlighted = match (key, menu_key) {
                        (_, popover::MenuKey::Up) => {
                            popover::menu_step(current, count, -1).unwrap_or_default()
                        }
                        (_, popover::MenuKey::Down) => {
                            popover::menu_step(current, count, 1).unwrap_or_default()
                        }
                        ("home", _) => 0,
                        ("end", _) => count.saturating_sub(1),
                        (_, popover::MenuKey::Escape) => {
                            cx.stop_propagation();
                            close_select_rc(view, &reach, cx);
                            return;
                        }
                        _ => return,
                    };
                    cx.stop_propagation();
                    cx.notify();
                })
            })
            .children(trigger_leading.map(|leading| leading()))
            .child(select_value(value))
            .child(select_chevron(&theme, open));

        if state.menu.get().is_some() {
            let highlighted = state.highlighted;
            let rows = options.into_iter().enumerate().map(|(ix, option)| {
                let row_id: SharedString = format!("{id}-option-{ix}").into();
                let reach = reach.clone();
                let on_select = on_select.clone();
                select_row(&theme, ix == selected, ix == highlighted, row_id.clone())
                    .id(row_id.clone())
                    .debug_selector(move || row_id.to_string())
                    .role(gpui::Role::MenuItemRadio)
                    .aria_label(option.label.clone())
                    .aria_toggled(if ix == selected {
                        gpui::Toggled::True
                    } else {
                        gpui::Toggled::False
                    })
                    .on_click(cx.listener(move |view, _, window, cx| {
                        cx.stop_propagation();
                        commit_select(view, ix, &reach, &on_select, window, cx);
                    }))
                    .children(option.leading.map(|leading| leading()))
                    .child(div().flex_1().min_w_0().truncate().child(option.label))
                    .when_some(option.detail, |row, detail| {
                        row.child(
                            div()
                                .flex_none()
                                .text_size(crate::typography::ui_rems(10.5))
                                .text_color(theme.text_muted)
                                .child(detail),
                        )
                    })
                    .child(select_check(&theme, ix == selected))
                    .into_any_element()
            });
            let menu = select_menu(&theme)
                .w(px(menu_width
                    .or(width)
                    .unwrap_or(0.0)
                    .max(SELECT_MENU_MIN_WIDTH)))
                .when_some(font, |menu, font| menu.font_family(font))
                .on_mouse_down_out({
                    let reach = reach.clone();
                    cx.listener(move |view, _, _, cx| close_select_rc(view, &reach, cx))
                })
                .when_some(heading, |menu, heading| {
                    menu.child(popover::menu_heading(&theme, heading))
                })
                .child(dropdown_rows(
                    format!("{id}-list"),
                    rows,
                    SELECT_HEIGHT,
                    if heading.is_some() { 32.0 } else { 8.0 },
                ));
            trigger = trigger.child(SettingsDropdown {
                id: format!("{id}-menu").into(),
                content: menu,
                closing: state.menu.closing_since(),
                trigger_height: SELECT_HEIGHT,
                align,
            });
        }
        trigger
    }
}
