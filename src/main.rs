use std::rc::Rc;

use gpui_kit::ElementId;
use gpui_kit::base::{self as base, StyledExt as _, spring};
use gpui_kit::component::{
    ActiveTheme, AxisExt as _, Colorize as _, Disableable, Icon, IconName, IconNamed as _, Root,
    Selectable as _, Sizable, Size, Theme, ThemeStyled as _, TitleBar, WindowExt,
    button::{Button, ButtonVariants as _},
    checkbox::Checkbox,
    color_picker::{ColorPicker, ColorPickerEvent, ColorPickerState},
    combobox::{Combobox, ComboboxEvent, ComboboxState},
    h_flex,
    label::Label,
    menu::{ContextMenuExt as _, PopupMenuItem},
    notification::Notification,
    searchable_list::{SearchableListDelegate, SearchableListItem, SearchableVec},
    select::{Select, SelectEvent, SelectState},
    setting::{RenderOptions, SettingField, SettingGroup, SettingItem, SettingPage, Settings},
    slider::{Slider, SliderEvent, SliderState, SliderValue},
    switch::Switch,
    tab::{Tab, TabBar},
    v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{AnimationExt, Pixels, SpringAnimation, SpringConfig};
use gpui_kit::{
    AnyElement, App, AppContext as _, Axis, Bounds, ClipboardItem, Context, Entity, Fill,
    FocusHandle, Focusable, FontWeight, Global, Hsla, InteractiveElement as _, IntoElement,
    KeyBinding, Menu, MenuItem, MouseButton, MouseMoveEvent, ParentElement as _, Render,
    SharedString, StatefulInteractiveElement as _, StyleRefinement, Styled as _, Subscription,
    TitlebarOptions, Window, WindowBackgroundAppearance, WindowBounds, WindowDecorations,
    WindowOptions, div, img, point, px, relative, size, svg,
};
use input_field::{FieldStatus, InputFieldConfig, InputFieldView, Validation};
use shell::{AppShell, ShellState};
use status_chip::{StatusChip, StatusIcon, StatusTone};
use theme::Appearance;
use tokens::UiTokens;

mod assets;
mod cadence;
mod i18n;
mod input_field;
mod persist;
mod shell;
mod status_chip;
mod theme;
mod tokens;

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
    appearance: Appearance,
    /// Brand accent, or `None` for the theme's own. Persisted as `#RRGGBB`.
    accent: Option<Hsla>,
    language: SharedString,
    font_size: f64,
    /// Whether the window background is translucent (the glass master
    /// switch). `false` renders the shipped opaque palette.
    background_translucent: bool,
    /// The single frost-shell opacity in percent while translucent; clamped
    /// to 40–100. One layer for the whole window, zeron-style.
    window_opacity: f64,
    /// Sidebar ink tone in percent of the max wash, 0–100. Painted *over* the
    /// frost (never its own glass base), so the rail separates from the
    /// content without a second alpha surface.
    sidebar_tone: f64,
    /// Persisted rail width in logical pixels (242–400). The drag seam writes
    /// it live and persists on release, like the percentage sliders.
    sidebar_width: f64,
    /// Whether the window asks the platform to blur what is behind it. Only
    /// meaningful while `background_translucent` is on.
    background_blur: bool,
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
            appearance: Appearance::default(),
            accent: None,
            language: "en".into(),
            // Matches the theme's `font.size` (14px) so the applied default
            // preserves the approved visual baseline. `theme::tests` pins the
            // two together.
            font_size: 14.0,
            // Glass is opt-in; when it is switched on, the default frost
            // opacity matches monocode's 0.85 surface (see
            // `theme::WINDOW_OPACITY_DEFAULT`), and the sidebar keeps the
            // subtle 5% ink tone zeron's glass sidebar uses.
            background_translucent: false,
            window_opacity: theme::WINDOW_OPACITY_DEFAULT,
            sidebar_tone: theme::SIDEBAR_TONE_DEFAULT,
            sidebar_width: theme::SIDEBAR_WIDTH_DEFAULT,
            background_blur: false,
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
            appearance: Appearance::parse(&s.appearance).unwrap_or_default(),
            accent: theme::accent_from_str(&s.accent),
            language: s.language.into(),
            font_size: if s.font_size.is_finite() {
                s.font_size
            } else {
                14.0
            },
            background_translucent: s.background_translucent,
            // The codec already clamps; this is the same guard as
            // `font_size` for a settings struct built in memory.
            window_opacity: if s.window_opacity.is_finite() {
                theme::clamp_window_opacity(s.window_opacity)
            } else {
                AppSettings::default().window_opacity
            },
            sidebar_tone: if s.sidebar_tone.is_finite() {
                theme::clamp_sidebar_tone(s.sidebar_tone)
            } else {
                AppSettings::default().sidebar_tone
            },
            sidebar_width: if s.sidebar_width.is_finite() {
                theme::clamp_sidebar_width(s.sidebar_width)
            } else {
                AppSettings::default().sidebar_width
            },
            background_blur: s.background_blur,
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
            appearance: self.appearance.as_str().to_string(),
            accent: self
                .accent
                .map(|accent| accent.to_hex())
                .unwrap_or_default(),
            language: self.language.to_string(),
            font_size: self.font_size,
            background_translucent: self.background_translucent,
            window_opacity: self.window_opacity,
            sidebar_tone: self.sidebar_tone,
            sidebar_width: self.sidebar_width,
            background_blur: self.background_blur,
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

/// Glass (translucent background) plumbing, zeron-style.
///
/// GPUI exposes one window-level knob for this — `WindowBackgroundAppearance` —
/// and it makes the *window surface* transparent/blurred. The app then paints
/// exactly one glass layer on top: the `AppShell` base (`shell_frost_bg`), a
/// full-window frost at the Window-opacity alpha. Everything above it is
/// transparent chrome: the title bar paints nothing, the sidebar column adds
/// only an ink tone (`sidebar_tone_bg`) plus a 1px right hairline, and the
/// content sits flush on the frost. The vendored `Root` would paint an opaque
/// base of its own, so it is switched off through `WindowRoot`.
///
/// The old model painted each pane at its own alpha; stacking two such layers
/// composites to `1-(1-a)^2` and reads opaque, so it is replaced rather than
/// extended.
impl AppSettings {
    /// The Window-opacity percent the frost layer paints with.
    ///
    /// While translucency is off this is the opaque end, so the palette is
    /// never alpha'd without the user asking for it and the default rendering
    /// stays byte-identical to `mono.json`.
    pub(crate) fn window_opacity(&self) -> f64 {
        if !self.background_translucent {
            return theme::WINDOW_OPACITY_MAX;
        }
        theme::clamp_window_opacity(self.window_opacity)
    }

    /// The stored Window opacity, clamped — independent of the master switch.
    /// The slider shows this (and reset compares against it) while the painted
    /// alpha uses [`Self::window_opacity`], which is fully opaque when off.
    pub(crate) fn stored_window_opacity(&self) -> f64 {
        theme::clamp_window_opacity(self.window_opacity)
    }

    /// The sidebar ink-tone percent painted by the rail.
    ///
    /// Unconditional: zeron paints `wash(0.05)` on its opaque shell too, and
    /// the tone is ink over whatever base is present — only the frost shell
    /// itself is gated by the translucency master switch.
    pub(crate) fn sidebar_tone(&self) -> f64 {
        theme::clamp_sidebar_tone(self.sidebar_tone)
    }

    /// The stored sidebar tone, clamped (the slider's own value).
    pub(crate) fn stored_sidebar_tone(&self) -> f64 {
        theme::clamp_sidebar_tone(self.sidebar_tone)
    }

    /// The persisted rail width, clamped, as layout pixels.
    pub(crate) fn sidebar_width(&self) -> Pixels {
        px(theme::clamp_sidebar_width(self.sidebar_width) as f32)
    }

    /// The stored (clamped) value of one glass percentage control.
    fn stored(&self, control: GlassControl) -> f64 {
        match control {
            GlassControl::WindowOpacity => self.stored_window_opacity(),
            GlassControl::SidebarTone => self.stored_sidebar_tone(),
        }
    }
}

/// The colour the single full-window frost layer paints right now.
pub(crate) fn shell_frost_bg(cx: &App) -> Hsla {
    theme::glass_surface(
        cx.theme().background,
        AppSettings::global(cx).window_opacity(),
    )
}

/// The ink the sidebar column paints over the frost right now.
///
/// A *tone*, not a surface: it is the theme foreground at the tone alpha, so
/// the sidebar separates from the content without becoming a second base
/// layer (zeron paints `wash(0.05)` for the same reason).
pub(crate) fn sidebar_tone_bg(cx: &App) -> Hsla {
    let alpha = theme::sidebar_tone_alpha(AppSettings::global(cx).sidebar_tone());
    Hsla {
        a: alpha,
        ..cx.theme().foreground
    }
}

/// `true` while the window paints a translucent base.
pub(crate) fn glass_enabled(cx: &App) -> bool {
    AppSettings::global(cx).background_translucent
}

/// The window-background appearance the glass settings ask for.
///
/// `Blurred` is a *request*, not a guarantee: macOS installs a vibrancy view
/// behind the content, Windows uses DWM acrylic (experimental under GPUI's
/// DirectComposition windows), and Linux only blurs when the compositor
/// advertises a blur protocol — elsewhere the engine degrades to plain
/// transparency instead of failing.
fn window_background_for(translucent: bool, blur: bool) -> WindowBackgroundAppearance {
    if !translucent {
        return WindowBackgroundAppearance::Opaque;
    }
    if blur {
        WindowBackgroundAppearance::Blurred
    } else {
        WindowBackgroundAppearance::Transparent
    }
}

/// Handle to the window's vendored `Root` element.
///
/// `Root` renders the window's outermost `div` and paints its own opaque
/// `tokens.background`; a view inside it cannot reach that paint. Glass needs
/// this handle to switch the paint off — without it the opaque Root layer
/// would sit behind every translucent app surface and the desktop would never
/// show through.
///
/// Single-window by design (NG7): the app opens one window, so the last
/// handle published by `open_window` is the only one there is.
struct WindowRoot(Entity<Root>);

impl Global for WindowRoot {}

/// The `Root` style override for the current glass state.
///
/// `Some(clear)` while glass is on (the app's panes own the only base paint);
/// `None` restores the vendored token paint for the default look. One helper
/// so startup (`open_window`'s first frame) and [`apply_glass`] cannot drift.
fn root_glass_bg(cx: &App, translucent: bool) -> Option<Fill> {
    translucent.then(|| cx.theme().transparent.into())
}

/// Applies the glass settings: the window background appearance (the part
/// that makes the OS window transparent/blurred), the `Root` base-paint
/// override, and a repaint so every per-surface paint helper re-reads the new
/// alpha.
fn apply_glass(window: &mut Window, cx: &mut App) {
    let settings = AppSettings::global(cx);
    let translucent = settings.background_translucent;
    let background = window_background_for(translucent, settings.background_blur);
    window.set_background_appearance(background);
    if let Some(root) = cx.try_global::<WindowRoot>().map(|root| root.0.clone()) {
        // `None` restores the vendored token paint, i.e. exactly the default
        // look; a transparent fill hands the base to the app's panes. Written
        // only when it actually changes: the mode flips and the reset funnel
        // re-enter here with an unchanged value, and the opacity slider never
        // reaches this function at all (it only repaints).
        let base = root_glass_bg(cx, translucent);
        root.update(cx, |root, _| {
            if root.style().background.is_some() != translucent {
                root.style().background = base;
            }
        });
    }
    cx.refresh_windows();
}

/// The Translucent-background switch's effect.
fn set_glass_translucent(enabled: bool, window: &mut Window, cx: &mut App) {
    AppSettings::global_mut(cx).background_translucent = enabled;
    apply_glass(window, cx);
    save_settings(cx);
}

/// The Background-blur switch's effect.
fn set_glass_blur(blur: bool, window: &mut Window, cx: &mut App) {
    AppSettings::global_mut(cx).background_blur = blur;
    apply_glass(window, cx);
    save_settings(cx);
}

/// One glass percentage control's effect. `persist` is false while the
/// slider is being dragged, so one drag does not rewrite the config per frame
/// (same contract as the Font Size slider).
///
/// Only painted alpha/tone changes, so this repaints instead of going through
/// [`apply_glass`] — reconfiguring the window surface (and its GPU alpha mode)
/// on every drag frame would be pure overhead.
fn set_glass_percent(
    control: GlassControl,
    percent: f64,
    persist: bool,
    _window: &mut Window,
    cx: &mut App,
) {
    control.set(AppSettings::global_mut(cx), percent);
    cx.refresh_windows();
    if persist {
        save_settings(cx);
    }
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

/// Keyed-state slot holding the Accent `ColorPickerState` and the
/// subscription that commits `ColorPickerEvent::Change` into
/// `AppSettings.accent`. Living in keyed state keeps the open popover and its
/// sliders across page re-renders.
struct AccentPickerSlot {
    state: Entity<ColorPickerState>,
    _subscription: Subscription,
}

/// The Accent field: the palette's named presets as swatches, plus the
/// vendored `ColorPicker` for any other color.
///
/// The global is the source of truth — a page reset or an external write is
/// adopted on the next render, the same contract `language_field` follows.
/// Picking writes through `set_accent`, which re-applies the theme overrides
/// so the change is visible immediately.
fn accent_field(label: SharedString) -> SettingField<SharedString> {
    SettingField::render(move |options, window, cx| {
        let slot = window.use_keyed_state("accent-picker-state", cx, |window, cx| {
            let initial = AppSettings::global(cx)
                .accent
                .unwrap_or_else(|| cx.theme().colors.primary);
            let state = cx.new(|cx| ColorPickerState::new(window, cx).default_value(initial));
            let subscription =
                cx.subscribe_in(&state, window, |_, _, event: &ColorPickerEvent, _, cx| {
                    // The event has one variant today; `set_accent(None)`
                    // would mean "back to the theme's own", which the picker
                    // does not currently offer.
                    let ColorPickerEvent::Change(value) = event;
                    set_accent(*value, cx);
                });
            AccentPickerSlot {
                state,
                _subscription: subscription,
            }
        });
        let state = slot.read(cx).state.clone();
        let selected = AppSettings::global(cx).accent;
        if let Some(selected) = selected
            && state.read(cx).value() != Some(selected)
        {
            state.update(cx, |state, cx| state.set_value(selected, window, cx));
        }
        // `filter_map`, not `expect`: this runs inside `render`, so a panic
        // here would abort the app mid-frame rather than fail a gate. The
        // hex strings are compile-time constants and
        // `stored_accent_parses_or_falls_back` already asserts every preset
        // parses, so no swatch is actually dropped — but a bad future edit
        // now loses one swatch instead of crashing the window.
        let presets: Vec<(SharedString, Hsla)> = theme::ACCENT_PRESETS
            .iter()
            .filter_map(|(name, hex)| {
                theme::accent_from_str(hex).map(|color| (SharedString::from(*name), color))
            })
            .collect();
        div()
            .id("accent-control")
            .when(options.layout().is_horizontal(), |this| this.w_64())
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .debug_selector(|| "accent-row".into())
                    .children(
                        presets
                            .iter()
                            .map(|(name, color)| {
                                accent_swatch(name, *color, selected == Some(*color), cx)
                            })
                            .collect::<Vec<_>>(),
                    )
                    .child(
                        ColorPicker::new(&state)
                            .featured_colors(presets.iter().map(|(_, color)| *color).collect())
                            .accessibility_label(label.clone())
                            .with_size(options.size()),
                    ),
            )
    })
    .on_reset(
        |cx: &App| AppSettings::global(cx).accent.is_some(),
        |_window, cx: &mut App| set_accent(None, cx),
    )
}

/// Keyed-state slot holding the Font Size `SliderState` and the subscription
/// that commits it into `AppSettings.font_size`.
struct FontSizeSlot {
    state: Entity<SliderState>,
    _subscription: Subscription,
}

/// Whether the Font Size setting still holds its default — the reset
/// affordance is inert while it does.
fn font_size_at_default(size: f64) -> bool {
    size == AppSettings::default().font_size
}

/// The Font Size field: a slider over the app's 8–32 clamp, a preview line
/// drawn at the chosen size, and a reset that is inert at the default.
///
/// The slider reports continuously while dragging, so the live theme is
/// updated on every `Change` but the file is written only on `Release` —
/// a drag would otherwise rewrite the config dozens of times.
fn font_size_field(label: SharedString) -> SettingField<SharedString> {
    SettingField::render(move |options, window, cx| {
        let slot = window.use_keyed_state("font-size-slider-state", cx, |window, cx| {
            // Read before `cx.new`: the closure must not borrow the app.
            let initial = AppSettings::global(cx).font_size as f32;
            let state = cx.new(|_| {
                SliderState::new()
                    .min(theme::FONT_SIZE_MIN as f32)
                    .max(theme::FONT_SIZE_MAX as f32)
                    .step(1.0)
                    .default_value(initial)
            });
            let subscription =
                cx.subscribe_in(&state, window, |_, _, event: &SliderEvent, _window, cx| {
                    let (value, persist) = match event {
                        SliderEvent::Change(value) => (value, false),
                        SliderEvent::Release(value) => (value, true),
                    };
                    let SliderValue::Single(size) = value else {
                        return;
                    };
                    // Same boundary guard the NumberField needed: a non-finite
                    // value would poison `px()`/`set_rem_size`.
                    let Some(size) = normalize_font_size(*size as f64) else {
                        return;
                    };
                    AppSettings::global_mut(cx).font_size = size;
                    apply_font_size(cx);
                    if persist {
                        save_settings(cx);
                    }
                    cx.refresh_windows();
                });
            FontSizeSlot {
                state,
                _subscription: subscription,
            }
        });
        let state = slot.read(cx).state.clone();
        // The global is the source of truth: adopt reset/external writes.
        let want = AppSettings::global(cx).font_size as f32;
        let SliderValue::Single(shown) = state.read(cx).value() else {
            unreachable!("the font size slider is single-valued")
        };
        if shown != want {
            state.update(cx, |state, cx| {
                state.set_value(want, window, cx);
            });
        }
        let size = AppSettings::global(cx).font_size;
        div()
            .id("font-size-control")
            .when(options.layout().is_horizontal(), |this| this.w_64())
            .child(
                v_flex()
                    .gap_2()
                    .debug_selector(|| "font-size-row".into())
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(
                                div().flex_1().min_w_0().child(
                                    // `accessibility_label` comes from the
                                    // vendored slider-accessibility-label
                                    // patch: without it the row announces
                                    // only "14, slider", with no name.
                                    Slider::new(&state)
                                        .horizontal()
                                        .accessibility_label(label.clone())
                                        .disabled(options.is_disabled()),
                                ),
                            )
                            .child(div().flex_none().child(format!("{size:.0} px")))
                            .child(
                                Button::new("font-size-reset")
                                    .ghost()
                                    .icon(IconName::RotateCw)
                                    .accessibility_label(label.clone())
                                    .disabled(font_size_at_default(size))
                                    .on_click(|_, _, cx| {
                                        AppSettings::global_mut(cx).font_size =
                                            AppSettings::default().font_size;
                                        apply_font_size(cx);
                                        save_settings(cx);
                                        cx.refresh_windows();
                                    }),
                            ),
                    )
                    .child(
                        div().flex().child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .text_size(px(size as f32))
                                .child(tr("preview.sample", &AppSettings::global(cx).language)),
                        ),
                    ),
            )
    })
    .on_reset(
        |cx: &App| !font_size_at_default(AppSettings::global(cx).font_size),
        |_window, cx: &mut App| {
            AppSettings::global_mut(cx).font_size = AppSettings::default().font_size;
            apply_font_size(cx);
            save_settings(cx);
            cx.refresh_windows();
        },
    )
}

/// The two glass percentage controls (Window opacity, Sidebar tone).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GlassControl {
    WindowOpacity,
    SidebarTone,
}

impl GlassControl {
    /// Stable suffix for element/test ids.
    fn key(self) -> &'static str {
        match self {
            GlassControl::WindowOpacity => "window-opacity",
            GlassControl::SidebarTone => "sidebar-tone",
        }
    }

    /// Slider range. The shell keeps the 40% readability floor; the sidebar
    /// tone legitimately goes to zero (hairline-only separation).
    fn range(self) -> (f32, f32) {
        match self {
            GlassControl::WindowOpacity => (
                theme::WINDOW_OPACITY_MIN as f32,
                theme::WINDOW_OPACITY_MAX as f32,
            ),
            GlassControl::SidebarTone => (
                theme::SIDEBAR_TONE_MIN as f32,
                theme::SIDEBAR_TONE_MAX as f32,
            ),
        }
    }

    /// The shipped default, used by page reset.
    fn default_value(self) -> f64 {
        match self {
            GlassControl::WindowOpacity => theme::WINDOW_OPACITY_DEFAULT,
            GlassControl::SidebarTone => theme::SIDEBAR_TONE_DEFAULT,
        }
    }

    /// Write one value, clamped.
    fn set(self, settings: &mut AppSettings, percent: f64) {
        match self {
            GlassControl::WindowOpacity => {
                settings.window_opacity = theme::clamp_window_opacity(percent);
            }
            GlassControl::SidebarTone => {
                settings.sidebar_tone = theme::clamp_sidebar_tone(percent);
            }
        }
    }
}

/// Keyed-state slot holding one glass `SliderState` and the subscription that
/// commits it into `AppSettings`.
struct GlassPercentSlot {
    state: Entity<SliderState>,
    _subscription: Subscription,
}

/// One glass percentage field. Inert while the Translucent-window master
/// switch is off, since nothing paints at a reduced alpha or tone then.
///
/// Like Font Size, the slider reports continuously while dragging, so the live
/// paint is updated on every `Change` but the file is written only on
/// `Release`.
fn glass_percent_field(control: GlassControl, label: SharedString) -> SettingField<SharedString> {
    SettingField::render(move |options, window, cx| {
        let state_id: ElementId = format!("glass-slider-state-{}", control.key()).into();
        let slot = window.use_keyed_state(state_id, cx, |window, cx| {
            // Read before `cx.new`: the closure must not borrow the app.
            let initial = AppSettings::global(cx).stored(control) as f32;
            let (min, max) = control.range();
            let state = cx.new(|_| {
                SliderState::new()
                    .min(min)
                    .max(max)
                    .step(1.0)
                    .default_value(initial)
            });
            let subscription = cx.subscribe_in(
                &state,
                window,
                move |_, _, event: &SliderEvent, window, cx| {
                    let (value, persist) = match event {
                        SliderEvent::Change(value) => (value, false),
                        SliderEvent::Release(value) => (value, true),
                    };
                    let SliderValue::Single(percent) = value else {
                        return;
                    };
                    if !percent.is_finite() {
                        return;
                    }
                    set_glass_percent(control, *percent as f64, persist, window, cx);
                },
            );
            GlassPercentSlot {
                state,
                _subscription: subscription,
            }
        });
        let state = slot.read(cx).state.clone();
        // The global is the source of truth: adopt reset/external writes.
        let want = AppSettings::global(cx).stored(control) as f32;
        let SliderValue::Single(shown) = state.read(cx).value() else {
            unreachable!("a glass percentage slider is single-valued")
        };
        if shown != want {
            state.update(cx, |state, cx| {
                state.set_value(want, window, cx);
            });
        }
        let percent = AppSettings::global(cx).stored(control);
        let enabled = glass_enabled(cx);
        let row = format!("{}-row", control.key());
        let value = format!("{}-value", control.key());
        div()
            .id(ElementId::Name(format!("{}-control", control.key()).into()))
            .when(options.layout().is_horizontal(), |this| this.w_64())
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .debug_selector(move || row.clone())
                    .child(
                        div().flex_1().min_w_0().child(
                            // Named through the vendored
                            // `slider-accessibility-label` patch; the
                            // value/min/max/orientation contract is the
                            // patched behaviour root's.
                            Slider::new(&state)
                                .horizontal()
                                .accessibility_label(label.clone())
                                .disabled(options.is_disabled() || !enabled),
                        ),
                    )
                    .child(
                        div()
                            .flex_none()
                            .child(percent_text(percent, &AppSettings::global(cx).language))
                            // The percentage duplicates the slider's a11y
                            // numeric value for sighted users.
                            .debug_selector(move || value.clone()),
                    ),
            )
    })
    .on_reset(
        move |cx: &App| AppSettings::global(cx).stored(control) != control.default_value(),
        move |window, cx: &mut App| {
            // Page reset runs every field handler, even with translucency
            // off; the stored default is the right target, not the effective
            // (opaque) value the switch implies.
            set_glass_percent(control, control.default_value(), true, window, cx);
        },
    )
}

/// One setting whose current value differs from its default.
///
/// `key` is an i18n key; the two values are already-formatted words, because
/// the panel only has to show them, not compare them.
struct ChangedSetting {
    key: &'static str,
    current: String,
    default: String,
}

/// Every setting that differs from `AppSettings::default()`.
///
/// Computed app-side: the app owns both sides of the comparison, so this
/// needs no vendor access and cannot drift from what the pages actually show.
/// A setting is included only when its *effective* value differs — a
/// hand-edited config that already equals the default is not "changed".
fn changed_settings(cx: &App) -> Vec<ChangedSetting> {
    let now = AppSettings::global(cx);
    let default = AppSettings::default();
    let mut changed = Vec::new();
    let mut push = |key: &'static str, differs: bool, current: String, default: String| {
        if differs {
            changed.push(ChangedSetting {
                key,
                current,
                default,
            });
        }
    };

    push(
        "item.appearance",
        now.appearance != default.appearance,
        now.appearance.as_str().to_string(),
        default.appearance.as_str().to_string(),
    );
    push(
        "item.accent",
        now.accent != default.accent,
        now.accent
            .map(|accent| accent.to_hex())
            .unwrap_or_else(|| tr("reset.theme_default", &now.language).to_string()),
        tr("reset.theme_default", &default.language).to_string(),
    );
    push(
        "item.font_size",
        now.font_size != default.font_size,
        format!("{:.0} px", now.font_size),
        format!("{:.0} px", default.font_size),
    );
    push(
        "item.background_translucent",
        now.background_translucent != default.background_translucent,
        on_off(now.background_translucent, &now.language),
        on_off(default.background_translucent, &default.language),
    );
    push(
        "item.window_opacity",
        now.window_opacity != default.window_opacity,
        percent_text(now.window_opacity, &now.language),
        percent_text(default.window_opacity, &default.language),
    );
    push(
        "item.sidebar_tone",
        now.sidebar_tone != default.sidebar_tone,
        percent_text(now.sidebar_tone, &now.language),
        percent_text(default.sidebar_tone, &default.language),
    );
    push(
        "item.background_blur",
        now.background_blur != default.background_blur,
        on_off(now.background_blur, &now.language),
        on_off(default.background_blur, &default.language),
    );
    push(
        "item.language",
        now.language != default.language,
        now.language.to_string(),
        default.language.to_string(),
    );
    push(
        "item.launch_at_login",
        now.launch_at_login != default.launch_at_login,
        on_off(now.launch_at_login, &now.language),
        on_off(default.launch_at_login, &default.language),
    );
    push(
        "item.notifications",
        now.notifications_enabled != default.notifications_enabled,
        on_off(now.notifications_enabled, &now.language),
        on_off(default.notifications_enabled, &default.language),
    );
    push(
        "item.username",
        now.username != default.username,
        now.username.to_string(),
        default.username.to_string(),
    );
    push(
        "item.display_name",
        now.display_name != default.display_name,
        now.display_name.to_string(),
        default.display_name.to_string(),
    );
    // The channels are one setting split over three switches, so they are
    // one row: listing them separately would let "reset" mean something
    // different from what the Notifications page shows.
    let channels_changed = now.channel_email != default.channel_email
        || now.channel_desktop != default.channel_desktop
        || now.channel_digest != default.channel_digest;
    let channel_summary = |email: bool, desktop: bool, digest: bool, lang: &SharedString| {
        let mark = |on: bool| tr(if on { "state.on" } else { "state.off" }, lang);
        format!(
            "{}: {} · {}: {} · {}: {}",
            tr("item.channel_email", lang),
            mark(email),
            tr("item.channel_desktop", lang),
            mark(desktop),
            tr("item.channel_digest", lang),
            mark(digest),
        )
    };
    push(
        "item.channels",
        channels_changed,
        channel_summary(
            now.channel_email,
            now.channel_desktop,
            now.channel_digest,
            &now.language,
        ),
        channel_summary(
            default.channel_email,
            default.channel_desktop,
            default.channel_digest,
            &default.language,
        ),
    );
    push(
        "item.digest",
        now.digest != default.digest,
        tr(now.digest.label_key(), &now.language).to_string(),
        tr(default.digest.label_key(), &default.language).to_string(),
    );
    push(
        "item.timezone",
        now.timezone != default.timezone,
        now.timezone.to_string(),
        default.timezone.to_string(),
    );
    changed
}

/// `On`/`Off` in the owner's language.
fn on_off(value: bool, lang: &SharedString) -> String {
    tr(if value { "state.on" } else { "state.off" }, lang).to_string()
}

/// `NN%` in the owner's language — the percent sign/space is a translatable
/// unit, not a hardcoded ASCII glyph.
fn percent_text(value: f64, lang: &SharedString) -> String {
    format!("{value:.0}{}", tr("unit.percent", lang))
}

/// Revert everything the reset panel lists, in one pass.
fn reset_everything(window: &mut Window, cx: &mut App) {
    let default = AppSettings::default();
    let language = default.language.clone();
    *AppSettings::global_mut(cx) = default;
    gpui_kit::component::set_locale(component_locale(&language));
    theme::install(
        cx,
        AppSettings::global(cx).font_size,
        AppSettings::global(cx).accent,
    );
    apply_appearance(AppSettings::global(cx).appearance, cx);
    // The reset returns the glass setting to "off", which also has to put the
    // window surface and Root's base paint back (the theme install above does
    // not know about window-level state).
    apply_glass(window, cx);
    persist::autostart_converge(AppSettings::global(cx).launch_at_login);
    save_settings(cx);
    cx.refresh_windows();
}

/// The Reset page: every setting that differs from its default, a way back
/// for each, and a "Reset all" that asks twice. With nothing changed it says
/// so rather than drawing an empty box.
fn reset_page(cx: &App) -> SettingPage {
    let lang = AppSettings::global(cx).language.clone();
    let changed = changed_settings(cx);
    let muted = cx.theme().muted_foreground;
    let has_changes = !changed.is_empty();
    let mut group = SettingGroup::new().title(tr("group.changed", &lang));

    if changed.is_empty() {
        group = group.item(
            SettingItem::new(
                tr("reset.nothing", &lang),
                SettingField::render(|_, _, _| div()),
            )
            .description(tr("reset.nothing.desc", &lang)),
        );
    } else {
        for each in changed {
            let key = each.key;
            let words = format!("{} → {}", each.current, each.default);
            group = group.item(
                SettingItem::new(
                    tr(key, &lang),
                    SettingField::render(move |_, _, _| {
                        div().text_color(muted).child(words.clone())
                    }),
                )
                .description(tr(key, &lang)),
            );
        }
    }

    // "Reset all" asks twice: the first press arms it, the second reverts.
    // The armed flag lives in keyed state so it survives re-renders.
    let label: SharedString = tr("reset.all", &lang).into();
    let armed_label: SharedString = tr("reset.all.confirm", &lang).into();
    group = group.item(
        SettingItem::new(
            label.clone(),
            SettingField::render(move |_, window, cx| {
                let armed = window.use_keyed_state("reset-all-armed", cx, |_, _| ResetArmed(false));
                let is_armed = armed.read(cx).0;
                let text = if is_armed {
                    armed_label.clone()
                } else {
                    label.clone()
                };
                div().child(
                    Button::new("reset-all")
                        .label(text)
                        .disabled(!has_changes)
                        .on_click(move |_, window, cx| {
                            if is_armed {
                                reset_everything(window, cx);
                            } else {
                                armed.update(cx, |armed, cx| {
                                    armed.0 = true;
                                    cx.notify();
                                });
                            }
                        }),
                )
            }),
        )
        .description(tr("reset.all.desc", &lang)),
    );

    SettingPage::new(tr("page.reset", &lang))
        .icon(Icon::new(IconName::RotateCw))
        .resettable(false)
        .groups(vec![group])
}

/// Keyed state for the two-press "Reset all".
struct ResetArmed(bool);

impl Global for ResetArmed {}

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
            // TODO(a11y): the Timezone `Combobox` is announced without a name.
            // `Combobox` exposes no `accessibility_label` at any layer (unlike
            // `Select`, which does), and a plain `div` wrapper with
            // `aria_label` is filtered out of the a11y tree, so a name here
            // would announce nothing while looking like it worked. Naming it
            // needs either a vendored patch following the
            // `select.rs:642` pattern or a switch to `Select`, which is a
            // product change (Combobox is searchable here).
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

/// The single funnel for every theme-mode write.
///
/// `Theme::change` rebuilds the theme from the config, which resets
/// `font_size` **and** the accent to their config values, so
/// `apply_font_size` (which applies both) must follow it on every path —
/// startup, the segmented control, and the OS observer alike. Routing them
/// all here is what keeps a mode flip from silently dropping a non-default
/// font size (see `docs/memory/gpui-component-setting-field-pitfalls.md`
/// item 3).
///
/// Deliberately does not persist: a caller that represents a user choice adds
/// `save_settings`, and startup must not rewrite the file it just read.
fn apply_appearance(appearance: Appearance, cx: &mut App) {
    let mode = appearance.resolve(cx.window_appearance());
    Theme::change(mode, None, cx);
    apply_font_size(cx);
    cx.refresh_windows();
}

/// The Appearance control's effect: remember the choice, then funnel it.
/// Shared by the row's handler and the page reset so the two cannot drift.
fn set_appearance(appearance: Appearance, cx: &mut App) {
    AppSettings::global_mut(cx).appearance = appearance;
    apply_appearance(appearance, cx);
    save_settings(cx);
}

/// The Accent control's effect: remember the color and re-apply the theme
/// overrides. No `Theme::change` is needed — the accent is written on top of
/// the resolved palette, so it survives mode switches through
/// `apply_overrides`.
fn set_accent(accent: Option<Hsla>, cx: &mut App) {
    AppSettings::global_mut(cx).accent = accent;
    apply_font_size(cx);
    save_settings(cx);
    cx.refresh_windows();
}

/// One clickable accent swatch. `selected` draws the focus ring color as its
/// border, so the current choice is visible without a checkmark.
fn accent_swatch(name: &SharedString, color: Hsla, selected: bool, cx: &App) -> impl IntoElement {
    let theme = cx.theme();
    let border = if selected {
        theme.colors.ring
    } else {
        theme.colors.border
    };
    let debug: SharedString = format!("accent-{name}").into();
    div()
        .id(ElementId::Name(debug.clone()))
        .debug_selector(move || debug.to_string())
        .size(px(20.))
        .flex_none()
        .rounded_full()
        .bg(color)
        .border_1()
        .border_color(border)
        .cursor_pointer()
        // A swatch is colour-only, so the name is the only thing a screen
        // reader can announce. `role` is required, not decorative: a `div`
        // with no role is filtered out of the accessibility tree, so the
        // label alone would announce nothing.
        .role(gpui_kit::Role::Button)
        .aria_label(name.clone())
        // Selection is a *description*, not part of the name: "violet,
        // selected" made the state read as the swatch's name.
        //
        // Checked against the running app. `aria_selected` does not survive
        // the trip — gpui writes `AccessKit::Node::selected`, but macOS
        // exposes no `AXSelected` on a plain AXButton, and `Role::RadioButton`
        // (which would) drops the swatch out of the tree entirely.
        // `aria_description` does survive, landing in `AXHelp`, so a screen
        // reader still announces the selected swatch.
        .when(selected, |this| this.aria_description("Selected"))
        .on_click(move |_, _, cx| set_accent(Some(color), cx))
}

/// The Launch at login checkbox's effect — shared by the row's click handler
/// and the page reset.
fn set_launch_at_login(val: bool, cx: &mut App) {
    AppSettings::global_mut(cx).launch_at_login = val;
    persist::autostart_converge(val);
    save_settings(cx);
    // The row mirrors the global on render; refresh so the change shows up
    // without waiting for the next unrelated re-render.
    cx.refresh_windows();
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

/// The icon a segment shows. `Sun`/`Moon` carry Light/Dark; the catalog has
/// no monitor glyph, so System borrows `Cpu` — "this machine decides".
fn appearance_icon(appearance: Appearance) -> IconName {
    match appearance {
        Appearance::Light => IconName::Sun,
        Appearance::Dark => IconName::Moon,
        Appearance::System => IconName::Cpu,
    }
}

/// The Appearance row's control: Light / Dark / System as one segmented
/// control, the chosen segment drawn as the raised pill.
///
/// Composed from vendored `Button`s rather than a new vendor control —
/// `Button::toggled` is exactly the segment state, and the phase's surfaces
/// allow no new vendor patch.
fn appearance_segmented(
    selected: Appearance,
    labels: [(Appearance, SharedString); 3],
    options: &RenderOptions,
    on_pick: impl Fn(Appearance, &mut Window, &mut App) + Clone + 'static,
) -> impl IntoElement {
    h_flex()
        .gap_1()
        .debug_selector(|| "appearance-segmented".into())
        .children(labels.into_iter().map(move |(appearance, label)| {
            let on_pick = on_pick.clone();
            let debug: SharedString = format!("appearance-{}", appearance.as_str()).into();
            Button::new(ElementId::Name(debug.clone()))
                .debug_selector(move || debug.to_string())
                .label(label.clone())
                .icon(appearance_icon(appearance))
                .accessibility_label(label)
                .toggled(appearance == selected)
                .disabled(options.is_disabled())
                .with_size(options.size())
                .on_click(move |_, window, cx| on_pick(appearance, window, cx))
        }))
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
    /// Rail-resize drag state lives on the view, not in a global.
    resizing: bool,
    /// First width limit hit in the current drag (latched per drag).
    latched: Option<SidebarEdge>,
    /// Selected settings page index (single source for nav + Settings).
    current_page: usize,
    /// Collapsible section open flags, in nav order.
    sections_open: [bool; 2],
    /// Header filter cycle state.
    nav_filter: NavFilter,
}

/// Sidebar nav filter: all pages, or one section's group.
#[derive(Clone, Copy, PartialEq, Eq)]
enum NavFilter {
    All,
    Preferences,
    Workspace,
}

impl NavFilter {
    fn next(self) -> Self {
        match self {
            NavFilter::All => NavFilter::Preferences,
            NavFilter::Preferences => NavFilter::Workspace,
            NavFilter::Workspace => NavFilter::All,
        }
    }

    fn label_key(self) -> &'static str {
        match self {
            NavFilter::All => "nav.all_settings",
            NavFilter::Preferences => "nav.preferences",
            NavFilter::Workspace => "nav.workspace",
        }
    }
}

/// The two zeron-style sidebar sections and the pages they hold.
#[derive(Clone, Copy, PartialEq, Eq)]
enum NavGroup {
    Preferences,
    Workspace,
}

impl NavGroup {
    fn index(self) -> usize {
        match self {
            NavGroup::Preferences => 0,
            NavGroup::Workspace => 1,
        }
    }

    fn label_key(self) -> &'static str {
        match self {
            NavGroup::Preferences => "nav.preferences",
            NavGroup::Workspace => "nav.workspace",
        }
    }
}

/// One custom-nav page row. `page_ix` indexes `Settings::pages`.
struct NavEntry {
    page_ix: usize,
    title_key: &'static str,
    icon: IconName,
    group: NavGroup,
}

const NAV_ENTRIES: [NavEntry; 6] = [
    NavEntry {
        page_ix: 0,
        title_key: "page.general",
        icon: IconName::Settings2,
        group: NavGroup::Preferences,
    },
    NavEntry {
        page_ix: 1,
        title_key: "page.notifications",
        icon: IconName::Bell,
        group: NavGroup::Preferences,
    },
    NavEntry {
        page_ix: 2,
        title_key: "page.profile",
        icon: IconName::User,
        group: NavGroup::Preferences,
    },
    NavEntry {
        page_ix: 3,
        title_key: "page.finance",
        icon: IconName::LayoutDashboard,
        group: NavGroup::Workspace,
    },
    NavEntry {
        page_ix: 4,
        title_key: "page.reset",
        icon: IconName::RotateCw,
        group: NavGroup::Workspace,
    },
    NavEntry {
        page_ix: 5,
        title_key: "page.about",
        icon: IconName::Info,
        group: NavGroup::Workspace,
    },
];

/// Width limit a drag pressed past; latched once per drag so a held pointer
/// does not re-trigger the limit on every move.
#[derive(Clone, Copy, PartialEq, Eq)]
enum SidebarEdge {
    Min,
    Max,
}

impl SettingsWindow {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        cx.set_global(ShellState::default());
        cx.set_global(cadence::DemoState::demo());
        // `System` follows the OS while the app runs. The observer lives on
        // the window rather than in the row, so the theme still flips when
        // the Appearance row is off screen or the page is filtered away.
        window
            .observe_window_appearance(|window, cx| {
                // An explicit Light or Dark ignores the OS entirely.
                if AppSettings::global(cx).appearance != Appearance::System {
                    return;
                }
                let mode = Appearance::System.resolve(window.appearance());
                if cx.theme().mode != mode {
                    set_appearance(Appearance::System, cx);
                }
            })
            .detach();
        Self {
            focus_handle: cx.focus_handle(),
            resizing: false,
            latched: None,
            current_page: 0,
            sections_open: [true, true],
            nav_filter: NavFilter::All,
        }
    }

    fn setting_pages(cx: &App) -> Vec<SettingPage> {
        let lang = AppSettings::global(cx).language.clone();
        let defaults = AppSettings::default();
        // Copied out of `defaults` so the reset closure of the element field
        // below can be `'static`.
        let notifications_default = defaults.notifications_enabled;
        // Same for the glass group's three fields: `AppSettings` itself is
        // not `Copy` (it owns `SharedString`s), the scalar defaults are.
        let glass_defaults = (
            defaults.background_translucent,
            defaults.window_opacity,
            defaults.sidebar_tone,
            defaults.background_blur,
        );

        vec![
            SettingPage::new(tr("page.general", &lang))
                .icon(Icon::new(IconName::Settings2))
                .resettable(true)
                .groups(vec![
                    SettingGroup::new()
                        .title(tr("group.appearance", &lang))
                        .items(vec![
                            SettingItem::new(
                                tr("item.appearance", &lang),
                                SettingField::render({
                                    let labels: [(Appearance, SharedString); 3] = [
                                        (Appearance::Light, tr("appearance.light", &lang).into()),
                                        (Appearance::Dark, tr("appearance.dark", &lang).into()),
                                        (Appearance::System, tr("appearance.system", &lang).into()),
                                    ];
                                    move |options, _window, cx| {
                                        let selected = AppSettings::global(cx).appearance;
                                        div().debug_selector(|| "appearance-row".into()).child(
                                            appearance_segmented(
                                                selected,
                                                labels.clone(),
                                                options,
                                                |each, _window, cx| set_appearance(each, cx),
                                            ),
                                        )
                                    }
                                })
                                // An element field carries no typed value, so
                                // the page reset needs the handler explicitly.
                                .on_reset(
                                    move |cx: &App| {
                                        AppSettings::global(cx).appearance != defaults.appearance
                                    },
                                    move |_window, cx: &mut App| {
                                        set_appearance(defaults.appearance, cx);
                                    },
                                ),
                            )
                            .description(tr("item.appearance.desc", &lang))
                            .keywords(["theme", "mode", "dark", "light", "system"]),
                            SettingItem::new(
                                tr("item.accent", &lang),
                                accent_field(tr("item.accent", &lang).into()),
                            )
                            .description(tr("item.accent.desc", &lang))
                            .keywords([
                                "color",
                                "colour",
                                "brand",
                                "primary",
                                "highlight",
                            ]),
                            SettingItem::new(
                                tr("item.font_size", &lang),
                                font_size_field(tr("item.font_size", &lang).into()),
                            )
                            .description(tr("item.font_size.desc", &lang))
                            .keywords(["text", "size", "zoom", "scale"]),
                        ]),
                    SettingGroup::new()
                        .title(tr("group.background", &lang))
                        .items(vec![
                            SettingItem::new(
                                tr("item.background_translucent", &lang),
                                SettingField::render({
                                    let label = tr("item.background_translucent", &lang);
                                    move |options, _window, cx| {
                                        div()
                                            .debug_selector(|| "background-translucent-row".into())
                                            .child(settings_switch(
                                                "background-translucent-switch",
                                                AppSettings::global(cx).background_translucent,
                                                label,
                                                options,
                                                |checked: &bool, window, cx| {
                                                    set_glass_translucent(*checked, window, cx);
                                                },
                                            ))
                                    }
                                })
                                // An element field carries no typed value, so
                                // the page reset needs the handler explicitly.
                                .on_reset(
                                    move |cx: &App| {
                                        AppSettings::global(cx).background_translucent
                                            != glass_defaults.0
                                    },
                                    move |window, cx: &mut App| {
                                        set_glass_translucent(glass_defaults.0, window, cx);
                                    },
                                ),
                            )
                            .description(tr("item.background_translucent.desc", &lang))
                            .keywords([
                                "glass",
                                "transparent",
                                "translucent",
                                "desktop",
                                "alpha",
                            ]),
                            SettingItem::new(
                                tr("item.window_opacity", &lang),
                                glass_percent_field(
                                    GlassControl::WindowOpacity,
                                    tr("item.window_opacity", &lang).into(),
                                ),
                            )
                            .description(tr("item.window_opacity.desc", &lang))
                            .keywords(["opacity", "alpha", "frost", "glass", "window"]),
                            SettingItem::new(
                                tr("item.sidebar_tone", &lang),
                                glass_percent_field(
                                    GlassControl::SidebarTone,
                                    tr("item.sidebar_tone", &lang).into(),
                                ),
                            )
                            .description(tr("item.sidebar_tone.desc", &lang))
                            .keywords(["tone", "wash", "sidebar", "rail", "glass"]),
                            SettingItem::new(
                                tr("item.background_blur", &lang),
                                SettingField::render({
                                    let label = tr("item.background_blur", &lang);
                                    move |options, _window, cx| {
                                        let translucent =
                                            AppSettings::global(cx).background_translucent;
                                        div().debug_selector(|| "background-blur-row".into()).child(
                                            settings_switch(
                                                "background-blur-switch",
                                                AppSettings::global(cx).background_blur,
                                                label,
                                                options,
                                                |blur: &bool, window, cx| {
                                                    set_glass_blur(*blur, window, cx);
                                                },
                                            )
                                            // Blur is only meaningful on a
                                            // translucent window, so the
                                            // switch is inert while the
                                            // master switch is off.
                                            .disabled(options.is_disabled() || !translucent),
                                        )
                                    }
                                })
                                .on_reset(
                                    move |cx: &App| {
                                        AppSettings::global(cx).background_blur != glass_defaults.3
                                    },
                                    move |window, cx: &mut App| {
                                        set_glass_blur(glass_defaults.3, window, cx);
                                    },
                                ),
                            )
                            .description(tr("item.background_blur.desc", &lang))
                            .keywords(["blur", "frosted", "vibrancy", "backdrop", "glass"]),
                        ]),
                    SettingGroup::new()
                        .title(tr("group.region", &lang))
                        .items(vec![
                            SettingItem::new(
                                tr("item.language", &lang),
                                language_field(SharedString::from(tr("item.language", &lang))),
                            )
                            .description(tr("item.language.desc", &lang))
                            .keywords([
                                "locale",
                                "translation",
                                "i18n",
                            ]),
                            SettingItem::new(
                                tr("item.timezone", &lang),
                                timezone_field(
                                    tr("tz.placeholder", &lang),
                                    tr("tz.search_placeholder", &lang),
                                ),
                            )
                            .description(tr("item.timezone.desc", &lang))
                            .keywords(["tz", "clock", "time", "region"]),
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
                            .description(tr("item.launch_at_login.desc", &lang))
                            .keywords([
                                "startup",
                                "boot",
                                "autostart",
                                "login",
                            ]),
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
                                        .child(settings_switch(
                                            "notifications-enabled",
                                            enabled,
                                            tr("item.notifications", &settings.language),
                                            options,
                                            |checked: &bool, _, cx: &mut App| {
                                                AppSettings::global_mut(cx).notifications_enabled =
                                                    *checked;
                                                save_settings(cx);
                                                cx.refresh_windows();
                                            },
                                        ))
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
                                        // The field mirrors the global on
                                        // render; refresh so the reset shows
                                        // up without waiting for the next
                                        // unrelated re-render.
                                        cx.refresh_windows();
                                    },
                                ),
                            )
                            .description(tr("item.notifications.desc", &lang))
                            .keywords(["alerts", "notify", "push"]),
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
                        .keywords(["delivery", "routes", "email", "desktop"])
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
                        .keywords(["summary", "frequency", "cadence"])
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
                        .description(tr("item.test_notification.desc", &lang))
                        .keywords(["preview", "sample", "try"]),
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
                            .description(tr("item.username.desc", &lang))
                            .keywords(["login", "handle", "account", "name"]),
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
                            .description(tr("item.display_name.desc", &lang))
                            .keywords(["nickname", "name", "profile"]),
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
                            .description(tr("item.preview.desc", &lang))
                            .keywords(["sample", "example", "avatar", "profile"]),
                        ]),
                ),
            cadence::showcase::page(&lang),
            reset_page(cx),
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
                            .description(tr("item.website.desc", &lang))
                            .keywords(["link", "url", "home", "docs"]),
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
                            .description(tr("item.about.desc", &lang))
                            .keywords(["version", "build", "info", "credits"]),
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
    let settings = AppSettings::global(cx);
    theme::apply_overrides(settings.font_size, settings.accent, cx);
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
/// Collapsed rail: zeron leaves only a hairline grab sliver, not an icon rail.
const SIDEBAR_COLLAPSED_W: Pixels = px(1.);

/// The spring target for the rail: the sliver when collapsed, else the
/// persisted width.
fn sidebar_target_width(collapsed: bool, stored_width: Pixels) -> Pixels {
    if collapsed {
        SIDEBAR_COLLAPSED_W
    } else {
        stored_width
    }
}

/// The app-owned chrome refinements, extracted from the render so the zeron
/// layout invariants are inspectable by tests without a rendered frame.
///
/// Both appearances share one structure: the shell paints the base (opaque
/// token or the glass frost), the rail is a transparent column carrying only
/// an ink tone plus a hairline, and the page is flush (no inset card).
pub(crate) fn sidebar_refinement(cx: &App, _glass: bool) -> StyleRefinement {
    // zeron's rail column is TRANSPARENT and carries only the ink tone plus a
    // 1px right hairline, in both appearances (zeron paints `wash(0.05)` on
    // its opaque shell too — it is a tone, not a glass-only effect). The
    // column still spans the full window height, while its content starts
    // below the overlay titlebar.
    // NOTE: the vendored `Sidebar` zeroes padding on its own root before
    // rendering (vendor/gpui-component/src/sidebar/mod.rs), so the 38px
    // chrome offset must live on the app-owned header element — see
    // `sidebar_header_padding()` — not on this refinement.
    StyleRefinement::default()
        .w_full()
        .border_0()
        .text_color(cx.theme().muted_foreground)
        .bg(sidebar_tone_bg(cx))
        .border_r_1()
        .border_color(cx.theme().border)
}

/// The content panel's style: zeron's flush page — no inset card in either
/// appearance, padded below the overlay titlebar.
pub(crate) fn content_panel_style(_cx: &App, _glass: bool) -> StyleRefinement {
    // zeron's layout has no inset cards: the page sits flush on the shell in
    // both appearances, padded below the overlay titlebar. The appearance
    // only changes the shell base (frost vs opaque), never the page chrome.
    StyleRefinement::default().pt(crate::shell::TITLEBAR_H)
}

/// Chrome offset the rail's own content needs below the overlay titlebar.
///
/// The vendored `Sidebar` discards padding from its style refinement, so the
/// app pads its header element explicitly instead; the menu rows follow the
/// header down because the panel lays them out in one column.
pub(crate) fn sidebar_header_padding() -> Pixels {
    crate::shell::TITLEBAR_H
}

/// Critically damped (ω₀ ≈ 32 rad/s), no overshoot: settles inside ~200 ms,
/// and a retarget keeps velocity instead of jumping.
const SIDEBAR_SPRING: SpringConfig = SpringConfig::new(1024.0, 64.0, 1.0);

/// Settling tolerance in pixels — sub-pixel, so the last visible step still
/// lands on the exact target.
const SIDEBAR_SPRING_EPSILON: f32 = 0.5;

impl SettingsWindow {
    /// The zeron header row: a filter "dropdown" (V1 cycles all → preferences
    /// → workspace) styled as the rail's title row.
    fn render_sidebar_filter(&self, cx: &mut Context<Self>) -> AnyElement {
        let lang = AppSettings::global(cx).language.clone();
        let label = tr(self.nav_filter.label_key(), &lang);
        Button::new("nav-filter")
            .ghost()
            .icon(IconName::Folder)
            .label(label)
            .tooltip(tr("nav.all_settings", &lang))
            .accessibility_label(tr("nav.all_settings", &lang))
            .w_full()
            .h(px(32.))
            .justify_start()
            .on_click(cx.listener(|this, _, _, cx| {
                this.nav_filter = this.nav_filter.next();
                cx.notify();
            }))
            .into_any_element()
    }

    /// The zeron nav tree: two collapsible sections holding the page rows.
    fn render_sidebar_nav(&self, cx: &mut Context<Self>) -> AnyElement {
        let lang = AppSettings::global(cx).language.clone();
        let current = self.current_page;
        let filter = self.nav_filter;
        let mut children: Vec<AnyElement> = Vec::new();
        for group in [NavGroup::Preferences, NavGroup::Workspace] {
            let group_visible = match filter {
                NavFilter::All => true,
                NavFilter::Preferences => group == NavGroup::Preferences,
                NavFilter::Workspace => group == NavGroup::Workspace,
            };
            if !group_visible {
                continue;
            }
            let open = self.sections_open[group.index()];
            let group_id = match group {
                NavGroup::Preferences => "nav-section-preferences",
                NavGroup::Workspace => "nav-section-workspace",
            };
            children.push(
                Button::new(group_id)
                    .ghost()
                    .icon(if open {
                        IconName::ChevronDown
                    } else {
                        IconName::ChevronRight
                    })
                    .label(tr(group.label_key(), &lang))
                    .w_full()
                    .h(px(28.))
                    .justify_start()
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.sections_open[group.index()] = !this.sections_open[group.index()];
                        cx.notify();
                    }))
                    .into_any_element(),
            );
            if !open {
                continue;
            }
            for entry in NAV_ENTRIES.iter().filter(|entry| entry.group == group) {
                let active = entry.page_ix == current;
                let page_ix = entry.page_ix;
                let row_id = format!("nav-page-{page_ix}");
                children.push(
                    Button::new(row_id)
                        .ghost()
                        .icon(entry.icon.clone())
                        .label(tr(entry.title_key, &lang))
                        .selected(active)
                        .w_full()
                        .h(px(30.))
                        .justify_start()
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.current_page = page_ix;
                            cx.notify();
                        }))
                        .into_any_element(),
                );
            }
        }
        v_flex()
            .w_full()
            .gap_1()
            .children(children)
            .into_any_element()
    }

    /// zeron's footer identity row: avatar initial + display name + a theme
    /// toggle icon button (the settings app has no account menu to open).
    fn render_sidebar_footer_identity(&self, cx: &mut Context<Self>) -> AnyElement {
        let settings = AppSettings::global(cx);
        let name = if settings.display_name.is_empty() {
            settings.username.clone()
        } else {
            settings.display_name.clone()
        };
        let initial = name
            .chars()
            .next()
            .map(|c| c.to_uppercase().to_string())
            .unwrap_or_else(|| "?".into());
        let lang = settings.language.clone();
        let icon = if cx.theme().mode.is_dark() {
            IconName::Moon
        } else {
            IconName::Sun
        };
        let tooltip = tr("appearance.cycle", &lang);
        h_flex()
            .id("nav-footer-identity")
            .debug_selector(|| "nav-footer-identity".into())
            .w_full()
            .h(px(40.))
            .px(px(8.))
            .gap_2()
            .items_center()
            .child(
                div()
                    .size(px(24.))
                    .rounded_full()
                    .bg(cx.theme().accent)
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(initial),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .text_sm()
                    .text_color(cx.theme().foreground)
                    .child(name),
            )
            .child(
                Button::new("nav-footer-theme")
                    .ghost()
                    .icon(icon)
                    .tooltip(tooltip)
                    .accessibility_label(tooltip)
                    .on_click(|_, _, cx| {
                        let next = match AppSettings::global(cx).appearance {
                            Appearance::Light => Appearance::Dark,
                            Appearance::Dark => Appearance::Light,
                            Appearance::System => Appearance::Light,
                        };
                        set_appearance(next, cx);
                    }),
            )
            .into_any_element()
    }

    /// End a rail drag: clear the latch, persist the width once, repaint.
    fn finish_sidebar_resize(&mut self, cx: &mut Context<Self>) {
        if !self.resizing {
            return;
        }
        self.resizing = false;
        self.latched = None;
        save_settings(cx);
        cx.notify();
    }
}

impl Render for SettingsWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let collapsed = cx.global::<ShellState>().collapsed;
        let target_w = sidebar_target_width(collapsed, AppSettings::global(cx).sidebar_width());
        // zeron-style glass: the rail is not a panel. It stays a transparent
        // column on the frost shell, carrying only an ink tone and a 1px right
        // hairline — no rounding, border box or inset margin. The content pane
        // is flush beside it, so the two read as one continuous glass window.
        let glass = glass_enabled(cx);
        let sidebar_style = sidebar_refinement(cx, glass);
        // zeron's collapsed rail is a bare 1px grab sliver, so its content
        // leaves the tree with it: nothing paints outside the sliver, no
        // invisible control stays in the tab order, and the vendored
        // `Sidebar` is never handed its icon-collapse mode — that mode pins
        // its own width to a 48px icon rail and overflows the 1px panel
        // across the page (the "nồi thập cẩm" collapse state).
        let expanded = !collapsed;
        let sidebar_filter = expanded.then(|| self.render_sidebar_filter(cx));
        let sidebar_nav = if expanded {
            self.render_sidebar_nav(cx)
        } else {
            div().into_any_element()
        };
        let sidebar_footer_identity = expanded.then(|| self.render_sidebar_footer_identity(cx));
        let lang = AppSettings::global(cx).language.clone();
        let page_title: SharedString = tr(NAV_ENTRIES[self.current_page].title_key, &lang).into();

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
            .sidebar_style(&sidebar_style)
            .sidebar_header(
                div()
                    .w_full()
                    .pt(crate::sidebar_header_padding())
                    .children(sidebar_filter),
            )
            .sidebar_nav(sidebar_nav)
            .sidebar_footer(
                div()
                    .w_full()
                    .px(px(8.))
                    .pb(px(8.))
                    .children(sidebar_footer_identity),
            )
            .selected_page(self.current_page)
            // Raised pill on the active sidebar item: the app-owned `sm`
            // elevation from the resolved token snapshot (O1).
            .active_item_style(&{
                // Under glass the rail is a tone, so the raised pill is a
                // translucent chip over it (70% of the token colour, one
                // layer above the tone); opaque mode keeps the flat token.
                let pill = if glass {
                    theme::glass_surface(cx.theme().group_box, 70.0)
                } else {
                    cx.theme().group_box
                };
                StyleRefinement::default()
                    .bg(pill)
                    .border_1()
                    .border_color(cx.theme().border)
                    .shadow(cx.global::<UiTokens>().shadow.sm.clone())
            })
            // zeron's flush page (no inset card in either appearance).
            .content_panel_style(&content_panel_style(cx, glass))
            .pages(Self::setting_pages(cx))
            // One spring drives the sidebar panel's width; the element id
            // preserves spring state across frames, so a mid-flight toggle
            // retargets with its current velocity instead of restarting.
            .with_spring(
                "sidebar-width",
                SpringAnimation::new(SIDEBAR_SPRING)
                    .to(target_w)
                    .with_epsilon(SIDEBAR_SPRING_EPSILON),
                // Never `.sidebar_collapsed(...)`: the vendored icon rail
                // would pin a 48px column inside the shrinking panel and
                // paint it over the page. The animated width is the only
                // collapse driver.
                move |this, w| this.sidebar_width(w).sidebar_size_range(w..w),
            );
        // No base paint here on purpose: `Root` owns the opaque palette
        // background and `AppShell` owns the single window base (opaque
        // palette, or clear while glass is on so the pane alpha is the only
        // layer). A second full-window fill would stack alpha and go opaque.
        // The rail seam is a zero-layout-width hit target straddling the
        // rail/content boundary, drawn above the page but below the dialogs.
        // While dragging, a full-window capture overlay owns every mouse move
        // (a 20px seam loses the pointer on a fast drag otherwise), shows the
        // column-resize cursor and occludes hover from the page.
        let collapsed = cx.global::<ShellState>().collapsed;
        let resizing = self.resizing;
        let rail_now = AppSettings::global(cx).sidebar_width();
        let seam = (!collapsed && !resizing).then(|| {
            div()
                .id("sidebar-resize-seam")
                .debug_selector(|| "sidebar-resize-seam".into())
                .absolute()
                .top(crate::shell::TITLEBAR_H)
                .bottom_0()
                .left(rail_now - px(10.))
                .w(px(20.))
                .cursor_col_resize()
                .occlude()
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, _, _, cx| {
                        this.resizing = true;
                        this.latched = None;
                        cx.notify();
                    }),
                )
        });
        let capture = resizing.then(|| {
            div()
                .id("sidebar-resize-capture")
                .debug_selector(|| "sidebar-resize-capture".into())
                .absolute()
                .top_0()
                .left_0()
                .right_0()
                .bottom_0()
                .cursor_col_resize()
                .occlude()
                .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _, cx| {
                    // A lost release (pointer released outside the window)
                    // arrives as a move with no button held: end the drag
                    // instead of leaving the overlay swallowing clicks.
                    if event.pressed_button.is_none() {
                        this.finish_sidebar_resize(cx);
                        return;
                    }
                    let raw = f32::from(event.position.x);
                    let width = raw.clamp(
                        theme::SIDEBAR_WIDTH_MIN as f32,
                        theme::SIDEBAR_WIDTH_MAX as f32,
                    );
                    AppSettings::global_mut(cx).sidebar_width = width as f64;
                    // Latch the first limit of this drag; a visual bounce is
                    // not animated yet (recorded as a follow-up).
                    if raw <= theme::SIDEBAR_WIDTH_MIN as f32 {
                        this.latched = this.latched.or(Some(SidebarEdge::Min));
                    } else if raw >= theme::SIDEBAR_WIDTH_MAX as f32 {
                        this.latched = this.latched.or(Some(SidebarEdge::Max));
                    } else {
                        this.latched = None;
                    }
                    cx.notify();
                }))
                .on_mouse_up(
                    MouseButton::Left,
                    cx.listener(|this, _, _, cx| this.finish_sidebar_resize(cx)),
                )
                .on_mouse_up_out(
                    MouseButton::Left,
                    cx.listener(|this, _, _, cx| this.finish_sidebar_resize(cx)),
                )
        });
        div()
            .size_full()
            .relative()
            .child(
                AppShell::new()
                    .page_title(page_title)
                    .settings_view(settings.into_any_element()),
            )
            .children(seam)
            .children(capture)
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
            let appearance = settings.appearance;
            let accent = settings.accent;
            let font_size = settings.font_size;
            let language = settings.language.clone();
            let launch_at_login = settings.launch_at_login;
            // Theme set + overrides (font size, focus ring), then the stored
            // mode, locale and autostart state.
            theme::install(cx, font_size, accent);
            // The same funnel the control uses, so startup cannot drift from
            // it. `System` resolves against the OS here, before any window
            // exists.
            apply_appearance(appearance, cx);
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
            // ⌘B toggles the rail between the persisted width and the 1px
            // grab sliver (same affordance as the titlebar collapse button).
            cx.on_action(|_: &ToggleSidebar, cx: &mut App| {
                let state = cx.global_mut::<ShellState>();
                state.collapsed = !state.collapsed;
                cx.refresh_windows();
            });
            // Frameless window driven by the vendored `TitleBar`: a 38px
            // overlay bar with macOS traffic lights at (14,14) and an
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
                // zeron's chrome floor: below this the overlay titlebar and the
                // rail/content split stop being usable.
                window_min_size: Some(size(px(900.), px(600.))),
                titlebar: Some(TitlebarOptions {
                    traffic_light_position: Some(shell::traffic_light_origin()),
                    ..TitleBar::title_bar_options()
                }),
                // Applied at creation so a stored translucent window opens
                // translucent, with no opaque first frame to flash.
                window_background: window_background_for(
                    AppSettings::global(cx).background_translucent,
                    AppSettings::global(cx).background_blur,
                ),
                window_decorations,
                ..TitleBar::window_options()
            };
            cx.spawn(async move |cx| {
                cx.open_window(window_options, |window, cx| {
                    let view = cx.new(|cx| SettingsWindow::new(window, cx));
                    // The first level on the window must be a Root.
                    let root = cx.new(|cx| Root::new(view, window, cx));
                    // Publish the handle so the glass rows can switch Root's
                    // opaque base paint off (see `apply_glass`), and apply
                    // the stored setting to the very first frame.
                    cx.set_global(WindowRoot(root.clone()));
                    let translucent = AppSettings::global(cx).background_translucent;
                    if let Some(base) = root_glass_bg(cx, translucent) {
                        root.update(cx, |root, _| {
                            root.style().background = Some(base);
                        });
                    }
                    root
                })
                .expect("failed to open window");
            })
            .detach();
        });
}

#[cfg(test)]
mod tests {
    use super::{normalize_font_size, window_background_for};
    use gpui_kit::WindowBackgroundAppearance;

    /// The window surface only leaves the opaque default when the master
    /// switch is on, and only blurs when both switches are on — this is what
    /// stops a translucent-but-unblurred or blur-but-opaque combination.
    #[test]
    fn window_appearance_follows_the_glass_switches() {
        assert_eq!(
            window_background_for(false, false),
            WindowBackgroundAppearance::Opaque
        );
        // Blur is ignored while the window stays opaque.
        assert_eq!(
            window_background_for(false, true),
            WindowBackgroundAppearance::Opaque
        );
        assert_eq!(
            window_background_for(true, false),
            WindowBackgroundAppearance::Transparent
        );
        assert_eq!(
            window_background_for(true, true),
            WindowBackgroundAppearance::Blurred
        );
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
