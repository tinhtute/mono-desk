//! mono-desk — a settings-style desktop demo.
//!
//! UI stack: zeron's GPUI fork (`github.com/zeronsh/zui`, pinned rev). The
//! window chrome, sidebar/settings layout and widget styling are ported from
//! zeron's MIT-licensed source (`github.com/zeronsh/zeron`) into
//! `vendor/zeron-ui/` and adapted to this settings-only app; the theme domain
//! model lives in `vendor/zeron-theme/`. See
//! `docs/plans/active/zeron-stack-port.md` and `THIRD_PARTY_NOTICES.md`.
//!
//! Current phase (`theme-foundation`): zeron's theme tokens, typography, icons
//! and frost/glass surfaces are installed on the new stack, the window
//! background follows the theme, and the root view paints the theme base in
//! both appearances. The shell/chrome replace this probe surface in the
//! `chrome` phase.

use gpui::{
    App, AppContext as _, AssetSource as _, Bounds, Div, FontWeight, IntoElement,
    ParentElement as _, Render, Styled as _, Window, WindowBackgroundAppearance, WindowBounds,
    WindowOptions, div, point, px, size,
};
use zeron_ui::icons;
use zeron_ui::theme::{AccentColor, Appearance, Theme};
use zeron_ui::typography::{self, UiFontFamily, UiFontSize};

/// zeron's titlebar height (`Theme::TITLEBAR_HEIGHT`): the window chrome is an
/// overlay strip in the `chrome` phase; the probe keeps the same geometry.
const TITLEBAR_HEIGHT: f32 = 38.0;

/// Root view for the theme-foundation phase; replaced by the zeron shell in the
/// `chrome` phase.
struct SettingsApp;

impl SettingsApp {
    /// One labelled colour chip.
    fn swatch(label: &'static str, color: gpui::Hsla, border: gpui::Hsla) -> Div {
        div()
            .flex()
            .flex_col()
            .gap_1()
            .child(
                div()
                    .h(px(36.))
                    .w_full()
                    .rounded_md()
                    .bg(color)
                    .border_1()
                    .border_color(border),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(gpui::hsla(0.0, 0.0, 0.45, 1.0))
                    .child(label),
            )
    }
}

impl Render for SettingsApp {
    fn render(&mut self, _window: &mut Window, cx: &mut gpui::Context<Self>) -> impl IntoElement {
        let theme = Theme::of(cx);
        let bg = theme.bg;
        let surface = theme.surface;
        let card = theme.surface_card;
        let border = theme.border;
        let hairline = theme.hairline(0.12);
        let text = theme.text;
        let muted = theme.text_muted;
        let faint = theme.text_faint;
        let accent = theme.accent;
        let accent_strong = theme.accent_strong;
        let sans = theme.font_sans.clone();
        let mono = theme.font_mono.clone();
        let display: gpui::SharedString = UiFontFamily::XanhMono.family_name().into();
        let appearance = theme.appearance;
        let frost = theme.is_frost();
        let window_background = theme.window_background_appearance();

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(bg)
            .text_color(text)
            .font_family(sans.clone())
            .child(
                // Titlebar strip: 38px, hairline at the bottom, title centered.
                div()
                    .h(px(TITLEBAR_HEIGHT))
                    .flex_none()
                    .flex()
                    .items_center()
                    .justify_center()
                    .border_b_1()
                    .border_color(hairline)
                    .text_sm()
                    .font_weight(FontWeight::MEDIUM)
                    .child("Theme foundation"),
            )
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .child(
                        // Sidebar column: full height, surface tone + 1px hairline.
                        div()
                            .w(px(256.))
                            .h_full()
                            .flex_none()
                            .flex()
                            .flex_col()
                            .gap_3()
                            .p_4()
                            .bg(surface)
                            .border_r_1()
                            .border_color(hairline)
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(faint)
                                    .child("SURFACES"),
                            )
                            .child(Self::swatch("bg", bg, border))
                            .child(Self::swatch("surface", surface, border))
                            .child(Self::swatch("surface_card", card, border))
                            .child(Self::swatch("element_hover", theme.element_hover, border))
                            .child(Self::swatch("accent", accent, border))
                            .child(Self::swatch("accent_strong", accent_strong, border)),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .h_full()
                            .flex()
                            .flex_col()
                            .gap_4()
                            .p_6()
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .child(div().text_xl().font_weight(FontWeight::SEMIBOLD).child(
                                        format!(
                                            "{} appearance",
                                            match appearance {
                                                Appearance::Dark => "Dark",
                                                Appearance::Light => "Light",
                                            }
                                        ),
                                    ))
                                    .child(div().text_xs().text_color(muted).child(format!(
                                        "window background: {window_background:?} · frost: {frost}"
                                    ))),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_3()
                                    .p_4()
                                    .rounded_lg()
                                    .bg(card)
                                    .border_1()
                                    .border_color(border)
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(muted)
                                            .child("Typography"),
                                    )
                                    .child(
                                        div()
                                            .font_family(display)
                                            .text_size(px(26.))
                                            .italic()
                                            .child("mono-desk"),
                                    )
                                    .child(
                                        div()
                                            .font_family(sans.clone())
                                            .text_lg()
                                            .child("Momo Trust Sans — the quick brown fox"),
                                    )
                                    .child(
                                        div()
                                            .font_family(sans.clone())
                                            .text_sm()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(muted)
                                            .child("Momo Trust Sans Medium — settings body copy"),
                                    )
                                    .child(
                                        div()
                                            .font_family(mono)
                                            .text_sm()
                                            .text_color(faint)
                                            .child("Google Sans Code — 0123456789 { keystroke }"),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_3()
                                    .p_4()
                                    .rounded_lg()
                                    .bg(card)
                                    .border_1()
                                    .border_color(border)
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(muted)
                                            .child("Solar icons (CC BY 4.0 — 480 Design)"),
                                    )
                                    .child(
                                        div().flex().items_center().gap_5().children(
                                            [
                                                icons::SUN,
                                                icons::MOON,
                                                icons::SETTINGS,
                                                icons::TUNING,
                                                icons::MAGNIFER,
                                            ]
                                            .into_iter()
                                            .map(
                                                |asset| {
                                                    icons::icon(asset)
                                                        .size(px(18.))
                                                        .text_color(accent)
                                                },
                                            ),
                                        ),
                                    ),
                            ),
                    ),
            )
    }
}

/// zeron's window options (`crates/ui/src/lib.rs` `open_main_window`): a
/// frameless-inset macOS chrome with custom-drawn titlebar, native traffic
/// lights at (14, 14), app-owned titlebar dragging, and a theme-driven window
/// background (frosted when the theme asks for it).
fn window_options(window_background: WindowBackgroundAppearance) -> WindowOptions {
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
        window_background,
        app_id: Some("mono-desk".into()),
        ..Default::default()
    }
}

/// Dev-only review hook: `MONO_DESK_APPEARANCE=dark|light` pins the palette so
/// both appearances can be captured without touching the machine's OS setting.
/// The agent has no Screen Recording grant, so review evidence comes from
/// in-process renders (`MONO_DESK_CAPTURE`), not `screencapture`.
fn forced_appearance() -> Option<Appearance> {
    match std::env::var("MONO_DESK_APPEARANCE").ok()?.as_str() {
        "dark" => Some(Appearance::Dark),
        "light" => Some(Appearance::Light),
        _ => None,
    }
}

/// The appearance the app should paint: the review pin, else the window's.
fn desired_appearance(window_appearance: gpui::WindowAppearance) -> Appearance {
    forced_appearance().unwrap_or_else(|| Appearance::from_window(window_appearance))
}

/// Dev-only review hook: `MONO_DESK_SELFTEST=1` prints the resolved theme
/// foundation (appearance, tokens, registered families, icon asset, native
/// window background) and quits. This is the machine-checkable half of the
/// phase check: the agent has no Screen Recording grant, so pixels are the
/// owner's check and these assertions are the agent's.
fn selftest_and_quit(cx: &mut App) {
    let theme = Theme::of(cx);
    let fonts = cx.text_system().all_font_names();
    let has = |family: &str| fonts.iter().any(|name| name == family);
    let icon = icons::Assets
        .load(icons::SUN)
        .ok()
        .flatten()
        .map(|bytes| bytes.starts_with(b"<svg"));
    eprintln!("selftest appearance={:?}", theme.appearance);
    eprintln!(
        "selftest bg={:?} surface={:?} text={:?} accent={:?}",
        theme.bg, theme.surface, theme.text, theme.accent
    );
    eprintln!(
        "selftest fonts momo={} hanken={} google_sans_code={} xanh_mono={} effective={}",
        has("Momo Trust Sans"),
        has("Hanken Grotesk"),
        has("Google Sans Code"),
        has("Xanh Mono"),
        theme.font_sans,
    );
    eprintln!("selftest icon_sun_svg={icon:?}");
    eprintln!(
        "selftest window_background={:?} frost={} rem={}",
        theme.window_background_appearance(),
        theme.is_frost(),
        typography::font_size(cx).pixels(),
    );
    cx.refresh_windows();
    cx.spawn(async move |cx: &mut gpui::AsyncApp| {
        cx.background_executor()
            .timer(std::time::Duration::from_millis(400))
            .await;
        cx.update(|cx| cx.quit());
    })
    .detach();
}

/// Re-resolve the theme for the window's actual appearance and point the native
/// window background at it. Runs at attach time (the window, not the app, knows
/// the appearance during launch) and on every OS appearance change.
fn sync_appearance(window: &mut Window, cx: &mut App) {
    let appearance = desired_appearance(window.appearance());
    if cx.try_global::<Theme>().map(|theme| theme.appearance) == Some(appearance) {
        return;
    }
    let accent = cx
        .try_global::<Theme>()
        .map(|theme| theme.accent_color)
        .unwrap_or_default();
    Theme::install_preferences(appearance, accent, cx);
    window.set_background_appearance(Theme::of(cx).window_background_appearance());
    cx.refresh_windows();
}

fn main() {
    gpui_platform::application()
        .with_assets(icons::Assets)
        .run(|cx: &mut App| {
            let system = desired_appearance(cx.window_appearance());
            let fonts = typography::register_fonts(cx);
            typography::init(
                UiFontFamily::MomoTrustSans,
                UiFontSize::default(),
                UiFontFamily::XanhMono,
                typography::TERMINAL_FONT_SIZE_DEFAULT,
                UiFontFamily::GoogleSansCode,
                typography::CODE_FONT_SIZE_DEFAULT,
                fonts,
                cx,
            );
            Theme::install_preferences(system, AccentColor::default(), cx);

            let window_background = Theme::of(cx).window_background_appearance();
            cx.open_window(window_options(window_background), |window, cx| {
                window.set_rem_size(px(typography::font_size(cx).pixels()));
                sync_appearance(window, cx);
                window.observe_window_appearance(sync_appearance).detach();
                cx.new(|_| SettingsApp)
            })
            .expect("failed to open window");

            if std::env::var_os("MONO_DESK_SELFTEST").is_some() {
                selftest_and_quit(cx);
            }
        });
}
