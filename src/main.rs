use gpui_kit::component::{
    ActiveTheme, Icon, IconName, Root, Sizable, Theme, ThemeMode, WindowExt,
    button::Button,
    label::Label,
    notification::Notification,
    setting::{
        NumberFieldOptions, SettingField, SettingGroup, SettingItem, SettingPage, Settings,
    },
    v_flex,
};
use gpui_kit::{
    App, AppContext as _, Context, FocusHandle, Focusable, Global, IntoElement,
    ParentElement as _, Render, SharedString, Styled as _, Window, WindowOptions, div,
};

/// Application state, stored as a GPUI global so `SettingField` closures can
/// reach it through `&App` / `&mut App`.
struct AppSettings {
    language: SharedString,
    font_size: f64,
    launch_at_login: bool,
    auto_update: bool,
    notifications_enabled: bool,
    digest: SharedString,
    username: SharedString,
    display_name: SharedString,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            language: "en".into(),
            font_size: 14.0,
            launch_at_login: false,
            auto_update: true,
            notifications_enabled: true,
            digest: "immediate".into(),
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

struct SettingsWindow {
    focus_handle: FocusHandle,
}

impl SettingsWindow {
    fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        cx.set_global(AppSettings::default());
        Self {
            focus_handle: cx.focus_handle(),
        }
    }

    fn setting_pages(&self, cx: &mut Context<Self>) -> Vec<SettingPage> {
        let _ = cx;
        let defaults = AppSettings::default();

        vec![
            SettingPage::new("General")
                .icon(Icon::new(IconName::Settings2))
                .default_open(true)
                .resettable(true)
                .groups(vec![
                    SettingGroup::new().title("Appearance").items(vec![
                        SettingItem::new(
                            "Dark Mode",
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
                                },
                            )
                            .default_value(false),
                        )
                        .description("Switch between light and dark themes."),
                        SettingItem::new(
                            "Language",
                            SettingField::dropdown(
                                vec![
                                    ("en".into(), "English".into()),
                                    ("vi".into(), "Tiếng Việt".into()),
                                    ("zh".into(), "简体中文".into()),
                                    ("ja".into(), "日本語".into()),
                                ],
                                |cx: &App| AppSettings::global(cx).language.clone(),
                                |val: SharedString, cx: &mut App| {
                                    AppSettings::global_mut(cx).language = val;
                                },
                            )
                            .default_value(defaults.language.clone()),
                        )
                        .description("Interface language (demo only, no i18n wired)."),
                        SettingItem::new(
                            "Font Size",
                            SettingField::number_input(
                                NumberFieldOptions {
                                    min: 8.0,
                                    max: 32.0,
                                    ..Default::default()
                                },
                                |cx: &App| AppSettings::global(cx).font_size,
                                |val: f64, cx: &mut App| {
                                    AppSettings::global_mut(cx).font_size = val;
                                },
                            )
                            .default_value(defaults.font_size),
                        )
                        .description("Base UI font size, 8–32."),
                    ]),
                    SettingGroup::new().title("Startup").items(vec![
                        SettingItem::new(
                            "Launch at Login",
                            SettingField::checkbox(
                                |cx: &App| AppSettings::global(cx).launch_at_login,
                                |val: bool, cx: &mut App| {
                                    AppSettings::global_mut(cx).launch_at_login = val;
                                },
                            )
                            .default_value(defaults.launch_at_login),
                        )
                        .description("Start the app when you sign in."),
                        SettingItem::new(
                            "Auto Update",
                            SettingField::switch(
                                |cx: &App| AppSettings::global(cx).auto_update,
                                |val: bool, cx: &mut App| {
                                    AppSettings::global_mut(cx).auto_update = val;
                                },
                            )
                            .default_value(defaults.auto_update),
                        )
                        .description("Download and install updates automatically."),
                    ]),
                ]),
            SettingPage::new("Notifications")
                .icon(Icon::new(IconName::Bell))
                .resettable(true)
                .groups(vec![
                    SettingGroup::new().title("Delivery").items(vec![
                        SettingItem::new(
                            "Enable Notifications",
                            SettingField::switch(
                                |cx: &App| AppSettings::global(cx).notifications_enabled,
                                |val: bool, cx: &mut App| {
                                    AppSettings::global_mut(cx).notifications_enabled = val;
                                },
                            )
                            .default_value(defaults.notifications_enabled),
                        )
                        .description("Allow in-app notifications."),
                        SettingItem::new(
                            "Digest",
                            SettingField::dropdown(
                                vec![
                                    ("immediate".into(), "Immediately".into()),
                                    ("daily".into(), "Daily summary".into()),
                                    ("weekly".into(), "Weekly summary".into()),
                                ],
                                |cx: &App| AppSettings::global(cx).digest.clone(),
                                |val: SharedString, cx: &mut App| {
                                    AppSettings::global_mut(cx).digest = val;
                                },
                            )
                            .default_value(defaults.digest.clone()),
                        )
                        .description("How often notifications are delivered."),
                    ]),
                    SettingGroup::new().title("Actions").item(
                        SettingItem::new(
                            "Test Notification",
                            SettingField::render(|options, _window, _cx| {
                                Button::new("send-test-notification")
                                    .outline()
                                    .with_size(options.size())
                                    .icon(IconName::Bell)
                                    .label("Send")
                                    .on_click(|_, window, cx| {
                                        window.push_notification(
                                            Notification::success(
                                                "Hello from settings-demo!",
                                            ),
                                            cx,
                                        );
                                    })
                            }),
                        )
                        .description("Push a sample in-app notification."),
                    ),
                ]),
            SettingPage::new("Profile")
                .icon(Icon::new(IconName::User))
                .resettable(true)
                .group(
                    SettingGroup::new().title("Account").items(vec![
                        SettingItem::new(
                            "Username",
                            SettingField::input(
                                |cx: &App| AppSettings::global(cx).username.clone(),
                                |val: SharedString, cx: &mut App| {
                                    AppSettings::global_mut(cx).username = val;
                                },
                            )
                            .default_value(defaults.username.clone()),
                        )
                        .description("Your unique handle."),
                        SettingItem::new(
                            "Display Name",
                            SettingField::input(
                                |cx: &App| AppSettings::global(cx).display_name.clone(),
                                |val: SharedString, cx: &mut App| {
                                    AppSettings::global_mut(cx).display_name = val;
                                },
                            )
                            .default_value(defaults.display_name.clone()),
                        )
                        .description("Shown instead of your username."),
                    ]),
                ),
            SettingPage::new("About")
                .icon(Icon::new(IconName::Info))
                .groups(vec![
                    SettingGroup::new().item(SettingItem::render(|_options, _window, cx| {
                        v_flex()
                            .gap_3()
                            .w_full()
                            .items_center()
                            .justify_center()
                            .child(Icon::new(IconName::GalleryVerticalEnd).size_16())
                            .child("settings-demo")
                            .child(
                                Label::new("A settings-style demo app built with GPUI Kit.")
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground),
                            )
                    })),
                    SettingGroup::new().title("Links").items(vec![
                        SettingItem::new(
                            "Website",
                            SettingField::render(|options, _window, _cx| {
                                Button::new("open-website")
                                    .outline()
                                    .with_size(options.size())
                                    .icon(IconName::ExternalLink)
                                    .label("gpui-kit.com")
                                    .on_click(|_, _window, cx| {
                                        cx.open_url("https://gpui-kit.com/");
                                    })
                            }),
                        )
                        .description("Framework website and documentation."),
                        SettingItem::new(
                            "About",
                            SettingField::render(|options, _window, _cx| {
                                Button::new("open-about")
                                    .outline()
                                    .with_size(options.size())
                                    .icon(IconName::Info)
                                    .label("About…")
                                    .on_click(|_, window, cx| {
                                        window.open_dialog(cx, |dialog, _, _| {
                                            dialog.title("About settings-demo").child(
                                                "Built with GPUI Kit — the Rust desktop \
                                                 framework from Longbridge.",
                                            )
                                        });
                                    })
                            }),
                        )
                        .description("Show the about dialog."),
                    ]),
                ]),
        ]
    }
}

impl Focusable for SettingsWindow {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for SettingsWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .bg(cx.theme().background)
            .child(Settings::new("settings-demo").pages(self.setting_pages(cx)))
            .children(Root::render_dialog_layer(window, cx))
            .children(Root::render_sheet_layer(window, cx))
            .children(Root::render_notification_layer(window, cx))
    }
}

fn main() {
    gpui_kit::application()
        .with_assets(gpui_kit::assets::Assets)
        .run(|cx| {
            // This must be called before using any GPUI Component features.
            gpui_kit::init(cx);
            cx.spawn(async move |cx| {
                cx.open_window(WindowOptions::default(), |window, cx| {
                    let view = cx.new(|cx| SettingsWindow::new(window, cx));
                    // The first level on the window must be a Root.
                    cx.new(|cx| Root::new(view, window, cx))
                })
                .expect("failed to open window");
            })
            .detach();
        });
}
