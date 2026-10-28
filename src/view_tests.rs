//! View-level tests for the settings app: the existing surfaces (switch and
//! checkbox rows, collapsed-rail tooltips, notifications) and the beUI P1
//! composed surfaces (Channels mixed checkboxes, Digest radio group, Language
//! listbox, Time Zone combobox, About tabs, brand context menu).
//!
//! Two harnesses share this file:
//!
//! - `app_window` mounts the real `SettingsWindow` inside the real `Root`
//!   (dialog, sheet and notification layers present) and drives it through a
//!   `VisualTestContext`; `test_env::install` redirects `persist`'s config dir
//!   to a scratch location so row handlers cannot touch the owner's
//!   `~/.config`.
//! - `SettingsHarness` mounts the `Settings` pages alone (the app shell holds
//!   the window chrome) under `Root` — deferred popovers (select/combobox
//!   listboxes) need it — and drives them through `TestAppContext` +
//!   `update_window`.
//!
//! Named imports only: with `test-support` on, `use gpui_kit::*` would also
//! pull in the `test` attribute macro and shadow the built-in `#[test]`.

use gpui_kit::component::setting::{SelectIndex, Settings};
use gpui_kit::component::{Disableable as _, Root, WindowExt};
use gpui_kit::test::{TestAppContextExt as _, TestWindowExt as _};
use gpui_kit::{
    App, AppContext as _, Bounds, Context, ElementId, InputEvent as _, IntoElement, KeyDownEvent,
    KeyUpEvent, Keystroke, Modifiers, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
    ParentElement as _, Pixels, Point, Render, Styled as _, TestAppContext, VisualContext as _,
    VisualTestContext, Window, WindowBounds, WindowOptions, div, point, px, size,
};
use std::time::Duration;

use crate::theme::Appearance;
use crate::{AppSettings, DigestCadence, SettingsWindow, persist, test_env, theme};
use gpui_kit::component::ActiveTheme as _;

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

/// Bounds of one labelled glass-opacity slider's accessibility node.
///
/// The vendored `slider-accessibility-label` patch puts the name on the node
/// that already owns `Role::Slider`.
fn opacity_slider_bounds(window: &Window, label: &'static str) -> Option<Bounds<Pixels>> {
    gpui_kit::base::test_support::snapshots(window)
        .into_iter()
        .find(|s| s.role() == Some(gpui_kit::Role::Slider) && s.label() == Some(label))
        .map(|s| s.bounds())
}

/// Send one activation keystroke (down + up): the focused element's click
/// listeners fire on an unmodified Enter/Space key-up, so a bare `press`
/// (key-down only) cannot toggle a focused checkbox.
fn press_key(window: &mut Window, cx: &mut App, key: &str) {
    let keystroke = Keystroke::parse(key).expect("valid test keystroke");
    window.render_frame(cx);
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

/// Mount the real settings window inside `Root`, ready for interaction.
fn app_window(cx: &mut TestAppContext) -> &mut VisualTestContext {
    test_env::install();
    cx.update(|cx| {
        gpui_kit::init(cx);
        theme::install(
            cx,
            AppSettings::default().font_size,
            AppSettings::default().accent,
        );
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

/// Bounds of the Nth custom-nav row (`nav-page-N`) in the zeron sidebar.
fn nav_page_bounds(cx: &mut VisualTestContext, ix: usize) -> Bounds<Pixels> {
    let id = ElementId::Name(format!("nav-page-{ix}").into());
    cx.update(|window, _| {
        gpui_kit::base::test_support::snapshots(window)
            .into_iter()
            .filter(|s| s.path().last() == Some(&id))
            .map(|s| s.bounds())
            .next()
            .unwrap_or_else(|| panic!("nav page {id:?} must be registered"))
    })
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
                // Notifications, 2 Profile, 3 Finance, 4 Reset, 5 About.
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
    // The General page now carries brand + three glass rows; a 720px viewport
    // puts the Region/Startup rows below the fold, where the test renderer
    // does not register them.
    open_settings_sized(cx, page_ix, px(1_000.))
}

/// [`open_settings`] with an explicit window height.
///
/// A row that sits below the fold is not registered by the test renderer, so
/// tests that click observed descendants of a long page need a viewport tall
/// enough to hold the page they exercise.
fn open_settings_sized(
    cx: &mut TestAppContext,
    page_ix: usize,
    height: Pixels,
) -> gpui_kit::WindowHandle<Root> {
    // Row handlers persist through `persist::save`; keep them off the owner's
    // real config dir.
    test_env::install();
    cx.open_window(size(px(960.), height), move |window, cx| {
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
fn appearance_segmented_click_keyboard_a11y_and_reset(cx: &mut TestAppContext) {
    let cx = app_window(cx);

    // Three named segments, in the order the control draws them.
    cx.update(|window, _| {
        for (id, label) in [
            ("appearance-light", "Light"),
            ("appearance-dark", "Dark"),
            ("appearance-system", "System"),
        ] {
            assert_eq!(
                window.find(id).label(),
                Some(label),
                "{id} carries its segment name"
            );
        }
    });

    // Click Dark: the global records the choice and the theme follows it.
    cx.update(|window, cx| window.click("appearance-dark", cx));
    assert_eq!(
        cx.update(|_, cx| AppSettings::global(cx).appearance),
        Appearance::Dark
    );
    assert!(
        cx.update(|_, cx| cx.theme().mode.is_dark()),
        "picking Dark repaints the theme"
    );

    // Click Light: the other explicit segment.
    cx.update(|window, cx| window.click("appearance-light", cx));
    assert_eq!(
        cx.update(|_, cx| AppSettings::global(cx).appearance),
        Appearance::Light
    );
    assert!(!cx.update(|_, cx| cx.theme().mode.is_dark()));

    // Click System: the global holds the *choice*, not the resolved mode, so
    // the OS stays in charge and a later OS flip still moves the theme.
    cx.update(|window, cx| window.click("appearance-system", cx));
    assert_eq!(
        cx.update(|_, cx| AppSettings::global(cx).appearance),
        Appearance::System
    );

    // Page reset (General is resettable) restores the default choice.
    cx.update(|window, cx| {
        window.render_frame(cx);
        window.click("reset", cx);
    });
    assert_eq!(
        cx.update(|_, cx| AppSettings::global(cx).appearance),
        Appearance::default()
    );
}

/// The Reset page lists exactly the settings that differ from the defaults,
/// in a fixed order, and nothing else.
#[gpui_kit::test]
fn reset_panel_lists_only_what_changed(cx: &mut TestAppContext) {
    let cx = app_window(cx);

    // A fresh app differs from its own defaults nowhere.
    let empty = cx.update(|_, cx| crate::changed_settings(cx).len());
    assert_eq!(empty, 0, "nothing changed on a fresh app");

    // Move two settings, one of them back-to-front in the list order.
    cx.update(|_, cx| {
        AppSettings::global_mut(cx).font_size = 20.0;
        AppSettings::global_mut(cx).notifications_enabled = false;
    });
    let keys = cx.update(|_, cx| {
        crate::changed_settings(cx)
            .iter()
            .map(|each| each.key)
            .collect::<Vec<_>>()
    });
    assert_eq!(
        keys,
        vec!["item.font_size", "item.notifications"],
        "only the two moved settings, in panel order"
    );

    // Putting one back drops it from the list.
    cx.update(|_, cx| {
        AppSettings::global_mut(cx).font_size = AppSettings::default().font_size;
    });
    let keys = cx.update(|_, cx| {
        crate::changed_settings(cx)
            .iter()
            .map(|each| each.key)
            .collect::<Vec<_>>()
    });
    assert_eq!(keys, vec!["item.notifications"]);
}

/// The sidebar search matcher (`settings-search-terms.patch`), ported from
/// Ely's pinned cases: every whitespace-separated term must land somewhere in
/// the lowercased haystack — title, description and `keywords` joined — where
/// the vendored matcher used to require the whole query as one substring.
#[test]
fn search_matcher_requires_every_term() {
    use gpui_kit::component::setting::SettingItem;

    // The real haystacks `is_match` builds, spelled out.
    let font_size = "font size interface text size. text size zoom scale";
    let appearance =
        "appearance always light or dark, or follow the system. theme mode dark light system";

    // One word behaves exactly as the old whole-string `contains` did.
    assert!(SettingItem::query_matches(font_size, "size"));
    assert!(SettingItem::query_matches(font_size, "SIZE"));

    // Every term must land, across title and keywords.
    assert!(SettingItem::query_matches(font_size, "text zoom"));
    // A term belonging to a different item does not.
    assert!(!SettingItem::query_matches(font_size, "theme zoom"));
    // Order does not matter.
    assert!(SettingItem::query_matches(appearance, "dark appearance"));

    // An empty query matches everything here; `is_match` is what turns that
    // into "not searching" and keeps custom elements visible.
    assert!(SettingItem::query_matches(font_size, ""));
    assert!(SettingItem::query_matches(font_size, "   "));
}

/// The Font Size row's reset affordance is inert exactly at the default, so
/// it cannot offer a no-op and cannot go missing once the size moves.
#[test]
fn font_size_reset_is_inert_at_the_default_only() {
    let default = AppSettings::default().font_size;
    assert!(crate::font_size_at_default(default));
    assert!(!crate::font_size_at_default(default - 1.0));
    assert!(!crate::font_size_at_default(default + 1.0));
    // The clamp bounds are not the default, so reset stays available there.
    assert!(!crate::font_size_at_default(theme::FONT_SIZE_MIN));
    assert!(!crate::font_size_at_default(theme::FONT_SIZE_MAX));
}

/// `Theme::change` rebuilds the theme from the config, which resets
/// `font_size` to the config value. Every mode path must therefore run the
/// same funnel; if one forgets, a non-default font size silently snaps back
/// to 14 on the next mode flip.
#[gpui_kit::test]
fn mode_flip_keeps_a_non_default_font_size(cx: &mut TestAppContext) {
    let cx = app_window(cx);

    // A non-default size, written the way the Font Size row writes it.
    cx.update(|_, cx| {
        AppSettings::global_mut(cx).font_size = 20.0;
        crate::apply_font_size(cx);
    });
    assert_eq!(
        cx.update(|_, cx| cx.theme().font_size),
        gpui_kit::px(20.),
        "the size reaches the live theme"
    );

    // Each flip goes through the funnel and must leave the size alone.
    for appearance in [Appearance::Dark, Appearance::Light, Appearance::System] {
        cx.update(|_, cx| crate::set_appearance(appearance, cx));
        assert_eq!(
            cx.update(|_, cx| cx.theme().font_size),
            gpui_kit::px(20.),
            "a {appearance:?} flip must not reset the font size"
        );
    }

    // The size the app persists is the one the owner chose, not the config's.
    assert_eq!(cx.update(|_, cx| AppSettings::global(cx).font_size), 20.0);
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

/// The Background (glass) group is opt-in and gated: the master switch
/// persists the trio, the dependent rows are inert until it is on, and the
/// page reset returns everything to the opaque default.
#[gpui_kit::test]
fn glass_rows_gate_persist_and_reset(cx: &mut TestAppContext) {
    let cx = app_window(cx);

    cx.update(|window, _| {
        let master = window.find("background-translucent-switch");
        assert_eq!(master.role(), Some(gpui_kit::Role::Switch));
        assert_eq!(master.label(), Some("Translucent window"));
        assert_eq!(master.checked(), Some(false));
    });

    // Both glass sliders carry accessible names from the vendored
    // `slider-accessibility-label` patch; without it a screen reader
    // announces only "85, slider".
    cx.update(|window, _| {
        for label in ["Font Size", "Window opacity", "Sidebar tone"] {
            let named = gpui_kit::base::test_support::snapshots(window)
                .into_iter()
                .any(|s| s.role() == Some(gpui_kit::Role::Slider) && s.label() == Some(label));
            assert!(named, "the {label} slider must expose its accessible name");
        }
    });
    // Inert while translucency is off: a real click on the track must not
    // move the value (the disabled slider attaches no track listeners).
    let track = cx
        .update(|window, _| opacity_slider_bounds(window, "Window opacity"))
        .expect("window opacity slider bounds");
    let at = |fraction: f32| {
        point(
            track.origin.x + track.size.width * fraction,
            track.center().y,
        )
    };
    cx.simulate_click(at(0.2), Modifiers::default());
    assert_eq!(
        cx.update(|_, cx| AppSettings::global(cx).window_opacity),
        theme::WINDOW_OPACITY_DEFAULT,
        "a disabled opacity slider must ignore a track click"
    );

    // Blur is inert until the master switch is on.
    cx.update(|window, cx| window.click("background-blur-switch", cx));
    assert!(!cx.update(|_, cx| AppSettings::global(cx).background_blur));

    // Master on: the setting lands, the row mirrors it, and the persisted
    // mirror carries it (checked in memory — the shared test config dir is
    // written by other view tests in parallel, so reading the file here would
    // race; `persist::tests` covers the on-disk path).
    cx.update(|window, cx| window.click("background-translucent-switch", cx));
    assert!(cx.update(|_, cx| AppSettings::global(cx).background_translucent));
    cx.update(|window, _| {
        assert_eq!(
            window.find("background-translucent-switch").checked(),
            Some(true)
        );
    });
    let stored = cx.update(|_, cx| AppSettings::global(cx).to_stored());
    assert!(stored.background_translucent);
    assert_eq!(stored.window_opacity, theme::WINDOW_OPACITY_DEFAULT);
    assert_eq!(stored.sidebar_tone, theme::SIDEBAR_TONE_DEFAULT);
    assert!(!stored.background_blur);
    assert_eq!(persist::parse(&persist::serialize(&stored)), Some(stored));

    // With a translucent window the blur row becomes interactive.
    cx.update(|window, cx| window.click("background-blur-switch", cx));
    assert!(cx.update(|_, cx| AppSettings::global(cx).background_blur));
    let stored = cx.update(|_, cx| AppSettings::global(cx).to_stored());
    assert!(stored.background_blur);

    // A real track click drives the widget's own events: mouse-down reports
    // through `Change` (40–100 mapped over the track, so 50% ≈ 70), then
    // mouse-up through `Release` persists that value — not the handler called
    // directly.
    // The enabled page can shift (flush layout vs card), so render once and
    // re-measure the track instead of reusing the disabled-state bounds.
    cx.update(|window, cx| window.render_frame(cx));
    let track = cx
        .update(|window, _| opacity_slider_bounds(window, "Window opacity"))
        .expect("enabled window opacity slider bounds");
    // `at` closed over the disabled-state bounds; the enabled layout moved,
    // so address the freshly measured track explicitly.
    let at_live = |fraction: f32| {
        point(
            track.origin.x + track.size.width * fraction,
            track.center().y,
        )
    };
    cx.simulate_mouse_down(at_live(0.5), MouseButton::Left, Modifiers::default());
    let live = cx.update(|_, cx| AppSettings::global(cx).window_opacity);
    assert!(
        (65.0..=75.0).contains(&live),
        "a click at 50% of the 40–100 track should land near 70, got {live}"
    );
    cx.simulate_mouse_up(at_live(0.5), MouseButton::Left, Modifiers::default());
    assert_eq!(
        cx.update(|_, cx| AppSettings::global(cx).to_stored().window_opacity),
        live,
        "releasing the slider must persist the value Change reported"
    );

    // The General page reset puts the whole group back to the opaque
    // defaults, and the row mirrors it again.
    cx.update(|window, cx| window.click("reset", cx));
    cx.update(|_, cx| {
        let settings = AppSettings::global(cx);
        assert!(!settings.background_translucent);
        assert_eq!(settings.window_opacity, theme::WINDOW_OPACITY_DEFAULT);
        assert_eq!(settings.sidebar_tone, theme::SIDEBAR_TONE_DEFAULT);
        assert!(!settings.background_blur);
    });
    let stored = cx.update(|_, cx| AppSettings::global(cx).to_stored());
    assert!(!stored.background_translucent);
    assert!(!stored.background_blur);
    // And the dependent row is inert again after the reset.
    cx.update(|window, cx| window.click("background-blur-switch", cx));
    assert!(!cx.update(|_, cx| AppSettings::global(cx).background_blur));
}

/// Zeron rail behaviour: collapse targets the 1px sliver (never a 48px icon
/// rail), the rail content unmounts with it (no invisible rows over the page
/// or in the tab order), reopen restores the persisted width, and dragging
/// the seam writes the width live and persists it on release within the
/// 242–400 clamp.
#[gpui_kit::test]
fn sidebar_collapses_to_sliver_and_resizes(cx: &mut TestAppContext) {
    let cx = app_window(cx);

    cx.update(|_, _| {
        assert_eq!(crate::sidebar_target_width(true, px(300.)), px(1.));
        assert_eq!(crate::sidebar_target_width(false, px(300.)), px(300.));
    });

    // Collapse from the titlebar toggle: the rail target becomes the sliver
    // and its content leaves the tree — the collapsed rail is a bare grab
    // strip, not a 48px icon rail painted over the page (the vendored
    // `Sidebar` icon-collapse mode), and not a stack of invisible controls
    // still reachable by keyboard.
    cx.update(|window, cx| window.click("sidebar-collapse-toggle", cx));
    assert!(cx.update(|_, cx| cx.global::<crate::shell::ShellState>().collapsed));
    cx.update(|window, cx| window.render_frame(cx));
    cx.update(|window, _| {
        assert!(
            window.try_find("nav-page-0").is_none(),
            "the collapsed rail must not keep its nav rows mounted"
        );
        assert!(
            window.try_find("nav-footer-theme").is_none(),
            "the collapsed rail must not keep its footer identity mounted"
        );
        assert!(
            window.try_find("appearance-light").is_some(),
            "collapsing the rail must not unmount the page"
        );
    });
    cx.update(|window, cx| window.click("sidebar-collapse-toggle", cx));
    assert!(!cx.update(|_, cx| cx.global::<crate::shell::ShellState>().collapsed));
    cx.update(|window, cx| window.render_frame(cx));
    cx.update(|window, _| {
        assert!(
            window.try_find("nav-page-0").is_some(),
            "reopening the rail must restore the nav tree"
        );
    });

    // Drag the seam: down on the rail edge, move right, release.
    let rail = cx.update(|_, cx| AppSettings::global(cx).sidebar_width());
    let at = |x: Pixels| point(x, px(500.));
    cx.simulate_mouse_down(at(rail), MouseButton::Left, Modifiers::default());
    // The capture overlay mounts on the next frame and owns the moves; a move
    // that changes the width proves it is active.
    cx.update(|window, cx| window.render_frame(cx));
    cx.simulate_mouse_move(at(px(320.)), MouseButton::Left, Modifiers::default());
    assert_eq!(
        cx.update(|_, cx| AppSettings::global(cx).sidebar_width),
        320.0
    );
    // Clamp at both limits while dragging.
    cx.simulate_mouse_move(at(px(600.)), MouseButton::Left, Modifiers::default());
    assert_eq!(
        cx.update(|_, cx| AppSettings::global(cx).sidebar_width),
        theme::SIDEBAR_WIDTH_MAX
    );
    cx.simulate_mouse_move(at(px(100.)), MouseButton::Left, Modifiers::default());
    assert_eq!(
        cx.update(|_, cx| AppSettings::global(cx).sidebar_width),
        theme::SIDEBAR_WIDTH_MIN
    );
    cx.simulate_mouse_up(at(px(100.)), MouseButton::Left, Modifiers::default());
    cx.update(|window, cx| window.render_frame(cx));
    assert_eq!(
        cx.update(|_, cx| AppSettings::global(cx).to_stored().sidebar_width),
        theme::SIDEBAR_WIDTH_MIN,
        "releasing the seam persists the clamped width"
    );
    // The capture overlay is gone: a later move with the button no longer
    // resizes, so a lost release cannot leave the window swallowed.
    cx.simulate_mouse_move(at(px(500.)), MouseButton::Left, Modifiers::default());
    assert_eq!(
        cx.update(|_, cx| AppSettings::global(cx).sidebar_width),
        theme::SIDEBAR_WIDTH_MIN,
        "the drag must not survive its release"
    );
}

/// zeron layout invariants, in BOTH appearances: no inset cards (flush page
/// padded below the bar), a transparent-ish rail column that always carries
/// the ink tone + right hairline, and that tone painted even with
/// translucency off (zeron's `wash(0.05)` is unconditional).
#[gpui_kit::test]
fn layout_matches_zeron_shell_in_both_modes(cx: &mut TestAppContext) {
    let cx = app_window(cx);
    cx.update(|_, cx| {
        let top = |style: &gpui_kit::StyleRefinement| style.padding.top.map(|top| top.to_string());
        let width = |edge: Option<gpui_kit::AbsoluteLength>| {
            edge.map(|edge| edge.to_string()).unwrap_or_default()
        };
        for glass in [true, false] {
            let content = crate::content_panel_style(cx, glass);
            assert!(
                content.background.is_none(),
                "zeron has no inset cards (glass={glass})"
            );
            assert!(
                content.margin.left.is_none(),
                "content is flush, no 6px inset (glass={glass})"
            );
            assert_eq!(top(&content), Some("38px".into()));
        }
        let rail = crate::sidebar_refinement(cx, true);
        assert!(rail.background.is_some(), "rail paints the ink tone");
        assert_eq!(width(rail.border_widths.right), "1px");
        assert_eq!(width(rail.border_widths.left), "0px");
        assert_eq!(crate::sidebar_header_padding(), px(38.));
        let rail_opaque = crate::sidebar_refinement(cx, false);
        assert!(
            rail_opaque.background.is_some(),
            "the tone is painted in opaque mode too (zeron wash)"
        );
        assert_eq!(width(rail_opaque.border_widths.right), "1px");

        // Default app settings: translucency is off, yet the tone still
        // paints — only the frost shell disappears.
        assert!(!AppSettings::global(cx).background_translucent);
        assert!(
            crate::sidebar_tone_bg(cx).a > 0.0,
            "the sidebar tone must not be gated by the translucency master switch"
        );
    });
}

/// The glass rows reach the window's vendored `Root`: enabling the master
/// switch switches Root's opaque token paint off, disabling restores it. The
/// headless platform's `set_background_appearance` is a no-op, so the
/// appearance decision itself is covered by
/// `window_appearance_follows_the_glass_switches`.
#[gpui_kit::test]
fn glass_switch_toggles_the_root_base_paint(cx: &mut TestAppContext) {
    test_env::install();
    cx.update(|cx| {
        gpui_kit::init(cx);
        theme::install(
            cx,
            AppSettings::default().font_size,
            AppSettings::default().accent,
        );
        cx.set_global(AppSettings::default());
    });
    let handle = cx.update(|cx| {
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::default(),
                    size: size(px(960.), px(720.)),
                })),
                ..Default::default()
            },
            |window, cx| {
                let view = cx.new(|cx| SettingsWindow::new(window, cx));
                let root = cx.new(|cx| Root::new(view, window, cx));
                // Mirror `main()`: the rows reach the Root through this
                // global, which is what `apply_glass` rewrites.
                cx.set_global(crate::WindowRoot(root.clone()));
                root
            },
        )
        .expect("test window")
    });
    let root_has_override = |cx: &mut App| {
        let root = cx
            .try_global::<crate::WindowRoot>()
            .map(|root| root.0.clone())
            .expect("the window must publish its Root exactly like main()");
        root.update(cx, |root, _| root.style().background.is_some())
    };

    cx.update_window(handle.into(), |_, window, cx| {
        window.activate_window();
        window.render_frame(cx);
        window.click("background-translucent-switch", cx);
    })
    .unwrap();
    assert!(cx.update(|cx| AppSettings::global(cx).background_translucent));
    assert!(
        cx.update(|cx| root_has_override(cx)),
        "glass must switch Root's opaque base paint off"
    );

    // The opacity slider's effect: painted live, persisted only on release,
    // and clamped at the app boundary.
    cx.update_window(handle.into(), |_, window, cx| {
        crate::set_glass_percent(crate::GlassControl::WindowOpacity, 55.0, true, window, cx);
    })
    .unwrap();
    assert_eq!(cx.update(|cx| AppSettings::global(cx).window_opacity), 55.0);
    assert_eq!(
        cx.update(|cx| AppSettings::global(cx).to_stored().window_opacity),
        55.0
    );
    cx.update_window(handle.into(), |_, window, cx| {
        crate::set_glass_percent(crate::GlassControl::WindowOpacity, 1.0, false, window, cx);
    })
    .unwrap();
    assert_eq!(
        cx.update(|cx| AppSettings::global(cx).window_opacity),
        theme::WINDOW_OPACITY_MIN
    );

    // A theme-mode flip rebuilds the theme from `mono.json`; it must not drop
    // the glass state, which lives on the window surface and Root style, not
    // in the palette.
    cx.update_window(handle.into(), |_, window, cx| {
        window.click("appearance-dark", cx);
    })
    .unwrap();
    assert_eq!(
        cx.update(|cx| AppSettings::global(cx).appearance),
        Appearance::Dark
    );
    assert!(cx.update(|cx| AppSettings::global(cx).background_translucent));
    assert!(
        cx.update(|cx| root_has_override(cx)),
        "a mode flip must keep Root's glass override"
    );
    // ...and the painted frost alpha itself survives: the shell still
    // resolves to the clamped 40% colour after the theme rebuild.
    cx.update(|cx| {
        let alpha = crate::shell_frost_bg(cx).a;
        let expected = (theme::WINDOW_OPACITY_MIN / 100.0) as f32;
        assert!(
            (alpha - expected).abs() < 1e-4,
            "frost alpha after the mode flip is {alpha}, expected {expected}"
        );
    });

    // Turning the master switch off restores Root's token paint...
    cx.update_window(handle.into(), |_, window, cx| {
        window.click("background-translucent-switch", cx);
    })
    .unwrap();
    assert!(!cx.update(|cx| AppSettings::global(cx).background_translucent));
    assert!(
        !cx.update(|cx| root_has_override(cx)),
        "opaque mode must restore Root's token paint"
    );

    // ...and "reset all" must too, including the window surface.
    cx.update_window(handle.into(), |_, window, cx| {
        window.click("background-translucent-switch", cx);
        crate::reset_everything(window, cx);
    })
    .unwrap();
    assert!(!cx.update(|cx| AppSettings::global(cx).background_translucent));
    assert_eq!(
        cx.update(|cx| AppSettings::global(cx).window_opacity),
        AppSettings::default().window_opacity
    );
    assert!(
        !cx.update(|cx| root_has_override(cx)),
        "reset-all must restore Root's token paint"
    );
}

/// The notifications-enabled switch is a raw vendored control inside an
/// element field; it now carries the row title as its accessible name.
#[gpui_kit::test]
fn notifications_switch_is_labelled_and_controlled(cx: &mut TestAppContext) {
    let cx = app_window(cx);

    // Notifications is the second page in the sidebar.
    let point = nav_page_bounds(cx, 1).center();
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
/// still lands instantly with no thumb travel. The Appearance row is a
/// segmented control now, but the switch spring still runs on the other rows.
#[gpui_kit::test]
fn switch_toggle_lands_under_reduce_motion(cx: &mut TestAppContext) {
    let cx = app_window(cx);
    cx.update(|_, cx| cx.set_reduce_motion(true));

    // The Notifications row still carries a switch; under reduced motion its
    // thumb reports the terminal value instead of travelling. The Appearance
    // row is a segmented control now, so the switch under test lives on the
    // second page.
    let point = nav_page_bounds(cx, 1).center();
    cx.simulate_click(point, Modifiers::default());
    cx.update(|window, cx| window.render_frame(cx));

    cx.update(|window, cx| window.click("notifications-enabled", cx));
    assert!(!cx.update(|_, cx| AppSettings::global(cx).notifications_enabled));
    cx.update(|window, _| {
        assert_eq!(window.find("notifications-enabled").checked(), Some(false));
    });
}

/// The zeron sidebar nav: section disclosure, the header filter cycle, page
/// switching through the custom rows, and the footer theme toggle.
#[gpui_kit::test]
fn zeron_nav_sections_filter_and_page_switch(cx: &mut TestAppContext) {
    let cx = app_window(cx);
    let present = |cx: &mut VisualTestContext, id: &'static str| {
        cx.update(|window, _| window.try_find(id).is_some())
    };

    // Both sections start open and every page row is reachable.
    assert!(present(cx, "nav-page-0"), "General row must exist");
    assert!(present(cx, "nav-page-4"), "Reset row must exist");

    // Collapsing Workspace hides its rows; reopening restores them.
    cx.update(|window, cx| window.click("nav-section-workspace", cx));
    assert!(!present(cx, "nav-page-4"), "collapsed section hides rows");
    assert!(present(cx, "nav-page-1"), "other section stays open");
    cx.update(|window, cx| window.click("nav-section-workspace", cx));
    assert!(present(cx, "nav-page-4"), "reopening restores rows");

    // The header filter cycles All -> Preferences -> Workspace -> All.
    cx.update(|window, cx| window.click("nav-filter", cx));
    assert!(
        !present(cx, "nav-page-4"),
        "Preferences filter hides Workspace"
    );
    assert!(
        present(cx, "nav-page-1"),
        "Preferences filter keeps its rows"
    );
    cx.update(|window, cx| window.click("nav-filter", cx));
    assert!(
        present(cx, "nav-page-4"),
        "Workspace filter shows Workspace"
    );
    assert!(
        !present(cx, "nav-page-0"),
        "Workspace filter hides Preferences"
    );
    cx.update(|window, cx| window.click("nav-filter", cx));
    assert!(present(cx, "nav-page-0"), "third click returns to All");

    // A custom row drives the content pane.
    cx.update(|window, cx| window.click("nav-page-1", cx));
    assert!(
        present(cx, "notifications-enabled"),
        "selecting the Notifications row must render that page"
    );

    // Footer identity row carries a working theme toggle.
    let before = cx.update(|_, cx| AppSettings::global(cx).appearance);
    cx.update(|window, cx| window.click("nav-footer-theme", cx));
    let after = cx.update(|_, cx| AppSettings::global(cx).appearance);
    assert_ne!(before, after, "footer theme toggle must flip appearance");
}

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
    let item = nav_page_bounds(cx, 1).center();
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
        press_key(window, cx, "space");
        assert!(!AppSettings::global(cx).channel_digest);
        press_key(window, cx, "enter");
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
        press_key(window, cx, "space");
        press_key(window, cx, "enter");
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
    // The General page is taller than the default 720px viewport now that it
    // carries the Background group; the Time Zone row has to be on screen for
    // its observed bounds to exist.
    let handle = open_settings_sized(cx, 0, px(1_000.));
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
    let handle = open_settings(cx, 5);
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
    with_settings(cx, 5, |window, cx| {
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
    let handle = open_settings(cx, 5);
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

/// The reset panel must list *every* setting that differs from its default —
/// otherwise a user who changed only a channel, a digest cadence or a
/// timezone sees "Nothing has changed" and the Reset all button stays
/// disabled, so the change cannot be reverted from the UI at all.
#[gpui_kit::test]
fn reset_panel_covers_the_channel_digest_and_timezone_settings(cx: &mut TestAppContext) {
    let cx = app_window(cx);
    assert_eq!(cx.update(|_, cx| crate::changed_settings(cx).len()), 0);

    // Move each structurally-distinct field the panel used to ignore.
    cx.update(|_, cx| {
        AppSettings::global_mut(cx).channel_email = false;
        AppSettings::global_mut(cx).channel_desktop = false;
        AppSettings::global_mut(cx).channel_digest = false;
    });
    let keys = cx.update(|_, cx| {
        crate::changed_settings(cx)
            .iter()
            .map(|each| each.key)
            .collect::<Vec<_>>()
    });
    assert!(
        keys.contains(&"item.channels"),
        "channels changed but the panel lists {keys:?}"
    );

    // Reset, then move only the timezone.
    cx.update(|_, cx| {
        *AppSettings::global_mut(cx) = AppSettings::default();
        AppSettings::global_mut(cx).timezone = "Asia/Tokyo".into();
    });
    let keys = cx.update(|_, cx| {
        crate::changed_settings(cx)
            .iter()
            .map(|each| each.key)
            .collect::<Vec<_>>()
    });
    assert_eq!(
        keys,
        vec!["item.timezone"],
        "a timezone change must be listed and revertable"
    );
}

/// The sidebar collapse toggle is icon-only, so its accessible name is all a
/// screen reader has. It must sit on the `Button` itself: a plain `div`
/// wrapper has no a11y role and is filtered out of the tree entirely, so
/// naming the wrapper announces nothing.
#[gpui_kit::test]
fn sidebar_collapse_toggle_is_named(cx: &mut TestAppContext) {
    let cx = app_window(cx);
    cx.update(|window, cx| window.render_frame(cx));
    cx.update(|window, _| {
        let toggles: Vec<_> = gpui_kit::base::test_support::snapshots(window)
            .into_iter()
            .filter(|s| format!("{:?}", s.path()).contains("sidebar-collapse-toggle"))
            .collect();
        assert_eq!(toggles.len(), 1, "expected exactly one toggle snapshot");
        assert_eq!(toggles[0].label(), Some("Toggle sidebar"));
        assert_eq!(toggles[0].role(), Some(gpui_kit::Role::Button));
    });
}
