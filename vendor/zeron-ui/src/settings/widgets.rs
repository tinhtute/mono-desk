//! zeron's settings widgets, ported from `crates/ui/src/settings/widgets.rs`
//! (MIT, Copyright 2026 Wing, pinned rev 8b7afb3).
//!
//! Ported in phase `chrome` T2: the section-list row ([`section_tab`]) and its
//! retargetable selected-state fade ([`tab_selection_t`]).
//!
//! Ported in phase `settings-surface` T1: the page scaffold ([`page_column`],
//! [`page_header`], [`page_subtitle`], [`field_label`], [`section`],
//! [`section_label`], [`details_label`], [`section_card`], [`card_row`],
//! [`block_fill`], [`row_divider`], [`row_tile`], [`row_title`], [`meta_line`]),
//! the option cards ([`option_card_row`], [`option_card`]) and the toggle
//! switch ([`toggle_switch`], `SwitchVisual`).
//!
//! Ported in phase `settings-surface` T2: the settings actions
//! ([`action_button`]/[`text_action`]/[`ghost_action`] with [`ActionTone`]),
//! the [`error_strip`]/[`warning_strip`] notes, the sidebar overflow fade
//! ([`scroll_faded`]) and the hover notes ([`text_tooltip`],
//! [`text_tooltip_above`]). T2 also ports zeron's accent swatch
//! ([`accent_swatch`]) for the Appearance page.
//!
//! Not ported: `PageScroll`/`rail` (this app's shell paints its own nav and
//! uses gpui's default scrollbars) and zeron's wallpaper/import machinery.
//! The menu and select halves of upstream's one file live in the sibling
//! `settings::menu` and `settings::select` modules and are re-exported here
//! through `crate::settings`.
//!
//! Adaptation principle: upstream code, formatting and doc comments stay
//! verbatim; only imports/paths change, and the widgets resolve through this
//! crate's vendored `theme`, `motion`, `typography` and `icons` modules.
//! [`scroll_faded`] uses the vendored `edge_fade` module.

use gpui::prelude::*;
use gpui::{AnyElement, Div, FontWeight, SharedString, Stateful, Window, div, px};

use crate::motion;
use crate::theme::{self, Theme};
use crate::typography;

/// Shared typography for a settings component's title and description. The
/// Shortcuts page established this compact rhythm; list-style settings reuse
/// it instead of drifting by page.
pub const ROW_TITLE_SIZE: f32 = 13.0;
pub const ROW_DESCRIPTION_SIZE: f32 = 12.0;

/// The page column's outer max width and side padding: a centered ~680px
/// content measure with generous air on either side.
const PAGE_MAX_WIDTH: f32 = 760.0;
const PAGE_PAD_X: f32 = 40.0;

/// Centered page column under the titlebar strip.
pub fn page_column() -> gpui::Div {
    div()
        .w_full()
        .max_w(px(PAGE_MAX_WIDTH))
        .mx_auto()
        .px(px(PAGE_PAD_X))
        // Titlebar clearance lives inside the scroll so content can scroll
        // up to the window edge and fade there, mirroring the bottom.
        .pt(px(Theme::TITLEBAR_HEIGHT + 16.0))
        .pb(px(48.0))
        .flex()
        .flex_col()
}

/// Page headline row: a large title, optionally followed by a muted count on
/// the same baseline. Inset to line up with the section labels.
pub fn page_header(theme: &Theme, title: &str, count: Option<usize>) -> gpui::Div {
    div()
        .px(px(SECTION_LABEL_INSET))
        .flex()
        .flex_row()
        .items_baseline()
        .gap(px(10.0))
        .child(
            div()
                .text_size(crate::typography::ui_rems(20.0))
                .line_height(crate::typography::ui_rems(26.0))
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(theme.text)
                .child(SharedString::from(title.to_string())),
        )
        .when_some(count, |el, count| {
            el.child(
                div()
                    .text_size(crate::typography::ui_rems(13.0))
                    .text_color(theme.text_muted)
                    .child(SharedString::from(format!("{count}"))),
            )
        })
}

/// Subtitle under the headline: `mt-1 text-[13px] text-muted-foreground`.
pub fn page_subtitle(theme: &Theme, copy: impl Into<SharedString>) -> gpui::Div {
    div()
        .mt(px(4.0))
        .px(px(SECTION_LABEL_INSET))
        .min_w_0()
        .text_size(crate::typography::ui_rems(13.0))
        .line_height(crate::typography::ui_rems(17.0))
        .text_color(theme.text_muted)
        .child(copy.into())
}

/// Small label above a group of controls (`text-[13px] font-medium`) — the
/// "Theme" caption over a picker, not a page headline.
pub fn field_label(theme: &Theme, label: impl Into<SharedString>) -> gpui::Div {
    div()
        .text_size(crate::typography::ui_rems(13.0))
        .font_weight(gpui::FontWeight::MEDIUM)
        .text_color(theme.text)
        .child(label.into())
}

/// A row of equally-sized preview cards for picking one of N *visual* options.
///
/// Deliberately knows nothing about themes: the caller supplies each preview as
/// an arbitrary element and picks however many cards it wants, so the same
/// control works for a density picker, a layout picker or anything else where
/// the choice is easier to show than to describe. Pair with [`option_card`].
pub fn option_card_row() -> gpui::Div {
    div().flex().flex_row().items_start().gap(px(16.0)).w_full()
}

/// Default height of an [`option_card`] preview frame.
pub const OPTION_CARD_HEIGHT: f32 = 148.0;
/// Corner radius of the preview frame.
///
/// Public because the preview has to round *itself* to this. gpui content masks
/// are axis-aligned rectangles, so `overflow_hidden` on the frame clips to its
/// bounding box and not to its corner radius — a preview that paints its own
/// background will square off the corners and cover the frame's border with it.
pub const OPTION_CARD_RADIUS: f32 = 6.0;

/// One card in an [`option_card_row`]: a fixed-height preview frame with a quiet
/// selected edge and caption underneath. There is deliberately no outer card
/// or ring; the preview itself is the control.
///
/// `preview` fills the frame and **must round its own corners** to
/// [`OPTION_CARD_RADIUS`] if it paints a background — see that constant.
///
/// Returns a plain `Div` like the rest of this module — the caller adds `.id(..)`
/// and `.on_click(..)`, so selection behaviour stays with the page that owns the
/// state.
pub fn option_card(
    theme: &Theme,
    icon_path: &'static str,
    label: impl Into<SharedString>,
    selected: bool,
    selection_t: f32,
    preview: AnyElement,
) -> gpui::Div {
    let selected_color = crate::motion::mix(theme.text_muted, theme.accent, selection_t);
    div()
        .flex_1()
        .min_w_0()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(8.0))
        .cursor_pointer()
        .child(
            div()
                .h(px(OPTION_CARD_HEIGHT))
                .flex_none()
                .w_full()
                .rounded(px(OPTION_CARD_RADIUS))
                .overflow_hidden()
                .border_1()
                .border_color(crate::motion::mix(theme.border, theme.accent, selection_t))
                .child(preview),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(6.0))
                .text_size(crate::typography::ui_rems(13.0))
                .font_weight(if selected {
                    gpui::FontWeight::MEDIUM
                } else {
                    gpui::FontWeight::NORMAL
                })
                .text_color(selected_color)
                .child(
                    crate::icons::icon(icon_path)
                        .size(px(16.0))
                        .text_color(selected_color),
                )
                .child(label.into()),
        )
}

/// Section labels and page titles sit a little inside the blocks' edge.
const SECTION_LABEL_INSET: f32 = 8.0;
/// Air between one section and the next.
const SECTION_GAP: f32 = 32.0;

/// The small plain label above a block ("Startup", "Fonts"): sentence case,
/// muted, never an uppercase eyebrow.
pub fn section_label(theme: &Theme, label: impl Into<SharedString>) -> gpui::Div {
    div()
        .px(px(SECTION_LABEL_INSET))
        .text_size(crate::typography::ui_rems(13.0))
        .line_height(crate::typography::ui_rems(17.0))
        .text_color(theme.text_muted)
        .child(label.into())
}

/// A [`section_label`] for a group nested inside a block row (an expanded
/// provider's "Completion", "Accounts"): flush with the row text it sits
/// under, and as tall as a small button so a trailing action never shifts it.
pub fn details_label(theme: &Theme, label: impl Into<SharedString>) -> gpui::Div {
    div()
        .min_h(px(32.0))
        .flex()
        .flex_row()
        .items_center()
        .text_size(crate::typography::ui_rems(13.0))
        .line_height(crate::typography::ui_rems(17.0))
        .text_color(theme.text_muted)
        .child(label.into())
}

/// A labeled section: the label, then its block tucked close underneath.
/// Pass a [`section_card`] (or any block) as `block`.
pub fn section(
    theme: &Theme,
    label: impl Into<SharedString>,
    block: impl IntoElement,
) -> gpui::Div {
    div()
        .mt(px(SECTION_GAP))
        .flex()
        .flex_col()
        .gap(px(8.0))
        .child(section_label(theme, label))
        .child(block)
}

/// The block's fill: a faint solid tint of the theme's ink, no outline.
pub fn block_fill(theme: &Theme) -> gpui::Hsla {
    theme.wash(0.045)
}

/// The hairline between rows inside a block.
pub fn row_divider(theme: &Theme) -> gpui::Hsla {
    theme.border.opacity(0.6)
}

/// The corners of a [`section_card`].
pub const BLOCK_RADIUS: f32 = 12.0;

/// A filled, rounded block of related settings rows. Standalone blocks keep
/// a section's worth of air above them; inside [`section`] the wrapper owns
/// the spacing and zeroes this margin.
pub fn section_card(theme: &Theme) -> gpui::Div {
    div()
        .mt(px(24.0))
        .rounded(px(BLOCK_RADIUS))
        .bg(block_fill(theme))
        .overflow_hidden()
        .flex()
        .flex_col()
}

/// One row in a [`section_card`]: title and muted description on the left,
/// the control on the right, split from the previous row by a hairline
/// inset to the text edge.
pub fn card_row(theme: &Theme, first: bool) -> gpui::Div {
    div()
        .mx(px(16.0))
        .py(px(12.0))
        .min_h(px(60.0))
        .when(!first, |el| {
            el.border_t_1().border_color(row_divider(theme))
        })
        .flex()
        .flex_row()
        .flex_wrap()
        .items_center()
        .gap(px(16.0))
}

/// Bare leading glyph for rows whose icon carries identity (a device's
/// platform), sized to sit beside a two-line title.
pub fn row_tile(theme: &Theme, icon_path: &'static str) -> gpui::Div {
    div()
        .flex_none()
        .w(px(24.0))
        .h(px(32.0))
        .flex()
        .items_center()
        .justify_center()
        .child(
            crate::icons::icon(icon_path)
                .size(px(16.0))
                .text_color(theme.text_muted),
        )
}

/// Row title. These metrics intentionally match the Shortcuts rows, whose
/// title/description rhythm is the reference for the other settings cards.
pub fn row_title(theme: &Theme, title: impl Into<SharedString>) -> gpui::Div {
    let title = title.into();
    #[cfg(test)]
    ROW_TITLES.with_borrow_mut(|titles| titles.push(title.clone()));
    div()
        .min_w_0()
        .text_size(crate::typography::ui_rems(ROW_TITLE_SIZE))
        .line_height(crate::typography::ui_rems(17.0))
        .font_weight(gpui::FontWeight::MEDIUM)
        .text_color(theme.text)
        .child(title)
}

#[cfg(test)]
thread_local! {
    /// Every [`row_title`] rendered since the last [`take_row_titles`].
    static ROW_TITLES: std::cell::RefCell<Vec<SharedString>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// The row titles rendered since the last call: what the settings search
/// must be able to find.
#[cfg(test)]
pub(crate) fn take_row_titles() -> Vec<SharedString> {
    ROW_TITLES.take()
}

/// The quiet meta line under a row title: `text-[12px]
/// text-muted-foreground/65` fragments joined by dots.
pub fn meta_line(theme: &Theme, fragments: Vec<AnyElement>) -> gpui::Div {
    let mut line = div()
        .mt(px(Theme::TEXT_STACK_GAP))
        .min_w_0()
        .max_w_full()
        .flex()
        .flex_row()
        .flex_wrap()
        .items_center()
        .gap_x(px(8.0))
        .gap_y(px(2.0))
        .text_size(crate::typography::ui_rems(ROW_DESCRIPTION_SIZE))
        .line_height(crate::typography::ui_rems(16.0))
        .text_color(theme.text_muted);
    let mut first = true;
    for fragment in fragments {
        if !first {
            line = line.child(
                div()
                    .text_color(theme.text_muted.opacity(0.3))
                    .child(SharedString::from("·")),
            );
        }
        line = line.child(div().min_w_0().max_w_full().child(fragment));
        first = false;
    }
    line
}

/// Right-anchored badge pill: a filled wash, like the settings blocks it
/// sits on (no outline).
pub fn badge(theme: &Theme, label: impl Into<SharedString>) -> gpui::Div {
    div()
        .flex_none()
        .px(px(8.0))
        .py(px(2.0))
        .rounded_full()
        .bg(theme.wash(0.08))
        .text_size(crate::typography::ui_rems(10.5))
        .text_color(theme.text_muted)
        .child(label.into())
}

/// Emerald status pill (the Accounts "Active" badge:
/// `bg-emerald-400/[0.12] text-emerald-300/90`).
pub fn badge_active(theme: &Theme, label: impl Into<SharedString>) -> gpui::Div {
    let emerald = theme.success;
    let emerald_text = theme.success_muted; // emerald-300
    div()
        .flex_none()
        .px(px(8.0))
        .py(px(2.0))
        .rounded_full()
        .bg(emerald.opacity(0.12))
        .text_size(crate::typography::ui_rems(10.5))
        .text_color(emerald_text.opacity(0.9))
        .child(label.into())
}

pub const SWITCH_WIDTH: f32 = 44.8;
const SWITCH_HEIGHT: f32 = 28.8;
const SWITCH_TRACK_HEIGHT: f32 = 20.8;
const SWITCH_SIDE_INSET: f32 = 1.6;
const SWITCH_THUMB_WIDTH: f32 = 24.0;
const SWITCH_THUMB_HEIGHT: f32 = SWITCH_TRACK_HEIGHT - 2.0 * SWITCH_SIDE_INSET;
const SWITCH_MARK_SIZE: f32 = 7.2;

/// A pill switch with the on/off marks nested beneath a sliding thumb.
/// The caller owns activation and accessibility; only the thumb interpolates.
pub fn toggle_switch(theme: &Theme, on: bool, key: impl Into<SharedString>) -> gpui::Div {
    let key: SharedString = key.into();
    div()
        .flex_none()
        .w(px(SWITCH_WIDTH))
        .h(px(SWITCH_HEIGHT))
        .child(SwitchVisual {
            theme: theme.clone(),
            on,
            key: format!("settings-switch-{key}").into(),
        })
}

#[derive(IntoElement)]
struct SwitchVisual {
    theme: Theme,
    on: bool,
    key: SharedString,
}

struct SwitchTravel {
    from: f32,
    target: f32,
    started: std::time::Instant,
}

impl SwitchTravel {
    fn value(&self, now: std::time::Instant) -> f32 {
        let t = (now.duration_since(self.started).as_secs_f32() / 0.18).min(1.0);
        self.from + (self.target - self.from) * (1.0 - (1.0 - t).powi(3))
    }
}

fn switch_track_color(theme: &Theme, on: bool) -> gpui::Hsla {
    let dark = theme.appearance.is_dark();
    if on {
        if dark {
            // Keep the accent saturated and opaque, but give the enabled
            // track more depth against the dark settings surface.
            crate::theme::flatten(gpui::black().opacity(0.14), theme.accent_strong)
        } else {
            // Preserve the current light opaque treatment.
            let accent = theme.accent;
            crate::theme::flatten(
                gpui::hsla(accent.h, accent.s, accent.l + (1.0 - accent.l) * 0.10, 0.98),
                theme.surface,
            )
        }
    } else {
        let opacity = match (dark, theme.is_frost()) {
            (true, true) => 0.22,
            (true, false) => 0.18,
            (false, true) => 0.12,
            (false, false) => 0.10,
        };
        crate::theme::flatten(theme.ink(opacity), theme.surface)
    }
}

fn switch_thumb_color(theme: &Theme) -> gpui::Hsla {
    let white = if theme.is_frost() {
        if theme.appearance.is_dark() {
            0.94
        } else {
            0.96
        }
    } else if theme.appearance.is_dark() {
        0.96
    } else {
        1.0
    };
    crate::theme::flatten(gpui::white().opacity(white), theme.surface)
}

/// Frosted switches catch a little light across their rim and thumb. Both
/// gradient stops are composited to opaque colors before painting.
fn switch_surface_tones(theme: &Theme, base: gpui::Hsla, thumb: bool) -> (gpui::Hsla, gpui::Hsla) {
    if !theme.is_frost() {
        return (base, base);
    }
    let (light, shade) = if thumb { (0.12, 0.07) } else { (0.07, 0.09) };
    (
        crate::theme::flatten(gpui::white().opacity(light), base),
        crate::theme::flatten(gpui::black().opacity(shade), base),
    )
}

impl RenderOnce for SwitchVisual {
    fn render(self, window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let now = std::time::Instant::now();
        let target = if self.on { 1.0 } else { 0.0 };
        let reduced = crate::motion::reduced_motion(cx);
        let position = window.with_global_id(self.key.into(), |id, window| {
            window.with_element_state(id, |previous: Option<SwitchTravel>, _| {
                let mut travel = previous.unwrap_or(SwitchTravel {
                    from: target,
                    target,
                    started: now,
                });
                let current = travel.value(now);
                if travel.target != target {
                    travel = SwitchTravel {
                        from: current,
                        target,
                        started: now,
                    };
                }
                if reduced {
                    travel.from = target;
                    travel.target = target;
                }
                (travel.value(now), travel)
            })
        });
        if (position - target).abs() > 0.001 {
            window.request_animation_frame();
        }
        let dark = self.theme.appearance.is_dark();
        let track = switch_track_color(&self.theme, self.on);
        let (track_light, track_shade) = switch_surface_tones(&self.theme, track, false);
        let thumb = switch_thumb_color(&self.theme);
        let (thumb_light, thumb_shade) = switch_surface_tones(&self.theme, thumb, true);
        let empty_width = SWITCH_WIDTH - SWITCH_THUMB_WIDTH - SWITCH_SIDE_INSET;
        let mark_padding = (empty_width - SWITCH_MARK_SIZE) / 2.0;
        let thumb_left = SWITCH_SIDE_INSET
            + (SWITCH_WIDTH - SWITCH_THUMB_WIDTH - 2.0 * SWITCH_SIDE_INSET) * position;
        let track_element = div()
            .absolute()
            .top(px((SWITCH_HEIGHT - SWITCH_TRACK_HEIGHT) / 2.0))
            .left_0()
            .w(px(SWITCH_WIDTH))
            .h(px(SWITCH_TRACK_HEIGHT))
            .rounded_full()
            .bg(gpui::linear_gradient(
                180.0,
                gpui::linear_color_stop(track_light, 0.0),
                gpui::linear_color_stop(track_shade, 1.0),
            ))
            .border_1()
            .border_color(if self.on {
                crate::theme::flatten(
                    gpui::white().opacity(if self.theme.is_frost() { 0.16 } else { 0.12 }),
                    track,
                )
            } else {
                crate::theme::flatten(self.theme.border, track)
            })
            .child(
                div()
                    .absolute()
                    .inset_0()
                    .px(px(mark_padding))
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .size(px(SWITCH_MARK_SIZE))
                            .flex()
                            .items_center()
                            .justify_center()
                            .opacity(position)
                            .child(
                                div()
                                    .w(px(1.2))
                                    .h(px(7.2))
                                    .rounded_full()
                                    .bg(gpui::white().opacity(0.96)),
                            ),
                    )
                    .child(
                        div()
                            .size(px(SWITCH_MARK_SIZE))
                            .flex()
                            .items_center()
                            .justify_center()
                            .opacity(1.0 - position)
                            .child(
                                div()
                                    .size(px(6.4))
                                    .rounded_full()
                                    .border(px(1.0))
                                    .border_color(gpui::white().opacity(0.92)),
                            ),
                    ),
            );
        let thumb_element = div()
            .absolute()
            .top(px((SWITCH_HEIGHT - SWITCH_THUMB_HEIGHT) / 2.0))
            .left(px(thumb_left))
            .w(px(SWITCH_THUMB_WIDTH))
            .h(px(SWITCH_THUMB_HEIGHT))
            .rounded_full()
            .bg(gpui::linear_gradient(
                180.0,
                gpui::linear_color_stop(thumb_light, 0.0),
                gpui::linear_color_stop(thumb_shade, 1.0),
            ))
            .border_1()
            .border_color(crate::theme::flatten(
                gpui::black().opacity(if dark { 0.10 } else { 0.08 }),
                thumb,
            ))
            // The rim highlight only belongs to the on state; it fades with
            // the thumb's travel so switching off doesn't pop.
            .when(self.theme.is_frost() && position > 0.001, |el| {
                el.child(
                    div()
                        .absolute()
                        .top(px(1.6))
                        .left(px(7.2))
                        .w(px(9.6))
                        .h(px(1.0))
                        .opacity(position)
                        .rounded_full()
                        .bg(crate::theme::flatten(
                            gpui::white().opacity(0.45),
                            thumb_light,
                        )),
                )
            });
        div()
            .relative()
            .w(px(SWITCH_WIDTH))
            .h(px(SWITCH_HEIGHT))
            .child(track_element)
            .child(thumb_element)
    }
}

/// A retargetable tab-selection fade: retargeting starts from the painted
/// value so a mid-flight flip reverses smoothly (zeron `TabSelectionTravel`).
struct TabSelectionTravel {
    from: f32,
    target: f32,
    started: std::time::Instant,
}

impl TabSelectionTravel {
    fn value(&self, now: std::time::Instant) -> f32 {
        let seconds = motion::TAB_SLIDE.total().as_secs_f32() * motion::speed_scale();
        let elapsed = now.saturating_duration_since(self.started).as_secs_f32();
        let progress = motion::TAB_SLIDE.progress((elapsed / seconds).min(1.0));
        self.from + (self.target - self.from) * progress
    }
}

/// Retargetable selected-state fade for a settings tab. First paint and
/// reduced-motion changes settle immediately instead of flashing an entrance.
pub fn tab_selection_t(
    window: &mut Window,
    key: impl Into<SharedString>,
    selected: bool,
    reduced_motion: bool,
) -> f32 {
    let now = std::time::Instant::now();
    let target = if selected { 1.0 } else { 0.0 };
    let value = window.with_global_id(key.into().into(), |id, window| {
        window.with_element_state(id, |previous: Option<TabSelectionTravel>, _| {
            let mut travel = previous.unwrap_or(TabSelectionTravel {
                from: target,
                target,
                started: now,
            });
            let current = travel.value(now);
            if travel.target != target {
                travel = TabSelectionTravel {
                    from: current,
                    target,
                    started: now,
                };
            }
            if reduced_motion {
                travel.from = target;
                travel.target = target;
            }
            (travel.value(now), travel)
        })
    });
    if (value - target).abs() > 0.001 {
        window.request_animation_frame();
    }
    value
}

/// One treatment for settings section tabs: the selected wash and text ease
/// over Zeron's tab timing, while hover keeps the normal sidebar color fade.
pub fn section_tab(
    theme: &Theme,
    selected: bool,
    selection_t: f32,
    id: impl Into<SharedString>,
    hover_key: impl Into<SharedString>,
) -> Stateful<Div> {
    let hover_key = hover_key.into();
    let base_bg = motion::mix(theme::wash(0.0), theme::glass_selected_bg(), selection_t);
    let base_text = motion::mix(theme.text_muted, theme.text, selection_t);
    let hover_bg = if selected {
        base_bg
    } else {
        theme.glass_hover()
    };
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(8.0))
        .rounded(px(8.0))
        .px(px(Theme::SPACE_SM))
        .py(px(6.0))
        .min_h(px(32.0))
        .flex_shrink_0()
        .text_size(typography::ui_rems(13.0))
        .when(selected, |el| el.font_weight(FontWeight::MEDIUM))
        .text_color(motion::hover_blend(&hover_key, base_text, theme.text))
        .bg(motion::hover_blend(&hover_key, base_bg, hover_bg))
        .id(id.into())
        .on_hover(motion::hover_listener(hover_key))
}

/// Settings actions share one size, corner radius, and hover language. Use
/// outlined for selectors, quiet for secondary actions, filled for row
/// actions on a block (the dropdown trigger's glass wash), solid for
/// commits, and danger for destructive actions behind a confirmation.
#[derive(Clone, Copy)]
pub enum ActionTone {
    Quiet,
    /// The secondary action: a borderless filled pill, the same as a
    /// dropdown trigger beside it.
    Filled,
    Solid,
    Danger,
}

pub fn action_button(theme: &Theme, tone: ActionTone) -> gpui::Div {
    let button = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(6.0))
        .rounded(px(8.0))
        .min_h(px(32.0))
        .px(px(10.0))
        .py(px(5.0))
        .text_size(crate::typography::ui_rems(12.5))
        .cursor_pointer();
    match tone {
        ActionTone::Quiet => button
            .text_color(theme.text_muted)
            .hover(|s| s.bg(theme.glass_hover()).text_color(theme.text)),
        ActionTone::Filled => {
            let lifted = crate::settings::select::select_fill(theme, true);
            button
                .bg(crate::settings::select::select_fill(theme, false))
                .text_color(theme.text)
                .hover(move |s| s.bg(lifted))
        }
        ActionTone::Solid => button
            .bg(theme.solid)
            .font_weight(gpui::FontWeight::MEDIUM)
            .text_color(theme.on_solid)
            .hover(|s| s.opacity(0.9)),
        // The dialogs' destructive fill (`popover::btn_danger`).
        ActionTone::Danger => button
            .bg(theme.danger_strong)
            .text_color(gpui::white())
            .hover(|s| s.opacity(0.9)),
    }
}

pub fn text_action(theme: &Theme, tone: ActionTone, label: impl Into<SharedString>) -> gpui::Div {
    action_button(theme, tone).child(label.into())
}

pub fn ghost_action(theme: &Theme) -> gpui::Div {
    action_button(theme, ActionTone::Quiet)
}

/// The dismissible red error strip (`flex items-start gap-2 rounded-xl border
/// border-red-400/20 bg-red-400/[0.06] text-red-300/90` with a leading
/// `DangerTriangle mt-0.5 size-4`).
pub fn error_strip(theme: &Theme, message: impl Into<SharedString>) -> gpui::Div {
    let red = theme.danger; // red-400
    let red_text = theme.danger_muted; // red-300
    div()
        .mt(px(16.0))
        .px(px(16.0))
        .py(px(12.0))
        .rounded(px(12.0))
        .border_1()
        .border_color(red.opacity(0.2))
        .bg(red.opacity(0.06))
        .text_size(crate::typography::ui_rems(12.5))
        .text_color(red_text.opacity(0.9))
        .flex()
        .flex_row()
        .items_start()
        .gap(px(8.0))
        .child(
            div().flex_none().mt(px(2.0)).child(
                crate::icons::icon(crate::icons::DANGER_TRIANGLE)
                    .size(px(16.0))
                    .text_color(red_text.opacity(0.9)),
            ),
        )
        .child(div().min_w_0().child(message.into()))
}

/// The amber warning strip (`flex items-start gap-2 border-amber-400/20
/// bg-amber-400/[0.06] text-amber-200/90` with a leading `DangerTriangle
/// mt-0.5 size-3.5`).
pub fn warning_strip(theme: &Theme, message: impl Into<SharedString>) -> gpui::Div {
    let amber = theme.warning; // amber-400
    let amber_text = theme.warning_muted; // amber-200
    div()
        .mt(px(8.0))
        .px(px(16.0))
        .py(px(10.0))
        .rounded(px(12.0))
        .border_1()
        .border_color(amber.opacity(0.2))
        .bg(amber.opacity(0.06))
        .text_size(crate::typography::ui_rems(12.0))
        .text_color(amber_text.opacity(0.9))
        .flex()
        .flex_row()
        .items_start()
        .gap(px(8.0))
        .child(
            div().flex_none().mt(px(2.0)).child(
                crate::icons::icon(crate::icons::DANGER_TRIANGLE)
                    .size(px(14.0))
                    .text_color(amber_text.opacity(0.9)),
            ),
        )
        .child(div().min_w_0().child(message.into()))
}

/// The sidebar's paint-time overflow fade, with a persistent scroll handle per
/// settings surface. No fade is painted when the content fits or at a reached edge.
pub fn scroll_faded(
    key: impl Into<SharedString>,
    area: gpui::Stateful<gpui::Div>,
) -> impl IntoElement {
    SettingsScroll {
        key: key.into(),
        area,
    }
}

#[derive(IntoElement)]
struct SettingsScroll {
    key: SharedString,
    area: gpui::Stateful<gpui::Div>,
}

impl RenderOnce for SettingsScroll {
    fn render(self, window: &mut gpui::Window, _: &mut gpui::App) -> impl IntoElement {
        let scroll = window.with_global_id(self.key.into(), |id, window| {
            window.with_element_state(id, |previous: Option<gpui::ScrollHandle>, _| {
                let scroll = previous.unwrap_or_default();
                (scroll.clone(), scroll)
            })
        });
        crate::edge_fade::edge_faded(16.0, true, true, self.area.track_scroll(&scroll))
            .fade_overflow_y(&scroll)
    }
}

/// A one-line hover note for settings controls (reset times, icon-only
/// actions), in the same frosted chip as the rest of the app's tooltips.
pub struct TextTooltip {
    text: SharedString,
    above: bool,
}

/// How far a [`text_tooltip_above`] chip sits over the pointer; clears a
/// small control the pointer is anywhere inside.
const TOOLTIP_ABOVE_GAP: f32 = 20.0;

impl Render for TextTooltip {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl IntoElement {
        let theme = Theme::of(cx);
        let card = div()
            .max_w(px(320.0))
            .px(px(9.0))
            .py(px(6.0))
            .rounded(px(6.0))
            .border_1()
            .border_color(theme.border)
            .bg(crate::popover::surface_bg(theme))
            .text_size(px(11.0))
            .text_color(theme.text_muted)
            .child(self.text.clone());
        let card = crate::frost::frosted(6.0, crate::frost::MENU_BLUR, card);
        if !self.above {
            return card.into_any_element();
        }
        // gpui hangs a tooltip below-right of the pointer. A zero-height box
        // with end-aligned content makes the chip overflow upward instead.
        div()
            .h(px(0.0))
            .flex()
            .flex_col()
            .justify_end()
            .child(div().relative().bottom(px(TOOLTIP_ABOVE_GAP)).child(card))
            .into_any_element()
    }
}

/// `.tooltip(...)` builder for a [`TextTooltip`].
pub fn text_tooltip(
    text: impl Into<SharedString>,
) -> impl Fn(&mut gpui::Window, &mut gpui::App) -> gpui::AnyView + 'static {
    let text: SharedString = text.into();
    move |_, cx| {
        cx.new(|_| TextTooltip {
            text: text.clone(),
            above: false,
        })
        .into()
    }
}

/// Like [`text_tooltip`], shown above the pointer instead of below it, for
/// controls with content underneath that the chip would otherwise cover.
pub fn text_tooltip_above(
    text: impl Into<SharedString>,
) -> impl Fn(&mut gpui::Window, &mut gpui::App) -> gpui::AnyView + 'static {
    let text: SharedString = text.into();
    move |_, cx| {
        cx.new(|_| TextTooltip {
            text: text.clone(),
            above: true,
        })
        .into()
    }
}

/// One accent choice in the Appearance page's swatch row: a 30×34 swatch with
/// a selected underline, built by re-resolving the theme for the candidate
/// selection so the sample shows the accent it actually selects.
pub fn accent_swatch(
    page_theme: &Theme,
    selection: zeron_theme::AccentSelection,
    selected: bool,
) -> gpui::Stateful<gpui::Div> {
    use zeron_theme::AccentSelection;
    let swatch_theme = Theme::for_selection(
        page_theme.appearance,
        page_theme.variant_id.as_ref(),
        selection,
        page_theme.surface_preference,
    );
    let sample = match selection {
        AccentSelection::ThemeDefault => div()
            .size_full()
            .rounded(px(6.0))
            .bg(swatch_theme.accent_wash)
            .flex()
            .items_center()
            .justify_center()
            .gap(px(2.0))
            .child(
                div()
                    .w(px(4.0))
                    .h(px(13.0))
                    .rounded(px(2.0))
                    .bg(swatch_theme.glyph.light),
            )
            .child(
                div()
                    .w(px(4.0))
                    .h(px(16.0))
                    .rounded(px(2.0))
                    .bg(swatch_theme.glyph.mid),
            )
            .child(
                div()
                    .w(px(4.0))
                    .h(px(11.0))
                    .rounded(px(2.0))
                    .bg(swatch_theme.glyph.deep),
            ),
        AccentSelection::Preset(_) => div().size_full().rounded(px(6.0)).bg(swatch_theme.accent),
    };
    div()
        .id(SharedString::from(format!("accent-{}", selection.label())))
        .aria_label(selection.label())
        .aria_selected(selected)
        .tooltip(text_tooltip(selection.label()))
        .flex_none()
        .w(px(30.0))
        .h(px(34.0))
        .pb(px(4.0))
        .border_b_2()
        .border_color(if selected {
            swatch_theme.accent
        } else {
            gpui::transparent_black()
        })
        .cursor_pointer()
        .child(
            div()
                .size(px(30.0))
                .p(px(2.0))
                .rounded(px(8.0))
                .border_1()
                .border_color(if selected {
                    page_theme.border_strong
                } else {
                    page_theme.border
                })
                .bg(page_theme.surface_raised.opacity(0.42))
                .child(sample),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tab_selection_reverses_from_its_current_opacity() {
        use std::time::{Duration, Instant};

        let start = Instant::now();
        let forward = TabSelectionTravel {
            from: 0.0,
            target: 1.0,
            started: start,
        };
        let halfway = start + Duration::from_millis(75);
        let current = forward.value(halfway);
        let reverse = TabSelectionTravel {
            from: current,
            target: 0.0,
            started: halfway,
        };
        assert_eq!(reverse.value(halfway), current);
        assert_eq!(
            reverse.value(
                halfway
                    + motion::TAB_SLIDE
                        .total()
                        .mul_f32(motion::speed_scale())
            ),
            0.0
        );
    }

    #[test]
    fn selection_travel_settles_on_its_target() {
        // First paint never animates in from the other state.
        let now = std::time::Instant::now();
        for target in [0.0, 1.0] {
            let travel = TabSelectionTravel {
                from: target,
                target,
                started: now,
            };
            assert_eq!(travel.value(now), target);
        }
    }
}

#[cfg(test)]
mod switch_tests {
    use super::*;

    #[test]
    fn switch_material_keeps_dark_accent_and_opaque_fills() {
        use zeron_theme::SurfaceTreatment;

        let mut dark = Theme::dark();
        dark.surface_treatment = SurfaceTreatment::Opaque;
        let dark_on = switch_track_color(&dark, true);
        assert_eq!(dark_on.a, 1.0);
        assert!(dark_on.l < dark.accent_strong.l);
        assert_eq!(switch_track_color(&dark, false).a, 1.0);
        assert_eq!(switch_thumb_color(&dark).a, 1.0);
        assert_eq!(
            switch_surface_tones(&dark, dark_on, false),
            (dark_on, dark_on)
        );
        let opaque_off = switch_track_color(&dark, false);

        dark.surface_treatment = SurfaceTreatment::Frosted;
        assert_eq!(switch_track_color(&dark, true), dark_on);
        assert_eq!(switch_track_color(&dark, false).a, 1.0);
        assert_eq!(switch_thumb_color(&dark).a, 1.0);
        assert_ne!(switch_track_color(&dark, false), opaque_off);
        for (base, thumb) in [(dark_on, false), (switch_thumb_color(&dark), true)] {
            let (light, shade) = switch_surface_tones(&dark, base, thumb);
            assert_eq!((light.a, shade.a), (1.0, 1.0));
            assert!(light.l > base.l && shade.l < base.l);
        }

        let mut light = Theme::light();
        light.surface_treatment = SurfaceTreatment::Opaque;
        assert_eq!(switch_track_color(&light, true).a, 1.0);
        assert!(switch_track_color(&light, true).l > light.accent.l);
        let light_on = switch_track_color(&light, true);
        assert_eq!(
            switch_surface_tones(&light, light_on, false),
            (light_on, light_on)
        );
    }

    #[test]
    fn switch_reversal_keeps_current_position_and_settles() {
        use std::time::{Duration, Instant};
        let now = Instant::now();
        let forward = SwitchTravel {
            from: 0.0,
            target: 1.0,
            started: now,
        };
        let halfway = now + Duration::from_millis(90);
        let current = forward.value(halfway);
        let reverse = SwitchTravel {
            from: current,
            target: 0.0,
            started: halfway,
        };
        assert_eq!(reverse.value(halfway), current);
        assert_eq!(reverse.value(halfway + Duration::from_millis(180)), 0.0);
        assert_eq!(forward.value(now + Duration::from_millis(180)), 1.0);
    }
}
