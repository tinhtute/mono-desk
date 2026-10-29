//! The Reset page (`settings-surface` T2): zeron's page scaffolding with
//! mono-desk's own content — every setting that differs from its default with
//! a way back, plus the two-press "Reset all".
//!
//! Ported from the pre-port page (`wip/gpui-kit-shell:src/main.rs`,
//! `changed_settings` ~1325 and `reset_page` ~1494): the changed rows carry
//! their `current → default` words and one quiet reset action, and the bulk
//! action arms on the first press (writing nothing) then reverts everything on
//! the second. With nothing changed the card says so instead of drawing an
//! empty block, and the bulk action stays inert (the pre-port page disabled it
//! in the same state).
//!
//! Two deliberate carry-overs from the pre-port list: `sidebar_width` is not
//! listed (nothing on the zeron shell resizes the rail; the old page did not
//! list it either), and the three channel switches are one row — they are one
//! setting split over three switches, so "reset" means resetting the group.

use gpui::prelude::*;
use gpui::{Context, IntoElement, Role, ScrollHandle, SharedString, Window, div, px};

use zeron_ui::settings::widgets::{self, ActionTone};
use zeron_ui::theme::Theme;

use crate::app_settings::{AppSettings, accent_to_str};
use crate::i18n::tr;

/// Every setting the page lists, in the store's declaration order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Setting {
    Appearance,
    Accent,
    FontSize,
    BackgroundTranslucent,
    WindowOpacity,
    SidebarTone,
    BackgroundBlur,
    Language,
    LaunchAtLogin,
    Notifications,
    Username,
    DisplayName,
    Channels,
    Digest,
    Timezone,
}

impl Setting {
    /// The same order (and grouping) as the pre-port `changed_settings`.
    const ALL: [Self; 15] = [
        Self::Appearance,
        Self::Accent,
        Self::FontSize,
        Self::BackgroundTranslucent,
        Self::WindowOpacity,
        Self::SidebarTone,
        Self::BackgroundBlur,
        Self::Language,
        Self::LaunchAtLogin,
        Self::Notifications,
        Self::Username,
        Self::DisplayName,
        Self::Channels,
        Self::Digest,
        Self::Timezone,
    ];

    /// The row title's i18n key (the same `item.*` keys the pre-port page
    /// used, so the reset rows name exactly what the pages call the setting).
    fn title_key(self) -> &'static str {
        match self {
            Self::Appearance => "item.appearance",
            Self::Accent => "item.accent",
            Self::FontSize => "item.font_size",
            Self::BackgroundTranslucent => "item.background_translucent",
            Self::WindowOpacity => "item.window_opacity",
            Self::SidebarTone => "item.sidebar_tone",
            Self::BackgroundBlur => "item.background_blur",
            Self::Language => "item.language",
            Self::LaunchAtLogin => "item.launch_at_login",
            Self::Notifications => "item.notifications",
            Self::Username => "item.username",
            Self::DisplayName => "item.display_name",
            Self::Channels => "item.channels",
            Self::Digest => "item.digest",
            Self::Timezone => "item.timezone",
        }
    }

    /// Whether the setting's *effective* value differs from the default — a
    /// hand-edited config that already equals the default is not "changed".
    fn differs(self, now: &AppSettings, default: &AppSettings) -> bool {
        match self {
            Self::Appearance => now.appearance != default.appearance,
            Self::Accent => now.accent != default.accent,
            Self::FontSize => now.font_size != default.font_size,
            Self::BackgroundTranslucent => {
                now.background_translucent != default.background_translucent
            }
            Self::WindowOpacity => now.window_opacity != default.window_opacity,
            Self::SidebarTone => now.sidebar_tone != default.sidebar_tone,
            Self::BackgroundBlur => now.background_blur != default.background_blur,
            Self::Language => now.language != default.language,
            Self::LaunchAtLogin => now.launch_at_login != default.launch_at_login,
            Self::Notifications => now.notifications_enabled != default.notifications_enabled,
            Self::Username => now.username != default.username,
            Self::DisplayName => now.display_name != default.display_name,
            // One setting split over three switches (see the module docs).
            Self::Channels => {
                now.channel_email != default.channel_email
                    || now.channel_desktop != default.channel_desktop
                    || now.channel_digest != default.channel_digest
            }
            Self::Digest => now.digest != default.digest,
            Self::Timezone => now.timezone != default.timezone,
        }
    }

    /// The setting's value as the word the row shows, in the current UI
    /// language (the pre-port page formatted the default side in the *default*
    /// language; one page is readable in only one locale, so both sides use
    /// `lang`).
    fn words(self, settings: &AppSettings, lang: &str) -> String {
        match self {
            Self::Appearance => tr(settings.appearance.label_key(), lang).to_string(),
            Self::Accent => settings
                .accent
                .map(accent_to_str)
                .unwrap_or_else(|| tr("reset.theme_default", lang).to_string()),
            Self::FontSize => format!("{:.0} px", settings.font_size),
            Self::BackgroundTranslucent => on_off(settings.background_translucent, lang),
            Self::WindowOpacity => percent_text(settings.window_opacity, lang),
            Self::SidebarTone => percent_text(settings.sidebar_tone, lang),
            Self::BackgroundBlur => on_off(settings.background_blur, lang),
            Self::Language => settings.language.to_string(),
            Self::LaunchAtLogin => on_off(settings.launch_at_login, lang),
            Self::Notifications => on_off(settings.notifications_enabled, lang),
            Self::Username => settings.username.to_string(),
            Self::DisplayName => settings.display_name.to_string(),
            Self::Channels => channel_summary(
                settings.channel_email,
                settings.channel_desktop,
                settings.channel_digest,
                lang,
            ),
            Self::Digest => tr(settings.digest.label_key(), lang).to_string(),
            Self::Timezone => settings.timezone.to_string(),
        }
    }

    /// Revert this setting (or, for [`Self::Channels`], the whole channel
    /// group) to `default`, leaving every other field alone.
    fn reset(self, settings: &mut AppSettings, default: &AppSettings) {
        match self {
            Self::Appearance => settings.appearance = default.appearance,
            Self::Accent => settings.accent = default.accent,
            Self::FontSize => settings.font_size = default.font_size,
            Self::BackgroundTranslucent => {
                settings.background_translucent = default.background_translucent;
            }
            Self::WindowOpacity => settings.window_opacity = default.window_opacity,
            Self::SidebarTone => settings.sidebar_tone = default.sidebar_tone,
            Self::BackgroundBlur => settings.background_blur = default.background_blur,
            Self::Language => settings.language = default.language.clone(),
            Self::LaunchAtLogin => settings.launch_at_login = default.launch_at_login,
            Self::Notifications => {
                settings.notifications_enabled = default.notifications_enabled;
            }
            Self::Username => settings.username = default.username.clone(),
            Self::DisplayName => settings.display_name = default.display_name.clone(),
            Self::Channels => {
                settings.channel_email = default.channel_email;
                settings.channel_desktop = default.channel_desktop;
                settings.channel_digest = default.channel_digest;
            }
            Self::Digest => settings.digest = default.digest,
            Self::Timezone => settings.timezone = default.timezone.clone(),
        }
    }
}

/// One setting whose current value differs from its default.
///
/// The values are already-formatted words: the page only has to show them,
/// not compare them.
struct ChangedRow {
    setting: Setting,
    current: String,
    default: String,
}

/// Every setting that differs from [`AppSettings::default()`], in
/// [`Setting::ALL`] order. Computed app-side so it cannot drift from the
/// fields the pages actually write.
fn changed_rows(now: &AppSettings, lang: &str) -> Vec<ChangedRow> {
    let default = AppSettings::default();
    Setting::ALL
        .into_iter()
        .filter(|setting| setting.differs(now, &default))
        .map(|setting| ChangedRow {
            setting,
            current: setting.words(now, lang),
            default: setting.words(&default, lang),
        })
        .collect()
}

/// `On`/`Off` in the owner's language.
fn on_off(value: bool, lang: &str) -> String {
    tr(if value { "state.on" } else { "state.off" }, lang).to_string()
}

/// `NN%` in the owner's language — the percent sign is a translatable unit,
/// not a hardcoded ASCII glyph (same shape as the pre-port `percent_text`).
fn percent_text(value: f64, lang: &str) -> String {
    format!("{value:.0}{}", tr("unit.percent", lang))
}

/// The three channel switches as one `Email: On · Desktop: On · Digest: Off`
/// line (the pre-port channel summary's shape).
fn channel_summary(email: bool, desktop: bool, digest: bool, lang: &str) -> String {
    format!(
        "{}: {} · {}: {} · {}: {}",
        tr("item.channel_email", lang),
        on_off(email, lang),
        tr("item.channel_desktop", lang),
        on_off(desktop, lang),
        tr("item.channel_digest", lang),
        on_off(digest, lang),
    )
}

/// Enter/Space activate a focused action, like zeron's own interactive rows.
fn is_activation(event: &gpui::KeyDownEvent) -> bool {
    !event.is_held && matches!(event.keystroke.key.as_str(), "enter" | "space")
}

pub struct ResetPage {
    /// The page's scroll offset (zeron's pages own their scroll handle).
    scroll: ScrollHandle,
    /// The two-press "Reset all": the first press only arms it, and nothing
    /// is written until the second. Held on the view so it survives re-renders
    /// (the pre-port page kept it in window-keyed state).
    armed: bool,
}

impl ResetPage {
    pub fn new() -> Self {
        Self {
            scroll: ScrollHandle::new(),
            armed: false,
        }
    }

    /// Revert one changed setting. Every row funnels through the appearance
    /// path as well: accent, font size and the glass fields are live theme
    /// inputs, and the funnel is idempotent for the rows that are not.
    fn reset_one(&mut self, setting: Setting, window: &mut Window, cx: &mut Context<Self>) {
        self.armed = false;
        let default = AppSettings::default();
        setting.reset(AppSettings::global_mut(cx), &default);
        super::persist_now(cx);
        super::apply_appearance(window, cx);
        super::apply_launch_at_login(cx);
        cx.notify();
    }

    /// "Reset all", second press: back to the shipped defaults, in one pass.
    fn reset_all(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.armed = false;
        let default = AppSettings::default();
        *AppSettings::global_mut(cx) = default;
        super::persist_now(cx);
        // Appearance/font size/glass are theme inputs, so the reset re-applies
        // them (and the native window background) exactly like a settings
        // write does.
        super::apply_appearance(window, cx);
        super::apply_launch_at_login(cx);
        cx.notify();
    }

    /// First press arms, second reverts. Inert when there is nothing to
    /// revert (the row does not attach this handler in that state either).
    fn press_reset_all(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let has_changes = !changed_rows(AppSettings::global(cx), &lang(cx)).is_empty();
        if !has_changes {
            return;
        }
        if self.armed {
            self.reset_all(window, cx);
        } else {
            self.armed = true;
            cx.notify();
        }
    }
}

/// The current UI language, typed for `tr`.
fn lang(cx: &gpui::App) -> SharedString {
    AppSettings::global(cx).language.clone()
}

impl Render for ResetPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::of(cx).clone();
        let language = lang(cx);
        let rows = changed_rows(AppSettings::global(cx), &language);
        let has_changes = !rows.is_empty();
        // A reset elsewhere (or the last per-row reset) must not leave a
        // primed confirm button behind.
        if !has_changes {
            self.armed = false;
        }

        let mut card = widgets::section_card(&theme).mt_0();
        if rows.is_empty() {
            card = card.child(
                widgets::card_row(&theme, true).child(
                    div()
                        .flex_1()
                        .min_w(px(160.0))
                        .child(widgets::row_title(&theme, tr("reset.nothing", &language)))
                        .child(widgets::meta_line(
                            &theme,
                            vec![
                                div()
                                    .child(tr("reset.nothing.desc", &language))
                                    .into_any_element(),
                            ],
                        )),
                ),
            );
        } else {
            for (ix, row) in rows.into_iter().enumerate() {
                let setting = row.setting;
                let action_id: SharedString = format!("reset-row-action-{ix}").into();
                let reset_label = tr("action.reset", &language);
                card = card.child(super::control_row(
                    &theme,
                    ix == 0,
                    tr(setting.title_key(), &language),
                    format!("{} → {}", row.current, row.default),
                    widgets::text_action(&theme, ActionTone::Quiet, reset_label)
                        .id(action_id)
                        .tab_index(0)
                        .role(Role::Button)
                        .aria_label(format!(
                            "{reset_label} {}",
                            tr(setting.title_key(), &language)
                        ))
                        .focus_visible(|style| style.border_2().border_color(theme.accent))
                        .cursor_pointer()
                        .on_key_down(cx.listener(
                            move |page, event: &gpui::KeyDownEvent, window, cx| {
                                if is_activation(event) {
                                    cx.stop_propagation();
                                    page.reset_one(setting, window, cx);
                                }
                            },
                        ))
                        .on_click(cx.listener(move |page, _, window, cx| {
                            page.reset_one(setting, window, cx)
                        })),
                ));
            }
        }

        // "Reset all" is the last row of the same block (the pre-port page
        // kept it inside the changed group): solid before arming, danger
        // once armed, and inert while nothing has changed.
        let (tone, bulk_label) = if self.armed {
            (ActionTone::Danger, tr("reset.all.confirm", &language))
        } else {
            (ActionTone::Solid, tr("reset.all", &language))
        };
        let mut bulk = widgets::text_action(&theme, tone, bulk_label)
            .id("reset-all")
            .role(Role::Button)
            .aria_label(bulk_label)
            .focus_visible(|style| style.border_2().border_color(theme.accent));
        bulk = if has_changes {
            bulk.tab_index(0)
                .cursor_pointer()
                .on_key_down(cx.listener(|page, event: &gpui::KeyDownEvent, window, cx| {
                    if is_activation(event) {
                        cx.stop_propagation();
                        page.press_reset_all(window, cx);
                    }
                }))
                .on_click(cx.listener(|page, _, window, cx| page.press_reset_all(window, cx)))
        } else {
            bulk.tab_index(-1)
                .cursor_not_allowed()
                .opacity(0.55)
                .aria_description(tr("reset.nothing.desc", &language))
        };
        card = card.child(super::control_row(
            &theme,
            false,
            tr("reset.all", &language),
            tr("reset.all.desc", &language),
            bulk,
        ));

        div()
            .id("reset-settings-page")
            .size_full()
            .overflow_y_scroll()
            .track_scroll(&self.scroll)
            .child(
                widgets::page_column()
                    .child(widgets::page_header(
                        &theme,
                        tr("page.reset", &language),
                        None,
                    ))
                    .child(widgets::section(
                        &theme,
                        tr("group.changed", &language),
                        card,
                    )),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fresh store has nothing to revert, so the page shows its empty state.
    #[test]
    fn default_settings_have_no_changed_rows() {
        assert!(changed_rows(&AppSettings::default(), "en").is_empty());
    }

    /// Only the changed fields are listed, in the store's order, with the
    /// current word first and the default word second.
    #[test]
    fn changed_rows_name_the_field_and_its_two_words() {
        let settings = AppSettings {
            language: "vi".into(),
            background_translucent: true,
            channel_digest: true,
            ..AppSettings::default()
        };
        let rows = changed_rows(&settings, "vi");
        let keys: Vec<_> = rows.iter().map(|row| row.setting.title_key()).collect();
        assert_eq!(
            keys,
            [
                "item.background_translucent",
                "item.language",
                "item.channels",
            ]
        );
        assert_eq!(rows[0].current, tr("state.on", "vi"));
        assert_eq!(rows[0].default, tr("state.off", "vi"));
        assert_eq!(rows[1].current, "vi");
        assert_eq!(rows[1].default, "en");
        // The channel row is the summary of all three switches: one off in
        // the default set, none off in this one.
        let off = tr("state.off", "vi");
        assert_eq!(rows[2].current.matches(off).count(), 0);
        assert_eq!(rows[2].default.matches(off).count(), 1);
    }

    /// A per-row reset restores only its own field, and the row disappears.
    #[test]
    fn resetting_a_row_restores_only_that_field() {
        let default = AppSettings::default();
        let mut settings = AppSettings {
            font_size: 22.0,
            language: "ja".into(),
            ..AppSettings::default()
        };
        assert_eq!(changed_rows(&settings, "en").len(), 2);
        Setting::FontSize.reset(&mut settings, &default);
        assert_eq!(settings.font_size, default.font_size);
        assert_eq!(settings.language, "ja");
        let rows = changed_rows(&settings, "en");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].setting, Setting::Language);
    }

    /// A stored accent shows its persisted word; the default side says
    /// "theme default" instead of a color.
    #[test]
    fn accent_row_uses_the_persisted_word_or_the_theme_default() {
        let default = AppSettings::default();
        let mut settings = AppSettings {
            accent: crate::app_settings::accent_from_str("#124b68"),
            ..AppSettings::default()
        };
        let rows = changed_rows(&settings, "en");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].setting, Setting::Accent);
        assert!(rows[0].current.starts_with('#'), "{}", rows[0].current);
        assert_eq!(rows[0].default, tr("reset.theme_default", "en"));
        Setting::Accent.reset(&mut settings, &default);
        assert_eq!(settings.accent, None);
        assert!(changed_rows(&settings, "en").is_empty());
    }
}
