use gpui_kit::component::{
<<<<<<< HEAD
    ActiveTheme, Icon, IconName, Root, Sizable, Theme, ThemeMode, TitleBar, WindowExt,
    button::Button,
=======
    ActiveTheme, Disableable, Icon, IconName, Root, Sizable, Theme, ThemeMode, WindowExt,
    button::{Button, ButtonVariants},
>>>>>>> origin/main
    h_flex,
    label::Label,
    notification::Notification,
    setting::{NumberFieldOptions, SettingField, SettingGroup, SettingItem, SettingPage, Settings},
    switch::Switch,
    v_flex,
};
use gpui_kit::{
    App, AppContext as _, Bounds, Context, FocusHandle, Focusable, FontWeight, Global, IntoElement,
    KeyBinding, Menu, MenuItem, ParentElement as _, Render, SharedString, StyleRefinement,
    Styled as _, TitlebarOptions, Window, WindowBounds, WindowDecorations, WindowOptions, div, img,
    point, px, size,
};
use rust_i18n::t;
use shell::{AppShell, ShellState};
use status_chip::{StatusChip, StatusIcon, StatusTone};

mod assets;
<<<<<<< HEAD
mod shell;
mod status_chip;
mod theme;

// App translations, loaded with the same stack the vendored gpui-component
// uses (`rust_i18n`). `rust_i18n::set_locale` is global, so switching it
// re-renders both these strings and the component-internal ones (search
// placeholder, dialog buttons, …). `fallback = "en"` covers locales the
// component ships no column for (vi, ja).
rust_i18n::i18n!("locales", fallback = "en");

gpui_kit::actions!(mono_desk, [Quit, CloseWindow, ToggleSidebar]);

/// Translate `key` for the current locale into an owned `String`.
///
/// The component builders take `impl Into<SharedString>`, which `Cow<str>`
/// does not satisfy directly, so the macro owns the result.
macro_rules! tr {
    ($key:literal) => {
        t!($key).to_string()
    };
}

/// Map the Language setting's value to a `rust_i18n` locale code.
///
/// The dropdown offers `zh`; the vendored component locale file keys Chinese
/// as `zh-CN`, so the two are bridged here — otherwise component-internal
/// strings would fall back to English for Chinese.
fn locale_for(language: &str) -> &str {
    match language {
        "zh" => "zh-CN",
        other => other,
    }
}
=======
mod i18n;
mod persist;
mod theme;

use i18n::tr;
use persist::StoredSettings;

gpui_kit::actions!(mono_desk, [Quit]);
>>>>>>> origin/main

/// Application state, stored as a GPUI global so `SettingField` closures can
/// reach it through `&App` / `&mut App`.
struct AppSettings {
    dark_mode: bool,
    language: SharedString,
    font_size: f64,
    launch_at_login: bool,
    notifications_enabled: bool,
    username: SharedString,
    display_name: SharedString,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            dark_mode: false,
            language: "en".into(),
            // Matches the theme's `font.size` (13px) so the applied default
            // preserves the approved visual baseline.
            font_size: 13.0,
            launch_at_login: false,
            notifications_enabled: true,
            username: "ninja".into(),
            display_name: "Ninja".into(),
        }
    }
}

impl Global for AppSettings {}

impl AppSettings {
    fn global(cx: &App) -> &Self {
        cx.global::<AppSettings>()
    }

    fn global_mut(cx: &mut App) -> &mut Self {
        cx.global_mut::<AppSettings>()
    }
}

impl AppSettings {
    fn from_stored(s: StoredSettings) -> Self {
        Self {
            dark_mode: s.dark_mode,
            language: s.language.into(),
            font_size: if s.font_size.is_finite() {
                s.font_size
            } else {
                14.0
            },
            launch_at_login: s.launch_at_login,
            notifications_enabled: s.notifications_enabled,
            username: s.username.into(),
            display_name: s.display_name.into(),
        }
    }

    fn to_stored(&self) -> StoredSettings {
        StoredSettings {
            dark_mode: self.dark_mode,
            language: self.language.to_string(),
            font_size: self.font_size,
            launch_at_login: self.launch_at_login,
            notifications_enabled: self.notifications_enabled,
            username: self.username.to_string(),
            display_name: self.display_name.to_string(),
        }
    }
}

/// Persists the current settings; errors are logged inside `persist::save`.
fn save_settings(cx: &App) {
    persist::save(&AppSettings::global(cx).to_stored());
}


struct SettingsWindow {
    focus_handle: FocusHandle,
}

impl SettingsWindow {
    fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
<<<<<<< HEAD
        cx.set_global(AppSettings::default());
        cx.set_global(ShellState::default());
=======
>>>>>>> origin/main
        Self {
            focus_handle: cx.focus_handle(),
        }
    }

    fn setting_pages(&self, cx: &mut Context<Self>) -> Vec<SettingPage> {
        let lang = AppSettings::global(cx).language.clone();
        let defaults = AppSettings::default();
        // Copied out of `defaults` so the reset closures of the element fields
        // below can be `'static`.
        let auto_update_default = defaults.auto_update;
        let notifications_default = defaults.notifications_enabled;

        vec![
<<<<<<< HEAD
            SettingPage::new(tr!("Page.general"))
=======
            SettingPage::new(tr("page.general", &lang))
>>>>>>> origin/main
                .icon(Icon::new(IconName::Settings2))
                .resettable(true)
                .groups(vec![
<<<<<<< HEAD
                    SettingGroup::new()
                        .title(tr!("Group.appearance"))
                        .items(vec![
                            SettingItem::new(
                                tr!("Item.dark_mode"),
                                SettingField::switch(
                                    |cx: &App| cx.theme().mode.is_dark(),
                                    |val: bool, cx: &mut App| {
                                        let mode = if val {
                                            ThemeMode::Dark
                                        } else {
                                            ThemeMode::Light
                                        };
                                        Theme::global_mut(cx).mode = mode;
                                        Theme::change(mode, None, cx);
                                        // `Theme::change` rebuilds the theme from
                                        // the config, so re-apply the app-side
                                        // overrides (font size, focus ring).
                                        theme::apply_overrides(
                                            AppSettings::global(cx).font_size,
                                            cx,
                                        );
                                    },
                                )
                                .default_value(false),
                            )
                            .description(tr!("Item.dark_mode_desc")),
                            SettingItem::new(
                                tr!("Item.language"),
                                SettingField::dropdown(
                                    vec![
                                        ("en".into(), "English".into()),
                                        ("vi".into(), "Tiếng Việt".into()),
                                        ("zh".into(), "简体中文".into()),
                                        ("ja".into(), "日本語".into()),
                                    ],
                                    |cx: &App| AppSettings::global(cx).language.clone(),
                                    |val: SharedString, cx: &mut App| {
                                        rust_i18n::set_locale(locale_for(&val));
                                        AppSettings::global_mut(cx).language = val;
                                    },
                                )
                                .default_value(defaults.language.clone()),
                            )
                            .description(tr!("Item.language_desc")),
                            SettingItem::new(
                                tr!("Item.font_size"),
                                SettingField::number_input(
                                    NumberFieldOptions {
                                        min: theme::FONT_SIZE_MIN,
                                        max: theme::FONT_SIZE_MAX,
                                        ..Default::default()
                                    },
                                    |cx: &App| AppSettings::global(cx).font_size,
                                    |val: f64, cx: &mut App| {
                                        let val = theme::clamp_font_size(val);
                                        AppSettings::global_mut(cx).font_size = val;
                                        theme::apply_overrides(val, cx);
                                    },
                                )
                                .default_value(defaults.font_size),
                            )
                            .description(tr!("Item.font_size_desc")),
                        ]),
                    SettingGroup::new().title(tr!("Group.startup")).items(vec![
                        SettingItem::new(
                            tr!("Item.launch_at_login"),
=======
                    SettingGroup::new().title(tr("group.appearance", &lang)).items(vec![
                        SettingItem::new(
                            tr("item.dark_mode", &lang),
                            SettingField::switch(
                                |cx: &App| cx.theme().mode.is_dark(),
                                |val: bool, cx: &mut App| {
                                    let mode = if val {
                                        ThemeMode::Dark
                                    } else {
                                        ThemeMode::Light
                                    };
                                    Theme::change(mode, None, cx);
                                    AppSettings::global_mut(cx).dark_mode = val;
                                    apply_font_size(cx);
                                    save_settings(cx);
                                    cx.refresh_windows();
                                },
                            )
                            .default_value(false),
                        )
                        .description(tr("item.dark_mode.desc", &lang)),
                        SettingItem::new(
                            tr("item.language", &lang),
                            SettingField::dropdown(
                                vec![
                                    ("en".into(), "English".into()),
                                    ("vi".into(), "Tiếng Việt".into()),
                                    ("zh-CN".into(), "简体中文".into()),
                                    ("ja".into(), "日本語".into()),
                                ],
                                |cx: &App| AppSettings::global(cx).language.clone(),
                                |val: SharedString, cx: &mut App| {
                                    AppSettings::global_mut(cx).language = val.clone();
                                    gpui_kit::component::set_locale(component_locale(&val));
                                    save_settings(cx);
                                    cx.refresh_windows();
                                },
                            )
                            .default_value(defaults.language.clone()),
                        )
                        .description(tr("item.language.desc", &lang)),
                        SettingItem::new(
                            tr("item.font_size", &lang),
                            SettingField::number_input(
                                NumberFieldOptions {
                                    min: 8.0,
                                    max: 32.0,
                                    ..Default::default()
                                },
                                |cx: &App| AppSettings::global(cx).font_size,
                                |val: f64, cx: &mut App| {
                                    let Some(size) = normalize_font_size(val) else {
                                        return;
                                    };
                                    AppSettings::global_mut(cx).font_size = size;
                                    apply_font_size(cx);
                                    save_settings(cx);
                                    cx.refresh_windows();
                                },
                            )
                            .default_value(defaults.font_size),
                        )
                        .description(tr("item.font_size.desc", &lang)),
                    ]),
                    SettingGroup::new().title(tr("group.startup", &lang)).items(vec![
                        SettingItem::new(
                            tr("item.launch_at_login", &lang),
>>>>>>> origin/main
                            SettingField::checkbox(
                                |cx: &App| AppSettings::global(cx).launch_at_login,
                                |val: bool, cx: &mut App| {
                                    AppSettings::global_mut(cx).launch_at_login = val;
                                    persist::autostart_converge(val);
                                    save_settings(cx);
                                },
                            )
                            .default_value(defaults.launch_at_login),
                        )
<<<<<<< HEAD
                        .description(tr!("Item.launch_at_login_desc")),
                        SettingItem::new(
                            tr!("Item.auto_update"),
                            SettingField::render(|options, _window, cx| {
                                let enabled = AppSettings::global(cx).auto_update;
                                // Auto update is either up to date or held
                                // back, so the chip carries the tone's own
                                // status name ("Ready" / "Paused").
                                let tone = if enabled {
                                    StatusTone::Ready
                                } else {
                                    StatusTone::Paused
                                };
                                h_flex().gap_2().child(StatusChip::new(tone)).child(
                                    Switch::new("auto-update")
                                        .checked(enabled)
                                        .with_size(options.size())
                                        .on_click(|checked: &bool, _, cx: &mut App| {
                                            AppSettings::global_mut(cx).auto_update = *checked;
                                        }),
                                )
                            })
                            // An element field carries no typed value, so
                            // the page reset needs the handler explicitly.
                            .on_reset(
                                move |cx: &App| {
                                    AppSettings::global(cx).auto_update != auto_update_default
                                },
                                move |_window, cx: &mut App| {
                                    AppSettings::global_mut(cx).auto_update = auto_update_default;
                                },
                            ),
                        )
                        .description(tr!("Item.auto_update_desc")),
                    ]),
                ]),
            SettingPage::new(tr!("Page.notifications"))
                .icon(Icon::new(IconName::Bell))
                .resettable(true)
                .groups(vec![
                    SettingGroup::new().title(tr!("Group.delivery")).items(vec![
                        SettingItem::new(
                            tr!("Item.notifications_enabled"),
                            SettingField::render(|options, _window, cx| {
                                let enabled = AppSettings::global(cx).notifications_enabled;
                                let (tone, label) = if enabled {
                                    (StatusTone::Ready, tr!("Status.enabled"))
                                } else {
                                    (StatusTone::Paused, tr!("Status.disabled"))
                                };
                                h_flex()
                                    .gap_2()
                                    .child(StatusChip::labelled(tone, label))
                                    .child(
                                        Switch::new("notifications-enabled")
                                            .checked(enabled)
                                            .with_size(options.size())
                                            .on_click(|checked: &bool, _, cx: &mut App| {
                                                AppSettings::global_mut(cx).notifications_enabled =
                                                    *checked;
                                            }),
                                    )
                            })
                            .on_reset(
                                move |cx: &App| {
                                    AppSettings::global(cx).notifications_enabled
                                        != notifications_default
=======
                        .description(tr("item.launch_at_login.desc", &lang)),
                    ]),
                ]),
            SettingPage::new(tr("page.notifications", &lang))
                .icon(Icon::new(IconName::Bell))
                .resettable(true)
                .groups(vec![
                    SettingGroup::new().title(tr("group.delivery", &lang)).items(vec![
                        SettingItem::new(
                            tr("item.notifications", &lang),
                            SettingField::switch(
                                |cx: &App| AppSettings::global(cx).notifications_enabled,
                                |val: bool, cx: &mut App| {
                                    AppSettings::global_mut(cx).notifications_enabled = val;
                                    save_settings(cx);
                                    cx.refresh_windows();
>>>>>>> origin/main
                                },
                                move |_window, cx: &mut App| {
                                    AppSettings::global_mut(cx).notifications_enabled =
                                        notifications_default;
                                },
                            ),
                        )
<<<<<<< HEAD
                        .description(tr!("Item.notifications_enabled_desc")),
                        SettingItem::new(
                            tr!("Item.digest"),
                            SettingField::dropdown(
                                vec![
                                    ("immediate".into(), tr!("Digest.immediate").into()),
                                    ("daily".into(), tr!("Digest.daily").into()),
                                    ("weekly".into(), tr!("Digest.weekly").into()),
                                ],
                                |cx: &App| AppSettings::global(cx).digest.clone(),
                                |val: SharedString, cx: &mut App| {
                                    AppSettings::global_mut(cx).digest = val;
                                },
                            )
                            .default_value(defaults.digest.clone()),
                        )
                        .description(tr!("Item.digest_desc")),
                    ]),
                    SettingGroup::new().title(tr!("Group.actions")).item(
                        SettingItem::new(
                            tr!("Item.test_notification"),
                            SettingField::render(|options, _window, _cx| {
=======
                        .description(tr("item.notifications.desc", &lang)),
                    ]),
                    SettingGroup::new().title(tr("group.actions", &lang)).item(
                        SettingItem::new(
                            tr("item.test_notification", &lang),
                            SettingField::render(|options, _window, cx| {
                                let settings = AppSettings::global(cx);
                                let enabled = settings.notifications_enabled;
                                let lang = settings.language.clone();
>>>>>>> origin/main
                                Button::new("send-test-notification")
                                    .outline()
                                    .with_size(options.size())
                                    .icon(IconName::Bell)
<<<<<<< HEAD
                                    .label(tr!("Action.send"))
=======
                                    .label(tr("action.send", &lang))
                                    .disabled(!enabled)
>>>>>>> origin/main
                                    .on_click(|_, window, cx| {
                                        let settings = AppSettings::global(cx);
                                        if !settings.notifications_enabled {
                                            return;
                                        }
                                        window.push_notification(
<<<<<<< HEAD
                                            Notification::success(tr!("Notification.test")),
=======
                                            Notification::success(tr(
                                                "notify.test_message",
                                                &settings.language,
                                            )),
>>>>>>> origin/main
                                            cx,
                                        );
                                    })
                            }),
                        )
<<<<<<< HEAD
                        .description(tr!("Item.test_notification_desc")),
                    ),
                ]),
            SettingPage::new(tr!("Page.profile"))
                .icon(Icon::new(IconName::User))
                .resettable(true)
                .group(SettingGroup::new().title(tr!("Group.account")).items(vec![
                            SettingItem::new(
                                tr!("Item.username"),
                                SettingField::input(
                                    |cx: &App| AppSettings::global(cx).username.clone(),
                                    |val: SharedString, cx: &mut App| {
                                        AppSettings::global_mut(cx).username = val;
                                    },
                                )
                                .default_value(defaults.username.clone()),
                            )
                            .description(tr!("Item.username_desc")),
                            SettingItem::new(
                                tr!("Item.display_name"),
                                SettingField::input(
                                    |cx: &App| AppSettings::global(cx).display_name.clone(),
                                    |val: SharedString, cx: &mut App| {
                                        AppSettings::global_mut(cx).display_name = val;
                                    },
                                )
                                .default_value(defaults.display_name.clone()),
                            )
                            .description(tr!("Item.display_name_desc")),
                        ])),
            SettingPage::new(tr!("Page.about"))
=======
                        .description(tr("item.test_notification.desc", &lang)),
                    ),
                ]),
            SettingPage::new(tr("page.profile", &lang))
                .icon(Icon::new(IconName::User))
                .resettable(true)
                .group(SettingGroup::new().title(tr("group.account", &lang)).items(vec![
                        SettingItem::new(
                            tr("item.username", &lang),
                            SettingField::input(
                                |cx: &App| AppSettings::global(cx).username.clone(),
                                |val: SharedString, cx: &mut App| {
                                    AppSettings::global_mut(cx).username = val;
                                    save_settings(cx);
                                    cx.refresh_windows();
                                },
                            )
                            .default_value(defaults.username.clone()),
                        )
                        .description(tr("item.username.desc", &lang)),
                        SettingItem::new(
                            tr("item.display_name", &lang),
                            SettingField::input(
                                |cx: &App| AppSettings::global(cx).display_name.clone(),
                                |val: SharedString, cx: &mut App| {
                                    AppSettings::global_mut(cx).display_name = val;
                                    save_settings(cx);
                                    cx.refresh_windows();
                                },
                            )
                            .default_value(defaults.display_name.clone()),
                        )
                        .description(tr("item.display_name.desc", &lang)),
                        SettingItem::new(
                            tr("item.preview", &lang),
                            SettingField::render(|_options, _window, cx| {
                                let settings = AppSettings::global(cx);
                                div().text_color(cx.theme().muted_foreground).child(
                                    SharedString::from(format!(
                                        "{} (@{})",
                                        settings.display_name, settings.username
                                    )),
                                )
                            }),
                        )
                        .description(tr("item.preview.desc", &lang)),
                    ])),
            SettingPage::new(tr("page.about", &lang))
>>>>>>> origin/main
                .icon(Icon::new(IconName::Info))
                .groups(vec![
                    SettingGroup::new().item(SettingItem::render(|_options, _window, cx| {
                        v_flex()
                            .gap_3()
                            .w_full()
                            .items_center()
                            .justify_center()
                            .child(img("brand/brand-mark.png").size(px(64.)))
                            .child(
                                div()
                                    .font_family("Xanh Mono")
                                    .italic()
                                    // Same derived 600 face as the sidebar
                                    // wordmark; the family's own italic is 400.
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_size(px(17.))
                                    .text_color(cx.theme().foreground)
                                    .child("settings-demo"),
                            )
                            .child(
<<<<<<< HEAD
                                Label::new(tr!("About.tagline"))
=======
                                Label::new(tr("about.tagline", &AppSettings::global(cx).language))
>>>>>>> origin/main
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground),
                            )
                            // The version borrows the blue informational tone
                            // with a release-tag glyph: a calendar would read as
                            // a date, not a release.
                            .child(
                                StatusChip::labelled(
                                    StatusTone::Scheduled,
                                    format!(
                                        "{} {}",
                                        tr!("Status.version"),
                                        env!("CARGO_PKG_VERSION")
                                    ),
                                )
                                .icon(StatusIcon::Tag),
                            )
                    })),
<<<<<<< HEAD
                    SettingGroup::new().title(tr!("Group.links")).items(vec![
                        SettingItem::new(
                            tr!("Item.website"),
                            SettingField::render(|options, _window, _cx| {
=======
                    SettingGroup::new().title(tr("group.links", &lang)).items(vec![
                        SettingItem::new(
                            tr("item.website", &lang),
                            SettingField::render(|options, _window, cx| {
>>>>>>> origin/main
                                Button::new("open-website")
                                    .outline()
                                    .with_size(options.size())
                                    .icon(IconName::ExternalLink)
                                    .label(tr("action.website_label", &AppSettings::global(cx).language))
                                    .on_click(|_, _window, cx| {
                                        cx.open_url("https://gpui-kit.com/");
                                    })
                            }),
                        )
<<<<<<< HEAD
                        .description(tr!("Item.website_desc")),
                        SettingItem::new(
                            tr!("Item.about"),
                            SettingField::render(|options, _window, _cx| {
=======
                        .description(tr("item.website.desc", &lang)),
                        SettingItem::new(
                            tr("item.about", &lang),
                            SettingField::render(|options, _window, cx| {
>>>>>>> origin/main
                                Button::new("open-about")
                                    .outline()
                                    .with_size(options.size())
                                    .icon(IconName::Info)
<<<<<<< HEAD
                                    .label(tr!("Action.about"))
                                    .on_click(|_, window, cx| {
                                        window.open_dialog(cx, |dialog, _, _| {
                                            dialog
                                                .title(tr!("About.dialog_title"))
                                                .child(tr!("About.dialog_body"))
=======
                                    .label(tr("action.about_label", &AppSettings::global(cx).language))
                                    .on_click(|_, window, cx| {
                                        window.open_dialog(cx, |dialog, _, cx| {
                                            let lang = &AppSettings::global(cx).language;
                                            dialog.title(tr("dialog.about_title", lang)).child(
                                                tr("dialog.about_body", lang),
                                            )
>>>>>>> origin/main
                                        });
                                    })
                            }),
                        )
<<<<<<< HEAD
                        .description(tr!("Item.about_desc")),
=======
                        .description(tr("item.about.desc", &lang)),
>>>>>>> origin/main
                    ]),
                ]),
        ]
    }
}

/// Maps app language codes to gpui-component locale codes: the component
/// ships `zh-CN` translations, so `"zh"` maps to `"zh-CN"`; other codes
/// pass through and rust_i18n falls back to English for missing locales.
fn component_locale(lang: &str) -> &str {
    match lang {
        "zh" => "zh-CN",
        other => other,
    }
}

impl Focusable for SettingsWindow {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

<<<<<<< HEAD
=======
/// Rail brand block: its own row above the search, left-aligned.
///
/// `img()` (not `svg()`): GPUI renders SVG as a monochrome alpha mask, which
/// would flatten the mark's gradient into the rail's text color. The row adds
/// its own vertical padding so the mark reads as a block of its own rather
/// than part of the search field.
fn rail_brand() -> impl IntoElement {
    h_flex()
        .w_full()
        .gap_2()
        .items_center()
        .pt_1()
        .pb_2()
        .child(img("brand/brand-mark.png").size(px(48.)))
        .child(
            div()
                .flex_1()
                .min_w(px(0.))
                .font_family("Xanh Mono")
                .italic()
                .text_xl()
                .truncate()
                .child("settings-demo"),
        )
}
fn rail_footer(cx: &App) -> impl IntoElement {
    let dark = cx.theme().mode.is_dark();
    let lang = AppSettings::global(cx).language.clone();
    Button::new("rail-theme-toggle")
        .primary()
        .w_full()
        .icon(if dark { IconName::Sun } else { IconName::Moon })
        .label(if dark {
            tr("rail.light_mode", &lang)
        } else {
            tr("rail.dark_mode", &lang)
        })
        .on_click(|_, window, cx| {
            let next = if cx.theme().mode.is_dark() {
                ThemeMode::Light
            } else {
                ThemeMode::Dark
            };
            Theme::change(next, Some(window), cx);
            AppSettings::global_mut(cx).dark_mode = next.is_dark();
            apply_font_size(cx);
            save_settings(cx);
        })
}

/// Re-applies the user's font size after `Theme::change`, which resets
/// `font_size` from the active `ThemeConfig` (`mono.json` pins it at 14).
/// `Root::render` pushes `Theme.font_size` into `window.set_rem_size`, so
/// the whole UI scales from this one value.
fn apply_font_size(cx: &mut App) {
    let size = AppSettings::global(cx).font_size;
    Theme::global_mut(cx).font_size = px(size as f32);
    Theme::sync_base(cx);
}

/// Guards the font-size boundary: the vendored `NumberField` parses
/// `"NaN"`/`"inf"` to non-finite `f64` and its clamp passes NaN through,
/// which would poison `px()`/`set_rem_size`. Non-finite input is rejected
/// (setter keeps the previous value); finite input passes through — the
/// field already clamps to 8–32.
fn normalize_font_size(val: f64) -> Option<f64> {
    val.is_finite().then_some(val)
}


>>>>>>> origin/main
impl Render for SettingsWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let collapsed = cx.global::<ShellState>().collapsed;
        let sidebar_w = px(if collapsed { 48. } else { 240. });
        div()
            .size_full()
            .bg(cx.theme().background)
            .child(
                AppShell::new().settings_view(
                    Settings::new("settings-demo")
                        .sidebar_width(sidebar_w)
                        // Pin the resizable panel to the current sidebar width —
                        // ResizablePanel min/max are authoritative, so a
                        // collapsed-sized pin keeps the seam from dragging.
                        .sidebar_size_range(sidebar_w..sidebar_w)
                        // Icon-collapse (48px rail) vs expanded (240px) sidebar.
                        .sidebar_collapsed(collapsed)
                        // Suppress the painted 1px resize seam between sidebar
                        // and card (hit area stays but the pin prevents drag).
                        .show_handle(false)
                        // Brand row above the search input (`sidebar_header`
                        // slot); collapses to the mark alone, search hides.
                        .sidebar_header(shell::sidebar_brand(collapsed, cx.theme().foreground))
                        // zdesk rail: borderless, inactive items in the muted
                        // foreground. The colour lands as the inherited text
                        // color on the sidebar root — active/hover items set
                        // their own full-contrast colors. (zdesk's 70%-opacity
                        // variant resolved to 3.7:1 in light mode, below WCAG
                        // AA; `muted.foreground` is 5.9:1 / 6.1:1.)
                        .sidebar_style(
                            &StyleRefinement::default()
                                .w(sidebar_w)
                                .border_0()
                                .text_color(cx.theme().muted_foreground),
                        )
                        // Raised pill on the active sidebar item.
                        .active_item_style(
                            &StyleRefinement::default()
                                .bg(cx.theme().group_box)
                                .border_1()
                                .border_color(cx.theme().border),
                        )
                        // Raised zdesk card around the settings content:
                        // rounded, stroked, lighter than the window base, with
                        // a 6px inset on all four sides, clipped so page
                        // content can't bleed past the radius.
                        .content_panel_style(
                            &StyleRefinement::default()
                                .rounded_lg()
                                .border_1()
                                .border_color(cx.theme().border)
                                .bg(cx.theme().group_box)
                                .m(px(6.))
                                .overflow_hidden(),
                        )
                        .pages(self.setting_pages(cx)),
                ),
            )
            .children(Root::render_dialog_layer(window, cx))
            .children(Root::render_sheet_layer(window, cx))
            .children(Root::render_notification_layer(window, cx))
    }
}

/// Fraction of gpui's default new-window bounds used for the settings window.
///
/// gpui's default is `DEFAULT_WINDOW_SIZE` (1536×1095) clipped to the display
/// and centered; this window opens at 70% of that, keeping the same center.
const WINDOW_SCALE: f32 = 0.7;

fn default_window_bounds(cx: &App) -> WindowBounds {
    let base = cx
        .primary_display()
        .map(|display| display.default_bounds())
        .unwrap_or_else(|| Bounds::new(point(px(0.), px(0.)), gpui_kit::DEFAULT_WINDOW_SIZE));
    let scaled = size(
        base.size.width * WINDOW_SCALE,
        base.size.height * WINDOW_SCALE,
    );
    let center = base.center();

    WindowBounds::Windowed(Bounds::new(
        point(
            center.x - scaled.width * 0.5,
            center.y - scaled.height * 0.5,
        ),
        scaled,
    ))
}

fn main() {
    gpui_kit::application()
        .with_assets(assets::AppAssets)
        .run(|cx| {
            // This must be called before using any GPUI Component features.
            gpui_kit::init(cx);
            // Apply the startup defaults: locale (switches both the app and
            // the component-internal strings) and the theme overrides (font
            // size, focus ring).
            let defaults = AppSettings::default();
            rust_i18n::set_locale(locale_for(&defaults.language));
            theme::install(cx, defaults.font_size);
            // ⌘Q: gpui ships no default Quit action or menu item, and macOS
            // uses `QuitMode::Explicit`, so both have to be registered here.
            // Bind first — `set_menus` reads the keymap for key equivalents.
            cx.bind_keys([
                KeyBinding::new("cmd-q", Quit, None),
                KeyBinding::new("cmd-w", CloseWindow, None),
                KeyBinding::new("cmd-b", ToggleSidebar, None),
            ]);
            cx.set_menus(vec![
                Menu::new("mono-desk").items([MenuItem::action("Quit", Quit)]),
            ]);
            cx.on_action(|_: &Quit, cx: &mut App| cx.quit());
<<<<<<< HEAD
            // ⌘W is close-window semantics; with a single window and no tabs
            // to close, closing the window quits the app.
            cx.on_action(|_: &CloseWindow, cx: &mut App| cx.quit());
            // ⌘B toggles the settings sidebar between 240px and the 48px
            // icon rail (same affordance as the titlebar collapse button).
            cx.on_action(|_: &ToggleSidebar, cx: &mut App| {
                let state = cx.global_mut::<ShellState>();
                state.collapsed = !state.collapsed;
                cx.refresh_windows();
            });
            // Frameless window driven by the vendored `TitleBar`: transparent
            // titlebar with macOS traffic lights at zdesk's (16,18) and an
            // app-owned drag region; only the bounds are customized.
            //
            // On Linux (GNOME/Wayland + XWayland) the compositor otherwise
            // stacks its own GTK/system titlebar above the vendored one.
            // Client-side decorations make the vendored `TitleBar` the only
            // chrome: it draws minimize/maximize/close itself
            // (`WindowControls`, gated on `Decorations::Client`) and owns
            // dragging, and `Root` already renders `window_border()` for the
            // resize edges. macOS keeps its native titlebar (traffic lights).
            #[cfg(target_os = "linux")]
            let window_decorations = Some(WindowDecorations::Client);
            #[cfg(not(target_os = "linux"))]
            let window_decorations: Option<WindowDecorations> = None;

=======
            cx.set_global(AppSettings::from_stored(persist::load()));
            let settings = AppSettings::global(cx);
            let dark_mode = settings.dark_mode;
            let language = settings.language.clone();
            let launch_at_login = settings.launch_at_login;
            if dark_mode {
                Theme::change(ThemeMode::Dark, None, cx);
            }
            apply_font_size(cx);
            gpui_kit::component::set_locale(component_locale(&language));
            persist::autostart_converge(launch_at_login);
            // Native titlebar: gpui only calls `setTitle` when a title is set,
            // so `WindowOptions::default()` leaves the window title empty.
>>>>>>> origin/main
            let window_options = WindowOptions {
                window_bounds: Some(default_window_bounds(cx)),
                titlebar: Some(TitlebarOptions {
                    traffic_light_position: Some(point(px(16.), px(18.))),
                    ..TitleBar::title_bar_options()
                }),
                window_decorations,
                ..TitleBar::window_options()
            };
            cx.spawn(async move |cx| {
                cx.open_window(window_options, |window, cx| {
                    let view = cx.new(|cx| SettingsWindow::new(window, cx));
                    // The first level on the window must be a Root.
                    cx.new(|cx| Root::new(view, window, cx))
                })
                .expect("failed to open window");
            })
            .detach();
        });
}

#[cfg(test)]
mod tests {
<<<<<<< HEAD
    use std::collections::BTreeSet;

    /// The sidebar tree shows children only for pages with at least two
    /// titled groups — that's exactly what keeps About (untitled brand
    /// group + one titled "Links" group) and Profile (one titled group)
    /// rendered as flat bold items.
    #[test]
    fn sidebar_children_require_two_titled_groups() {
        use gpui_kit::component::setting::{SettingGroup, SettingPage, sidebar_child_group_ixs};

        let titled = |t: &str| SettingGroup::new().title(t);

        // General / Notifications shape: every group titled.
        let two_titled =
            SettingPage::new("General").groups([titled("Appearance"), titled("Startup")]);
        assert_eq!(sidebar_child_group_ixs(&two_titled), vec![0, 1]);

        // About: an untitled group doesn't count toward the threshold.
        let about = SettingPage::new("About").groups([SettingGroup::new(), titled("Links")]);
        assert!(sidebar_child_group_ixs(&about).is_empty());

        // Profile: a single titled group renders flat.
        let profile = SettingPage::new("Profile").groups([titled("Account")]);
        assert!(sidebar_child_group_ixs(&profile).is_empty());

        // Untitled groups keep their slots — child indices address the
        // group's position in `page.groups`, which is the index the scroll-
        // to-group action consumes.
        let mixed = SettingPage::new("Mixed").groups([
            SettingGroup::new(),
            titled("First"),
            titled("Second"),
        ]);
        assert_eq!(sidebar_child_group_ixs(&mixed), vec![1, 2]);
    }

    /// Every key in `locales/app.yml` must carry a value for every language the
    /// Language dropdown offers, so switching never silently falls back to
    /// English for a key that exists in another column.
    #[test]
    fn locale_files_cover_every_offered_language() {
        let backend = crate::_rust_i18n_backend();
        let locales = ["en", "vi", "zh-CN", "ja"];

        let mut reference: Option<BTreeSet<String>> = None;
        for locale in locales {
            let messages = backend
                .messages_for_locale(locale)
                .unwrap_or_else(|| panic!("locale {locale} missing from locales/app.yml"));
            let keys: BTreeSet<String> = messages.iter().map(|(key, _)| key.to_string()).collect();
            assert!(!keys.is_empty(), "locale {locale} has no keys");

            match &reference {
                None => {
                    assert!(
                        keys.contains("Page.general"),
                        "expected key Page.general in the locale file"
                    );
                    reference = Some(keys);
                }
                Some(expected) => {
                    let missing: Vec<_> = expected.difference(&keys).collect();
                    let extra: Vec<_> = keys.difference(expected).collect();
                    assert!(
                        missing.is_empty() && extra.is_empty(),
                        "locale {locale} key mismatch — missing {missing:?}, extra {extra:?}"
                    );
                }
            }
        }
    }

    /// The dropdown value `zh` must bridge to the component's `zh-CN` column.
    #[test]
    fn locale_for_bridges_chinese() {
        assert_eq!(super::locale_for("zh"), "zh-CN");
        assert_eq!(super::locale_for("en"), "en");
        assert_eq!(super::locale_for("vi"), "vi");
        assert_eq!(super::locale_for("ja"), "ja");
=======
    use super::normalize_font_size;

    #[test]
    fn font_size_rejects_non_finite_input() {
        assert_eq!(normalize_font_size(f64::NAN), None);
        assert_eq!(normalize_font_size(f64::INFINITY), None);
        assert_eq!(normalize_font_size(f64::NEG_INFINITY), None);
        assert_eq!(normalize_font_size(14.0), Some(14.0));
        assert_eq!(normalize_font_size(8.0), Some(8.0));
        assert_eq!(normalize_font_size(32.0), Some(32.0));
>>>>>>> origin/main
    }
}
