// (the tests below depend on these ids — keep them stable)

//! View-level tests for the beUI P1 composed surfaces: the Channels mixed
//! checkboxes, the Digest radio group, the Language listbox, the Time Zone
//! combobox, the About tabs and the brand context menu.
//!
//! These render the real `Settings` tree (not `SettingsWindow`: the app shell
//! holds the window chrome, while the pages alone cover the new controls)
//! under `Root` — deferred popovers (select/combobox listboxes) need it — in a
//! headless window, exercising pointer/keyboard flows against `AppSettings`,
//! the global the field closures read and write.

use super::{AppSettings, DigestCadence, SettingsWindow};
use gpui_kit::component::Root;
use gpui_kit::component::setting::{SelectIndex, Settings};
use gpui_kit::test::{TestAppContextExt as _, TestWindowExt as _};
use gpui_kit::{
    App, AppContext as _, Bounds, Context, ElementId, InputEvent as _, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, ParentElement as _, Pixels, Point, Render, Styled as _,
    TestAppContext, Window, div, px, size,
};
use std::time::Duration;

/// The bounds of the first visible observed element anywhere below `ancestor`
/// in its element path — for controls like the vendored `Combobox` that never
/// register their own id, this reaches the trigger through a descendant.
fn bounds_below(window: &Window, ancestor: &'static str) -> Bounds<Pixels> {
    let needle: ElementId = ancestor.into();
    gpui_kit::base::test_support::snapshots(window)
        .into_iter()
        .find(|s| s.visible() && s.path().contains(&needle))
        .map(|s| s.bounds())
        .unwrap_or_else(|| panic!("no observed element below {ancestor}"))
}

/// Dispatches a full mouse click at a window position — the same primitives
/// `TestWindowExt::click` composes — for targets with no observed id.
fn click_at_position(
    window: &mut Window,
    button: MouseButton,
    position: Point<Pixels>,
    cx: &mut App,
) {
    window.dispatch_event(
        MouseMoveEvent {
            position,
            pressed_button: None,
            modifiers: Default::default(),
        }
        .to_platform_input(),
        cx,
    );
    window.dispatch_event(
        MouseDownEvent {
            button,
            position,
            modifiers: Default::default(),
            click_count: 1,
            first_mouse: false,
        }
        .to_platform_input(),
        cx,
    );
    // A frame between down and up lets deferred work (e.g. a popover being
    // built and focused) land before the release, matching the vendored
    // helper's per-event render cadence.
    window.render_frame(cx);
    window.dispatch_event(
        MouseUpEvent {
            button,
            position,
            modifiers: Default::default(),
            click_count: 1,
        }
        .to_platform_input(),
        cx,
    );
    window.render_frame(cx);
}

/// Dispatches a full key press (down + up): the div-based activation path
/// maps Enter/Space to a click on the key-up, so a bare `press` (key-down
/// only) cannot toggle a focused checkbox.
fn press_key(window: &mut Window, key: &str, cx: &mut App) {
    let keystroke = gpui_kit::Keystroke::parse(key).expect("a valid test keystroke");
    window.render_frame(cx);
    window.dispatch_event(
        gpui_kit::KeyDownEvent {
            keystroke: keystroke.clone(),
            is_held: false,
            prefer_character_input: false,
        }
        .to_platform_input(),
        cx,
    );
    window.dispatch_event(gpui_kit::KeyUpEvent { keystroke }.to_platform_input(), cx);
    window.render_frame(cx);
}

/// Down+up in one burst, without an intervening frame: for targets that get
/// blurred on mouse-down and would lose their hitbox before the release
/// (e.g. the combobox row when the search field is focused).
fn down_up_at_position(
    window: &mut Window,
    button: MouseButton,
    position: Point<Pixels>,
    cx: &mut App,
) {
    window.dispatch_event(
        MouseDownEvent {
            button,
            position,
            modifiers: Default::default(),
            click_count: 1,
            first_mouse: false,
        }
        .to_platform_input(),
        cx,
    );
    window.dispatch_event(
        MouseUpEvent {
            button,
            position,
            modifiers: Default::default(),
            click_count: 1,
        }
        .to_platform_input(),
        cx,
    );
}
/// The `Settings` element with the real pages mounted, under `Root`.
struct SettingsHarness {
    page_ix: usize,
}

impl Render for SettingsHarness {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl gpui_kit::IntoElement {
        div().size_full().child(
            Settings::new("test-settings")
                // Page order mirrors `setting_pages`: 0 General, 1
                // Notifications, 2 Profile, 3 Finance, 4 About.
                .default_selected_index(SelectIndex {
                    page_ix: self.page_ix,
                    group_ix: None,
                })
                .pages(SettingsWindow::setting_pages(cx)),
        )
    }
}

/// Mounts the harness (under `Root`), settles a frame, runs `f` inside the
/// window. Returns the handle so async tests can `wait_for` afterwards.
fn open_settings(cx: &mut TestAppContext, page_ix: usize) -> gpui_kit::WindowHandle<Root> {
    cx.open_window(size(px(960.), px(720.)), move |window, cx| {
        let view = cx.new(|_| SettingsHarness { page_ix });
        Root::new(view, window, cx)
    })
}

fn with_settings<F>(cx: &mut TestAppContext, page_ix: usize, f: F)
where
    F: FnOnce(&mut Window, &mut App),
{
    let handle = open_settings(cx, page_ix);
    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        f(window, cx)
    })
    .unwrap();
}

#[gpui_kit::test]
fn channels_parent_reports_mixed_state(cx: &mut TestAppContext) {
    cx.update(gpui_kit::init);
    // AppSettings::default() enables email + desktop but not digest, so the
    // composed parent starts indeterminate.
    cx.update(|cx| cx.set_global(AppSettings::default()));
    with_settings(cx, 1, |window, cx| {
        let parent = window.find("channels-all");
        assert_eq!(parent.indeterminate(), Some(true));
        assert_eq!(parent.checked(), Some(false));
        assert_eq!(window.find("channel-email").checked(), Some(true));
        assert_eq!(window.find("channel-desktop").checked(), Some(true));
        assert_eq!(window.find("channel-digest").checked(), Some(false));

        // Parent click clears the mixed state by selecting everything.
        window.click("channels-all", cx);
        assert_eq!(window.find("channels-all").checked(), Some(true));
        assert_eq!(window.find("channel-digest").checked(), Some(true));
        assert!(AppSettings::global(cx).channel_digest);

        // Unchecking one child returns the parent to mixed.
        window.click("channel-email", cx);
        assert_eq!(window.find("channels-all").indeterminate(), Some(true));
        assert_eq!(window.find("channel-email").checked(), Some(false));

        // Parent click from mixed selects all again; clicking once more
        // unchecks all — full toggle semantics from the composed control.
        window.click("channels-all", cx);
        window.click("channels-all", cx);
        assert_eq!(window.find("channels-all").checked(), Some(false));
        let settings = AppSettings::global(cx);
        assert!(!settings.channel_email && !settings.channel_desktop && !settings.channel_digest);
    });
}

#[gpui_kit::test]
fn channels_disable_with_notifications(cx: &mut TestAppContext) {
    cx.update(gpui_kit::init);
    cx.update(|cx| {
        cx.set_global(AppSettings {
            notifications_enabled: false,
            ..Default::default()
        });
    });
    with_settings(cx, 1, |window, cx| {
        // gpui-base snapshots don't surface `disabled` as a fact, so the
        // behavioral proof is that clicks leave the global untouched.
        window.click("channels-all", cx);
        window.click("channel-digest", cx);
        let settings = AppSettings::global(cx);
        assert!(settings.channel_email && !settings.channel_digest);
    });
}

#[gpui_kit::test]
fn digest_disable_with_notifications(cx: &mut TestAppContext) {
    cx.update(gpui_kit::init);
    cx.update(|cx| {
        cx.set_global(AppSettings {
            notifications_enabled: false,
            ..Default::default()
        });
    });
    with_settings(cx, 1, |window, cx| {
        // The Digest group is gated on notifications like the Channels group:
        // clicks leave the global untouched.
        window.click("digest-weekly", cx);
        assert_eq!(AppSettings::global(cx).digest, DigestCadence::Immediate);
    });
}

#[gpui_kit::test]
fn channels_keyboard_and_reduced_motion(cx: &mut TestAppContext) {
    cx.update(gpui_kit::init);
    cx.update(|cx| cx.set_global(AppSettings::default()));
    with_settings(cx, 1, |window, cx| {
        // Clicking focuses the composed control; Space and Enter then toggle
        // it from the keyboard (base Checkbox activation).
        window.click("channels-all", cx);
        assert_eq!(window.find("channels-all").focused(), Some(true));
        assert!(AppSettings::global(cx).channel_digest);
        press_key(window, "space", cx);
        assert!(!AppSettings::global(cx).channel_digest);
        press_key(window, "enter", cx);
        assert!(AppSettings::global(cx).channel_digest);

        // Reduced motion: the mark spring resolves immediately; the toggle
        // still lands and writes the global.
        cx.set_reduce_motion(true);
        window.click("channel-email", cx);
        assert!(!AppSettings::global(cx).channel_email);
        cx.set_reduce_motion(false);
    });
}

#[gpui_kit::test]
fn digest_radio_keyboard_and_reduced_motion(cx: &mut TestAppContext) {
    cx.update(gpui_kit::init);
    cx.update(|cx| cx.set_global(AppSettings::default()));
    with_settings(cx, 1, |window, cx| {
        // Clicking focuses the option; Space/Enter on the selected option are
        // inert (a radio cannot be unchecked by activation).
        window.click("digest-daily", cx);
        assert_eq!(window.find("digest-daily").focused(), Some(true));
        assert_eq!(window.find("digest-daily").selected(), Some(true));
        assert_eq!(AppSettings::global(cx).digest, DigestCadence::Daily);
        press_key(window, "space", cx);
        press_key(window, "enter", cx);
        assert_eq!(AppSettings::global(cx).digest, DigestCadence::Daily);

        // Reduced motion: the dot spring resolves immediately; selection
        // still lands.
        cx.set_reduce_motion(true);
        window.click("digest-weekly", cx);
        assert_eq!(AppSettings::global(cx).digest, DigestCadence::Weekly);
        cx.set_reduce_motion(false);
    });
}

#[gpui_kit::test]
fn digest_radio_selects_and_arrows_move(cx: &mut TestAppContext) {
    cx.update(gpui_kit::init);
    cx.update(|cx| cx.set_global(AppSettings::default()));
    with_settings(cx, 1, |window, cx| {
        assert_eq!(window.find("digest-immediate").checked(), Some(true));
        assert_eq!(window.find("digest-weekly").checked(), Some(false));

        // Click selects and persists through the global.
        window.click("digest-weekly", cx);
        assert_eq!(window.find("digest-weekly").checked(), Some(true));
        assert_eq!(AppSettings::global(cx).digest, DigestCadence::Weekly);

        // Arrow keys move focus and selection (beUI radio-group semantics).
        window.press("right", cx);
        assert_eq!(AppSettings::global(cx).digest, DigestCadence::Immediate);
        window.press("down", cx);
        assert_eq!(AppSettings::global(cx).digest, DigestCadence::Daily);
        assert_eq!(window.find("digest-daily").checked(), Some(true));
    });
}

#[gpui_kit::test]
async fn language_select_commits_via_keyboard(cx: &mut TestAppContext) {
    cx.update(gpui_kit::init);
    cx.update(|cx| cx.set_global(AppSettings::default()));
    let handle = open_settings(cx, 0);
    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        let select = window.find("language-select-control");
        assert_eq!(select.expanded(), Some(false));
        // Focus the trigger and open the listbox (same dispatch path as the
        // vendored select test: click the inner "input" hitbox).
        window.within("language-select-control").click("input", cx);
        assert_eq!(
            window.find("language-select-control").expanded(),
            Some(true)
        );
        window.press("down", cx);
        window.press("enter", cx);
    })
    .unwrap();
    // Select commits through defer_in after the dispatch returns.
    cx.wait_for(handle.into(), Duration::from_millis(100), |_, cx| {
        AppSettings::global(cx).language == "vi"
    })
    .await;
}

#[gpui_kit::test]
async fn language_select_escape_cancels(cx: &mut TestAppContext) {
    cx.update(gpui_kit::init);
    cx.update(|cx| cx.set_global(AppSettings::default()));
    let handle = open_settings(cx, 0);
    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        window.within("language-select-control").click("input", cx);
        assert_eq!(
            window.find("language-select-control").expanded(),
            Some(true)
        );
        // Escape closes the listbox without committing the cursor row.
        window.press("down", cx);
        window.press("escape", cx);
        assert_eq!(
            window.find("language-select-control").expanded(),
            Some(false)
        );
    })
    .unwrap();
    cx.run_until_parked();
    let language = cx.update(|cx| AppSettings::global(cx).language.clone());
    assert_eq!(language, "en");
}

#[gpui_kit::test]
async fn timezone_combobox_searches_and_commits(cx: &mut TestAppContext) {
    cx.update(gpui_kit::init);
    cx.update(|cx| cx.set_global(AppSettings::default()));
    let handle = open_settings(cx, 0);
    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        // The vendored Combobox never registers its own id, so the trigger
        // click goes through the observed descendant's bounds.
        click_at_position(
            window,
            MouseButton::Left,
            bounds_below(window, "timezone-combo").center(),
            cx,
        );
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        // Filter to the single row. The query lands in the popup's search
        // input; the Change events flush when this update returns, so the
        // filter is applied by the time the next frame renders.
        window.input("tokyo", cx);
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        let rows: Vec<_> = gpui_kit::base::test_support::snapshots(window)
            .into_iter()
            .filter(|s| s.path().last().is_some_and(|id| matches!(id, ElementId::NamedInteger(n, _) if n.as_ref() == "searchable-list-item")))
            .collect();
        let row = rows.into_iter().next()
            .map(|s| s.bounds())
            .expect("combobox row not observed after filtering");
        // Mouse-down on the row blurs the search input, which closes the
        // popup and removes the row's hitbox — the click only survives if the
        // release lands before any re-render can tear the hitbox down.
        down_up_at_position(window, MouseButton::Left, row.center(), cx);
    })
    .unwrap();
    // The pick commits through `defer_in` — park to flush the queued callback.
    cx.run_until_parked();
    cx.wait_for(handle.into(), Duration::from_millis(200), |_, cx| {
        AppSettings::global(cx).timezone == "Asia/Tokyo"
    })
    .await;
}

#[gpui_kit::test]
fn about_context_menu_escape_dismisses(cx: &mut TestAppContext) {
    cx.update(gpui_kit::init);
    cx.update(|cx| cx.set_global(AppSettings::default()));
    let handle = open_settings(cx, 4);
    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        click_at_position(
            window,
            MouseButton::Right,
            bounds_below(window, "about-brand-card").center(),
            cx,
        );
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        // Escape dismisses the menu; a following Enter has no item to
        // activate, so nothing reaches the clipboard.
        window.press("escape", cx);
        window.press("enter", cx);
    })
    .unwrap();
    cx.run_until_parked();
    assert!(cx.read_from_clipboard().is_none());
    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        window.remove_window();
    })
    .unwrap();
    cx.run_until_parked();
}

#[gpui_kit::test]
fn about_tabs_swap_panels(cx: &mut TestAppContext) {
    cx.update(gpui_kit::init);
    cx.update(|cx| cx.set_global(AppSettings::default()));
    with_settings(cx, 4, |window, cx| {
        assert_eq!(
            window.within("about-tabs").find(0usize).selected(),
            Some(true)
        );
        window.within("about-tabs").click(1usize, cx);
        assert_eq!(
            window.within("about-tabs").find(1usize).selected(),
            Some(true)
        );
        assert_eq!(
            window.within("about-tabs").find(0usize).selected(),
            Some(false)
        );
        window.within("about-tabs").click(2usize, cx);
        assert_eq!(
            window.within("about-tabs").find(2usize).selected(),
            Some(true)
        );
    });
}

#[gpui_kit::test]
fn about_context_menu_copies_version(cx: &mut TestAppContext) {
    cx.update(gpui_kit::init);
    cx.update(|cx| cx.set_global(AppSettings::default()));
    let handle = open_settings(cx, 4);
    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        // The card is a scope segment (the ContextMenu's hitbox), not an
        // observed leaf — dispatch the right-click at a descendant's center.
        click_at_position(
            window,
            MouseButton::Right,
            bounds_below(window, "about-brand-card").center(),
            cx,
        );
    })
    .unwrap();
    // The menu view is built in a deferred callback — settle, then it is
    // rendered and focused (vendored context-menu test does the same).
    cx.run_until_parked();
    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        // Down+Enter activates the focused menu's first item ("Copy version").
        window.press("down", cx);
        window.press("enter", cx);
    })
    .unwrap();
    cx.run_until_parked();
    let item = cx.read_from_clipboard().expect("clipboard is empty");
    assert_eq!(item.text(), Some(env!("CARGO_PKG_VERSION").to_string()));
    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        // The ContextMenu's mouse listener retains the PopupMenu entity for
        // the window's lifetime even after unmount; remove the window so its
        // listeners and element states drop before the leak check.
        window.remove_window();
    })
    .unwrap();
    cx.run_until_parked();
}
