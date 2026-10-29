//! The app shell: zeron's Settings surface and its window chrome.
//!
//! Ported from zeron's `crates/ui/src/shell.rs` (MIT, Copyright 2026 Wing).
//! The app has a single surface — zeron's *Settings* route — so this is that
//! route's shell, not the chat shell: the root paints the theme's glass tint
//! over the blurred window background (`Shell::render`'s `div().bg(frost)`
//! where `frost = theme.glass()`), the sidebar column stands on zeron's
//! `sidebar_tone` layer (`wash(0.05)` + 1px right hairline), and a 38px
//! overlay strip drags the window (`titlebar_drag_region`) without painting a
//! control cluster — zeron's settings route has none. The chat-only chrome
//! (cluster at 88px, sidebar resize/collapse, centered title) is deliberately
//! not ported; the plan's R2 records that decision.
//!
//! Phase `chrome` T1 (this task): the shell surface, the drag strip, the
//! sidebar tone/column and the page pane's titlebar clearance. T2 adds the
//! section list and the footer identity row.

use gpui::prelude::*;
use gpui::{Context, Div, Hsla, MouseButton, Render, Stateful, Window, WindowControlArea, div, px};

use zeron_ui::theme::{self, Theme};
use zeron_ui::typography;

/// zeron's default sidebar width (`settings.rs` `SIDEBAR_DEFAULT`). zeron's
/// settings route reads the persisted chat-sidebar width; mono-desk has no
/// resizable sidebar (settings route has no drag handle), so the column is
/// fixed at the default. Bounds for reference: `SIDEBAR_MIN` 242,
/// `SIDEBAR_MAX` 400.
pub const SIDEBAR_WIDTH: f32 = 256.0;

pub struct Shell {
    /// Armed by a left press on the titlebar strip: the next pointer move with
    /// the button down hands the drag to the compositor (zeron's
    /// `titlebar_should_move`).
    titlebar_should_move: bool,
}

impl Shell {
    pub fn new() -> Self {
        Self {
            titlebar_should_move: false,
        }
    }

    /// zeron `Shell::sidebar_tone`: the absolute full-height column that paints
    /// the shell's 5% ink wash under the sidebar, with a 1px hairline on its
    /// inner edge. Upstream rounds/two-tone-trims this layer only for Linux
    /// client-side-decorated corners (NG3), so the macOS port paints it square
    /// — the window server rounds the window itself.
    fn sidebar_tone(width: f32, border_color: Hsla) -> Div {
        div()
            .absolute()
            .top_0()
            .bottom_0()
            .left_0()
            .w(px(width))
            .bg(theme::wash(0.05))
            .border_r_1()
            .border_color(border_color)
    }

    /// Make a titlebar strip drag the window — zed's platform-titlebar pattern
    /// (zeron's `.drag` region): mark it a [`WindowControlArea::Drag`] (macOS
    /// app-owned titlebar), hand the drag to the compositor once the pointer
    /// moves with the button down, and double-click zooms.
    fn titlebar_drag_region(
        &self,
        id: &'static str,
        el: Div,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        el.id(id)
            .window_control_area(WindowControlArea::Drag)
            .on_mouse_down_out(cx.listener(|this, _, _, _| this.titlebar_should_move = false))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _, _, _| this.titlebar_should_move = false),
            )
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, _, _| this.titlebar_should_move = true),
            )
            // Hand the drag to the compositor only while the button is
            // actually held (`pressed_button` guard): on macOS
            // `start_window_move` runs AppKit's NATIVE drag session
            // (`performWindowDragWithEvent:`), and AppKit resolves a quick
            // second click inside that session as a titlebar double-click —
            // system zoom — natively, beyond gpui's reach.
            .on_mouse_move(
                cx.listener(|this, event: &gpui::MouseMoveEvent, window, _| {
                    if this.titlebar_should_move && event.pressed_button == Some(MouseButton::Left)
                    {
                        this.titlebar_should_move = false;
                        window.start_window_move();
                    }
                }),
            )
            .on_click(|event, window, _| {
                if event.click_count() == 2 {
                    if cfg!(target_os = "macos") {
                        // Native titlebar double-click action (zoom/minimize
                        // per system preference).
                        window.titlebar_double_click();
                    } else {
                        window.zoom_window();
                    }
                }
            })
    }

    /// The settings sidebar column (zeron `render_settings_page`): full height
    /// at the fixed width, with its content starting below the titlebar strip.
    /// T2 fills it with the section list and the footer identity row.
    fn render_sidebar(&self, theme: &Theme) -> impl IntoElement {
        div()
            .w(px(SIDEBAR_WIDTH))
            .h_full()
            .flex_none()
            .flex()
            .flex_col()
            .pt(px(Theme::TITLEBAR_HEIGHT))
            .px(px(Theme::SPACE_SM))
            .child(
                div()
                    .text_xs()
                    .text_color(theme.text_faint)
                    .child("Settings sections (chrome T2)"),
            )
    }

    /// The page pane: zeron's page column scrolls to the window's top edge and
    /// carries the titlebar clearance inside the scroll, so a page header can
    /// sit at the very top of the column once it scrolls (`page_column` in
    /// zeron's `settings/widgets.rs`). T1 paints the placeholder content that
    /// `settings-surface` replaces with the real pages.
    fn render_page(&self, theme: &Theme) -> impl IntoElement {
        div().flex_1().min_w_0().h_full().flex().flex_col().child(
            div()
                .id("settings-page-scroll")
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
                .pt(px(Theme::TITLEBAR_HEIGHT + Theme::SPACE_LG))
                .px(px(Theme::SPACE_LG * 2.0))
                .child(
                    div()
                        .text_sm()
                        .text_color(theme.text_muted)
                        .child("chrome phase · T1 — shell surface"),
                ),
        )
    }
}

impl Render for Shell {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::of(cx).clone();
        let drag = self.titlebar_drag_region(
            "settings-titlebar",
            div()
                .absolute()
                .top_0()
                .left_0()
                .right_0()
                .h(px(Theme::TITLEBAR_HEIGHT)),
            cx,
        );
        // zeron `Shell::render`'s root: `id("shell-root")`, row layout,
        // `bg(theme.glass())`, theme text colour and the 14px rem scale.
        div()
            .id("shell-root")
            .relative()
            .flex()
            .flex_row()
            .size_full()
            .bg(theme.glass())
            .text_color(theme.text)
            .font_family(theme.font_sans.clone())
            .text_size(typography::ui_rems(14.0))
            .child(Self::sidebar_tone(SIDEBAR_WIDTH, theme.border))
            .child(
                div()
                    .size_full()
                    .relative()
                    .child(
                        div()
                            .size_full()
                            .flex()
                            .flex_row()
                            .child(self.render_sidebar(&theme))
                            .child(self.render_page(&theme)),
                    )
                    .child(drag),
            )
    }
}
