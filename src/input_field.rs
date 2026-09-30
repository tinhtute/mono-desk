//! App-side input field: label + `Input` + validation message row.
//!
//! beUI's `input` (label, leading/trailing slots, error row, error shake) has
//! no GPUI Kit equivalent — the vendored `Input` is the bare control. This
//! composes the field app-side instead of patching the vendored crate:
//!
//! - the vendored [`Input`] keeps its own background, border and focus tint;
//!   the field only replaces the border colour for `error` / `success`,
//! - caller-owned prefix/suffix slots are drawn inside the input frame and
//!   can react to the field's status (beUI's success check),
//! - the message row below the input carries the validation text in the
//!   state's ink, and can reserve its line so the layout does not jump,
//! - entering an error plays a short horizontal shake through
//!   [`animate_keyframes`], which honours `App::reduce_motion` for free.
//!
//! [`FieldState`] is the whole state model and is window-free, so the
//! precedence rules are unit-tested without a GPUI context.

use std::rc::Rc;
use std::time::Duration;

use gpui_kit::base::{Keyframe, Keyframes, Timing, animate_keyframes};
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::{
    ActiveTheme, Sizable, Size, StyledExt, Theme, h_flex, label::Label, v_flex,
};
use gpui_kit::{
    AnyElement, App, AppContext as _, Context, ElementId, Entity, Hsla, IntoElement,
    ParentElement as _, Pixels, Render, SharedString, StyleRefinement, Styled as _, Subscription,
    Window, px,
};

/// How long the error shake runs.
const SHAKE_DURATION: Duration = Duration::from_millis(450);

/// Horizontal offsets of the error shake, as `(progress, px)` keyframes.
///
/// beUI's `x: [0, -6, 6, -4, 4, -2, 0]` over 0.45 s, spread over the same
/// even spacing. The first and last frames are at rest, so the field always
/// ends where it started.
const SHAKE_FRAMES: &[(f32, f32)] = &[
    (0.0, 0.0),
    (1.0 / 6.0, -6.0),
    (2.0 / 6.0, 6.0),
    (3.0 / 6.0, -4.0),
    (4.0 / 6.0, 4.0),
    (5.0 / 6.0, -2.0),
    (1.0, 0.0),
];

/// The shake keyframes, built once per field.
fn shake_keyframes() -> Keyframes<Pixels> {
    Keyframes::try_new(
        SHAKE_FRAMES
            .iter()
            .map(|(offset, x)| Keyframe::new(*offset, px(*x))),
    )
    .expect("SHAKE_FRAMES is a well-formed keyframe sequence")
}

/// Validation outcome for the field's current value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Validation {
    /// The value is invalid; the message says why.
    Error(SharedString),
    /// The value is valid; the message confirms it.
    Success(SharedString),
}

impl Validation {
    /// The text the message row shows.
    pub fn message(&self) -> &SharedString {
        match self {
            Self::Error(message) | Self::Success(message) => message,
        }
    }
}

/// Effective presentation state of an [`InputFieldView`].
///
/// Precedence: `Disabled` > `Error` > `Success` > `Focused` > `Idle`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldStatus {
    /// No focus, no validation result.
    Idle,
    /// Focused, no validation result.
    Focused,
    /// The value failed validation.
    Error,
    /// The value passed validation.
    Success,
    /// Not editable; the message row stays empty.
    Disabled,
}

/// The field's state, independent of any window.
///
/// Focus, disabled and the validation result are tracked separately and
/// resolved into a [`FieldStatus`] on demand, so a focus change never clears a
/// validation result and disabling never loses one.
#[derive(Clone, Debug, Default)]
pub struct FieldState {
    focused: bool,
    disabled: bool,
    validation: Option<Validation>,
    shake_generation: u64,
}

impl FieldState {
    /// A fresh, idle field.
    pub fn new() -> Self {
        Self::default()
    }

    /// The state the field presents right now.
    pub fn status(&self) -> FieldStatus {
        if self.disabled {
            FieldStatus::Disabled
        } else if matches!(self.validation, Some(Validation::Error(_))) {
            FieldStatus::Error
        } else if matches!(self.validation, Some(Validation::Success(_))) {
            FieldStatus::Success
        } else if self.focused {
            FieldStatus::Focused
        } else {
            FieldStatus::Idle
        }
    }

    /// Whether the field is disabled.
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// The message the row below the input shows: none while disabled,
    /// otherwise the validation text.
    pub fn visible_message(&self) -> Option<&SharedString> {
        if self.disabled {
            None
        } else {
            self.validation.as_ref().map(Validation::message)
        }
    }

    /// Bumped whenever a validation error is entered from a non-error state.
    ///
    /// The render keys the shake playback on it, so the shake replays for a
    /// new error but not while the user is typing through an existing one.
    pub fn shake_generation(&self) -> u64 {
        self.shake_generation
    }

    /// Set focus. Returns whether the state changed.
    pub fn set_focused(&mut self, focused: bool) -> bool {
        let changed = self.focused != focused;
        self.focused = focused;
        changed
    }

    /// Set disabled. Returns whether the state changed.
    pub fn set_disabled(&mut self, disabled: bool) -> bool {
        let changed = self.disabled != disabled;
        self.disabled = disabled;
        changed
    }

    /// Replace the validation result. Returns whether the state changed.
    ///
    /// Entering an error from a non-error state bumps the shake generation;
    /// staying in error does not, even when the message changes.
    pub fn set_validation(&mut self, validation: Option<Validation>) -> bool {
        self.replace_validation(validation, true)
    }

    /// Adopt a validation result computed for a value the user did not type —
    /// the initial value, an external write, a page reset.
    ///
    /// Returns whether the state changed. Unlike [`Self::set_validation`] this
    /// never bumps the shake generation: there is no keystroke to shake at.
    pub fn adopt_validation(&mut self, validation: Option<Validation>) -> bool {
        self.replace_validation(validation, false)
    }

    fn replace_validation(&mut self, validation: Option<Validation>, shake: bool) -> bool {
        let was_error = matches!(self.validation, Some(Validation::Error(_)));
        let is_error = matches!(validation, Some(Validation::Error(_)));
        if shake && is_error && !was_error {
            self.shake_generation += 1;
        }
        let changed = self.validation != validation;
        self.validation = validation;
        changed
    }
}

/// Builds the element for one of the input's inline slots.
///
/// A factory rather than an element: the field re-renders on every keystroke,
/// and an element is consumed by its first render. The factory receives the
/// field's current status, so a slot can react to the validation state — the
/// demo's success check, for example — and returns `None` to leave the slot
/// empty (and its padding unclaimed).
pub type InputSlot = Rc<dyn Fn(FieldStatus, &mut Window, &mut App) -> Option<AnyElement>>;

/// Validates the field's text; `None` means "no message".
pub type Validator = Rc<dyn Fn(&str) -> Option<Validation>>;

/// Reads the value the field mirrors from the app.
pub type ValueReader = Rc<dyn Fn(&App) -> SharedString>;

/// Writes a new value back to the app.
pub type ValueWriter = Rc<dyn Fn(SharedString, &mut App)>;

/// Everything an [`InputFieldView`] needs from its owner.
#[derive(Clone)]
pub struct InputFieldConfig {
    /// Label above the input. `None` when the surrounding layout already
    /// labels the field — a settings row's title, for example.
    pub label: Option<SharedString>,
    /// Placeholder shown while the input is empty.
    pub placeholder: SharedString,
    /// Keep the message row's height while there is no message, so the layout
    /// does not jump when a validation result appears.
    pub reserve_message_line: bool,
    /// Element drawn before the input text (a leading icon, say).
    pub prefix: Option<InputSlot>,
    /// Element drawn after the input text.
    pub suffix: Option<InputSlot>,
    /// Validation for the current text; `None` means "no message".
    pub validate: Validator,
    /// Reads the value the field mirrors.
    pub get_value: ValueReader,
    /// Writes a new value back to the app.
    pub set_value: ValueWriter,
}

impl InputFieldConfig {
    /// A config with the required callbacks and no label or slots.
    pub fn new(
        placeholder: impl Into<SharedString>,
        validate: impl Fn(&str) -> Option<Validation> + 'static,
        get_value: impl Fn(&App) -> SharedString + 'static,
        set_value: impl Fn(SharedString, &mut App) + 'static,
    ) -> Self {
        Self {
            label: None,
            placeholder: placeholder.into(),
            reserve_message_line: false,
            prefix: None,
            suffix: None,
            validate: Rc::new(validate),
            get_value: Rc::new(get_value),
            set_value: Rc::new(set_value),
        }
    }

    /// Reserve the message row's line while there is no message.
    pub fn reserve_message_line(mut self) -> Self {
        self.reserve_message_line = true;
        self
    }

    /// Draw `slot` before the input text.
    pub fn prefix(mut self, slot: InputSlot) -> Self {
        self.prefix = Some(slot);
        self
    }

    /// Draw `slot` after the input text.
    pub fn suffix(mut self, slot: InputSlot) -> Self {
        self.suffix = Some(slot);
        self
    }
}

/// The ink a status paints with.
///
/// `theme.danger` / `theme.success` are the pastel chip *backgrounds*
/// (`danger.background` / `success.background` in `mono.json`); text, icons
/// and borders need the matching `*_foreground` ink, or the message reads
/// faded against the card. Crate-visible so callers drawing their own state
/// affordances (the demo's success check) share the one mapping.
pub(crate) fn status_ink(status: FieldStatus, theme: &Theme) -> Hsla {
    match status {
        FieldStatus::Error => theme.danger_foreground,
        FieldStatus::Success => theme.success_foreground,
        _ => theme.muted_foreground,
    }
}

/// Reconcile the field with the value the app holds.
///
/// Returns the value the input has to be set to, or `None` when it already
/// shows it.
///
/// `InputState::set_value` emits no [`InputEvent::Change`], so validation
/// cannot ride on the input event alone: a value written from outside the
/// field — the initial value, a page reset, any external write — would
/// otherwise keep the message the previous value earned. The result is
/// adopted without shaking, because the user did not type this value.
fn reconcile(
    current: &str,
    value: &SharedString,
    validate: &Validator,
    field: &mut FieldState,
) -> Option<SharedString> {
    if current == value.as_ref() {
        return None;
    }
    field.adopt_validation(validate(value));
    Some(value.clone())
}

/// Keyed-state slot for the field view.
///
/// `use_keyed_state` observes the entity it stores and notifies the current
/// view when that entity notifies; holding the view inside a slot that never
/// notifies keeps a keystroke from re-rendering the whole page around the
/// field — the view re-renders itself.
struct FieldSlot(Entity<InputFieldView>);

/// A labelled input with a validation message row.
///
/// The view owns the [`InputState`] and the [`FieldState`]; the app's own
/// value stays the source of truth through `get_value` / `set_value`.
pub struct InputFieldView {
    input: Entity<InputState>,
    field: FieldState,
    config: InputFieldConfig,
    size: Size,
    shake: Keyframes<Pixels>,
    _subscription: Subscription,
}

impl InputFieldView {
    /// Get or create the field keyed by `id` in `window`.
    ///
    /// The keyed state survives re-renders of the surrounding view, so the
    /// field keeps its text, focus and validation while the page re-renders.
    /// `id` must be unique per window, like any other element id.
    pub fn keyed(
        window: &mut Window,
        cx: &mut App,
        id: impl Into<ElementId>,
        config: InputFieldConfig,
    ) -> Entity<Self> {
        let slot = window.use_keyed_state(id, cx, |window, cx| {
            FieldSlot(cx.new(|cx| Self::new(window, cx, config)))
        });
        slot.read(cx).0.clone()
    }

    fn new(window: &mut Window, cx: &mut Context<Self>, config: InputFieldConfig) -> Self {
        let initial = (config.get_value)(cx);
        let input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder(config.placeholder.clone())
                .default_value(initial.clone())
        });
        let subscription = cx.subscribe_in(
            &input,
            window,
            |this, input, event: &InputEvent, _window, cx| {
                let changed = match event {
                    InputEvent::Change => {
                        let value = input.read(cx).value();
                        (this.config.set_value)(value.clone(), cx);
                        this.field.set_validation((this.config.validate)(&value))
                    }
                    InputEvent::Focus => this.field.set_focused(true),
                    InputEvent::Blur => this.field.set_focused(false),
                    InputEvent::PressEnter { .. } => false,
                };
                if changed {
                    cx.notify();
                }
            },
        );
        // The value the field mounts with is validated up front, so a field
        // that starts on an invalid value shows its message without waiting
        // for a keystroke.
        let mut field = FieldState::new();
        field.adopt_validation((config.validate)(&initial));
        Self {
            input,
            field,
            config,
            size: Size::Medium,
            shake: shake_keyframes(),
            _subscription: subscription,
        }
    }

    /// Mirror the surrounding control's presentation.
    ///
    /// Called from the owner's render; notifies only when something changed.
    pub fn set_presentation(&mut self, size: Size, disabled: bool, cx: &mut Context<Self>) {
        let size_changed = self.size != size;
        self.size = size;
        if self.field.set_disabled(disabled) || size_changed {
            cx.notify();
        }
    }

    /// The border colour the validation state imposes, if any.
    ///
    /// `None` leaves the vendored input's own border — and its focus tint —
    /// alone. The error/success inks replace it and stay while the field is
    /// focused, so the state remains visible while the user fixes the value;
    /// the caret still marks focus.
    fn state_border(&self, status: FieldStatus, cx: &App) -> Option<Hsla> {
        match status {
            FieldStatus::Error | FieldStatus::Success => Some(status_ink(status, cx.theme())),
            _ => None,
        }
    }

    fn render_input(&self, status: FieldStatus, window: &mut Window, cx: &mut App) -> AnyElement {
        let mut input = Input::new(&self.input)
            .with_size(self.size)
            .disabled(self.field.is_disabled())
            .w_full();
        if let Some(prefix) = self
            .config
            .prefix
            .clone()
            .and_then(|slot| slot(status, window, cx))
        {
            input = input.prefix(prefix);
        }
        if let Some(suffix) = self
            .config
            .suffix
            .clone()
            .and_then(|slot| slot(status, window, cx))
        {
            input = input.suffix(suffix);
        }
        if let Some(border) = self.state_border(status, cx) {
            // `focus_bordered(false)` keeps the focused semantic style from
            // repainting the border with the ring colour over the state
            // colour.
            input = input
                .focus_bordered(false)
                .refine_style(&StyleRefinement::default().border_color(border));
        }
        input.into_any_element()
    }

    /// The message row: the validation text in the state's ink, or an empty
    /// line when the row is reserved.
    ///
    /// beUI's error row is text-only (`text-xs text-destructive`); the success
    /// state gets the same row in the success ink, and its glyph lives in the
    /// input's suffix slot instead.
    fn render_message(&self, status: FieldStatus, cx: &App) -> Option<AnyElement> {
        let ink = status_ink(status, cx.theme());
        let message = self.field.visible_message().cloned();
        if message.is_none() && !self.config.reserve_message_line {
            return None;
        }

        // `h_5` is `text_sm`'s line height, so the reserved line and the
        // filled one are the same height.
        let mut row = h_flex().h_5().w_full().text_sm().text_color(ink);
        if let Some(message) = message {
            row = row.child(Label::new(message).text_sm().text_color(ink));
        }
        Some(row.into_any_element())
    }
}

impl Render for InputFieldView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // The app stays the source of truth: mirror its value so an external
        // write (a page reset, say) shows up on the next render. The mirror
        // also re-runs the validation — `set_value` emits no `Change`, so
        // otherwise the message would outlive the value it described.
        let value = (self.config.get_value)(cx);
        let current = self.input.read(cx).value();
        if let Some(next) = reconcile(
            current.as_ref(),
            &value,
            &self.config.validate,
            &mut self.field,
        ) {
            self.input
                .update(cx, |input, cx| input.set_value(next, window, cx));
        }

        let status = self.field.status();
        let mut field = v_flex().w_full().gap_1();
        if let Some(label) = self.config.label.clone() {
            field = field.child(Label::new(label).text_sm());
        }
        field = field.child(self.render_input(status, window, cx));
        if let Some(message) = self.render_message(status, cx) {
            field = field.child(message);
        }

        // Generation 0 is the initial render: nothing to shake yet, and
        // skipping it avoids a no-op playback on every field's first frame.
        let generation = self.field.shake_generation();
        if generation > 0 {
            let offset = animate_keyframes(
                (
                    ElementId::View(cx.entity_id()),
                    SharedString::from(format!("shake-{generation}")),
                ),
                &self.shake,
                Timing::new(SHAKE_DURATION),
                window,
                cx,
            )
            .value;
            // Relative positioning shifts the paint without moving siblings.
            field = field.relative().left(offset);
        }
        field
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_kit::component::{ThemeMode, ThemeSet};

    fn error(message: &str) -> Option<Validation> {
        Some(Validation::Error(message.into()))
    }

    fn success(message: &str) -> Option<Validation> {
        Some(Validation::Success(message.into()))
    }

    #[test]
    fn status_resolves_by_precedence() {
        let mut state = FieldState::new();
        assert_eq!(state.status(), FieldStatus::Idle);

        state.set_focused(true);
        assert_eq!(state.status(), FieldStatus::Focused);

        state.set_validation(error("taken"));
        assert_eq!(state.status(), FieldStatus::Error);

        state.set_validation(success("available"));
        assert_eq!(state.status(), FieldStatus::Success);

        state.set_disabled(true);
        assert_eq!(state.status(), FieldStatus::Disabled);
    }

    #[test]
    fn focus_never_clears_a_validation_result() {
        let mut state = FieldState::new();
        state.set_validation(error("taken"));

        state.set_focused(true);
        assert_eq!(state.status(), FieldStatus::Error);
        state.set_focused(false);
        assert_eq!(state.status(), FieldStatus::Error);
    }

    #[test]
    fn disabled_hides_the_message_without_losing_it() {
        let mut state = FieldState::new();
        state.set_validation(error("taken"));
        assert_eq!(state.visible_message().map(|m| m.as_ref()), Some("taken"));

        state.set_disabled(true);
        assert!(state.visible_message().is_none());

        state.set_disabled(false);
        assert_eq!(state.visible_message().map(|m| m.as_ref()), Some("taken"));
    }

    #[test]
    fn shake_fires_once_per_error_entry() {
        let mut state = FieldState::new();
        assert_eq!(state.shake_generation(), 0);

        state.set_validation(error("too short"));
        assert_eq!(state.shake_generation(), 1);

        // Staying in error does not re-shake, even when the message changes.
        state.set_validation(error("invalid character"));
        assert_eq!(state.shake_generation(), 1);

        // Clearing and re-entering shakes again.
        state.set_validation(success("available"));
        assert_eq!(state.shake_generation(), 1);
        state.set_validation(error("too short"));
        assert_eq!(state.shake_generation(), 2);
    }

    #[test]
    fn set_validation_reports_whether_the_state_changed() {
        let mut state = FieldState::new();
        assert!(state.set_validation(error("taken")));
        assert!(!state.set_validation(error("taken")));
        assert!(state.set_validation(None));
        assert!(!state.set_validation(None));
    }

    /// A page reset writes the value from outside the field, and the vendored
    /// `InputState::set_value` emits no `Change` — so the validation has to be
    /// re-run against the new value. Without that, the error the previous
    /// entry earned outlives the text it described.
    #[test]
    fn external_value_sync_clears_a_stale_error_without_shaking() {
        // The demo's rule in miniature: empty is idle, under three characters
        // is an error, anything else is fine.
        let validate: Validator = Rc::new(|value: &str| match value.chars().count() {
            0 => None,
            n if n < 3 => error("too short"),
            _ => success("ok"),
        });

        let mut field = FieldState::new();

        // The user types "ab": the field reports the error and shakes.
        field.set_validation(validate("ab"));
        assert_eq!(field.status(), FieldStatus::Error);
        assert_eq!(field.shake_generation(), 1);

        // Reset writes "ninja" from outside. The input still shows "ab", so
        // the field is re-synced — and the error must not survive it.
        let next = reconcile("ab", &"ninja".into(), &validate, &mut field);
        assert_eq!(next.as_deref(), Some("ninja"));
        assert_eq!(field.status(), FieldStatus::Success);
        assert_eq!(field.visible_message().map(|m| m.as_ref()), Some("ok"));
        // Nothing was typed, so nothing shakes.
        assert_eq!(field.shake_generation(), 1);

        // An input that already shows the app's value needs no sync.
        assert!(reconcile("ninja", &"ninja".into(), &validate, &mut field).is_none());
        assert_eq!(field.status(), FieldStatus::Success);
    }

    #[test]
    fn set_focused_and_set_disabled_report_whether_the_state_changed() {
        let mut state = FieldState::new();
        assert!(state.set_focused(true));
        assert!(!state.set_focused(true));
        assert!(state.set_focused(false));

        assert!(state.set_disabled(true));
        assert!(!state.set_disabled(true));
        assert!(state.set_disabled(false));
    }

    /// The shake must start and end at rest, or the field would settle offset
    /// from where it belongs.
    #[test]
    fn shake_keyframes_start_and_end_at_rest() {
        let frames = shake_keyframes();
        assert_eq!(frames.sample(0.0), px(0.));
        assert_eq!(frames.sample(1.0), px(0.));
        assert_ne!(frames.sample(0.15), px(0.));
    }

    /// The app's own palette, resolved the way `Theme::change` resolves it.
    fn mono_theme(mode: ThemeMode) -> Theme {
        let set: ThemeSet = serde_json::from_str(crate::theme::MONO_THEMES)
            .expect("embedded mono.json must deserialize");
        let config = set
            .themes
            .into_iter()
            .find(|theme| theme.mode == mode)
            .expect("both modes are present in the embedded theme set");
        let mut theme = Theme::default();
        theme.apply_config(&Rc::new(config));
        theme
    }

    /// The field's ink must be the state's *foreground* token.
    ///
    /// `theme.danger` / `theme.success` are the pastel chip *backgrounds*
    /// (`danger.background` / `success.background`), which read as faded text
    /// and borders against the card — the bug this pins. The palette must keep
    /// the tint and the ink distinct for the assertion to mean anything.
    #[test]
    fn status_ink_uses_the_foreground_tokens() {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let theme = mono_theme(mode);
            assert_ne!(
                theme.danger, theme.danger_foreground,
                "{mode:?}: the palette must distinguish the danger tint from its ink"
            );
            assert_ne!(
                theme.success, theme.success_foreground,
                "{mode:?}: the palette must distinguish the success tint from its ink"
            );

            assert_eq!(
                status_ink(FieldStatus::Error, &theme),
                theme.danger_foreground,
                "{mode:?}: error ink must be `danger.foreground`, not the tint"
            );
            assert_eq!(
                status_ink(FieldStatus::Success, &theme),
                theme.success_foreground,
                "{mode:?}: success ink must be `success.foreground`, not the tint"
            );
            for status in [
                FieldStatus::Idle,
                FieldStatus::Focused,
                FieldStatus::Disabled,
            ] {
                assert_eq!(status_ink(status, &theme), theme.muted_foreground);
            }
        }
    }
}
