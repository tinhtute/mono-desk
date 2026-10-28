use gpui_kit::component::{
    ActiveTheme, IconName, Sizable as _, StyledExt, TitleBar,
    button::{Button, ButtonVariants},
    v_flex,
};
use gpui_kit::{
    AnyElement, App, Global, IntoElement, ParentElement as _, Pixels, Point, RenderOnce,
    SharedString, StyleRefinement, Styled as _, Window, div, point, px,
};

/// Unified window titlebar height: zeron's 38px overlay bar.
pub(crate) const TITLEBAR_H: Pixels = px(38.);

/// GPUI's `traffic_light_position.y` is the button TOP edge (Tauri's is the center).
pub(crate) fn traffic_light_origin() -> Point<Pixels> {
    // zeron pins the native lights at (14, 14) inside the 38px overlay bar.
    point(px(14.), px(14.))
}

/// zeron's top-left control cluster starts at 88px from the window edge
/// (`titlebar_cluster_start`, the same value its titlebar spacer uses), which
/// leaves a deliberate gap after the traffic lights instead of hugging them.
/// The vendored `TitleBar` defaults to 80px, so the shell overrides it.
const TITLEBAR_CLUSTER_START: Pixels = px(88.);

/// zeron's 24px cluster buttons ride 4px below the bar's geometric center
/// (`TITLEBAR_TOP_PAD`), which puts them on the traffic lights' axis.
const TITLEBAR_CLUSTER_TOP_PAD: Pixels = px(4.);

/// Shell chrome state, stored as a GPUI global so both the titlebar
/// collapse button (via `on_click`) and `main.rs` (sidebar sizing) can
/// reach it through `&mut App`.
#[derive(Default)]
pub struct ShellState {
    /// `true` = collapsed to the grab sliver; `false` = persisted rail width.
    pub collapsed: bool,
}

impl Global for ShellState {}

/// Stateless shell element for zeron's window layout: the settings page
/// (rail + content) fills the whole window, and the 38px `TitleBar` is an
/// absolute overlay at the top — borderless, no fill, its one child parked
/// right of the macOS traffic lights by the bar's built-in left padding.
/// The page pads its own content below the bar; the shell paints the single
/// window base (opaque token or the glass frost).
#[derive(IntoElement)]
pub struct AppShell {
    settings: Option<AnyElement>,
    /// Centered title in the overlay titlebar (zeron's centered session
    /// title); the settings app shows the active page name there.
    page_title: Option<SharedString>,
}

impl AppShell {
    pub fn new() -> Self {
        Self {
            settings: None,
            page_title: None,
        }
    }

    /// Set the centered titlebar title.
    pub fn page_title(mut self, title: impl Into<SharedString>) -> Self {
        self.page_title = Some(title.into());
        self
    }

    /// Attach the settings element rendered under the titlebar.
    pub fn settings_view(mut self, settings: AnyElement) -> Self {
        self.settings = Some(settings);
        self
    }
}

impl RenderOnce for AppShell {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        // One alpha layer per pixel: with glass on, the shell paints the
        // single full-window frost layer (`shell_frost_bg`). Everything above
        // it is transparent chrome — the title bar paints nothing, the sidebar
        // adds only an ink tone, and the content is flush — so no pixel is
        // covered by two alpha *bases* (the old per-pane model composited to
        // `1-(1-a)^2` and read opaque). Without glass the shipped opaque base
        // is painted exactly as before.
        let glass = crate::glass_enabled(cx);
        let base = if glass {
            crate::shell_frost_bg(cx)
        } else {
            cx.theme().background
        };
        // Under glass the title bar is a *window* band, not a surface: it
        // paints nothing and lets the frost below carry the tone. The opaque
        // path keeps the token gradient.
        // zeron's titlebar strip paints nothing in either appearance: it is
        // chrome over the page, not a surface. The vendored bar would paint a
        // token gradient of its own, so the refinement always kills it.
        let title_bar = StyleRefinement::default()
            .h(TITLEBAR_H)
            .border_0()
            .bg(cx.theme().transparent)
            // zeron's cluster inset (see `TITLEBAR_CLUSTER_START`); the
            // vendored bar's own 80px left padding lands first, so this wins.
            .pl(TITLEBAR_CLUSTER_START);
        // zeron layout: the page owns the full window height and pads its own
        // content below the bar; the title bar is an absolute overlay, not a
        // layout row, so the sidebar tone runs under it to the window top.
        let page = v_flex()
            .size_full()
            .child(div().size_full().children(self.settings));
        let title_bar_overlay = div().absolute().top_0().left_0().right_0().child(
            // `.refine_style` lands after the title bar's own
            // `.border_b_1()`, so `border_0()` wins and kills the 1px
            // seam under the title bar.
            TitleBar::new().refine_style(&title_bar).child(
                // zeron's cluster rhythm: a 24px button on the lights' axis
                // (4px top pad + centering in the 38px bar), starting past the
                // 88px cluster inset.
                div()
                    .h_full()
                    .pt(TITLEBAR_CLUSTER_TOP_PAD)
                    .flex()
                    .items_center()
                    .child(
                        // An icon-only toggle has no text for a screen reader,
                        // so it needs an explicit accessible name. This must go on
                        // the Button itself: a plain `div` wrapper has no a11y
                        // role and is filtered out of the tree.
                        Button::new("sidebar-collapse-toggle")
                            .ghost()
                            .small()
                            .icon(IconName::PanelLeft)
                            .accessibility_label("Toggle sidebar")
                            .tooltip("Toggle sidebar")
                            .on_click(|_, window, cx| {
                                let state = cx.global_mut::<ShellState>();
                                state.collapsed = !state.collapsed;
                                window.refresh();
                            }),
                    ),
            ),
        );
        div()
            .size_full()
            .relative()
            .bg(base)
            .child(page)
            .child(title_bar_overlay)
    }
}
