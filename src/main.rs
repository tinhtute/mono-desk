use std::rc::Rc;

use gpui_kit::ElementId;
use gpui_kit::base::{self as base, StyledExt as _, spring};
use gpui_kit::component::{
    ActiveTheme, AxisExt as _, Disableable, Icon, IconName, IconNamed as _, Root, Sizable, Size,
    Theme, ThemeMode, ThemeStyled as _, TitleBar, WindowExt,
    button::Button,
    checkbox::Checkbox,
    combobox::{Combobox, ComboboxEvent, ComboboxState},
    h_flex,
    label::Label,
    menu::{ContextMenuExt as _, PopupMenuItem},
    notification::Notification,
    searchable_list::{SearchableListDelegate, SearchableListItem, SearchableVec},
    select::{Select, SelectEvent, SelectState},
    setting::{
        NumberFieldOptions, RenderOptions, SettingField, SettingGroup, SettingItem, SettingPage,
        Settings,
    },
    switch::Switch,
    tab::{Tab, TabBar},
    v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{AnimationExt, Pixels, SpringAnimation, SpringConfig};
use gpui_kit::{
    AnyElement, App, AppContext as _, Axis, Bounds, ClipboardItem, Context, Entity, FocusHandle,
    Focusable, FontWeight, Global, InteractiveElement as _, IntoElement, KeyBinding, Menu,
    MenuItem, ParentElement as _, Render, SharedString, StyleRefinement, Styled as _, Subscription,
    TitlebarOptions, Window, WindowBounds, WindowDecorations, WindowOptions, div, img, point, px,
    relative, size, svg,
};
use input_field::{FieldStatus, InputFieldConfig, InputFieldView, Validation};
use shell::{AppShell, ShellState};
use status_chip::{StatusChip, StatusIcon, StatusTone};

mod assets;
mod cadence;
mod i18n;
mod input_field;
mod persist;
mod shell;
mod status_chip;
mod theme;

#[cfg(test)]
mod view_tests;

/// Test-only environment isolation.
///
/// View tests drive the real settings rows, whose handlers call
/// `persist::save` / `persist::autostart_converge`. Both resolve through
/// `$XDG_CONFIG_HOME`/`$HOME` at call time, so redirecting them to a per-
/// process temp dir keeps the owner's `~/.config` untouched.
#[cfg(test)]
pub(crate) mod test_env {
    /// Redirect `HOME` and `XDG_CONFIG_HOME` to a scratch dir. Safe to call
    /// from every test; only the first call does anything.
    pub(crate) fn install() {
        static ONCE: std::sync::Once = std::sync::Once::new();
        ONCE.call_once(|| {
            let root = std::env::temp_dir().join(format!("mono-desk-test-{}", std::process::id()));
            std::fs::create_dir_all(&root).expect("test scratch dir");
            // SAFETY: `set_var` can race other threads' `getenv`; the first
            // view test runs this before any test window (and therefore any
            // persist/font worker thread) exists, and the vars are never
            // touched again.
            unsafe {
                std::env::set_var("XDG_CONFIG_HOME", &root);
                std::env::set_var("HOME", &root);
            }
        });
    }
}

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
/// How often digest notifications arrive (the Notifications → Digest radio).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DigestCadence {
    Immediate,
    Daily,
    Weekly,
}

impl DigestCadence {
    /// The value persisted and used as each radio's key.
    fn value(self) -> &'static str {
        match self {
            Self::Immediate => "immediate",
            Self::Daily => "daily",
            Self::Weekly => "weekly",
        }
    }

    /// The i18n key for the radio's label.
    fn label_key(self) -> &'static str {
        match self {
            Self::Immediate => "digest.immediate",
            Self::Daily => "digest.daily",
            Self::Weekly => "digest.weekly",
        }
    }

    /// Parse a stored value; unknown text falls back to `Immediate` (the same
    /// whitelist `persist::parse` applies).
    fn parse(value: &str) -> Self {
        match value {
            "daily" => Self::Daily,
            "weekly" => Self::Weekly,
            _ => Self::Immediate,
        }
    }

    const ALL: [Self; 3] = [Self::Immediate, Self::Daily, Self::Weekly];
}

/// One toggleable notification channel (the Channels group checkboxes).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Channel {
    Email,
    Desktop,
    Digest,
}

impl Channel {
    const ALL: [Self; 3] = [Self::Email, Self::Desktop, Self::Digest];

    fn id(self) -> &'static str {
        match self {
            Self::Email => "channel-email",
            Self::Desktop => "channel-desktop",
            Self::Digest => "channel-digest",
        }
    }

    fn label_key(self) -> &'static str {
        match self {
            Self::Email => "item.channel_email",
            Self::Desktop => "item.channel_desktop",
            Self::Digest => "item.channel_digest",
        }
    }

    fn get(self, s: &AppSettings) -> bool {
        match self {
            Self::Email => s.channel_email,
            Self::Desktop => s.channel_desktop,
            Self::Digest => s.channel_digest,
        }
    }

    fn set(self, s: &mut AppSettings, enabled: bool) {
        match self {
            Self::Email => s.channel_email = enabled,
            Self::Desktop => s.channel_desktop = enabled,
            Self::Digest => s.channel_digest = enabled,
        }
    }
}

/// Parent "All channels" state: checked when every child is on, unchecked when
/// none is, mixed in between — the only genuine indeterminate surface in the
/// app. Pure so the tri-state mapping is unit-testable without a window.
fn channels_state(s: &AppSettings) -> base::CheckboxState {
    let on = Channel::ALL.iter().filter(|c| c.get(s)).count();
    match on {
        0 => base::CheckboxState::Unchecked,
        3 => base::CheckboxState::Checked,
        _ => base::CheckboxState::Indeterminate,
    }
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
    channel_email: bool,
    channel_desktop: bool,
    channel_digest: bool,
    digest: DigestCadence,
    timezone: SharedString,
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
            channel_email: true,
            channel_desktop: true,
            // One child off: the Channels parent opens in its mixed state.
            channel_digest: false,
            digest: DigestCadence::Immediate,
            timezone: "system".into(),
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
            channel_email: s.channel_email,
            channel_desktop: s.channel_desktop,
            channel_digest: s.channel_digest,
            digest: DigestCadence::parse(&s.digest),
            timezone: s.timezone.into(),
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
            channel_email: self.channel_email,
            channel_desktop: self.channel_desktop,
            channel_digest: self.channel_digest,
            digest: self.digest.value().to_string(),
            timezone: self.timezone.to_string(),
        }
    }
}

/// Persists the current settings; errors are logged inside `persist::save`.
fn save_settings(cx: &App) {
    persist::save(&AppSettings::global(cx).to_stored());
}

// MARK: beUI P1 composed controls
//
// These build on `gpui_base` primitives (unstyled `Checkbox`/`Radio`) or the
// vendored `Select`/`Combobox` states — no vendor patch needed. Indicator
// sizes, colors and the `spring_control` mark fade mirror the vendored
// `checkbox.rs`/`radio.rs` so composed controls sit next to vendored ones
// without a visual seam.

/// Indicator box edge for a composed checkbox/radio, matching the vendored
/// sizing (`size_3`/`size_3p5`/`size_4` are 12/14/16 px at the 14px rem; Large
/// is 18px, `rems(1.125)`).
fn indicator_box_px(size: Size) -> Pixels {
    match size {
        Size::XSmall => px(12.),
        Size::Small => px(14.),
        Size::Large => px(18.),
        _ => px(16.),
    }
}

/// The Channels checkboxes, composed on `gpui_base::Checkbox` so the "All
/// channels" parent can carry `Indeterminate` — the vendored `Checkbox` is
/// boolean-only. The mark (check or minus) fades in via the shared
/// `spring_control` token; under `reduce_motion` the spring resolves
/// immediately, so the mark appears without motion.
#[allow(clippy::too_many_arguments)]
fn channel_checkbox(
    id: &'static str,
    label: SharedString,
    state: base::CheckboxState,
    disabled: bool,
    indented: bool,
    size: Size,
    on_toggle: impl Fn(bool, &mut App) + 'static,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    let focus_handle = window
        .use_keyed_state(id, cx, |_, cx| cx.focus_handle())
        .read(cx)
        .clone();
    let is_focused = focus_handle.is_focused(window);
    let checked = state == base::CheckboxState::Checked;
    let mixed = state == base::CheckboxState::Indeterminate;
    let marked = checked || mixed;

    // Mirrors the vendored indicator: input border over the input background
    // unchecked, `primary` border + fill when marked, halved when disabled.
    let mut border_color = if marked {
        cx.theme().primary
    } else {
        cx.theme().input
    };
    if disabled {
        border_color = border_color.opacity(0.5);
    }
    let radius = cx.theme().radius.min(px(4.));
    let mark_color = if disabled {
        cx.theme().primary_foreground.opacity(0.5)
    } else {
        cx.theme().primary_foreground
    };
    let box_px = indicator_box_px(size);

    // Same fade spring as the vendored check icon; keyed per control so a
    // child's toggle doesn't replay the parent's animation.
    let opacity = spring(
        (id, "mark"),
        if marked { 1f32 } else { 0f32 },
        cx.theme().motion_tokens().spring_control,
        window,
        cx,
    );

    base::Checkbox::new(id)
        .state(state)
        .disabled(disabled)
        .track_focus(&focus_handle)
        .accessibility_label(label.clone())
        .on_change(move |next, _, _, cx| {
            on_toggle(next == base::CheckboxState::Checked, cx);
            save_settings(cx);
            cx.refresh_windows();
        })
        .h_flex()
        .gap_2()
        .items_center()
        .line_height(relative(1.))
        .when(!disabled, |this| this.cursor_pointer())
        .when(disabled, |this| {
            this.text_color(cx.theme().muted_foreground)
        })
        .rounded(radius * 0.5)
        .when(is_focused, |this| this.focus_ring_style(window, cx))
        .when(indented, |this| this.pl_6())
        .child(
            div()
                .size(box_px)
                .flex_shrink_0()
                .border_1()
                .rounded(radius)
                .border_color(border_color)
                .when(!marked, |this| this.bg(cx.theme().input_background()))
                .when(marked, |this| {
                    this.bg(if disabled {
                        cx.theme().primary.opacity(0.5)
                    } else {
                        cx.theme().primary
                    })
                })
                .child(
                    div()
                        .size_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .opacity(opacity)
                        .text_color(mark_color)
                        .child(
                            svg()
                                .map(|this| match size {
                                    Size::XSmall => this.size_2(),
                                    Size::Small => this.size_2p5(),
                                    Size::Large => this.size_3p5(),
                                    _ => this.size_3(),
                                })
                                .path(if mixed {
                                    IconName::Minus.path()
                                } else {
                                    IconName::Check.path()
                                }),
                        ),
                ),
        )
        .child(div().line_height(relative(1.2)).child(label))
        .into_any_element()
}

/// One option of the Digest radio group, composed on `gpui_base::Radio` with
/// the beUI mark — a ring with a fading inner dot, not the vendored check
/// glyph. Arrow keys (Up/Left/Down/Right) move selection and focus between
/// siblings, and only the selected option is a tab stop (roving tabindex).
#[allow(clippy::too_many_arguments)]
fn digest_radio(
    cadence: DigestCadence,
    handles: Vec<FocusHandle>,
    selected: DigestCadence,
    disabled: bool,
    size: Size,
    label: &'static str,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    let ix = DigestCadence::ALL
        .iter()
        .position(|c| *c == cadence)
        .unwrap_or(0);
    let checked = cadence == selected;
    let id: &'static str = match cadence {
        DigestCadence::Immediate => "digest-immediate",
        DigestCadence::Daily => "digest-daily",
        DigestCadence::Weekly => "digest-weekly",
    };
    let focus_handle = handles[ix].clone();
    let is_focused = focus_handle.is_focused(window);

    let mut border_color = if checked {
        cx.theme().primary
    } else {
        cx.theme().input
    };
    let mut dot_color = cx.theme().primary;
    if disabled {
        border_color = border_color.opacity(0.5);
        dot_color = dot_color.opacity(0.5);
    }
    let box_px = indicator_box_px(size);
    let opacity = spring(
        (id, "dot"),
        if checked { 1f32 } else { 0f32 },
        cx.theme().motion_tokens().spring_control,
        window,
        cx,
    );

    base::Radio::new(id)
        .checked(checked)
        .disabled(disabled)
        .track_focus(&focus_handle)
        // Roving tab stop: `digest` always holds a value, so Tab reaches the
        // selected option and arrows move between options.
        .tab_stop(checked)
        .set_position(ix + 1, DigestCadence::ALL.len())
        .accessibility_label(SharedString::from(label))
        .on_key_down(move |event, window, cx| {
            let delta = match event.keystroke.key.as_str() {
                "up" | "left" => -1isize,
                "down" | "right" => 1isize,
                _ => return,
            };
            window.prevent_default();
            let len = DigestCadence::ALL.len() as isize;
            let next_ix = ((ix as isize + delta).rem_euclid(len)) as usize;
            AppSettings::global_mut(cx).digest = DigestCadence::ALL[next_ix];
            save_settings(cx);
            handles[next_ix].focus(window, cx);
            cx.refresh_windows();
        })
        .on_change(move |_, _, window, cx| {
            window.prevent_default();
            AppSettings::global_mut(cx).digest = cadence;
            save_settings(cx);
            cx.refresh_windows();
        })
        .h_flex()
        .gap_x_2()
        .items_center()
        .line_height(relative(1.))
        .when(!disabled, |this| this.cursor_pointer())
        .when(disabled, |this| {
            this.text_color(cx.theme().muted_foreground)
        })
        .rounded(cx.theme().radius * 0.5)
        .when(is_focused, |this| this.focus_ring_style(window, cx))
        .child(
            div()
                .size(box_px)
                .flex_shrink_0()
                .rounded_full_style(cx)
                .border_1()
                .border_color(border_color)
                .bg(cx.theme().input_background())
                .flex()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .size(box_px * 0.5)
                        .rounded_full_style(cx)
                        .bg(dot_color)
                        .opacity(opacity),
                ),
        )
        .child(div().line_height(relative(1.2)).child(label))
        .into_any_element()
}

/// A language row for the Select listbox. `title()` is the endonym, which is
/// also the committed accessible value.
#[derive(Clone, Debug)]
struct LanguageItem {
    code: SharedString,
    name: SharedString,
}

impl SearchableListItem for LanguageItem {
    type Value = SharedString;

    fn title(&self) -> SharedString {
        self.name.clone()
    }

    fn value(&self) -> &Self::Value {
        &self.code
    }
}

fn language_items() -> SearchableVec<LanguageItem> {
    SearchableVec::new(vec![
        LanguageItem {
            code: "en".into(),
            name: "English".into(),
        },
        LanguageItem {
            code: "vi".into(),
            name: "Tiếng Việt".into(),
        },
        LanguageItem {
            code: "zh-CN".into(),
            name: "简体中文".into(),
        },
        LanguageItem {
            code: "ja".into(),
            name: "日本語".into(),
        },
    ])
}

/// Keyed-state slot holding the Language `SelectState` and the subscription
/// that commits `SelectEvent::Confirm` into `AppSettings.language`. Living in
/// keyed state keeps the open menu and cursor across page re-renders.
struct LanguageSelectSlot {
    state: Entity<SelectState<SearchableVec<LanguageItem>>>,
    _subscription: Subscription,
}

/// The Language field as a `Select` listbox (owner decision 3, replacing the
/// dropdown menu). The slot is created once per window; render reconciles the
/// selection with the global so reset/external writes show up — the same
/// contract the input field follows.
fn language_field(label: SharedString) -> SettingField<SharedString> {
    SettingField::render(move |options, window, cx| {
        let slot = window.use_keyed_state("language-select-state", cx, |window, cx| {
            let delegate = language_items();
            let selected = delegate.position(&AppSettings::global(cx).language);
            let state = cx.new(|cx| SelectState::new(delegate, selected, window, cx));
            // Commit on Enter/click: write the global, switch the component
            // locale, persist, re-render.
            let subscription = cx.subscribe_in(
                &state,
                window,
                |_, _, event: &SelectEvent<SearchableVec<LanguageItem>>, _, cx| {
                    let SelectEvent::Confirm(value) = event;
                    if let Some(lang) = value {
                        AppSettings::global_mut(cx).language = lang.clone();
                        gpui_kit::component::set_locale(component_locale(lang));
                        save_settings(cx);
                        cx.refresh_windows();
                    }
                },
            );
            LanguageSelectSlot {
                state,
                _subscription: subscription,
            }
        });
        let state = slot.read(cx).state.clone();
        // The global is the source of truth: adopt reset/external writes.
        let want = AppSettings::global(cx).language.clone();
        if state.read(cx).selected_value() != Some(&want) {
            state.update(cx, |state, cx| state.set_selected_value(&want, window, cx));
        }
        div()
            .id("language-select")
            .when(options.layout().is_horizontal(), |this| this.w_64())
            .child(
                Select::new(&state)
                    .id("language-select-control")
                    .accessibility_label(label.clone())
                    .with_size(options.size())
                    .disabled(options.is_disabled()),
            )
    })
    .on_reset(
        {
            let default = AppSettings::default().language;
            move |cx: &App| AppSettings::global(cx).language != default
        },
        {
            let default = AppSettings::default().language;
            move |_window, cx: &mut App| {
                AppSettings::global_mut(cx).language = default.clone();
                gpui_kit::component::set_locale(component_locale(&default));
                save_settings(cx);
                cx.refresh_windows();
            }
        },
    )
}

/// A time zone row for the Combobox. `title()` is localized for the "system"
/// sentinel and the IANA name otherwise; `matches` also searches the IANA
/// code so "bangkok" and "GMT+7"-style fragments both hit.
#[derive(Clone, Debug)]
struct TimezoneItem {
    value: SharedString,
    title: SharedString,
}

impl SearchableListItem for TimezoneItem {
    type Value = SharedString;

    fn title(&self) -> SharedString {
        self.title.clone()
    }

    fn value(&self) -> &Self::Value {
        &self.value
    }

    fn matches(&self, query: &str) -> bool {
        let q = query.to_lowercase();
        self.title.to_lowercase().contains(&q) || self.value.to_lowercase().contains(&q)
    }
}

/// The combobox's options: a localized "follow system" sentinel plus a
/// curated set of IANA zones spanning the offset range — long enough that
/// the search field earns its keep (owner decision 6: combobox kept here).
fn timezone_items(lang: &str) -> SearchableVec<TimezoneItem> {
    let item = |value: &'static str| TimezoneItem {
        value: value.into(),
        title: value.into(),
    };
    SearchableVec::new(vec![
        TimezoneItem {
            value: "system".into(),
            title: tr("tz.system", lang).into(),
        },
        item("Pacific/Auckland"),
        item("Pacific/Honolulu"),
        item("America/Anchorage"),
        item("America/Los_Angeles"),
        item("America/Denver"),
        item("America/Chicago"),
        item("America/New_York"),
        item("America/Sao_Paulo"),
        item("Atlantic/Reykjavik"),
        item("Europe/London"),
        item("Europe/Berlin"),
        item("Europe/Kyiv"),
        item("Africa/Cairo"),
        item("Asia/Dubai"),
        item("Asia/Kolkata"),
        item("Asia/Bangkok"),
        item("Asia/Saigon"),
        item("Asia/Singapore"),
        item("Asia/Tokyo"),
        item("Australia/Sydney"),
    ])
}

/// Keyed-state slot for the Timezone combobox: the `ComboboxState`, the
/// `Confirm` subscription that commits into `AppSettings.timezone`, and the
/// locale the delegate's sentinel title was built in.
struct TimezoneComboSlot {
    state: Entity<ComboboxState<SearchableVec<TimezoneItem>>>,
    items_lang: SharedString,
    _subscription: Subscription,
}

/// The Time Zone field: the app's one searchable surface, so it carries the
/// beUI combobox. Single-select with an in-menu search input; commits on
/// `Confirm` (item pick or popover close, both paths emit it).
fn timezone_field(
    placeholder: &'static str,
    search_placeholder: &'static str,
) -> SettingField<SharedString> {
    SettingField::render(move |options, window, cx| {
        let lang = AppSettings::global(cx).language.clone();
        let slot = window.use_keyed_state("timezone-combo-state", cx, |window, cx| {
            let delegate = timezone_items(&lang);
            let selected = delegate
                .position(&AppSettings::global(cx).timezone)
                .into_iter()
                .collect();
            let state =
                cx.new(|cx| ComboboxState::new(delegate, selected, window, cx).searchable(true));
            let subscription = cx.subscribe_in(
                &state,
                window,
                |_, _, event: &ComboboxEvent<SearchableVec<TimezoneItem>>, _, cx| {
                    if let ComboboxEvent::Confirm(values) = event
                        && let Some(tz) = values.first()
                    {
                        AppSettings::global_mut(cx).timezone = tz.clone();
                        save_settings(cx);
                        cx.refresh_windows();
                    }
                },
            );
            TimezoneComboSlot {
                state,
                items_lang: lang.clone(),
                _subscription: subscription,
            }
        });
        let state = slot.read(cx).state.clone();
        // The sentinel's title is localized: rebuild the delegate when the
        // locale changed since the slot was created.
        if slot.read(cx).items_lang != lang {
            let delegate = timezone_items(&lang);
            slot.update(cx, |slot, _| slot.items_lang = lang.clone());
            state.update(cx, |state, cx| state.set_items(delegate, window, cx));
        }
        let want = AppSettings::global(cx).timezone.clone();
        if state.read(cx).selected_value().as_ref() != Some(&want) {
            state.update(cx, |state, cx| {
                state.set_selected_values(std::slice::from_ref(&want), window, cx)
            });
        }
        div()
            .id("timezone-combo")
            .debug_selector(|| "timezone-combo".into())
            .when(options.layout().is_horizontal(), |this| this.w_64())
            .child(
                Combobox::new(&state)
                    .placeholder(placeholder)
                    .search_placeholder(search_placeholder)
                    .with_size(options.size())
                    .disabled(options.is_disabled()),
            )
    })
    .on_reset(
        {
            let default = AppSettings::default().timezone;
            move |cx: &App| AppSettings::global(cx).timezone != default
        },
        {
            let default = AppSettings::default().timezone;
            move |_window, cx: &mut App| {
                AppSettings::global_mut(cx).timezone = default.clone();
                save_settings(cx);
                cx.refresh_windows();
            }
        },
    )
}

/// The Dark Mode switch's effect: flip the theme, remember the choice, and
/// re-apply the derived overrides — shared by the row's click handler and the
/// page reset so the two cannot drift.
fn set_dark_mode(val: bool, cx: &mut App) {
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
}

/// The Launch at login checkbox's effect — shared by the row's click handler
/// and the page reset.
fn set_launch_at_login(val: bool, cx: &mut App) {
    AppSettings::global_mut(cx).launch_at_login = val;
    persist::autostart_converge(val);
    save_settings(cx);
}

/// The settings-row `Switch`.
///
/// `SettingField::switch` renders a bare vendored control with no accessible
/// name; composing the control here keeps the field's `on_reset` semantics
/// while every switch row carries its row title as the `accessibility_label`.
/// `test_support` on the underlying `gpui_base` control makes the role,
/// toggled state and label observable in view tests.
fn settings_switch(
    id: impl Into<ElementId>,
    checked: bool,
    accessibility_label: impl Into<SharedString>,
    options: &RenderOptions,
    on_click: impl Fn(&bool, &mut Window, &mut App) + 'static,
) -> Switch {
    Switch::new(id)
        .checked(checked)
        .accessibility_label(accessibility_label)
        .disabled(options.is_disabled())
        .with_size(options.size())
        .on_click(on_click)
}

/// The settings-row `Checkbox`; same reasoning as [`settings_switch`].
fn settings_checkbox(
    id: impl Into<ElementId>,
    checked: bool,
    accessibility_label: impl Into<SharedString>,
    options: &RenderOptions,
    on_click: impl Fn(&bool, &mut Window, &mut App) + 'static,
) -> Checkbox {
    Checkbox::new(id)
        .checked(checked)
        .accessibility_label(accessibility_label)
        .disabled(options.is_disabled())
        .with_size(options.size())
        .on_click(on_click)
}

struct SettingsWindow {
    focus_handle: FocusHandle,
}

impl SettingsWindow {
    fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        cx.set_global(ShellState::default());
        cx.set_global(cadence::DemoState::demo());
        Self {
            focus_handle: cx.focus_handle(),
        }
    }

    fn setting_pages(cx: &App) -> Vec<SettingPage> {
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
                                SettingField::render({
                                    let label = tr("item.dark_mode", &lang);
                                    move |options, _window, cx| {
                                        div().debug_selector(|| "dark-mode-row".into()).child(
                                            settings_switch(
                                                "dark-mode-switch",
                                                cx.theme().mode.is_dark(),
                                                label,
                                                options,
                                                move |checked: &bool, _, cx: &mut App| {
                                                    set_dark_mode(*checked, cx);
                                                },
                                            ),
                                        )
                                    }
                                })
                                // An element field carries no typed value, so
                                // the page reset needs the handler explicitly.
                                .on_reset(
                                    move |cx: &App| cx.theme().mode.is_dark() != defaults.dark_mode,
                                    move |_window, cx: &mut App| {
                                        set_dark_mode(defaults.dark_mode, cx);
                                    },
                                ),
                            )
                            .description(tr("item.dark_mode.desc", &lang)),
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
                        .title(tr("group.region", &lang))
                        .items(vec![
                            SettingItem::new(
                                tr("item.language", &lang),
                                language_field(SharedString::from(tr("item.language", &lang))),
                            )
                            .description(tr("item.language.desc", &lang)),
                            SettingItem::new(
                                tr("item.timezone", &lang),
                                timezone_field(
                                    tr("tz.placeholder", &lang),
                                    tr("tz.search_placeholder", &lang),
                                ),
                            )
                            .description(tr("item.timezone.desc", &lang)),
                        ]),
                    SettingGroup::new()
                        .title(tr("group.startup", &lang))
                        .items(vec![
                            SettingItem::new(
                                tr("item.launch_at_login", &lang),
                                SettingField::render({
                                    let label = tr("item.launch_at_login", &lang);
                                    move |options, _window, cx| {
                                        div().debug_selector(|| "launch-at-login-row".into()).child(
                                            settings_checkbox(
                                                "launch-at-login-checkbox",
                                                AppSettings::global(cx).launch_at_login,
                                                label,
                                                options,
                                                move |checked: &bool, _, cx: &mut App| {
                                                    set_launch_at_login(*checked, cx);
                                                },
                                            ),
                                        )
                                    }
                                })
                                // An element field carries no typed value, so
                                // the page reset needs the handler explicitly.
                                .on_reset(
                                    move |cx: &App| {
                                        AppSettings::global(cx).launch_at_login
                                            != defaults.launch_at_login
                                    },
                                    move |_window, cx: &mut App| {
                                        set_launch_at_login(defaults.launch_at_login, cx);
                                    },
                                ),
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
                                        .debug_selector(|| "notifications-enabled-row".into())
                                        .gap_2()
                                        .child(StatusChip::labelled(tone, label))
                                        .child(
                                            Switch::new("notifications-enabled")
                                                .checked(enabled)
                                                .accessibility_label(tr(
                                                    "item.notifications",
                                                    &settings.language,
                                                ))
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
                    SettingGroup::new().title(tr("group.channels", &lang)).item(
                        SettingItem::new(
                            tr("group.channels", &lang),
                            SettingField::render(|options, window, cx| {
                                let settings = AppSettings::global(cx);
                                let disabled =
                                    options.is_disabled() || !settings.notifications_enabled;
                                let size = options.size();
                                let lang = settings.language.clone();
                                let parent_state = channels_state(settings);
                                // Snapshot child states before handing `cx`
                                // and `window` to the control builders —
                                // `use_keyed_state` inside needs `&mut`.
                                let children: Vec<_> = Channel::ALL
                                    .iter()
                                    .map(|channel| {
                                        (
                                            *channel,
                                            tr(channel.label_key(), &lang),
                                            channel.get(settings),
                                        )
                                    })
                                    .collect();
                                let parent = channel_checkbox(
                                    "channels-all",
                                    tr("item.channel_all", &lang).into(),
                                    parent_state,
                                    disabled,
                                    false,
                                    size,
                                    |next, cx| {
                                        for channel in Channel::ALL {
                                            channel.set(AppSettings::global_mut(cx), next);
                                        }
                                    },
                                    window,
                                    cx,
                                );
                                v_flex()
                                    .id("channels-group")
                                    .gap_2()
                                    .w_full()
                                    // Parent: activating from mixed or checked
                                    // selects all; from unchecked does nothing
                                    // to a no-selection — it selects all.
                                    .child(parent)
                                    .children(children.into_iter().map(|(channel, label, on)| {
                                        channel_checkbox(
                                            channel.id(),
                                            label.into(),
                                            if on {
                                                base::CheckboxState::Checked
                                            } else {
                                                base::CheckboxState::Unchecked
                                            },
                                            disabled,
                                            true,
                                            size,
                                            move |next, cx| {
                                                channel.set(AppSettings::global_mut(cx), next);
                                            },
                                            window,
                                            cx,
                                        )
                                        .into_any_element()
                                    }))
                            })
                            .on_reset(
                                |cx: &App| {
                                    let settings = AppSettings::global(cx);
                                    Channel::ALL
                                        .iter()
                                        .any(|c| c.get(settings) != c.get(&AppSettings::default()))
                                },
                                |_window, cx: &mut App| {
                                    let defaults = AppSettings::default();
                                    let settings = AppSettings::global_mut(cx);
                                    for channel in Channel::ALL {
                                        channel.set(settings, channel.get(&defaults));
                                    }
                                    save_settings(cx);
                                    cx.refresh_windows();
                                },
                            ),
                        )
                        .description(tr("item.channels.desc", &lang))
                        .layout(Axis::Vertical),
                    ),
                    SettingGroup::new().title(tr("group.digest", &lang)).item(
                        SettingItem::new(
                            tr("group.digest", &lang),
                            SettingField::render(|options, window, cx| {
                                let (selected, lang, size) = {
                                    let settings = AppSettings::global(cx);
                                    (settings.digest, settings.language.clone(), options.size())
                                };
                                let disabled = options.is_disabled()
                                    || !AppSettings::global(cx).notifications_enabled;
                                // One handle per option, keyed in the window
                                // so arrow-key handling can move focus.
                                let handles: Vec<FocusHandle> = DigestCadence::ALL
                                    .iter()
                                    .map(|c| {
                                        window
                                            .use_keyed_state(c.value(), cx, |_, cx| {
                                                cx.focus_handle()
                                            })
                                            .read(cx)
                                            .clone()
                                    })
                                    .collect();
                                base::RadioGroup::new("digest-radio-group")
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .children(DigestCadence::ALL.map(|cadence| {
                                        digest_radio(
                                            cadence,
                                            handles.clone(),
                                            selected,
                                            disabled,
                                            size,
                                            tr(cadence.label_key(), &lang),
                                            window,
                                            cx,
                                        )
                                        .into_any_element()
                                    }))
                            })
                            .on_reset(
                                |cx: &App| {
                                    AppSettings::global(cx).digest != AppSettings::default().digest
                                },
                                |_window, cx: &mut App| {
                                    AppSettings::global_mut(cx).digest =
                                        AppSettings::default().digest;
                                    save_settings(cx);
                                    cx.refresh_windows();
                                },
                            ),
                        )
                        .description(tr("item.digest.desc", &lang))
                        .layout(Axis::Vertical),
                    ),
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
                                    // The row carries the visible title, so
                                    // the field's own label stays off — the
                                    // aria label still names the control for
                                    // screen readers (no placeholder fallback).
                                    .aria_label(tr("item.username", &lang))
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
            cadence::showcase::page(&lang),
            SettingPage::new(tr("page.about", &lang))
                .icon(Icon::new(IconName::Info))
                .groups(vec![
                    SettingGroup::new().item(SettingItem::render(|_options, window, cx| {
                        let lang = AppSettings::global(cx).language.clone();
                        // The About tab selection lives in keyed window state;
                        // `use_keyed_state` self-observes so writes re-render.
                        let tab_ix = window.use_keyed_state("about-tab", cx, |_, _| 0usize);
                        v_flex()
                            .id("about-brand-card")
                            .gap_3()
                            .w_full()
                            .items_center()
                            .justify_center()
                            // Owner decision 4: the brand block carries the
                            // app's context-menu surface (Copy version).
                            .context_menu(move |menu, _menu_window, cx| {
                                menu.item(
                                    PopupMenuItem::new(tr(
                                        "action.copy_version",
                                        &AppSettings::global(cx).language,
                                    ))
                                    .icon(IconName::Copy)
                                    .on_click(
                                        move |_, _, cx| {
                                            cx.write_to_clipboard(ClipboardItem::new_string(
                                                env!("CARGO_PKG_VERSION").to_string(),
                                            ));
                                        },
                                    ),
                                )
                            })
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
                                Label::new(tr("about.tagline", &lang))
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
                                        tr("status.version", &lang),
                                        env!("CARGO_PKG_VERSION")
                                    ),
                                )
                                .icon(StatusIcon::Tag),
                            )
                            .child(
                                // Owner decision 5: the app builds a real
                                // `TabBar` (Info / Licenses / Changelog). The
                                // vendored TabBar scrolls and closes like a
                                // document tab strip; for settings-style
                                // pinned tabs `menu(false)` hides its
                                // dropdown — the three tabs fit at every
                                // supported font size (32px max: 3 short
                                // labels ≪ the panel width).
                                TabBar::new("about-tabs")
                                    .underline()
                                    .pt_2()
                                    .w_full()
                                    .with_size(Size::Small)
                                    .menu(false)
                                    .selected_index(*tab_ix.read(cx))
                                    .justify_center()
                                    .child(Tab::new().label(tr("about.tab.info", &lang)))
                                    .child(Tab::new().label(tr("about.tab.licenses", &lang)))
                                    .child(Tab::new().label(tr("about.tab.changelog", &lang)))
                                    .on_click(move |ix: &usize, _, cx| {
                                        tab_ix.update(cx, |s, _| *s = *ix);
                                    }),
                            )
                    })),
                    SettingGroup::new().item(SettingItem::render(|_options, window, cx| {
                        let lang = AppSettings::global(cx).language.clone();
                        let ix = *window
                            .use_keyed_state("about-tab", cx, |_, _| 0usize)
                            .read(cx);
                        div()
                            .id("about-tab-panel")
                            .w_full()
                            .pt_2()
                            .child(match ix.min(2) {
                                0 => about_info_panel(&lang, cx),
                                1 => about_licenses_panel(&lang, cx),
                                _ => about_changelog_panel(&lang, cx),
                            })
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

/// Info tab panel: a short prose paragraph about the demo.
fn about_info_panel(lang: &str, cx: &App) -> AnyElement {
    div()
        .w_full()
        .max_w(px(480.))
        .mx_auto()
        .text_color(cx.theme().muted_foreground)
        .child(tr("about.info.body", lang))
        .into_any_element()
}

/// Licenses tab panel: the bundled licenses as quiet muted lines.
fn about_licenses_panel(lang: &str, cx: &App) -> AnyElement {
    v_flex()
        .w_full()
        .max_w(px(480.))
        .mx_auto()
        .gap_2()
        .text_color(cx.theme().muted_foreground)
        .children([
            tr("about.licenses.0", lang),
            tr("about.licenses.1", lang),
            tr("about.licenses.2", lang),
        ])
        .into_any_element()
}

/// Changelog tab panel: the release highlight, muted like release notes.
fn about_changelog_panel(lang: &str, cx: &App) -> AnyElement {
    div()
        .w_full()
        .max_w(px(480.))
        .mx_auto()
        .text_color(cx.theme().muted_foreground)
        .child(tr("about.changelog.0", lang))
        .into_any_element()
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
            .pages(Self::setting_pages(cx))
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
