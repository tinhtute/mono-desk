//! zeron's settings widgets, ported from `crates/ui/src/settings/widgets.rs`
//! (MIT, Copyright 2026 Wing).
//!
//! Ported in phase `chrome` T2: the section-list row ([`section_tab`]) and its
//! retargetable selected-state fade ([`tab_selection_t`]). The page scaffold
//! (`page_column`, `page_header`, `section`, `section_card`, `card_row`,
//! `row_title`, `meta_line`) and the controls (`toggle_switch`, `select_*`,
//! `block_fill`, `row_divider`, `field_label`, `option_card`, `accent_swatch`)
//! arrive with the `settings-surface` phase.
//!
//! Adaptations from upstream: none in this file — the widgets depend only on
//! the vendored theme, motion and typography modules. The scroll edge-fade
//! wrapper (`scroll_faded`) and the faded label are not ported yet (they use
//! zeron's `edge_fade` module); callers use plain overflow containers until
//! that module lands.

use gpui::prelude::*;
use gpui::{Div, FontWeight, SharedString, Stateful, Window, div, px};

use crate::motion;
use crate::theme::{self, Theme};
use crate::typography;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_travel_settles_on_its_target() {
        // A settled travel value is the target itself, in both directions —
        // first paint never animates in from the other state.
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
