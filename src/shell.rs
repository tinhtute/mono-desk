use gpui_kit::component::{
    ActiveTheme, IconName, StyledExt, TitleBar,
    button::{Button, ButtonVariants},
    setting::Settings,
    v_flex,
};
use gpui_kit::{
    App, FontWeight, Global, IntoElement, ParentElement as _, RenderOnce, StyleRefinement,
    Styled as _, Window, div, img, px,
};

/// Shell chrome state, stored as a GPUI global so both the titlebar
/// collapse button (via `on_click`) and `main.rs` (sidebar sizing) can
/// reach it through `&mut App`.
#[derive(Default)]
pub struct ShellState {
    /// `true` = 48px icon rail; `false` = full 240px sidebar.
    pub collapsed: bool,
}

impl Global for ShellState {}

/// Sidebar brand row: 32px-high raster mark (`img` — `svg()` would
/// flatten the gradient into a monochrome mask) plus the Xanh Mono
/// italic wordmark. Mounted via `Settings::sidebar_header`, above the
/// search input.
///
/// Metrics mirror the zdesk `NavView` brand row: h-8, `size-brand` (20px)
/// mark, 15px semibold wordmark, gap-2, px-2. When `collapsed`, the row
/// shrinks to a centered 20px mark on an 8px-wide (w-8) box so it stays
/// in the icon column above the rail's items.
pub(crate) fn sidebar_brand(collapsed: bool) -> impl IntoElement {
    let row = div()
        .h(px(32.))
        .flex_shrink_0()
        .flex()
        .items_center()
        .child(img("brand/brand-mark.png").size(px(20.)).flex_shrink_0());

    if collapsed {
        row.w(px(32.)).justify_center()
    } else {
        row.w_full().gap_2().px_2().child(
            div()
                .font_family("Xanh Mono")
                .italic()
                .font_weight(FontWeight::SEMIBOLD)
                .text_size(px(15.))
                .child("mono-desk"),
        )
    }
}

/// Stateless shell element: a vertical column of exactly two regions —
/// the transparent `TitleBar` (borderless; its one child is the sidebar
/// collapse toggle, parked right of the macOS traffic lights via the
/// title bar's built-in 80px left padding) and the `Settings` element
/// rendered directly below it. The raised content card is applied by
/// `Settings` itself via `content_panel_style`.
#[derive(IntoElement)]
pub struct AppShell {
    settings: Option<Settings>,
}

impl AppShell {
    pub fn new() -> Self {
        Self { settings: None }
    }

    /// Attach the `Settings` element rendered under the titlebar.
    pub fn settings_view(mut self, settings: Settings) -> Self {
        self.settings = Some(settings);
        self
    }
}

impl RenderOnce for AppShell {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        v_flex()
            .size_full()
            .bg(cx.theme().background)
            .child(
                // `.refine_style` lands after the title bar's own
                // `.border_b_1()`, so `border_0()` (refined
                // `border_widths = 0` on all edges) wins and kills the
                // 1px seam under the title bar.
                TitleBar::new()
                    .refine_style(&StyleRefinement::default().border_0())
                    .child(
                        Button::new("sidebar-collapse-toggle")
                            .ghost()
                            .icon(IconName::PanelLeft)
                            .on_click(|_, window, cx| {
                                let state = cx.global_mut::<ShellState>();
                                state.collapsed = !state.collapsed;
                                window.refresh();
                            }),
                    ),
            )
            .children(self.settings)
    }
}
