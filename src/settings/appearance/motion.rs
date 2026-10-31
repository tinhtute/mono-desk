//! The Appearance page's Motion section: zeron's three rows — Reduce motion
//! (select + system-aware helper), Pause animations in background and Smooth
//! scrolling.
//!
//! Ported from zeron's `crates/ui/src/settings/appearance.rs` (MIT, rev
//! 8b7afb3): motion rows :3958-4083, `reduce_motion_helper` :1549-1558,
//! `smooth_scrolling_helper` :1560-1566.
//!
//! Adaptations: zeron reads/writes its motion runtime and settings service
//! (`crate::motion::*`, `crate::settings::smooth_scrolling`); mono-desk reads
//! zeron's vendored motion runtime and the `AppSettings` global (the page's
//! handlers persist every change). Smooth scrolling stays a persisted
//! preference only — mono-desk has no scroll-animation runtime (NG2). Select
//! option words are localized (`appearance.motion.system/on/off`).

use gpui::prelude::*;
use gpui::{AnyElement, Context, Role, Toggled, div, px};

use zeron_ui::motion::ReduceMotion;
use zeron_ui::settings::select::{SelectOption, select};
use zeron_ui::settings::widgets;
use zeron_ui::theme::Theme;

use crate::i18n::tr;

use super::AppearancePage;

/// The Reduce motion select's option words, localized (R8); upstream uses the
/// vendored `ReduceMotion::label()` strings directly.
fn reduce_motion_label(preference: ReduceMotion, lang: &str) -> &'static str {
    match preference {
        ReduceMotion::System => tr("appearance.motion.system", lang),
        ReduceMotion::On => tr("appearance.motion.on", lang),
        ReduceMotion::Off => tr("appearance.motion.off", lang),
    }
}

/// The Motion section's resolved state for one frame, snapshotted by the
/// page's render.
pub(super) struct MotionState {
    pub(super) reduce: ReduceMotion,
    pub(super) system_reduces: bool,
    pub(super) pause_in_background: bool,
    pub(super) smooth_scrolling: bool,
    /// The resolved flag (`motion::reduced_motion`): the preference combined
    /// with the OS setting and background pause.
    pub(super) reduced_motion: bool,
}

/// zeron `reduce_motion_helper` (:1549-1558), localized: the sentence tracks
/// both the preference and what the OS currently asks for.
pub(super) fn reduce_motion_helper(
    preference: ReduceMotion,
    system: bool,
    lang: &str,
) -> &'static str {
    match (preference, system) {
        (ReduceMotion::System, true) => tr("appearance.motion.helper.system_reduced", lang),
        (ReduceMotion::System, false) => tr("appearance.motion.helper.system_allows", lang),
        (ReduceMotion::On, _) => tr("appearance.motion.helper.on", lang),
        (ReduceMotion::Off, _) => tr("appearance.motion.helper.off", lang),
    }
}

/// zeron `smooth_scrolling_helper` (:1560-1566), localized: with reduced
/// motion the wheels step instead of gliding.
pub(super) fn smooth_scrolling_helper(enabled: bool, reduced: bool, lang: &str) -> &'static str {
    if enabled && reduced {
        tr("appearance.motion.smooth.desc_reduced", lang)
    } else {
        tr("appearance.motion.smooth.desc", lang)
    }
}

/// The Motion card's three rows (upstream :3958-4083).
pub(super) fn motion_rows(
    page: &mut AppearancePage,
    theme: &Theme,
    state: &MotionState,
    lang: &str,
    cx: &mut Context<AppearancePage>,
) -> Vec<AnyElement> {
    let reduce_index = ReduceMotion::ALL
        .into_iter()
        .position(|preference| preference == state.reduce)
        .unwrap_or_default();
    let reduce_control = select(
        "reduce-motion",
        tr("appearance.motion.reduce", lang),
        theme,
        |page: &mut AppearancePage| &mut page.motion,
    )
    .options(
        ReduceMotion::ALL
            .into_iter()
            .map(|preference| SelectOption::new(reduce_motion_label(preference, lang))),
        reduce_index,
    )
    .width(128.0)
    .on_select(|page: &mut AppearancePage, ix, _window, cx| {
        page.set_reduce_motion(ReduceMotion::ALL[ix], cx)
    })
    .render(&page.motion, cx);

    let pause_in_background = state.pause_in_background;
    let pause_control =
        widgets::toggle_switch(theme, pause_in_background, "pause-animations-in-background")
            .id("pause-animations-in-background-toggle")
            .tab_index(0)
            .role(Role::Switch)
            .aria_label(tr("appearance.motion.pause", lang))
            .focus_visible(|style| style.border_2().border_color(theme.accent))
            .cursor_pointer()
            .aria_toggled(if pause_in_background {
                Toggled::True
            } else {
                Toggled::False
            })
            .on_click(cx.listener(move |page, _, _, cx| {
                page.set_pause_in_background(!pause_in_background, cx)
            }));

    let smooth_scrolling = state.smooth_scrolling;
    let smooth_control = widgets::toggle_switch(theme, smooth_scrolling, "smooth-scrolling")
        .id("smooth-scrolling-toggle")
        .tab_index(0)
        .role(Role::Switch)
        .aria_label(tr("appearance.motion.smooth", lang))
        .focus_visible(|style| style.border_2().border_color(theme.accent))
        .cursor_pointer()
        .aria_toggled(if smooth_scrolling {
            Toggled::True
        } else {
            Toggled::False
        })
        .on_click(
            cx.listener(move |page, _, _, cx| page.set_smooth_scrolling(!smooth_scrolling, cx)),
        );

    vec![
        widgets::card_row(theme, true)
            .child(
                div()
                    .flex_1()
                    .min_w(px(160.0))
                    .child(widgets::row_title(
                        theme,
                        tr("appearance.motion.reduce", lang),
                    ))
                    .child(widgets::meta_line(
                        theme,
                        vec![
                            div()
                                .child(reduce_motion_helper(
                                    state.reduce,
                                    state.system_reduces,
                                    lang,
                                ))
                                .into_any_element(),
                        ],
                    )),
            )
            .child(reduce_control)
            .into_any_element(),
        widgets::card_row(theme, false)
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .child(widgets::row_title(
                        theme,
                        tr("appearance.motion.pause", lang),
                    ))
                    .child(widgets::meta_line(
                        theme,
                        vec![
                            div()
                                .child(tr("appearance.motion.pause.desc", lang))
                                .into_any_element(),
                        ],
                    )),
            )
            .child(pause_control)
            .into_any_element(),
        widgets::card_row(theme, false)
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .child(widgets::row_title(
                        theme,
                        tr("appearance.motion.smooth", lang),
                    ))
                    .child(widgets::meta_line(
                        theme,
                        vec![
                            div()
                                .child(smooth_scrolling_helper(
                                    smooth_scrolling,
                                    state.reduced_motion,
                                    lang,
                                ))
                                .into_any_element(),
                        ],
                    )),
            )
            .child(smooth_control)
            .into_any_element(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_reduce_motion_preference_has_a_helper() {
        for preference in ReduceMotion::ALL {
            for system in [true, false] {
                let helper = reduce_motion_helper(preference, system, "en");
                assert!(!helper.is_empty());
                // The two System states describe different system settings.
                if preference == ReduceMotion::System {
                    assert_ne!(
                        reduce_motion_helper(ReduceMotion::System, true, "en"),
                        reduce_motion_helper(ReduceMotion::System, false, "en")
                    );
                }
            }
        }
    }

    #[test]
    fn smooth_scrolling_helper_names_the_reduced_motion_case() {
        assert_ne!(
            smooth_scrolling_helper(true, true, "en"),
            smooth_scrolling_helper(true, false, "en")
        );
        assert_eq!(
            smooth_scrolling_helper(false, true, "en"),
            smooth_scrolling_helper(false, false, "en")
        );
    }
}
