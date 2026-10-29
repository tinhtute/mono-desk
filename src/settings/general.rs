//! The General page (`settings-surface` T1): zeron's page scaffolding with
//! mono-desk's own content — region (language, timezone) and startup (launch
//! at login).
//!
//! The page is a view, like zeron's own settings pages, so it can own the open
//! state of its two [`select`] controls ([`SelectState`]) across frames. The
//! rows are zeron's `section`/`section_card`/`card_row`/`row_title`/`meta_line`
//! over `toggle_switch`/`select`; strings come from mono-desk's i18n table and
//! writes go to the `AppSettings` global + the persisted file.

use gpui::prelude::*;
use gpui::{Context, IntoElement, Render, Role, ScrollHandle, SharedString, Toggled, Window, div};

use zeron_ui::settings::select::{SelectOption, SelectState, select};
use zeron_ui::settings::widgets;
use zeron_ui::theme::Theme;

use crate::app_settings::AppSettings;
use crate::i18n::tr;

/// The language table: persisted code + endonym shown in the select (the same
/// four locales mono-desk has always shipped, in their own writing systems).
const LANGUAGES: [(&str, &str); 4] = [
    ("en", "English"),
    ("vi", "Tiếng Việt"),
    ("zh-CN", "简体中文"),
    ("ja", "日本語"),
];

/// The curated timezone set the old combobox offered: a localized
/// follow-the-system sentinel plus IANA zones spanning the offset range.
/// Index 0 is the sentinel, persisted as `system`.
const TIMEZONES: [&str; 21] = [
    "system",
    "Pacific/Auckland",
    "Pacific/Honolulu",
    "America/Anchorage",
    "America/Los_Angeles",
    "America/Denver",
    "America/Chicago",
    "America/New_York",
    "America/Sao_Paulo",
    "Atlantic/Reykjavik",
    "Europe/London",
    "Europe/Berlin",
    "Europe/Kyiv",
    "Africa/Cairo",
    "Asia/Dubai",
    "Asia/Kolkata",
    "Asia/Bangkok",
    "Asia/Saigon",
    "Asia/Singapore",
    "Asia/Tokyo",
    "Australia/Sydney",
];

pub struct GeneralPage {
    /// Open state of the language select.
    language: SelectState,
    /// Open state of the timezone select.
    timezone: SelectState,
    /// The page's scroll offset (zeron's pages own their scroll handle).
    scroll: ScrollHandle,
}

impl GeneralPage {
    pub fn new() -> Self {
        Self {
            language: SelectState::default(),
            timezone: SelectState::default(),
            scroll: ScrollHandle::new(),
        }
    }

    fn set_language(&mut self, ix: usize, cx: &mut Context<Self>) {
        let code: SharedString = LANGUAGES[ix].0.into();
        AppSettings::global_mut(cx).language = code;
        super::persist_now(cx);
        cx.notify();
    }

    fn set_timezone(&mut self, ix: usize, cx: &mut Context<Self>) {
        let zone: SharedString = TIMEZONES[ix].into();
        AppSettings::global_mut(cx).timezone = zone;
        super::persist_now(cx);
        cx.notify();
    }

    fn toggle_launch_at_login(&mut self, cx: &mut Context<Self>) {
        let enabled = AppSettings::global(cx).launch_at_login;
        AppSettings::global_mut(cx).launch_at_login = !enabled;
        super::persist_now(cx);
        // Converge the real autostart entry (pre-port R5) on every change.
        super::apply_launch_at_login(cx);
        cx.notify();
    }
}

impl Render for GeneralPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::of(cx).clone();
        let settings = AppSettings::global(cx);
        let lang: SharedString = settings.language.clone();
        let timezone: SharedString = settings.timezone.clone();
        let launch_at_login = settings.launch_at_login;

        let language_ix = LANGUAGES
            .iter()
            .position(|(code, _)| *code == lang.as_ref())
            .unwrap_or_default();
        let language = select(
            "general-language",
            tr("item.language", &lang),
            &theme,
            |page: &mut Self| &mut page.language,
        )
        .options(
            LANGUAGES.iter().map(|(_, name)| SelectOption::new(*name)),
            language_ix,
        )
        .width(176.0)
        .on_select(|page: &mut Self, ix, _window, cx| page.set_language(ix, cx))
        .render(&self.language, cx);

        let timezone_ix = TIMEZONES
            .iter()
            .position(|zone| *zone == timezone.as_ref())
            .unwrap_or_default();
        let timezone_options = TIMEZONES.iter().enumerate().map(|(ix, zone)| {
            let label = if ix == 0 {
                tr("tz.system", &lang)
            } else {
                zone
            };
            SelectOption::new(label)
        });
        let timezone = select(
            "general-timezone",
            tr("item.timezone", &lang),
            &theme,
            |page: &mut Self| &mut page.timezone,
        )
        .options(timezone_options, timezone_ix)
        .width(224.0)
        .menu_width(224.0)
        .heading(tr("item.timezone", &lang))
        .on_select(|page: &mut Self, ix, _window, cx| page.set_timezone(ix, cx))
        .render(&self.timezone, cx);

        let launch = widgets::toggle_switch(&theme, launch_at_login, "launch-at-login")
            .id("launch-at-login-toggle")
            .tab_index(0)
            .role(Role::Switch)
            .aria_label(tr("item.launch_at_login", &lang))
            .aria_toggled(if launch_at_login {
                Toggled::True
            } else {
                Toggled::False
            })
            .focus_visible(|style| style.border_2().border_color(theme.accent))
            .cursor_pointer()
            .on_click(cx.listener(|page, _, _, cx| page.toggle_launch_at_login(cx)));

        let region = widgets::section_card(&theme)
            .mt_0()
            .child(super::control_row(
                &theme,
                true,
                tr("item.language", &lang),
                tr("item.language.desc", &lang),
                language,
            ))
            .child(super::control_row(
                &theme,
                false,
                tr("item.timezone", &lang),
                tr("item.timezone.desc", &lang),
                timezone,
            ));

        let startup = widgets::section_card(&theme)
            .mt_0()
            .child(super::control_row(
                &theme,
                true,
                tr("item.launch_at_login", &lang),
                tr("item.launch_at_login.desc", &lang),
                launch,
            ));

        div()
            .id("general-settings-page")
            .size_full()
            .overflow_y_scroll()
            .track_scroll(&self.scroll)
            .child(
                widgets::page_column()
                    .child(widgets::page_header(
                        &theme,
                        tr("page.general", &lang),
                        None,
                    ))
                    .child(widgets::section(&theme, tr("group.region", &lang), region))
                    .child(widgets::section(
                        &theme,
                        tr("group.startup", &lang),
                        startup,
                    )),
            )
    }
}
