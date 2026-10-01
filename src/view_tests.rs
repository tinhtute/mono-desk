//! View-level tests for the existing settings surfaces — phase
//! `beui-p1-existing-surfaces` of `docs/plans/active/beui-p1-components.md`.
//!
//! The real `SettingsWindow` is mounted inside the real `Root` (dialog, sheet
//! and notification layers present) and driven through clicks, keyboard input
//! and the test clock. `test_env::install` redirects `persist`'s config dir to
//! a scratch location so row handlers cannot touch the owner's `~/.config`.
//!
//! Named imports only: with `test-support` on, `use gpui_kit::*` would also
//! pull in the `test` attribute macro and shadow the built-in `#[test]`.

use gpui_kit::component::{Disableable as _, Root, WindowExt};
use gpui_kit::test::{TestAppContextExt as _, TestWindowExt};
use gpui_kit::{
    App, AppContext as _, Bounds, Context, ElementId, InputEvent as _, IntoElement, KeyDownEvent,
    KeyUpEvent, Keystroke, Modifiers, ParentElement as _, Pixels, Render, TestAppContext,
    VisualContext as _, VisualTestContext, Window, div, point, px,
};

use std::time::Duration;

use crate::shell::ShellState;
use crate::{AppSettings, SettingsWindow, persist, test_env, theme};

/// Mount the real settings window inside `Root`, ready for interaction.
fn app_window(cx: &mut TestAppContext) -> &mut VisualTestContext {
    test_env::install();
    cx.update(|cx| {
        gpui_kit::init(cx);
        theme::install(cx, AppSettings::default().font_size);
        cx.set_global(AppSettings::default());
    });
    let (_, cx) = cx.add_window_view(|window, cx| {
        Root::new(cx.new(|cx| SettingsWindow::new(window, cx)), window, cx)
    });
    cx.update(|window, cx| {
        // Vendored tests activate the headless window; the toast advance task
        // only samples and ages entries while the window is live.
        window.activate_window();
        window.render_frame(cx);
    });
    cx
}

/// Send one activation keystroke (down + up): the focused element's click
/// listeners fire on an unmodified Enter/Space key-up.
fn press_key(window: &mut Window, cx: &mut App, key: &str) {
    let keystroke = Keystroke::parse(key).expect("valid test keystroke");
    window.dispatch_event(
        KeyDownEvent {
            keystroke: keystroke.clone(),
            is_held: false,
            prefer_character_input: false,
        }
        .to_platform_input(),
        cx,
    );
    window.dispatch_event(KeyUpEvent { keystroke }.to_platform_input(), cx);
    window.render_frame(cx);
}

/// Tab forward until the registered element `id` is focused.
fn tab_to(cx: &mut VisualTestContext, id: &'static str) {
    cx.update(|window, cx| {
        for _ in 0..48 {
            if window.find(id).focused() == Some(true) {
                return;
            }
            window.focus_next(cx);
            window.render_frame(cx);
        }
        panic!("{id} never received focus");
    });
}

/// Advance the test clock, run the shared scheduler to completion (the toast
/// advance task + timers both live on it), then repaint the window.
fn advance(cx: &mut VisualTestContext, by: Duration) {
    cx.update(|_, cx| cx.background_executor().advance_clock(by));
    cx.run_until_parked();
    cx.update(|window, cx| window.render_frame(cx));
}

/// Bounds of the Nth top-level sidebar page item (`"0-N"`).
///
/// Child (group) rows reuse the parent's element id inside the submenu, so
/// `window.find("0-N")` is ambiguous; the page item is the registered
/// candidate with the shortest path.
fn sidebar_item_bounds(cx: &mut VisualTestContext, ix: usize) -> Bounds<Pixels> {
    let id = ElementId::from(format!("0-{ix}"));
    cx.update(|window, _| {
        gpui_kit::base::test_support::snapshots(window)
            .into_iter()
            .filter(|s| s.path().last() == Some(&id))
            .min_by_key(|s| s.path().len())
            .map(|s| s.bounds())
            .unwrap_or_else(|| panic!("sidebar item {id:?} must be registered"))
    })
}

/// The Dark Mode switch on the General page: role + label + toggled state,
/// click toggles once and writes the global, Tab + Space/Enter toggles, and
/// the page reset restores the default through `on_reset`.
#[gpui_kit::test]
fn dark_mode_switch_click_keyboard_a11y_and_reset(cx: &mut TestAppContext) {
    let cx = app_window(cx);

    cx.update(|window, _| {
        let switch = window.find("dark-mode-switch");
        assert_eq!(switch.role(), Some(gpui_kit::Role::Switch));
        assert_eq!(switch.label(), Some("Dark Mode"));
        assert_eq!(switch.checked(), Some(false));
    });

    // Click toggles once and writes the global.
    cx.update(|window, cx| window.click("dark-mode-switch", cx));
    assert!(cx.update(|_, cx| AppSettings::global(cx).dark_mode));
    cx.update(|window, _| {
        assert_eq!(window.find("dark-mode-switch").checked(), Some(true));
    });

    // Tab focus + Space toggles back; Enter toggles again.
    tab_to(cx, "dark-mode-switch");
    cx.update(|window, cx| press_key(window, cx, "space"));
    assert!(!cx.update(|_, cx| AppSettings::global(cx).dark_mode));
    cx.update(|window, cx| press_key(window, cx, "enter"));
    assert!(cx.update(|_, cx| AppSettings::global(cx).dark_mode));

    // Page reset (General is resettable) restores the default.
    cx.update(|window, cx| {
        window.render_frame(cx);
        window.click("reset", cx);
    });
    assert!(!cx.update(|_, cx| AppSettings::global(cx).dark_mode));
    cx.update(|window, _| {
        assert_eq!(window.find("dark-mode-switch").checked(), Some(false));
    });
}

/// Same contract for the Launch at login checkbox, plus the autostart side
/// effect converging under the redirected config dir.
#[gpui_kit::test]
fn launch_at_login_checkbox_click_keyboard_a11y_and_reset(cx: &mut TestAppContext) {
    let cx = app_window(cx);

    cx.update(|window, _| {
        let checkbox = window.find("launch-at-login-checkbox");
        assert_eq!(checkbox.role(), Some(gpui_kit::Role::CheckBox));
        assert_eq!(checkbox.label(), Some("Launch at Login"));
        assert_eq!(checkbox.checked(), Some(false));
    });

    cx.update(|window, cx| window.click("launch-at-login-checkbox", cx));
    assert!(cx.update(|_, cx| AppSettings::global(cx).launch_at_login));
    if let Some(entry) = persist::autostart_path() {
        assert!(
            entry.exists(),
            "autostart converge must write the managed entry under the test config dir"
        );
    }

    tab_to(cx, "launch-at-login-checkbox");
    cx.update(|window, cx| press_key(window, cx, "space"));
    assert!(!cx.update(|_, cx| AppSettings::global(cx).launch_at_login));
    cx.update(|window, cx| press_key(window, cx, "enter"));
    assert!(cx.update(|_, cx| AppSettings::global(cx).launch_at_login));

    cx.update(|window, cx| window.click("reset", cx));
    assert!(!cx.update(|_, cx| AppSettings::global(cx).launch_at_login));
    cx.update(|window, _| {
        assert_eq!(
            window.find("launch-at-login-checkbox").checked(),
            Some(false)
        );
    });
}

/// The notifications-enabled switch is a raw vendored control inside an
/// element field; it now carries the row title as its accessible name.
#[gpui_kit::test]
fn notifications_switch_is_labelled_and_controlled(cx: &mut TestAppContext) {
    let cx = app_window(cx);

    // Notifications is the second page in the sidebar.
    let point = sidebar_item_bounds(cx, 1).center();
    cx.simulate_click(point, Modifiers::default());
    cx.update(|window, cx| window.render_frame(cx));

    cx.update(|window, _| {
        let switch = window.find("notifications-enabled");
        assert_eq!(switch.role(), Some(gpui_kit::Role::Switch));
        assert_eq!(switch.label(), Some("Enable Notifications"));
        assert_eq!(switch.checked(), Some(true));
    });

    cx.update(|window, cx| window.click("notifications-enabled", cx));
    assert!(!cx.update(|_, cx| AppSettings::global(cx).notifications_enabled));
    cx.update(|window, _| {
        assert_eq!(window.find("notifications-enabled").checked(), Some(false));
    });
}

/// A disabled control is inert: no toggle on click, skipped by Tab.
#[gpui_kit::test]
fn disabled_controls_are_inert(cx: &mut TestAppContext) {
    use gpui_kit::component::checkbox::Checkbox;
    use gpui_kit::component::switch::Switch;

    struct DisabledRow;
    impl Render for DisabledRow {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .child(
                    Switch::new("inert-switch")
                        .checked(false)
                        .disabled(true)
                        .on_click(|_, _, _| panic!("disabled switch must not toggle")),
                )
                .child(
                    Checkbox::new("inert-checkbox")
                        .checked(false)
                        .disabled(true)
                        .on_click(|_, _, _| panic!("disabled checkbox must not toggle")),
                )
        }
    }

    test_env::install();
    cx.update(gpui_kit::init);
    let (_, cx) = cx.add_window_view(|_, _| DisabledRow);
    cx.update(|window, cx| window.render_frame(cx));

    for id in ["inert-switch", "inert-checkbox"] {
        cx.update(|window, cx| window.click(id, cx));
        cx.update(|window, _| {
            assert_eq!(window.find(id).checked(), Some(false), "{id} toggled");
        });
    }

    // Neither disabled control is a tab stop.
    cx.update(|window, cx| {
        for _ in 0..8 {
            window.focus_next(cx);
            window.render_frame(cx);
            for id in ["inert-switch", "inert-checkbox"] {
                assert_ne!(window.find(id).focused(), Some(true));
            }
        }
    });
}

/// Reduced motion: the switch spring reports its terminal value, so a toggle
/// still lands instantly with no thumb travel.
#[gpui_kit::test]
fn switch_toggle_lands_under_reduce_motion(cx: &mut TestAppContext) {
    let cx = app_window(cx);
    cx.update(|_, cx| cx.set_reduce_motion(true));

    cx.update(|window, cx| window.click("dark-mode-switch", cx));
    assert!(cx.update(|_, cx| AppSettings::global(cx).dark_mode));
    cx.update(|window, _| {
        assert_eq!(window.find("dark-mode-switch").checked(), Some(true));
    });
}

/// Collapsed-rail tooltip wiring.
///
/// The icon-only page item's `managed_tooltip_at(Placement::Right, …)` trigger
/// runs the shared overlay lifecycle (500 ms show delay, 300 ms hide grace)
/// end-to-end. The popup itself is not observable headlessly — the vendored
/// `Tooltip` registers no `test_support` hitbox and `Root::tooltip_overlay`
/// is crate-private — so this exercises hover → delay → leave → grace without
/// asserting the popup's presence; the visual check stays with the owner
/// (plan `beui-p1-components` risk R6).
#[gpui_kit::test]
fn collapsed_rail_item_drives_tooltip_lifecycle(cx: &mut TestAppContext) {
    let cx = app_window(cx);

    // Collapse the rail to icon mode (the width spring lands instantly under
    // reduced motion, so the item is narrow on the next frame).
    cx.update(|_, cx| {
        cx.global_mut::<ShellState>().collapsed = true;
        cx.set_reduce_motion(true);
        cx.refresh_windows();
    });
    cx.update(|window, cx| window.render_frame(cx));

    let bounds = sidebar_item_bounds(cx, 1);
    assert!(
        bounds.size.width <= SIDEBAR_ICON_ITEM_MAX_W,
        "collapsed item must be icon-width, got {bounds:?}"
    );
    let item_center = bounds.center();

    // Hover enters the trigger; the show task fires after the 500 ms delay.
    cx.simulate_mouse_move(item_center, None, Modifiers::default());
    advance(cx, Duration::from_millis(600));

    // Leaving schedules the hide; the grace period clears it.
    cx.simulate_mouse_move(point(px(600.), item_center.y), None, Modifiers::default());
    advance(cx, Duration::from_millis(600));
}

/// Collapsed rail items render icon-only inside a 48px rail.
const SIDEBAR_ICON_ITEM_MAX_W: Pixels = px(48.);

/// Notifications → Send test notification drives the real push → dismiss →
/// autohide lifecycle on the window's notification layer.
///
/// The toast advance task is a foreground `spawn_in` future; only
/// `TestAppContextExt::wait_for` keeps the test scheduler polling it while
/// the clock moves (`run_until_parked` alone leaves the toast mounted).
#[gpui_kit::test]
async fn test_notification_pushes_closes_and_autohides(cx: &mut TestAppContext) {
    let cx = app_window(cx);
    let window = cx.window_handle();

    // Open the Notifications page and send a test notification.
    let item = sidebar_item_bounds(cx, 1).center();
    cx.simulate_click(item, Modifiers::default());
    cx.update(|window, cx| {
        window.render_frame(cx);
        window.click("send-test-notification", cx);
        window.render_frame(cx);
        assert!(window.try_find("notification").is_some());
        assert_eq!(window.notifications(cx).len(), 1);
    });

    // Dismissal: `Notification::dismiss` is the call the close button's
    // listener makes. The button's hitbox is hover-gated
    // (`invisible().group_hover`) inside the overlay layer and not reachable
    // by the headless click-target check, so the test drives the entity.
    cx.update(|window, cx| {
        let note = window.notifications(cx)[0].clone();
        note.update(cx, |note, cx| note.dismiss(window, cx));
    });
    cx.wait_for(window, Duration::from_secs(2), |window, cx| {
        window.notifications(cx).is_empty()
    })
    .await;
    cx.update(|window, _| {
        assert!(window.try_find("notification").is_none());
    });

    // Autohide: a fresh push mounts, then the 5 s timeout ages it out. The
    // 10 s wait bound is what proves the timeout fired on schedule.
    cx.update(|window, cx| {
        window.click("send-test-notification", cx);
        window.render_frame(cx);
        assert!(window.try_find("notification").is_some());
    });
    cx.wait_for(window, Duration::from_secs(10), |window, cx| {
        window.notifications(cx).is_empty()
    })
    .await;
    cx.update(|window, _| {
        assert!(window.try_find("notification").is_none());
    });
}
