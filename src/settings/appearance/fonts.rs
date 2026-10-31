//! Fonts and layout section (phase `fonts-layout`): zeron's Interface /
//! Terminal / Code & diff rows, each with a searchable family picker and a
//! size select, plus the unavailable-font error strips.
//!
//! Ported from zeron's `crates/ui/src/settings/appearance.rs` (MIT, rev
//! 8b7afb3): the section rows and error strips at :4094-4182, the picker and
//! size-dropdown renderers at :1958-2142, the font-menu state/handlers at
//! :735-1004, and `AppearancePage`'s font fields at :306-335. The pure model
//! (ladders, stepping, filtering) lives in [`super::font_model`].
//!
//! Adaptations:
//! - zeron writes through its settings service; mono-desk persists the
//!   `AppSettings` fields and then re-runs [`crate::settings::apply_appearance`],
//!   because the vendored typography setters no longer reinstall the theme.
//! - zeron's `ComposerInput` ("PaletteSearch") is replaced with the app's
//!   [`TextField`]: text edits notify the page through `Context::observe`, and
//!   the menu card uses the key-capture phase so Home/End reach the picker
//!   even though the focused field binds them for caret movement.
//! - search anchors (`SettingId`) are not ported — the settings search index
//!   lands with its own phase. `search_anchor` calls are therefore absent.
//! - The Conversation-width/math rows belong to [`super::layout`] and are
//!   appended inside the same card (upstream :4161-4163).
//! - strings come from [`crate::i18n::tr`]; the hidden fallback rows keep
//!   zeron's option shape.

use std::time::{Duration, Instant};

use gpui::prelude::*;
use gpui::{
    AnyElement, App, Context, Entity, FocusHandle, Focusable, KeyDownEvent, Pixels, Role,
    SharedString, Size, Subscription, Toggled, Window, div, px,
};

use zeron_ui::popover::{
    self, MenuKey, Popup, classify_key, faded_menu_list, menu_scroll_host, menu_scroll_list,
    picker_row, search_input_frame,
};
use zeron_ui::settings::menu::{dropdown, dropdown_list_height};
use zeron_ui::settings::select::{
    self as select_kit, SELECT_HEIGHT, SelectOption, SelectState, select, select_chevron,
    select_menu, select_trigger, select_value,
};
use zeron_ui::settings::widgets::{self, PageScroll};
use zeron_ui::theme::Theme;
use zeron_ui::typography::{self, FontAvailability, UiFontFamily, UiFontSize};

use crate::app_settings::AppSettings;
use crate::i18n::tr;
use crate::settings::text_field::TextField;

use super::AppearancePage;
use super::font_model::{self, FontKind, MONO_FONT_SIZES, filter_families, step_font};

/// View state the Fonts section owns (family menus/search, size selects;
/// upstream `AppearancePage` fields :306-335). Constructed by the page's
/// `Default`, so every child (search entity, focus handles) is created lazily
/// on the first render through [`ensure`].
#[derive(Default)]
pub(super) struct State {
    font_menu: Popup<()>,
    terminal_font_menu: Popup<()>,
    code_font_menu: Popup<()>,
    font_search: Option<Entity<TextField>>,
    _font_search_subscription: Option<Subscription>,
    font_focus: Option<FocusHandle>,
    terminal_font_focus: Option<FocusHandle>,
    code_font_focus: Option<FocusHandle>,
    font_list: PageScroll,
    terminal_font_list: PageScroll,
    code_font_list: PageScroll,
    font_menu_dismissed_at: Option<Instant>,
    terminal_font_menu_dismissed_at: Option<Instant>,
    code_font_menu_dismissed_at: Option<Instant>,
    size_select: SelectState,
    terminal_size_select: SelectState,
    code_size_select: SelectState,
    selected_font: UiFontFamily,
    selected_terminal_font: UiFontFamily,
    selected_code_font: UiFontFamily,
    initialized: bool,
}

impl State {
    fn font_menu(&self, kind: FontKind) -> &Popup<()> {
        match kind {
            FontKind::Ui => &self.font_menu,
            FontKind::Terminal => &self.terminal_font_menu,
            FontKind::Code => &self.code_font_menu,
        }
    }

    fn font_menu_mut(&mut self, kind: FontKind) -> &mut Popup<()> {
        match kind {
            FontKind::Ui => &mut self.font_menu,
            FontKind::Terminal => &mut self.terminal_font_menu,
            FontKind::Code => &mut self.code_font_menu,
        }
    }

    fn list_mut(&mut self, kind: FontKind) -> &mut PageScroll {
        match kind {
            FontKind::Ui => &mut self.font_list,
            FontKind::Terminal => &mut self.terminal_font_list,
            FontKind::Code => &mut self.code_font_list,
        }
    }

    fn focus(&self, kind: FontKind) -> &FocusHandle {
        match kind {
            FontKind::Ui => self.font_focus.as_ref(),
            FontKind::Terminal => self.terminal_font_focus.as_ref(),
            FontKind::Code => self.code_font_focus.as_ref(),
        }
        .expect("font focus handles are created by `ensure`")
    }

    fn dismissed_at_mut(&mut self, kind: FontKind) -> &mut Option<Instant> {
        match kind {
            FontKind::Ui => &mut self.font_menu_dismissed_at,
            FontKind::Terminal => &mut self.terminal_font_menu_dismissed_at,
            FontKind::Code => &mut self.code_font_menu_dismissed_at,
        }
    }

    fn selected(&self, kind: FontKind) -> &UiFontFamily {
        match kind {
            FontKind::Ui => &self.selected_font,
            FontKind::Terminal => &self.selected_terminal_font,
            FontKind::Code => &self.selected_code_font,
        }
    }

    fn set_selected(&mut self, kind: FontKind, family: UiFontFamily) {
        match kind {
            FontKind::Ui => self.selected_font = family,
            FontKind::Terminal => self.selected_terminal_font = family,
            FontKind::Code => self.selected_code_font = family,
        }
    }

    fn size_select(&self, kind: FontKind) -> &SelectState {
        match kind {
            FontKind::Ui => &self.size_select,
            FontKind::Terminal => &self.terminal_size_select,
            FontKind::Code => &self.code_size_select,
        }
    }

    fn size_select_mut(&mut self, kind: FontKind) -> &mut SelectState {
        match kind {
            FontKind::Ui => &mut self.size_select,
            FontKind::Terminal => &mut self.terminal_size_select,
            FontKind::Code => &mut self.code_size_select,
        }
    }
}

/// The per-frame context the pickers render with (kept in one struct so the
/// renderers stay under clippy's argument limit).
struct PickerCtx<'a> {
    theme: &'a Theme,
    availability: &'a FontAvailability,
    fixed: SharedString,
    viewport: Size<Pixels>,
    lang: &'a str,
}

/// Create the lazily-owned children (search field, focus handles) and seed the
/// highlight from the effective typography.
fn ensure(page: &mut AppearancePage, lang: &str, cx: &mut Context<AppearancePage>) {
    let state = &mut page.fonts;
    if state.font_focus.is_none() {
        state.font_focus = Some(cx.focus_handle());
        state.terminal_font_focus = Some(cx.focus_handle());
        state.code_font_focus = Some(cx.focus_handle());
    }
    if state.font_search.is_none() {
        let input =
            cx.new(|cx| TextField::new("font-search", tr("appearance.fonts.search", lang), cx));
        state._font_search_subscription = Some(cx.observe(&input, |page, _, cx| reclamp(page, cx)));
        state.font_search = Some(input);
    } else if let Some(input) = state.font_search.clone() {
        let placeholder = tr("appearance.fonts.search", lang);
        if input.read(cx).placeholder() != placeholder {
            input.update(cx, |field, cx| field.set_placeholder(placeholder, cx));
        }
    }
    if !state.initialized {
        state.selected_font = typography::effective(cx);
        state.selected_terminal_font = typography::terminal_effective(cx);
        state.selected_code_font = typography::code_effective(cx);
        state.initialized = true;
    }
}

/// The typed filter query of the shared search field.
fn query(page: &AppearancePage, cx: &App) -> String {
    page.fonts
        .font_search
        .as_ref()
        .map(|input| input.read(cx).value().to_owned())
        .unwrap_or_default()
}

/// The kind whose family menu is open, if any (upstream :857-862).
fn open_font_kind(page: &AppearancePage) -> Option<FontKind> {
    FontKind::ALL
        .into_iter()
        .find(|kind| page.fonts.font_menu(*kind).is_open())
}

/// Keep the highlighted row inside the filtered list after a search edit
/// (upstream :872-884): the next Enter must not commit a filtered-out family.
fn reclamp(page: &mut AppearancePage, cx: &mut Context<AppearancePage>) {
    let Some(kind) = open_font_kind(page) else {
        return;
    };
    let availability = typography::availability(cx);
    let visible = filter_families(&query(page, cx), kind.choices_for(&availability));
    if !visible.contains(page.fonts.selected(kind)) {
        let next = if visible.is_empty() {
            kind.effective(cx)
        } else {
            font_model::first_available(&visible, kind, &availability)
        };
        page.fonts.set_selected(kind, next);
    }
    cx.notify();
}

/// Clear the shared filter before a menu opens (upstream :929-932 saves the
/// resulting `Edited` event from clobbering the anchor highlight).
fn clear_query(page: &mut AppearancePage, cx: &mut Context<AppearancePage>) {
    if let Some(input) = page.fonts.font_search.clone() {
        input.update(cx, |field, cx| field.set_value("", cx));
    }
}

fn close_menu(page: &mut AppearancePage, kind: FontKind, cx: &mut Context<AppearancePage>) {
    if !page.fonts.font_menu_mut(kind).begin_close() {
        return;
    }
    match kind {
        FontKind::Ui => popover::reap_popup(cx, |page| &mut page.fonts.font_menu),
        FontKind::Terminal => popover::reap_popup(cx, |page| &mut page.fonts.terminal_font_menu),
        FontKind::Code => popover::reap_popup(cx, |page| &mut page.fonts.code_font_menu),
    }
}

/// Close the menu on an outside press and arm the 400ms reopen guard
/// (upstream :915-918).
fn dismiss_menu(page: &mut AppearancePage, kind: FontKind, cx: &mut Context<AppearancePage>) {
    *page.fonts.dismissed_at_mut(kind) = Some(Instant::now());
    close_menu(page, kind, cx);
}

/// Only one of this page's six font dropdowns may be open at a time; opening
/// any one closes the other five (upstream :897-913).
fn close_other_menus(
    page: &mut AppearancePage,
    keep_family: Option<FontKind>,
    keep_size: Option<FontKind>,
    cx: &mut Context<AppearancePage>,
) {
    for kind in FontKind::ALL {
        if Some(kind) != keep_family {
            close_menu(page, kind, cx);
        }
        if Some(kind) != keep_size {
            select_kit::close_select(page, move |page| page.fonts.size_select_mut(kind), cx);
        }
    }
}

/// Apply the highlighted family (upstream :791-800, :213-219), then persist
/// the requested family and reinstall the theme so the vendored `Theme` picks
/// up the new font fields.
fn commit_font(
    page: &mut AppearancePage,
    kind: FontKind,
    window: &mut Window,
    cx: &mut Context<AppearancePage>,
) {
    let family = page.fonts.selected(kind).clone();
    if !kind.is_available_for(&typography::availability(cx), &family) {
        return;
    }
    kind.apply_family(family.clone(), cx);
    {
        let settings = AppSettings::global_mut(cx);
        match kind {
            FontKind::Ui => settings.ui_font_family = family,
            FontKind::Terminal => settings.terminal_font_family = family,
            FontKind::Code => settings.code_font_family = family,
        }
    }
    let effective = kind.effective(cx);
    page.fonts.set_selected(kind, effective);
    close_menu(page, kind, cx);
    crate::settings::persist_now(cx);
    super::super::apply_appearance(window, cx);
    cx.notify();
}

/// Apply rung `ix` of this kind's size ladder (upstream :818-842) and persist
/// it, then reinstall the theme.
fn commit_size(kind: FontKind, ix: usize, window: &mut Window, cx: &mut Context<AppearancePage>) {
    let ix = ix.min(kind.size_count().saturating_sub(1));
    match kind {
        FontKind::Ui => {
            let size = UiFontSize::ALL[ix];
            typography::set_font_size(size, window, cx);
            AppSettings::global_mut(cx).font_size = size.pixels() as f64;
        }
        FontKind::Terminal => {
            let size = MONO_FONT_SIZES[ix];
            typography::set_terminal_font_size(size, cx);
            AppSettings::global_mut(cx).terminal_font_size = size as f64;
        }
        FontKind::Code => {
            let size = MONO_FONT_SIZES[ix];
            typography::set_code_font_size(size, cx);
            AppSettings::global_mut(cx).code_font_size = size as f64;
        }
    }
    crate::settings::persist_now(cx);
    super::super::apply_appearance(window, cx);
    cx.notify();
}

/// Toggle one family menu (upstream :920-942): close the other dropdowns,
/// honour the 400ms reopen guard, start at the top and focus the filter.
fn toggle_menu(
    page: &mut AppearancePage,
    kind: FontKind,
    window: &mut Window,
    cx: &mut Context<AppearancePage>,
) {
    close_other_menus(page, Some(kind), None, cx);
    let just_dismissed = page
        .fonts
        .dismissed_at_mut(kind)
        .take()
        .is_some_and(|at| at.elapsed() < Duration::from_millis(400));
    if page.fonts.font_menu(kind).is_open() {
        close_menu(page, kind, cx);
    } else if !just_dismissed {
        clear_query(page, cx);
        let effective = kind.effective(cx);
        page.fonts.set_selected(kind, effective);
        page.fonts.list_mut(kind).reset();
        page.fonts.font_menu_mut(kind).open(());
        let focus = page
            .fonts
            .font_search
            .as_ref()
            .expect("font search created by `ensure`")
            .read(cx)
            .focus_handle(cx);
        window.focus(&focus, cx);
    }
    cx.notify();
}

/// Keyboard model of a family menu (upstream :944-1004), returning whether
/// the key was consumed. The card listens in the capture phase and the trigger
/// re-checks `is_open` — Enter flips it — so a consumed key never bubbles into
/// the focused search field.
fn on_font_key_down(
    page: &mut AppearancePage,
    kind: FontKind,
    event: &KeyDownEvent,
    window: &mut Window,
    cx: &mut Context<AppearancePage>,
) -> bool {
    let key = event.keystroke.key.as_str();

    if !page.fonts.font_menu(kind).is_open() {
        // Closed: any list key opens the menu; the filter takes over from
        // there, so the highlight is not stepped through hundreds of rows.
        if matches!(
            key,
            "up" | "down" | "left" | "right" | "home" | "end" | "enter" | "space"
        ) {
            *page.fonts.dismissed_at_mut(kind) = None;
            toggle_menu(page, kind, window, cx);
            return true;
        }
        return false;
    }

    // Open: the filter input owns text and caret keys. Only navigation,
    // commit, and dismiss are claimed here.
    let availability = typography::availability(cx);
    let choices = filter_families(&query(page, cx), kind.choices_for(&availability));
    let modifiers = event.keystroke.modifiers;
    let next = match (
        key,
        classify_key(key, modifiers.platform, modifiers.control),
    ) {
        (_, MenuKey::Up) => step_font(page.fonts.selected(kind), -1, &choices, kind, &availability),
        (_, MenuKey::Down) => {
            step_font(page.fonts.selected(kind), 1, &choices, kind, &availability)
        }
        ("home", _) => font_model::first_available(&choices, kind, &availability),
        ("end", _) => font_model::last_available(&choices, kind, &availability),
        (_, MenuKey::Enter) => {
            commit_font(page, kind, window, cx);
            return true;
        }
        (_, MenuKey::Escape) => {
            let effective = kind.effective(cx);
            page.fonts.set_selected(kind, effective);
            close_menu(page, kind, cx);
            let focus = page.fonts.focus(kind).clone();
            window.focus(&focus, cx);
            cx.notify();
            return true;
        }
        _ => return false,
    };
    page.fonts.set_selected(kind, next);
    cx.notify();
    true
}

/// One family picker: a 220px trigger opening a 220px card with the search
/// row and the searchable, rail-scrolled family list (upstream
/// `render_font_picker`, :1959-2113).
fn picker(
    page: &mut AppearancePage,
    kind: FontKind,
    ctx: &PickerCtx<'_>,
    cx: &mut Context<AppearancePage>,
) -> AnyElement {
    let slug = kind.slug();
    // The scroll helpers key off `&'static str`, so the ids are spelled out
    // rather than formatted from the slug (upstream :1969-1983).
    let (host_id, list_id, rail_id) = match kind {
        FontKind::Ui => (
            "interface-font-host",
            "interface-font-scroll",
            "interface-font-scrollbar",
        ),
        FontKind::Terminal => (
            "terminal-font-host",
            "terminal-font-scroll",
            "terminal-font-scrollbar",
        ),
        FontKind::Code => ("code-font-host", "code-font-scroll", "code-font-scrollbar"),
    };
    let theme = ctx.theme;
    let effective = kind.effective(cx);
    let selected = page.fonts.selected(kind).clone();
    let query = query(page, cx);
    let filtered = !query.trim().is_empty();
    let visible = filter_families(&query, kind.choices_for(ctx.availability));
    let rows: Vec<AnyElement> = visible
        .into_iter()
        .enumerate()
        .map(|(ix, family)| {
            let available = kind.is_available_for(ctx.availability, &family);
            let active = family == effective;
            let focused = family == selected;
            let label = SharedString::from(family.label().to_owned());
            let fade_key = format!("{slug}-font-option-{ix}");
            let row_family = family.clone();
            picker_row(theme, active, focused, fade_key)
                .id(SharedString::from(format!("{slug}-font-option-{ix}")))
                .role(Role::MenuItemRadio)
                .aria_label(label.clone())
                .aria_toggled(if active {
                    Toggled::True
                } else {
                    Toggled::False
                })
                .when(available, |row| {
                    row.on_click(cx.listener(move |page, _, window, cx| {
                        cx.stop_propagation();
                        page.fonts.set_selected(kind, row_family.clone());
                        commit_font(page, kind, window, cx);
                    }))
                })
                .when(!available, |row| row.opacity(0.45))
                .child(div().flex_1().min_w_0().truncate().child(label))
                .into_any_element()
        })
        .collect();

    let search = page
        .fonts
        .font_search
        .clone()
        .expect("font search created by `ensure`");
    let search_el = search.into_any_element();
    let list: AnyElement = if rows.is_empty() {
        div()
            .px(px(8.0))
            .py(px(6.0))
            .text_size(px(12.0))
            .text_color(theme.for_popup().text_faint)
            .child(SharedString::from(if filtered {
                tr("appearance.fonts.no_match", ctx.lang)
            } else {
                tr("appearance.fonts.empty", ctx.lang)
            }))
            .into_any_element()
    } else {
        // Card-bleed scroll host (upstream :2031-2058): the rail mounts as a
        // sibling of the scroller, above its clip.
        let rail = widgets::rail(page.fonts.list_mut(kind), rail_id, theme, cx, move |page| {
            page.fonts.list_mut(kind)
        });
        let scroll = page.fonts.list_mut(kind).scroll.clone();
        let list_height = dropdown_list_height(ctx.viewport, SELECT_HEIGHT, 52.0);
        menu_scroll_host(host_id)
            .on_hover(cx.listener(move |page, hovered: &bool, _, cx| {
                if page.fonts.list_mut(kind).set_list_hovered(*hovered) {
                    cx.notify();
                }
            }))
            .child(faded_menu_list(
                &scroll,
                menu_scroll_list(list_id, &scroll)
                    .max_h(px(list_height))
                    .flex()
                    .flex_col()
                    .gap(px(2.0))
                    .children(rows),
            ))
            .children(rail)
            .into_any_element()
    };

    // The key handler sits on the card, not just the trigger: focus moves into
    // the filter input, so the card captures navigation keys before the field
    // can claim Home/End for caret movement (upstream :2061-2079 bubbles).
    let open = page.fonts.font_menu(kind).is_open();
    let closing = page.fonts.font_menu(kind).closing_since();
    let menu = select_menu(theme)
        .w(px(220.0))
        .font_family(ctx.fixed.clone())
        .on_mouse_down_out(cx.listener(move |page, _, _, cx| dismiss_menu(page, kind, cx)))
        .capture_key_down(cx.listener(move |page, event: &KeyDownEvent, window, cx| {
            if on_font_key_down(page, kind, event, window, cx) {
                cx.stop_propagation();
            }
        }))
        .flex()
        .flex_col()
        .child(search_input_frame(theme, search_el))
        .child(list);

    let value = SharedString::from(effective.label().to_owned());
    let trigger = select_trigger(theme, format!("{slug}-font-dropdown"), open)
        .w(px(220.0))
        .font_family(ctx.fixed.clone())
        .aria_label(
            tr("appearance.fonts.picker_aria", ctx.lang)
                .replace("{kind}", kind.label(ctx.lang))
                .replace("{family}", &value),
        )
        .track_focus(&page.fonts.focus(kind).clone().tab_stop(true))
        // Closed state only: once open, the card above owns these keys.
        .on_key_down(cx.listener(move |page, event: &KeyDownEvent, window, cx| {
            if !page.fonts.font_menu(kind).is_open()
                && on_font_key_down(page, kind, event, window, cx)
            {
                cx.stop_propagation();
            }
        }))
        .on_click(cx.listener(move |page, _, window, cx| {
            let focus = page.fonts.focus(kind).clone();
            window.focus(&focus, cx);
            toggle_menu(page, kind, window, cx);
        }))
        .child(select_value(value))
        .child(select_chevron(theme, open));

    if page.fonts.font_menu(kind).get().is_some() {
        trigger
            .child(dropdown(
                SharedString::from(format!("{slug}-font-menu")),
                menu,
                closing,
                SELECT_HEIGHT,
            ))
            .into_any_element()
    } else {
        trigger.into_any_element()
    }
}

/// One size select beside a family picker (upstream `render_size_picker`,
/// :2115-2142).
fn size_picker(
    page: &mut AppearancePage,
    kind: FontKind,
    ctx: &PickerCtx<'_>,
    cx: &mut Context<AppearancePage>,
) -> AnyElement {
    let slug = kind.slug();
    select(
        format!("{slug}-font-size-dropdown"),
        kind.size_label(ctx.lang),
        ctx.theme,
        move |page: &mut AppearancePage| page.fonts.size_select_mut(kind),
    )
    .options(
        kind.size_labels().into_iter().map(SelectOption::new),
        kind.size_ix(cx),
    )
    .width(128.0)
    .font_family(ctx.fixed.clone())
    .on_select(move |_page: &mut AppearancePage, ix, window, cx| commit_size(kind, ix, window, cx))
    .render(page.fonts.size_select(kind), cx)
    .into_any_element()
}

/// The "Fonts and layout" section plus its error strips (upstream :4094-4182),
/// in zeron's slot after the theme-library card. The Conversation-width/math
/// rows are appended inside the same card by [`super::layout::section`].
pub(super) fn section(
    page: &mut AppearancePage,
    theme: &Theme,
    lang: &str,
    window: &mut Window,
    cx: &mut Context<AppearancePage>,
) -> AnyElement {
    ensure(page, lang, cx);
    let availability = typography::availability(cx);
    let ctx = PickerCtx {
        theme,
        availability: &availability,
        fixed: theme.font_sans_fixed.clone(),
        viewport: window.viewport_size(),
        lang,
    };

    let ui_picker = picker(page, FontKind::Ui, &ctx, cx);
    let terminal_picker = picker(page, FontKind::Terminal, &ctx, cx);
    let code_picker = picker(page, FontKind::Code, &ctx, cx);
    let ui_size = size_picker(page, FontKind::Ui, &ctx, cx);
    let terminal_size = size_picker(page, FontKind::Terminal, &ctx, cx);
    let code_size = size_picker(page, FontKind::Code, &ctx, cx);

    let mut font_rows = widgets::section_card(theme)
        .mt_0()
        .font_family(ctx.fixed.clone());
    for (ix, (kind, picker, size_control)) in [
        (FontKind::Ui, ui_picker, ui_size),
        (FontKind::Terminal, terminal_picker, terminal_size),
        (FontKind::Code, code_picker, code_size),
    ]
    .into_iter()
    .enumerate()
    {
        font_rows = font_rows.child(
            widgets::card_row(theme, ix == 0)
                .justify_between()
                .child(
                    div()
                        .min_w(px(160.0))
                        .flex_1()
                        .child(widgets::row_title(theme, kind.label(lang)))
                        .child(widgets::meta_line(
                            theme,
                            vec![div().child(kind.description(lang)).into_any_element()],
                        )),
                )
                .child(
                    div()
                        .flex_none()
                        .max_w_full()
                        .flex()
                        .flex_row()
                        .flex_wrap()
                        .items_center()
                        .gap(px(8.0))
                        .child(picker)
                        .child(size_control),
                ),
        );
    }
    font_rows = font_rows.child(super::layout::section(page, theme, lang, window, cx));

    let mut font_section = widgets::section(theme, tr("appearance.fonts", lang), font_rows);
    for kind in FontKind::ALL {
        let (requested, effective) = (kind.requested(cx), kind.effective(cx));
        if requested != effective {
            let template = tr("appearance.fonts.unavailable", lang);
            let message = template
                .replace("{kind}", kind.label(lang))
                .replace("{requested}", requested.label())
                .replace("{effective}", effective.label());
            font_section = font_section.child(
                widgets::error_strip(theme, message)
                    .mt_0()
                    .font_family(ctx.fixed.clone()),
            );
        }
    }
    font_section.into_any_element()
}
