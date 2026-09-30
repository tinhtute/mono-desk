use gpui_kit::component::{
    ActiveTheme, IconName, StyledExt, TitleBar,
    button::{Button, ButtonVariants},
    v_flex,
};
use gpui_kit::{
    AnyElement, App, FontWeight, Global, Hsla, IntoElement, ParentElement as _, Pixels, Point,
    RenderOnce, StyleRefinement, Styled as _, Window, div, img, point, px,
};

/// Titlebar height, from zdesk `spacing.css`.
pub(crate) const TITLEBAR_H: Pixels = px(36.);
const LIGHT_SIZE: Pixels = px(14.);

/// GPUI's `traffic_light_position.y` is the button TOP edge (Tauri's is the center).
pub(crate) fn traffic_light_origin() -> Point<Pixels> {
    point(px(16.), TITLEBAR_H / 2. - LIGHT_SIZE / 2.)
}

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
/// mark, gap-2, px-2. The wordmark is 17px in the full-contrast
/// `theme.foreground`, set in the derived Xanh Mono SemiBold Italic face
/// (`assets/fonts/XanhMono-SemiBoldItalic.ttf`, weight 600 — see
/// `theme::XANH_MONO_SEMIBOLD_ITALIC`): the family's own 400 italic is too
/// thin to hold up at 17px on a 1.0-scale display, and GPUI does not
/// synthesize bold, so the weight has to come from a real face. When
/// `collapsed`, the row shrinks to a centered 20px mark on a 32px box so it
/// stays in the icon column above the rail's items.
pub(crate) fn sidebar_brand(collapsed: bool, foreground: Hsla) -> impl IntoElement {
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
                // Selects the derived 600 face; the family's own italic is 400.
                .font_weight(FontWeight::SEMIBOLD)
                .text_size(px(17.))
                .text_color(foreground)
                .child("mono-desk"),
        )
    }
}

/// Stateless shell element: a vertical column of exactly two regions —
/// the transparent `TitleBar` (borderless; its one child is the sidebar
/// collapse toggle, parked right of the macOS traffic lights via the
/// title bar's built-in 80px left padding) and the settings element
/// rendered directly below it — an `AnyElement` so `main.rs` can wrap the
/// `Settings` tree in the sidebar-width spring. The raised content card is
/// applied by `Settings` itself via `content_panel_style`.
#[derive(IntoElement)]
pub struct AppShell {
    settings: Option<AnyElement>,
}

impl AppShell {
    pub fn new() -> Self {
        Self { settings: None }
    }

    /// Attach the settings element rendered under the titlebar.
    pub fn settings_view(mut self, settings: AnyElement) -> Self {
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
                    .refine_style(&StyleRefinement::default().h(TITLEBAR_H).border_0())
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
