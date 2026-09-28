use gpui_kit::component::{
    ActiveTheme, Icon, IconName, Root, Sizable, Theme, ThemeMode, TitleBar, WindowExt,
    button::Button,
    label::Label,
    notification::Notification,
    setting::{NumberFieldOptions, SettingField, SettingGroup, SettingItem, SettingPage, Settings},
    v_flex,
};
use gpui_kit::{
    App, AppContext as _, Bounds, Context, FocusHandle, Focusable, Global, IntoElement, KeyBinding,
    Menu, MenuItem, ParentElement as _, Render, SharedString, StyleRefinement, Styled as _,
    TitlebarOptions, Window, WindowBounds, WindowOptions, div, img, point, px, size,
};
use shell::{AppShell, ShellState};

mod assets;
mod shell;
mod theme;

gpui_kit::actions!(mono_desk, [Quit, CloseWindow, ToggleSidebar]);

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
        cx.set_global(ShellState::default());
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
                    SettingGroup::new()
                        .title("Appearance")
                        .icon(IconName::Palette)
                        .items(vec![
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
                    SettingGroup::new()
                        .title("Startup")
                        .icon(IconName::Play)
                        .items(vec![
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
                    SettingGroup::new()
                        .title("Delivery")
                        .icon(IconName::Bell)
                        .items(vec![
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
                    SettingGroup::new()
                        .title("Actions")
                        .icon(IconName::SquareTerminal)
                        .item(
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
                                                Notification::success("Hello from settings-demo!"),
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
                    SettingGroup::new()
                        .title("Account")
                        .icon(IconName::CircleUser)
                        .items(vec![
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
                            .child(img("brand/brand-mark.png").size(px(64.)))
                            .child(
                                div()
                                    .font_family("Xanh Mono")
                                    .italic()
                                    .child("settings-demo"),
                            )
                            .child(
                                Label::new("A settings-style demo app built with GPUI Kit.")
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground),
                            )
                    })),
                    SettingGroup::new()
                        .title("Links")
                        .icon(IconName::ExternalLink)
                        .items(vec![
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
                        // slot); hidden entirely when collapsed.
                        .sidebar_header(shell::sidebar_brand(collapsed))
                        // zdesk rail: borderless, inactive items at 70%
                        // foreground. The opacity lands as the inherited text
                        // color on the sidebar root — active/hover items set
                        // their own full-opacity colors.
                        .sidebar_style(
                            &StyleRefinement::default()
                                .w(sidebar_w)
                                .border_0()
                                .text_color(cx.theme().sidebar_foreground.opacity(0.7)),
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
            theme::install(cx);
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
            let window_options = WindowOptions {
                window_bounds: Some(default_window_bounds(cx)),
                titlebar: Some(TitlebarOptions {
                    traffic_light_position: Some(point(px(16.), px(18.))),
                    ..TitleBar::title_bar_options()
                }),
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
