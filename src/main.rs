use gpui_kit::component::{
    ActiveTheme, AxisExt as _, Disableable, Icon, IconName, Root, Sizable, Size, Theme, ThemeMode,
    TitleBar, WindowExt,
    button::Button,
    h_flex,
    label::Label,
    notification::Notification,
    setting::{NumberFieldOptions, SettingField, SettingGroup, SettingItem, SettingPage, Settings},
    switch::Switch,
    v_flex,
};
use gpui_kit::{AnimationExt, Pixels, SpringAnimation, SpringConfig};
use gpui_kit::{
    App, AppContext as _, Bounds, Context, FocusHandle, Focusable, FontWeight, Global, IntoElement,
    KeyBinding, Menu, MenuItem, ParentElement as _, Render, SharedString, StyleRefinement,
    Styled as _, TitlebarOptions, Window, WindowBounds, WindowDecorations, WindowOptions, div, img,
    point, px, size,
};
use input_field::{FieldStatus, InputFieldConfig, InputFieldView, Validation};
use shell::{AppShell, ShellState};
use status_chip::{StatusChip, StatusIcon, StatusTone};
use std::rc::Rc;

mod assets;
mod i18n;
mod input_field;
mod persist;
mod shell;
mod status_chip;
mod theme;

use i18n::tr;
use persist::StoredSettings;

gpui_kit::actions!(mono_desk, [Quit, CloseWindow, ToggleSidebar]);

/// Why the demo's username field accepts or rejects a value.
///
/// The rule is a pure function so the demo's validation is testable without a
/// locale or a window; [`Self::validation`] adds the translated message.
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
    /// The validation the field shows for this rule, in `lang`.
    fn validation(self, lang: &str) -> Option<Validation> {
        match self {
            Self::Empty => None,
            Self::TooShort => Some(Validation::Error(
                tr("field.username_too_short", lang).into(),
            )),
            Self::TooLong => Some(Validation::Error(
                tr("field.username_too_long", lang).into(),
            )),
            Self::InvalidCharacter => {
                Some(Validation::Error(tr("field.username_invalid", lang).into()))
            }
            Self::Valid => Some(Validation::Success(tr("field.username_ok", lang).into())),
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
            // Matches the theme's `font.size` (14px) so the applied default
            // preserves the approved visual baseline. `theme::tests` pins the
            // two together.
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
        cx.set_global(ShellState::default());
        Self {
            focus_handle: cx.focus_handle(),
        }
    }

    fn setting_pages(&self, cx: &mut Context<Self>) -> Vec<SettingPage> {
        let lang = AppSettings::global(cx).language.clone();
        let defaults = AppSettings::default();
        // Copied out of `defaults` so the reset closure of the element field
        // below can be `'static`.
        let notifications_default = defaults.notifications_enabled;

        vec![
            SettingPage::new(tr("page.general", &lang))
                .icon(Icon::new(IconName::Settings2))
                .resettable(true)
                .groups(vec![
                    SettingGroup::new()
                        .title(tr("group.appearance", &lang))
                        .items(vec![
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
                                        min: theme::FONT_SIZE_MIN,
                                        max: theme::FONT_SIZE_MAX,
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
                    SettingGroup::new()
                        .title(tr("group.startup", &lang))
                        .items(vec![
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
                    SettingGroup::new()
                        .title(tr("group.delivery", &lang))
                        .items(vec![
                            SettingItem::new(
                                tr("item.notifications", &lang),
                                SettingField::render(|options, _window, cx| {
                                    let settings = AppSettings::global(cx);
                                    let enabled = settings.notifications_enabled;
                                    let (tone, label) = if enabled {
                                        (
                                            StatusTone::Ready,
                                            tr("status.enabled", &settings.language),
                                        )
                                    } else {
                                        (
                                            StatusTone::Paused,
                                            tr("status.disabled", &settings.language),
                                        )
                                    };
                                    h_flex()
                                        .gap_2()
                                        .child(StatusChip::labelled(tone, label))
                                        .child(
                                            Switch::new("notifications-enabled")
                                                .checked(enabled)
                                                .with_size(options.size())
                                                .on_click(|checked: &bool, _, cx: &mut App| {
                                                    AppSettings::global_mut(cx)
                                                        .notifications_enabled = *checked;
                                                    save_settings(cx);
                                                    cx.refresh_windows();
                                                }),
                                        )
                                })
                                // An element field carries no typed value, so
                                // the page reset needs the handler explicitly.
                                .on_reset(
                                    move |cx: &App| {
                                        AppSettings::global(cx).notifications_enabled
                                            != notifications_default
                                    },
                                    move |_window, cx: &mut App| {
                                        AppSettings::global_mut(cx).notifications_enabled =
                                            notifications_default;
                                        save_settings(cx);
                                    },
                                ),
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
                .group(
                    SettingGroup::new()
                        .title(tr("group.account", &lang))
                        .items(vec![
                            SettingItem::new(
                                tr("item.username", &lang),
                                // The composed input field (label + input +
                                // validation row) instead of the vendored
                                // `SettingField::input`: the settings row
                                // already carries the title, so the field's
                                // own label stays off and the leading icon
                                // slot stands in for it.
                                SettingField::render({
                                    // The validator is `'static`, so it owns
                                    // its own copy of the locale.
                                    let field_lang = lang.clone();
                                    let config = InputFieldConfig::new(
                                        tr("item.username_placeholder", &lang),
                                        move |value: &str| {
                                            username_rule(value).validation(&field_lang)
                                        },
                                        |cx: &App| AppSettings::global(cx).username.clone(),
                                        |val: SharedString, cx: &mut App| {
                                            AppSettings::global_mut(cx).username = val;
                                            save_settings(cx);
                                            cx.refresh_windows();
                                        },
                                    )
                                    // beUI's leading icon, muted like the
                                    // placeholder.
                                    .prefix(Rc::new(|_, _, cx| {
                                        Some(
                                            Icon::new(IconName::User)
                                                .text_color(cx.theme().muted_foreground)
                                                .into_any_element(),
                                        )
                                    }))
                                    // beUI's success check: it replaces the
                                    // trailing icon while the value is valid.
                                    // The ink comes from the field's own
                                    // status→ink mapping, so the icon cannot
                                    // drift onto the pastel tint token.
                                    .suffix(Rc::new(|status, window, cx| {
                                        (status == FieldStatus::Success).then(|| {
                                            Icon::new(IconName::CircleCheck)
                                                .with_size(Size::Size(window.rem_size()))
                                                .text_color(input_field::status_ink(
                                                    status,
                                                    cx.theme(),
                                                ))
                                                .into_any_element()
                                        })
                                    }))
                                    .reserve_message_line();
                                    move |options, window, cx| {
                                        let view = InputFieldView::keyed(
                                            window,
                                            cx,
                                            "profile-username",
                                            config.clone(),
                                        );
                                        view.update(cx, |view, cx| {
                                            view.set_presentation(
                                                options.size(),
                                                options.is_disabled(),
                                                cx,
                                            )
                                        });
                                        let field = div().child(view);
                                        if options.layout().is_horizontal() {
                                            field.w_64()
                                        } else {
                                            field.w_full()
                                        }
                                    }
                                })
                                .on_reset(
                                    {
                                        let username_default = defaults.username.clone();
                                        move |cx: &App| {
                                            AppSettings::global(cx).username != username_default
                                        }
                                    },
                                    {
                                        let username_default = defaults.username.clone();
                                        move |_window, cx: &mut App| {
                                            AppSettings::global_mut(cx).username =
                                                username_default.clone();
                                            save_settings(cx);
                                            // The field mirrors the global on
                                            // render; refresh so the reset
                                            // shows up without waiting for the
                                            // next unrelated re-render.
                                            cx.refresh_windows();
                                        }
                                    },
                                ),
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
                        ]),
                ),
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
                                    // Same derived 600 face as the sidebar
                                    // wordmark; the family's own italic is 400.
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_size(px(17.))
                                    .text_color(cx.theme().foreground)
                                    .child("settings-demo"),
                            )
                            .child(
                                Label::new(tr("about.tagline", &AppSettings::global(cx).language))
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
                                        tr("status.version", &AppSettings::global(cx).language),
                                        env!("CARGO_PKG_VERSION")
                                    ),
                                )
                                .icon(StatusIcon::Tag),
                            )
                    })),
                    SettingGroup::new()
                        .title(tr("group.links", &lang))
                        .items(vec![
                            SettingItem::new(
                                tr("item.website", &lang),
                                SettingField::render(|options, _window, cx| {
                                    Button::new("open-website")
                                        .outline()
                                        .with_size(options.size())
                                        .icon(IconName::ExternalLink)
                                        .label(tr(
                                            "action.website_label",
                                            &AppSettings::global(cx).language,
                                        ))
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
                                        .label(tr(
                                            "action.about_label",
                                            &AppSettings::global(cx).language,
                                        ))
                                        .on_click(|_, window, cx| {
                                            window.open_dialog(cx, |dialog, _, cx| {
                                                let lang = &AppSettings::global(cx).language;
                                                dialog
                                                    .title(tr("dialog.about_title", lang))
                                                    .child(tr("dialog.about_body", lang))
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
/// pass through and the component falls back to English for missing locales.
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

/// Re-applies the user's font size (and the app's other theme overrides)
/// after `Theme::change`, which resets `font_size` from the active
/// `ThemeConfig`. `Root::render` pushes `Theme.font_size` into
/// `window.set_rem_size`, so the whole UI scales from this one value.
fn apply_font_size(cx: &mut App) {
    theme::apply_overrides(AppSettings::global(cx).font_size, cx);
}

/// Guards the font-size boundary: the vendored `NumberField` parses
/// `"NaN"`/`"inf"` to non-finite `f64` and its clamp passes NaN through,
/// which would poison `px()`/`set_rem_size`. Non-finite input is rejected
/// (setter keeps the previous value); finite input passes through — the
/// field already clamps to 8–32.
fn normalize_font_size(val: f64) -> Option<f64> {
    val.is_finite().then_some(val)
}

/// Sidebar rail widths: collapsed icon rail and expanded navigation column.
const SIDEBAR_COLLAPSED_W: Pixels = px(48.);
const SIDEBAR_EXPANDED_W: Pixels = px(240.);

/// Once the expanding rail reaches 85% of `SIDEBAR_EXPANDED_W`, labels and
/// search un-hide — "near full" without waiting for the spring to settle.
/// (204px; `Pixels` exposes no const arithmetic, so the literal must be
/// kept in sync.)
const SIDEBAR_LABEL_REVEAL_W: Pixels = px(204.);

/// Critically damped (ω₀ ≈ 32 rad/s), no overshoot: settles inside ~200 ms,
/// and a retarget keeps velocity instead of jumping.
const SIDEBAR_SPRING: SpringConfig = SpringConfig::new(1024.0, 64.0, 1.0);

/// Settling tolerance in pixels — sub-pixel, so the last visible step still
/// lands on the exact target.
const SIDEBAR_SPRING_EPSILON: f32 = 0.5;

/// Whether the sidebar renders icon-only: immediately when collapsing, and
/// until the rail is nearly full width again when expanding.
fn sidebar_icon_mode(collapsed: bool, width: Pixels) -> bool {
    collapsed || width < SIDEBAR_LABEL_REVEAL_W
}

impl Render for SettingsWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let collapsed = cx.global::<ShellState>().collapsed;
        let foreground = cx.theme().foreground;
        let target_w = if collapsed {
            SIDEBAR_COLLAPSED_W
        } else {
            SIDEBAR_EXPANDED_W
        };

        let settings = Settings::new("settings-demo")
            .sidebar_width(target_w)
            // ResizablePanel min/max are authoritative, so pinning the range
            // to the animated width keeps the seam from dragging.
            .sidebar_size_range(target_w..target_w)
            // Suppress the painted 1px resize seam between sidebar and card
            // (hit area stays but the pin prevents drag).
            .show_handle(false)
            // zdesk rail: borderless, inactive items in the muted foreground.
            // The colour lands as the inherited text color on the sidebar
            // root — active/hover items set their own full-contrast colors.
            // (zdesk's 70%-opacity variant resolved to 3.7:1 in light mode,
            // below WCAG AA; `muted.foreground` is 5.9:1 / 6.1:1.)
            //
            // `w_full()` is a fractional, not pixel, width, so the vendored
            // `Sidebar` sees no expanded pixel width and skips its own
            // wrapper tween — the spring below is the single width driver
            // and the sidebar fills its panel exactly.
            .sidebar_style(
                &StyleRefinement::default()
                    .w_full()
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
            // Raised zdesk card around the settings content: rounded,
            // stroked, lighter than the window base, with a 6px inset on all
            // four sides, clipped so page content can't bleed past the
            // radius.
            .content_panel_style(
                &StyleRefinement::default()
                    .rounded_lg()
                    .border_1()
                    .border_color(cx.theme().border)
                    .bg(cx.theme().group_box)
                    .m(px(6.))
                    .overflow_hidden(),
            )
            .pages(self.setting_pages(cx))
            // One spring drives the sidebar panel's width; the element id
            // preserves spring state across frames, so a mid-flight toggle
            // retargets with its current velocity instead of restarting.
            .with_spring(
                "sidebar-width",
                SpringAnimation::new(SIDEBAR_SPRING)
                    .to(target_w)
                    .with_epsilon(SIDEBAR_SPRING_EPSILON),
                move |this, w| {
                    this.sidebar_width(w)
                        .sidebar_size_range(w..w)
                        // Icon-only while narrow: labels and search hide the
                        // moment collapse starts and reappear only once the
                        // rail is nearly back at full width.
                        .sidebar_collapsed(sidebar_icon_mode(collapsed, w))
                        .sidebar_header(shell::sidebar_brand(
                            sidebar_icon_mode(collapsed, w),
                            foreground,
                        ))
                },
            );
        div()
            .size_full()
            .bg(cx.theme().background)
            .child(AppShell::new().settings_view(settings.into_any_element()))
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
            cx.set_global(AppSettings::from_stored(persist::load()));
            let settings = AppSettings::global(cx);
            let dark_mode = settings.dark_mode;
            let font_size = settings.font_size;
            let language = settings.language.clone();
            let launch_at_login = settings.launch_at_login;
            // Theme set + overrides (font size, focus ring), then the stored
            // mode, locale and autostart state.
            theme::install(cx, font_size);
            if dark_mode {
                Theme::change(ThemeMode::Dark, None, cx);
                apply_font_size(cx);
            }
            gpui_kit::component::set_locale(component_locale(&language));
            persist::autostart_converge(launch_at_login);
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

            let window_options = WindowOptions {
                window_bounds: Some(default_window_bounds(cx)),
                titlebar: Some(TitlebarOptions {
                    traffic_light_position: Some(shell::traffic_light_origin()),
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
    use super::{
        SIDEBAR_COLLAPSED_W, SIDEBAR_EXPANDED_W, SIDEBAR_LABEL_REVEAL_W, normalize_font_size,
        sidebar_icon_mode,
    };
    use gpui_kit::px;

    /// Icon-mode is asymmetric: collapsing hides labels at the very first
    /// animated frame (width still ~full), while expanding only restores
    /// them once the rail is nearly back at `SIDEBAR_EXPANDED_W`.
    #[test]
    fn sidebar_icon_mode_hides_early_and_reveals_late() {
        // Collapse direction: `collapsed` alone forces icon mode at any width.
        assert!(sidebar_icon_mode(true, SIDEBAR_EXPANDED_W));
        assert!(sidebar_icon_mode(true, SIDEBAR_COLLAPSED_W));

        // Expand direction: labels stay hidden below the reveal width...
        assert!(sidebar_icon_mode(false, SIDEBAR_COLLAPSED_W));
        assert!(sidebar_icon_mode(false, SIDEBAR_LABEL_REVEAL_W - px(1.)));
        // ...and appear at the reveal width, including a settled full rail.
        assert!(!sidebar_icon_mode(false, SIDEBAR_LABEL_REVEAL_W));
        assert!(!sidebar_icon_mode(false, SIDEBAR_EXPANDED_W));

        // The reveal width must sit strictly between the two rail sizes.
        assert!(SIDEBAR_COLLAPSED_W < SIDEBAR_LABEL_REVEAL_W);
        assert!(SIDEBAR_LABEL_REVEAL_W < SIDEBAR_EXPANDED_W);
    }

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

    #[test]
    fn font_size_rejects_non_finite_input() {
        assert_eq!(normalize_font_size(f64::NAN), None);
        assert_eq!(normalize_font_size(f64::INFINITY), None);
        assert_eq!(normalize_font_size(f64::NEG_INFINITY), None);
        assert_eq!(normalize_font_size(14.0), Some(14.0));
        assert_eq!(normalize_font_size(8.0), Some(8.0));
        assert_eq!(normalize_font_size(32.0), Some(32.0));
    }

    /// The demo field's rules: empty is idle, 3–16 characters of lowercase
    /// letters, digits, `-` and `_` are valid, everything else is rejected.
    #[test]
    fn username_rules() {
        use super::{UsernameRule, username_rule};
        use crate::input_field::Validation;

        assert_eq!(username_rule(""), UsernameRule::Empty);
        assert_eq!(username_rule("ab"), UsernameRule::TooShort);
        assert_eq!(username_rule("abcdefghijklmnopq"), UsernameRule::TooLong);
        assert_eq!(username_rule("Ninja"), UsernameRule::InvalidCharacter);
        assert_eq!(username_rule("ninja!"), UsernameRule::InvalidCharacter);
        assert_eq!(username_rule("ninja"), UsernameRule::Valid);
        assert_eq!(username_rule("a-1"), UsernameRule::Valid);
        assert_eq!(username_rule("a_b_2"), UsernameRule::Valid);

        // Only a non-empty rule produces a message, and only the valid one is
        // a success. The message text itself is locale-dependent, so only the
        // kind is asserted.
        assert!(UsernameRule::Empty.validation("en").is_none());
        assert!(matches!(
            UsernameRule::TooShort.validation("en"),
            Some(Validation::Error(_))
        ));
        assert!(matches!(
            UsernameRule::TooLong.validation("en"),
            Some(Validation::Error(_))
        ));
        assert!(matches!(
            UsernameRule::InvalidCharacter.validation("en"),
            Some(Validation::Error(_))
        ));
        assert!(matches!(
            UsernameRule::Valid.validation("en"),
            Some(Validation::Success(_))
        ));
    }
}
