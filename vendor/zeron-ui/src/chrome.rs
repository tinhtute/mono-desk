//! Window chrome geometry and titlebar controls, ported from zeron's
//! `crates/ui/src/shell.rs` (MIT, Copyright 2026 Wing).
//!
//! These are the parts of zeron's shell that are pure presentation — theme and
//! numbers in, element out — so they can live in the vendored crate and be
//! unit-tested next to their upstream tests. Anything that needs app state
//! (the drag strip's "button is down" flag) is composed by the app shell in
//! `src/shell.rs` on top of these helpers.
//!
//! Ported verbatim: `TITLEBAR_CONTROL_GAP`, `TITLEBAR_GROUP_GAP`,
//! `TITLEBAR_IDENTITY_GAP`, `TITLEBAR_ACTION_EDGE_INSET`, `CLUSTER_BUTTONS_WIDTH`,
//! `TITLEBAR_ACTION_SLOT_WIDTH`, `TITLEBAR_CLUSTER_PAD`,
//! `titlebar_cluster_start`, `titlebar_spacer_width`, `caption_buttons_width`,
//! `cluster_buttons_start`, `cluster_clearance`, `titlebar_right_padding`,
//! `window_control_button`, `window_control_button_with` and their tests.
//!
//! Adaptations from upstream:
//! - the buttons' tooltip is dropped until zeron's
//!   `settings::widgets::text_tooltip` lands (`settings-surface`); the button
//!   keeps its accessibility label through [`gpui::InteractiveElement::aria_label`];
//! - the Windows/Linux caption buttons themselves are not ported (NG3, macOS
//!   first) while the geometry that reserves their space is, so the numbers
//!   stay comparable to upstream;
//! - [`titlebar_spacer`] is static: upstream tweens the fullscreen inset with
//!   its `WidthTween` clock, which is not part of this port yet.
//!
//! This app's only surface is zeron's *Settings* route, which paints no
//! control cluster (see the port plan, R2), so nothing here renders a
//! `titlebar_spacer`/button by default; the helpers exist because the page
//! header clearance and any future cluster need the same documented numbers.

use gpui::prelude::*;
use gpui::{App, ClickEvent, Div, IntoElement, MouseButton, Window, div, px};

use crate::icons;
use crate::motion;
use crate::theme::Theme;

/// Where a titlebar control cluster starts, from the window's left edge.
/// zeron `window-controls.tsx`: `left: fullscreen ? 12 : 88` — the cluster
/// clears the native macOS traffic lights at {14,15} and reclaims the inset
/// when fullscreen hides them.
pub fn titlebar_cluster_start(fullscreen: bool) -> f32 {
    if fullscreen { 12.0 } else { 88.0 }
}

/// Width of the spacer ahead of the control cluster for a strip that already
/// carries `container_pad` px of its own left padding. macOS only — on
/// Linux/Windows there are no traffic lights and the cluster hugs the edge.
pub fn titlebar_spacer_width(is_macos: bool, fullscreen: bool, container_pad: f32) -> f32 {
    if !is_macos {
        return 0.0;
    }
    (titlebar_cluster_start(fullscreen) - container_pad).max(0.0)
}

/// Within-group rhythm for Back/Forward.
pub const TITLEBAR_CONTROL_GAP: f32 = 2.0;
/// Structural separation between titlebar groups: sidebar, navigation,
/// transcript identity, and trailing actions.
pub const TITLEBAR_GROUP_GAP: f32 = Theme::SPACE_SM;
/// Breathing room between the navigation cluster and transcript identity.
pub const TITLEBAR_IDENTITY_GAP: f32 = Theme::SPACE_MD;
/// A 28px action centered in the 38px titlebar with its 2px downward optical
/// shift lands 6px from the top; use the same inset at the trailing edge.
pub const TITLEBAR_ACTION_EDGE_INSET: f32 = 6.0;
/// Width of the persistent top-left button cluster itself: a 24px sidebar
/// trigger, an 8px group gap, then two 24px history buttons on a 2px rhythm.
pub const CLUSTER_BUTTONS_WIDTH: f32 = 24.0 * 3.0 + TITLEBAR_GROUP_GAP + TITLEBAR_CONTROL_GAP;
/// Extra width consumed when a second action group joins the left controls.
pub const TITLEBAR_ACTION_SLOT_WIDTH: f32 = TITLEBAR_GROUP_GAP + 24.0;
/// Horizontal inset owned by the titlebar control row itself. Keep this value
/// paired with [`titlebar_spacer`]: using a different number for the spacer
/// shifts every control while leaving the declared cluster geometry unchanged.
pub const TITLEBAR_CLUSTER_PAD: f32 = 10.0;

const WINDOWS_CAPTION_BUTTON_WIDTH: f32 = 36.0;
const WINDOWS_CAPTION_WIDTH: f32 = WINDOWS_CAPTION_BUTTON_WIDTH * 3.0;

/// The animated spacer clearing the macOS traffic lights ahead of a titlebar
/// control cluster (zeron `Shell::titlebar_spacer`). `None` off macOS — no
/// phantom flex child — and `None` when the strip's own padding already clears
/// the cluster start.
pub fn titlebar_spacer(is_macos: bool, fullscreen: bool, container_pad: f32) -> Option<Div> {
    let width = titlebar_spacer_width(is_macos, fullscreen, container_pad);
    (width > 0.0).then(|| div().flex_none().h_full().w(px(width)))
}

/// Width of a row of `count` caption buttons, drawn at the cluster's own
/// 24px-button / 2px-gap rhythm.
pub fn caption_buttons_width(count: usize) -> f32 {
    if count == 0 {
        return 0.0;
    }
    count as f32 * 24.0 + (count as f32 - 1.0) * 2.0
}

/// Where the cluster's first button starts, from the window's left edge.
/// `linux_left_captions` is the number of caption buttons zeron draws at the
/// top-left on Linux (GNOME `close:…` layouts) — the app cluster follows them
/// at the shared 2px rhythm.
pub fn cluster_buttons_start(is_macos: bool, fullscreen: bool, linux_left_captions: usize) -> f32 {
    if is_macos {
        titlebar_cluster_start(fullscreen)
    } else if linux_left_captions > 0 {
        10.0 + caption_buttons_width(linux_left_captions) + 2.0
    } else {
        10.0
    }
}

/// Left clearance a full-bleed header (no cluster row of its own) needs so its
/// content starts past the overlay cluster, given the header's own
/// `container_pad`.
pub fn cluster_clearance(
    is_macos: bool,
    fullscreen: bool,
    linux_left_captions: usize,
    container_pad: f32,
) -> f32 {
    (cluster_buttons_start(is_macos, fullscreen, linux_left_captions) + CLUSTER_BUTTONS_WIDTH + 8.0
        - container_pad)
        .max(0.0)
}

/// Right padding titlebar content needs to clear the platform's caption
/// controls (native Windows cluster / zeron-drawn Linux buttons).
pub fn titlebar_right_padding(is_windows: bool, linux_right_captions: usize, base: f32) -> f32 {
    base + if is_windows {
        WINDOWS_CAPTION_WIDTH
    } else if linux_right_captions > 0 {
        10.0 + caption_buttons_width(linux_right_captions)
    } else {
        0.0
    }
}

/// A size-6 icon button for the titlebar strip (zeron `window-controls.tsx`:
/// `grid size-6 place-items-center rounded-md text-muted-foreground`).
pub fn window_control_button(
    id: &'static str,
    icon_path: &'static str,
    label: &'static str,
    theme: &Theme,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let glyph = icons::icon(icon_path)
        .size(px(16.0))
        .text_color(theme.text_muted);
    window_control_button_with(id, glyph, label, theme, on_click)
}

/// [`window_control_button`] around a caller-drawn glyph (zeron's morphing
/// sidebar glyph).
pub fn window_control_button_with(
    id: &'static str,
    glyph: impl IntoElement,
    label: &'static str,
    theme: &Theme,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let fade_key = format!("window-control-{id}");
    div()
        .id(id)
        .debug_selector(move || id.to_owned())
        .role(gpui::Role::Button)
        .aria_label(label)
        .size(px(24.0))
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(6.0))
        .cursor_pointer()
        // zeron window-controls.tsx: `transition-colors` — the wash fades.
        .bg(motion::hover_blend(
            &fade_key,
            theme.glass_hover().opacity(0.0),
            theme.glass_hover(),
        ))
        .on_hover(motion::hover_listener(fade_key))
        // Buttons in/over a titlebar drag strip must be EXCLUDED from the
        // strip's event surface entirely: `occlude()` makes the window
        // hit-test stop at the button, so the strip's drag listeners never
        // fire with the pointer over a button (and the native Drag control
        // area loses the button's rect).
        .occlude()
        .on_mouse_down(MouseButton::Left, |_, window, _| window.prevent_default())
        .on_click(move |event, window, cx| {
            cx.stop_propagation();
            on_click(event, window, cx)
        })
        .child(glyph)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn titlebar_cluster_matches_zeron_window_controls() {
        // zeron window-controls.tsx: `left: fullscreen ? 12 : 88` — the
        // cluster clears the {14,15} traffic lights, and reclaims the inset
        // when fullscreen hides them.
        assert_eq!(titlebar_cluster_start(false), 88.0);
        assert_eq!(titlebar_cluster_start(true), 12.0);
        assert_eq!(TITLEBAR_CONTROL_GAP, 2.0);
        assert_eq!(TITLEBAR_GROUP_GAP, Theme::SPACE_SM);
        assert_eq!(TITLEBAR_IDENTITY_GAP, Theme::SPACE_MD);
        assert_eq!(CLUSTER_BUTTONS_WIDTH, 82.0);
        assert_eq!(TITLEBAR_ACTION_SLOT_WIDTH, 32.0);
        assert_eq!(TITLEBAR_ACTION_EDGE_INSET, 6.0);
    }

    #[test]
    fn titlebar_spacer_selects_per_platform_and_fullscreen() {
        // macOS, lights visible: spacer fills up to the 88px cluster start.
        assert_eq!(titlebar_spacer_width(true, false, 10.0), 78.0);
        assert_eq!(titlebar_spacer_width(true, false, 12.0), 76.0);
        assert_eq!(titlebar_spacer_width(true, false, 26.0), 62.0);
        // macOS fullscreen: the inset animates away (clamped at zero when the
        // strip's own padding already exceeds the 12px cluster start).
        assert_eq!(titlebar_spacer_width(true, true, 10.0), 2.0);
        assert_eq!(titlebar_spacer_width(true, true, 26.0), 0.0);
        // Linux / Windows: never any inset.
        assert_eq!(titlebar_spacer_width(false, false, 10.0), 0.0);
        assert_eq!(titlebar_spacer_width(false, true, 10.0), 0.0);
        assert_eq!(
            TITLEBAR_CLUSTER_PAD + titlebar_spacer_width(true, false, TITLEBAR_CLUSTER_PAD),
            titlebar_cluster_start(false),
            "the rendered row padding and spacer must land on the declared cluster start"
        );
    }

    #[test]
    fn windows_caption_controls_reserve_titlebar_space() {
        assert_eq!(titlebar_right_padding(true, 0, 16.0), 124.0);
        assert_eq!(titlebar_right_padding(false, 0, 16.0), 16.0);
    }

    #[test]
    fn linux_caption_controls_reserve_titlebar_space() {
        // 24px buttons on the cluster's 2px rhythm.
        assert_eq!(caption_buttons_width(0), 0.0);
        assert_eq!(caption_buttons_width(1), 24.0);
        assert_eq!(caption_buttons_width(3), 76.0);
        // Right-side captions (the Linux default: minimize,maximize,close):
        // content pads past the 10px edge inset + the button row.
        assert_eq!(titlebar_right_padding(false, 3, 16.0), 16.0 + 10.0 + 76.0);
        // GNOME-vanilla ":close" — a single right button.
        assert_eq!(titlebar_right_padding(false, 1, 16.0), 16.0 + 10.0 + 24.0);
        // Left-side captions ("close:…" layouts) shift the app cluster right
        // by the button row + one 2px gap.
        assert_eq!(cluster_buttons_start(false, false, 0), 10.0);
        assert_eq!(cluster_buttons_start(false, false, 1), 10.0 + 24.0 + 2.0);
        assert_eq!(cluster_buttons_start(false, false, 3), 10.0 + 76.0 + 2.0);
        // macOS ignores the Linux caption count entirely.
        assert_eq!(cluster_buttons_start(true, false, 3), 88.0);
    }

    #[test]
    fn cluster_clearance_clears_the_overlay_buttons() {
        // Linux: buttons at 10..92; a 16px-padded header needs 84 more px to
        // put content at 92 + 8 breathing room.
        assert_eq!(cluster_clearance(false, false, 0, 16.0), 84.0);
        assert_eq!(cluster_clearance(false, false, 0, 10.0), 90.0);
        // Linux with a left-side close caption: everything shifts one slot.
        assert_eq!(cluster_clearance(false, false, 1, 16.0), 84.0 + 26.0);
        // macOS: buttons start at the 88px traffic-light cluster start.
        assert_eq!(
            cluster_clearance(true, false, 0, 16.0),
            88.0 + CLUSTER_BUTTONS_WIDTH + 8.0 - 16.0
        );
        // macOS fullscreen: cluster reclaims the inset (starts at 12).
        assert_eq!(
            cluster_clearance(true, true, 0, 16.0),
            12.0 + CLUSTER_BUTTONS_WIDTH + 8.0 - 16.0
        );
    }
}
