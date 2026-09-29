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
use rust_i18n::t;
use shell::{AppShell, ShellState};

mod assets;
mod shell;
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
            // Matches the theme's `font.size` (13px) so the applied default
            // preserves the approved visual baseline.
            font_size: 13.0,
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
            SettingPage::new(tr!("Page.general"))
                .icon(Icon::new(IconName::Settings2))
                .default_open(true)
                .resettable(true)
                .groups(vec![
                    SettingGroup::new()
                        .title(tr!("Group.appearance"))
                        .icon(IconName::Palette)
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
                    SettingGroup::new()
                        .title(tr!("Group.startup"))
                        .icon(IconName::Play)
                        .items(vec![
                            SettingItem::new(
                                tr!("Item.launch_at_login"),
                                SettingField::checkbox(
                                    |cx: &App| AppSettings::global(cx).launch_at_login,
                                    |val: bool, cx: &mut App| {
                                        AppSettings::global_mut(cx).launch_at_login = val;
                                    },
                                )
                                .default_value(defaults.launch_at_login),
                            )
                            .description(tr!("Item.launch_at_login_desc")),
                            SettingItem::new(
                                tr!("Item.auto_update"),
                                SettingField::switch(
                                    |cx: &App| AppSettings::global(cx).auto_update,
                                    |val: bool, cx: &mut App| {
                                        AppSettings::global_mut(cx).auto_update = val;
                                    },
                                )
                                .default_value(defaults.auto_update),
                            )
                            .description(tr!("Item.auto_update_desc")),
                        ]),
                ]),
            SettingPage::new(tr!("Page.notifications"))
                .icon(Icon::new(IconName::Bell))
                .resettable(true)
                .groups(vec![
                    SettingGroup::new()
                        .title(tr!("Group.delivery"))
                        .icon(IconName::Bell)
                        .items(vec![
                            SettingItem::new(
                                tr!("Item.notifications_enabled"),
                                SettingField::switch(
                                    |cx: &App| AppSettings::global(cx).notifications_enabled,
                                    |val: bool, cx: &mut App| {
                                        AppSettings::global_mut(cx).notifications_enabled = val;
                                    },
                                )
                                .default_value(defaults.notifications_enabled),
                            )
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
                    SettingGroup::new()
                        .title(tr!("Group.actions"))
                        .icon(IconName::SquareTerminal)
                        .item(
                            SettingItem::new(
                                tr!("Item.test_notification"),
                                SettingField::render(|options, _window, _cx| {
                                    Button::new("send-test-notification")
                                        .outline()
                                        .with_size(options.size())
                                        .icon(IconName::Bell)
                                        .label(tr!("Action.send"))
                                        .on_click(|_, window, cx| {
                                            window.push_notification(
                                                Notification::success(tr!("Notification.test")),
                                                cx,
                                            );
                                        })
                                }),
                            )
                            .description(tr!("Item.test_notification_desc")),
                        ),
                ]),
            SettingPage::new(tr!("Page.profile"))
                .icon(Icon::new(IconName::User))
                .resettable(true)
                .group(
                    SettingGroup::new()
                        .title(tr!("Group.account"))
                        .icon(IconName::CircleUser)
                        .items(vec![
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
                        ]),
                ),
            SettingPage::new(tr!("Page.about"))
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
                                Label::new(tr!("About.tagline"))
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground),
                            )
                    })),
                    SettingGroup::new()
                        .title(tr!("Group.links"))
                        .icon(IconName::ExternalLink)
                        .items(vec![
                            SettingItem::new(
                                tr!("Item.website"),
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
                            .description(tr!("Item.website_desc")),
                            SettingItem::new(
                                tr!("Item.about"),
                                SettingField::render(|options, _window, _cx| {
                                    Button::new("open-about")
                                        .outline()
                                        .with_size(options.size())
                                        .icon(IconName::Info)
                                        .label(tr!("Action.about"))
                                        .on_click(|_, window, cx| {
                                            window.open_dialog(cx, |dialog, _, _| {
                                                dialog
                                                    .title(tr!("About.dialog_title"))
                                                    .child(tr!("About.dialog_body"))
                                            });
                                        })
                                }),
                            )
                            .description(tr!("Item.about_desc")),
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
                        // slot); collapses to the mark alone, search hides.
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

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
    }
}
