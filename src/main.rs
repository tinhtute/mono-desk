//! mono-desk — a settings-style desktop demo.
//!
//! UI stack: zeron's GPUI fork (`github.com/zeronsh/zui`, pinned rev). The
//! window chrome, sidebar/settings layout and widget styling are ported from
//! zeron's MIT-licensed source (`github.com/zeronsh/zeron`) into
//! `vendor/zeron-ui/` and adapted to this settings-only app; see
//! `docs/plans/active/zeron-stack-port.md` and `THIRD_PARTY_NOTICES.md`.
//!
//! Phase 1 (stack bring-up): a client-side-decorated window with zeron's
//! options — 900×600 minimum, 1320×880 default, transparent titlebar, traffic
//! lights at (14, 14), app-owned titlebar drag — and a placeholder root view.
//! Theme/chrome/settings land in the following phases.

use gpui::{
    App, AppContext as _, Bounds, IntoElement, ParentElement as _, Render, Styled as _, Window,
    WindowBackgroundAppearance, WindowBounds, WindowOptions, div, hsla, point, px, size,
};

/// Root view for the bring-up phase; replaced by the zeron shell in Phase 3.
struct SettingsApp;

impl Render for SettingsApp {
    fn render(&mut self, _window: &mut Window, _cx: &mut gpui::Context<Self>) -> impl IntoElement {
        // Placeholder base until the zeron theme/frost port lands (Phase 2).
        div()
            .size_full()
            .bg(hsla(0.0, 0.0, 0.968, 1.0))
            .flex()
            .items_center()
            .justify_center()
            .text_color(hsla(0.0, 0.0, 0.45, 1.0))
            .child("mono-desk — zeron stack bring-up")
    }
}

/// zeron's window options (`crates/ui/src/lib.rs` `open_main_window`): a
/// frameless-inset macOS chrome with custom-drawn titlebar, native traffic
/// lights at (14, 14), and app-owned titlebar dragging.
fn window_options() -> WindowOptions {
    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(Bounds::new(
            point(px(120.), px(120.)),
            size(px(1320.), px(880.)),
        ))),
        window_min_size: Some(size(px(900.), px(600.))),
        titlebar: Some(gpui::TitlebarOptions {
            title: None,
            appears_transparent: true,
            traffic_light_position: Some(point(px(14.), px(14.))),
        }),
        // The shell's own 38px strip drags the window (WindowControlArea::Drag
        // + start_window_move), so AppKit must treat the content view as
        // app-owned.
        app_owns_titlebar_drag: true,
        window_background: WindowBackgroundAppearance::Opaque,
        app_id: Some("mono-desk".into()),
        ..Default::default()
    }
}

fn main() {
    gpui_platform::application().run(|cx: &mut App| {
        cx.open_window(window_options(), |window, cx| {
            window.set_rem_size(px(16.));
            cx.new(|_| SettingsApp)
        })
        .expect("failed to open window");
    });
}
