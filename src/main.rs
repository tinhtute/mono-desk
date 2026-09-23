use gpui_kit::component::{
    ActiveTheme, Disableable, Icon, IconName, Root, Sizable, Theme, ThemeMode, WindowExt,
    button::{Button, ButtonVariants},
    h_flex,
    label::Label,
    notification::Notification,
    setting::{NumberFieldOptions, SettingField, SettingGroup, SettingItem, SettingPage, Settings},
    v_flex,
};
use gpui_kit::{
    App, AppContext as _, Bounds, Context, FocusHandle, Focusable, Global, IntoElement, KeyBinding,
    Menu, MenuItem, ParentElement as _, Render, SharedString, Styled as _, TitlebarOptions, Window,
    WindowBounds, WindowOptions, div, img, point, px, size,
};

mod assets;
mod i18n;
mod persist;
mod theme;

use i18n::tr;
use persist::StoredSettings;

gpui_kit::actions!(mono_desk, [Quit]);

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
            font_size: 14.0,
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
        Self {
            focus_handle: cx.focus_handle(),
        }
    }

    fn setting_pages(&self, cx: &mut Context<Self>) -> Vec<SettingPage> {
        let lang = AppSettings::global(cx).language.clone();
        let defaults = AppSettings::default();

        vec![
            SettingPage::new(tr("page.general", &lang))
                .icon(Icon::new(IconName::Settings2))
                .default_open(true)
                .resettable(true)
                .groups(vec![
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
                                },
                            )
                            .default_value(defaults.notifications_enabled),
                        )
                        .description(tr("item.notifications.desc", &lang)),
                    ]),
                    SettingGroup::new().title(tr("group.actions", &lang)).item(
                        SettingItem::new(
                            tr("item.test_notification", &lang),
                            SettingField::render(|options, _window, cx| {
                                let settings = AppSettings::global(cx);
                                let enabled = settings.notifications_enabled;
                                let lang = settings.language.clone();
                                Button::new("send-test-notification")
                                    .outline()
                                    .with_size(options.size())
                                    .icon(IconName::Bell)
                                    .label(tr("action.send", &lang))
                                    .disabled(!enabled)
                                    .on_click(|_, window, cx| {
                                        let settings = AppSettings::global(cx);
                                        if !settings.notifications_enabled {
                                            return;
                                        }
                                        window.push_notification(
                                            Notification::success(tr(
                                                "notify.test_message",
                                                &settings.language,
                                            )),
                                            cx,
                                        );
                                    })
                            }),
                        )
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
                                    .child("settings-demo"),
                            )
                            .child(
                                Label::new(tr("about.tagline", &AppSettings::global(cx).language))
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground),
                            )
                    })),
                    SettingGroup::new().title(tr("group.links", &lang)).items(vec![
                        SettingItem::new(
                            tr("item.website", &lang),
                            SettingField::render(|options, _window, cx| {
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
                        .description(tr("item.website.desc", &lang)),
                        SettingItem::new(
                            tr("item.about", &lang),
                            SettingField::render(|options, _window, cx| {
                                Button::new("open-about")
                                    .outline()
                                    .with_size(options.size())
                                    .icon(IconName::Info)
                                    .label(tr("action.about_label", &AppSettings::global(cx).language))
                                    .on_click(|_, window, cx| {
                                        window.open_dialog(cx, |dialog, _, cx| {
                                            let lang = &AppSettings::global(cx).language;
                                            dialog.title(tr("dialog.about_title", lang)).child(
                                                tr("dialog.about_body", lang),
                                            )
                                        });
                                    })
                            }),
                        )
                        .description(tr("item.about.desc", &lang)),
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


impl Render for SettingsWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .bg(cx.theme().background)
            .child(
                Settings::new("settings-demo")
                    .sidebar_width(px(240.))
                    .sidebar_header(rail_brand())
                    .sidebar_footer(rail_footer(cx))
                    .pages(self.setting_pages(cx)),
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
            theme::install(cx);
            // ⌘Q: gpui ships no default Quit action or menu item, and macOS
            // uses `QuitMode::Explicit`, so both have to be registered here.
            // Bind first — `set_menus` reads the keymap for key equivalents.
            cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
            cx.set_menus(vec![
                Menu::new("mono-desk").items([MenuItem::action("Quit", Quit)]),
            ]);
            cx.on_action(|_: &Quit, cx: &mut App| cx.quit());
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
            let window_options = WindowOptions {
                window_bounds: Some(default_window_bounds(cx)),
                titlebar: Some(TitlebarOptions {
                    title: Some("mono-desk".into()),
                    ..Default::default()
                }),
                ..Default::default()
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
    use super::normalize_font_size;

    #[test]
    fn font_size_rejects_non_finite_input() {
        assert_eq!(normalize_font_size(f64::NAN), None);
        assert_eq!(normalize_font_size(f64::INFINITY), None);
        assert_eq!(normalize_font_size(f64::NEG_INFINITY), None);
        assert_eq!(normalize_font_size(14.0), Some(14.0));
        assert_eq!(normalize_font_size(8.0), Some(8.0));
        assert_eq!(normalize_font_size(32.0), Some(32.0));
    }
}
