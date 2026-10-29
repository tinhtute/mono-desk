//! The Profile page (`settings-surface` T2): zeron's page scaffolding with
//! mono-desk's own content — the username/display-name fields and a preview of
//! how the identity reads.
//!
//! Behavior is ported from the pre-port page on `wip/gpui-kit-shell`
//! (`src/main.rs`): usernames are 3–16 characters of lowercase letters, digits,
//! `-` and `_` (empty is idle, not an error), the `field.username_*` messages
//! validate inline, and the preview reads `Display Name (@username)` with the
//! display name falling back to the username while empty.
//!
//! zeron's settings kit has no text field (its field is the chat composer,
//! which the port plan puts out of scope), so the editors are the vendored
//! [`TextField`] entities wrapped in a box shaped like zeron's select trigger:
//! same 32px height, radius and wash fill, with a focus-visible accent edge.
//! Writes go to the `AppSettings` global and through [`super::persist_now`],
//! like every other page.

use gpui::prelude::*;
use gpui::{
    Context, CursorStyle, Entity, FocusHandle, Focusable, FontWeight, IntoElement, Render, Role,
    ScrollHandle, SharedString, Subscription, Window, div, px,
};

use zeron_ui::settings::widgets;
use zeron_ui::theme::Theme;
use zeron_ui::typography;

use crate::app_settings::AppSettings;
use crate::i18n::tr;

use super::text_field::TextField;

/// The box the text field sits in: zeron's select trigger metrics.
const FIELD_HEIGHT: f32 = 32.0;
const FIELD_WIDTH: f32 = 240.0;
const FIELD_RADIUS: f32 = 8.0;

/// Why the username field accepts or rejects a value.
///
/// The rule is a pure function so validation is testable without a locale or a
/// window; [`Self::validation`] adds the translated message and its tone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UsernameRule {
    /// Empty: no message, the field is simply idle.
    Empty,
    TooShort,
    TooLong,
    InvalidCharacter,
    Valid,
}

impl UsernameRule {
    /// The inline message for this rule in `lang`, as `(message, is_valid)`.
    fn validation(self, lang: &str) -> Option<(&'static str, bool)> {
        match self {
            Self::Empty => None,
            Self::TooShort => Some((tr("field.username_too_short", lang), false)),
            Self::TooLong => Some((tr("field.username_too_long", lang), false)),
            Self::InvalidCharacter => Some((tr("field.username_invalid", lang), false)),
            Self::Valid => Some((tr("field.username_ok", lang), true)),
        }
    }
}

/// Usernames are 3–16 characters of lowercase letters, digits, `-` and `_`.
fn username_rule(value: &str) -> UsernameRule {
    if value.is_empty() {
        return UsernameRule::Empty;
    }
    let length = value.chars().count();
    if length < 3 {
        return UsernameRule::TooShort;
    }
    if length > 16 {
        return UsernameRule::TooLong;
    }
    if !value
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
    {
        return UsernameRule::InvalidCharacter;
    }
    UsernameRule::Valid
}

pub struct ProfilePage {
    /// The username editor. Created on the first render: the shell builds the
    /// page with `cx.new(|_| ProfilePage::new())`, so `new()` has no `cx`.
    username_field: Option<Entity<TextField>>,
    /// The display-name editor.
    display_name_field: Option<Entity<TextField>>,
    /// The page language the editors' placeholders were built for; a language
    /// switch rebuilds them so the placeholders follow the page language.
    fields_lang: SharedString,
    /// Keeps the page repainting while a field changes: the inline username
    /// validation reads the field's live value.
    field_observers: Vec<Subscription>,
    /// The page's scroll offset (zeron's pages own their scroll handle).
    scroll: ScrollHandle,
}

impl ProfilePage {
    pub fn new() -> Self {
        Self {
            username_field: None,
            display_name_field: None,
            fields_lang: SharedString::default(),
            field_observers: Vec::new(),
            scroll: ScrollHandle::new(),
        }
    }

    /// Build the text-field entities for `lang` (or rebuild them when the page
    /// language changed) and mirror external `AppSettings` edits — e.g. the
    /// Reset page restoring a default — into a field that is not being edited.
    fn ensure_fields(
        &mut self,
        lang: &str,
        username: &str,
        display_name: &str,
        window: &Window,
        cx: &mut Context<Self>,
    ) {
        if self.fields_lang.as_ref() != lang {
            let field_lang = lang.to_owned();
            let username_field = cx.new(|cx| {
                TextField::new(
                    "profile-username",
                    tr("item.username_placeholder", &field_lang),
                    cx,
                )
                .with_value(username.to_owned())
                .on_change(|value, _window, cx| {
                    AppSettings::global_mut(cx).username = value.into();
                    super::persist_now(cx);
                    // The sidebar footer shows the identity; repaint it too.
                    cx.refresh_windows();
                })
            });
            let display_name_field = cx.new(|cx| {
                TextField::new(
                    "profile-display-name",
                    tr("item.display_name", &field_lang),
                    cx,
                )
                .with_value(display_name.to_owned())
                .on_change(|value, _window, cx| {
                    AppSettings::global_mut(cx).display_name = value.into();
                    super::persist_now(cx);
                    cx.refresh_windows();
                })
            });
            let mut field_observers = Vec::new();
            field_observers.push(cx.observe(&username_field, |_page, _field, cx| {
                cx.notify();
            }));
            field_observers.push(cx.observe(&display_name_field, |_page, _field, cx| {
                cx.notify();
            }));
            self.username_field = Some(username_field);
            self.display_name_field = Some(display_name_field);
            self.field_observers = field_observers;
            self.fields_lang = lang.to_owned().into();
        }

        Self::sync_field(self.username_field.clone(), username, window, cx);
        Self::sync_field(self.display_name_field.clone(), display_name, window, cx);
    }

    /// Adopt an `AppSettings` value that changed outside the field (a reset),
    /// but never fight the user's cursor while the field is focused.
    fn sync_field(
        field: Option<Entity<TextField>>,
        value: &str,
        window: &Window,
        cx: &mut Context<Self>,
    ) {
        let Some(field) = field else {
            return;
        };
        let focused = field.read(cx).focus_handle(cx).is_focused(window);
        if !focused && field.read(cx).value() != value {
            field.update(cx, |field, cx| field.set_value(value.to_owned(), cx));
        }
    }
}

impl Render for ProfilePage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::of(cx).clone();
        let settings = AppSettings::global(cx);
        let lang: SharedString = settings.language.clone();
        let username_setting = settings.username.to_string();
        let display_name_setting = settings.display_name.to_string();

        self.ensure_fields(&lang, &username_setting, &display_name_setting, window, cx);

        let username_field = self
            .username_field
            .clone()
            .expect("ensure_fields creates the username editor");
        let display_name_field = self
            .display_name_field
            .clone()
            .expect("ensure_fields creates the display-name editor");
        let username_handle = username_field.read(cx).focus_handle(cx);
        let display_name_handle = display_name_field.read(cx).focus_handle(cx);
        let username_value = username_field.read(cx).value().to_owned();
        let display_name_value = display_name_field.read(cx).value().to_owned();

        let account_card = widgets::section_card(&theme)
            .mt_0()
            .child(
                widgets::card_row(&theme, true)
                    .child(row_text(
                        &theme,
                        tr("item.username", &lang),
                        tr("item.username.desc", &lang),
                        username_rule(&username_value).validation(&lang),
                    ))
                    .child(field_box(
                        username_field,
                        username_handle,
                        &theme,
                        "profile-username-field",
                        tr("item.username", &lang),
                    )),
            )
            .child(
                widgets::card_row(&theme, false)
                    .child(row_text(
                        &theme,
                        tr("item.display_name", &lang),
                        tr("item.display_name.desc", &lang),
                        None,
                    ))
                    .child(field_box(
                        display_name_field,
                        display_name_handle,
                        &theme,
                        "profile-display-name-field",
                        tr("item.display_name", &lang),
                    )),
            )
            .child(
                widgets::card_row(&theme, false)
                    .child(row_text(
                        &theme,
                        tr("item.preview", &lang),
                        tr("item.preview.desc", &lang),
                        None,
                    ))
                    .child(preview_chip(&theme, &display_name_value, &username_value)),
            );

        div()
            .id("profile-settings-page")
            .size_full()
            .overflow_y_scroll()
            .track_scroll(&self.scroll)
            .child(
                widgets::page_column()
                    .child(widgets::page_header(
                        &theme,
                        tr("page.profile", &lang),
                        None,
                    ))
                    .child(widgets::page_subtitle(
                        &theme,
                        tr("item.preview.desc", &lang),
                    ))
                    .child(widgets::section(
                        &theme,
                        tr("group.account", &lang),
                        account_card,
                    )),
            )
    }
}

/// The row's left column: zeron's title + muted meta, plus the inline
/// validation message under them when the field has one.
fn row_text(
    theme: &Theme,
    title: &'static str,
    meta: &'static str,
    validation: Option<(&'static str, bool)>,
) -> gpui::Div {
    let mut column = div()
        .flex_1()
        .min_w(px(160.0))
        .child(widgets::row_title(theme, title))
        .child(widgets::meta_line(
            theme,
            vec![div().child(meta).into_any_element()],
        ));
    if let Some((message, is_valid)) = validation {
        column = column.child(
            div()
                .mt(px(4.0))
                .text_size(typography::ui_rems(12.0))
                .line_height(typography::ui_rems(16.0))
                .text_color(if is_valid {
                    theme.success
                } else {
                    theme.danger
                })
                .child(message),
        );
    }
    column
}

/// A [`TextField`] in zeron's select-trigger box: the same 32px height,
/// 8px radius and wash fill, a transparent edge held for the focus ring, and
/// the field's own focus handle as the tab stop (`tab_index`/`focus_visible`
/// style the box while the caret lives in the field).
fn field_box(
    field: Entity<TextField>,
    handle: FocusHandle,
    theme: &Theme,
    id: &'static str,
    label: &'static str,
) -> gpui::Stateful<gpui::Div> {
    let accent = theme.accent;
    div()
        .id(id)
        .flex_none()
        .w(px(FIELD_WIDTH))
        .h(px(FIELD_HEIGHT))
        .px(px(10.0))
        .rounded(px(FIELD_RADIUS))
        .border_1()
        .border_color(gpui::transparent_black())
        .bg(theme.wash(0.06))
        .text_size(typography::ui_rems(12.5))
        .line_height(typography::ui_rems(16.0))
        .text_color(theme.text)
        .flex()
        .flex_row()
        .items_center()
        .cursor(CursorStyle::IBeam)
        .tab_index(0)
        .role(Role::TextInput)
        .aria_label(label)
        .focus_visible(move |style| style.border_color(accent))
        .track_focus(&handle)
        .child(div().flex_1().min_w_0().child(field))
}

/// The preview row's control: an avatar initial, the display name (falling
/// back to the username while empty) and the `@username` handle as a badge —
/// `Display Name (@username)` in the pre-port page, with a small identity
/// card's shape.
fn preview_chip(theme: &Theme, display_name: &str, username: &str) -> gpui::Div {
    let name = if display_name.is_empty() {
        username
    } else {
        display_name
    };
    let initial = name
        .chars()
        .next()
        .map(|c| c.to_uppercase().to_string())
        .unwrap_or_else(|| "?".to_owned());
    div()
        .flex_none()
        .h(px(FIELD_HEIGHT))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(8.0))
        .px(px(10.0))
        .rounded(px(FIELD_RADIUS))
        .bg(theme.wash(0.06))
        .child(
            div()
                .flex_none()
                .w(px(20.0))
                .h(px(20.0))
                .rounded_full()
                .bg(theme.accent_strong)
                .flex()
                .items_center()
                .justify_center()
                .text_size(typography::ui_rems(11.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.on_accent)
                .child(initial),
        )
        .child(
            div()
                .text_size(typography::ui_rems(12.5))
                .text_color(theme.text)
                .child(name.to_owned()),
        )
        .child(widgets::badge(theme, format!("@{username}")))
}

#[cfg(test)]
mod tests {
    use super::{UsernameRule, username_rule};

    #[test]
    fn username_rules() {
        assert_eq!(username_rule(""), UsernameRule::Empty);
        assert_eq!(username_rule("ab"), UsernameRule::TooShort);
        assert_eq!(username_rule("abcdefghijklmnopq"), UsernameRule::TooLong);
        assert_eq!(username_rule("Ninja"), UsernameRule::InvalidCharacter);
        assert_eq!(username_rule("ninja!"), UsernameRule::InvalidCharacter);
        assert_eq!(username_rule("ninja"), UsernameRule::Valid);
        assert_eq!(username_rule("a-1"), UsernameRule::Valid);
        assert_eq!(username_rule("a_b_2"), UsernameRule::Valid);
    }
}
