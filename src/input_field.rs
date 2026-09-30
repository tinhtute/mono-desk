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
use gpui_kit::component::{ActiveTheme, Sizable, Size, StyledExt, h_flex, label::Label, v_flex};
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
        let was_error = matches!(self.validation, Some(Validation::Error(_)));
        let is_error = matches!(validation, Some(Validation::Error(_)));
        if is_error && !was_error {
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
        let input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder(config.placeholder.clone())
                .default_value((config.get_value)(cx))
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
        Self {
            input,
            field: FieldState::new(),
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
    /// alone. The error/success colours replace it and stay while the field is
    /// focused, so the state remains visible while the user fixes the value;
    /// the caret still marks focus.
    fn state_border(&self, status: FieldStatus, cx: &App) -> Option<Hsla> {
        match status {
            FieldStatus::Error => Some(cx.theme().danger),
            FieldStatus::Success => Some(cx.theme().success),
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
        let ink = match status {
            FieldStatus::Error => cx.theme().danger,
            FieldStatus::Success => cx.theme().success,
            _ => cx.theme().muted_foreground,
        };
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
        // write (a page reset, say) shows up on the next render.
        let value = (self.config.get_value)(cx);
        if self.input.read(cx).value() != value {
            self.input
                .update(cx, |input, cx| input.set_value(value, window, cx));
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
}
