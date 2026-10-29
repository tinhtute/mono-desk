//! mono-desk — a settings-style desktop demo.
//!
//! UI stack: zeron's GPUI fork (`github.com/zeronsh/zui`, pinned rev). The
//! window chrome, sidebar/settings layout and widget styling are ported from
//! zeron's MIT-licensed source (`github.com/zeronsh/zeron`) into
//! `vendor/zeron-ui/` and adapted to this settings-only app; the theme domain
//! model lives in `vendor/zeron-theme/`. See
//! `docs/plans/active/zeron-stack-port.md` and `THIRD_PARTY_NOTICES.md`.
//!
//! Current phase (`chrome`): the window runs on zeron's shell — glass root
//! surface over the blurred window background, the 38px titlebar drag strip and
//! the settings sidebar tone/column. `src/shell.rs` owns that chrome; the
//! settings section list, the settings pages and the vendored widget kit land
//! in the following tasks/phases.

mod shell;

use gpui::{
    App, AppContext as _, AssetSource as _, Bounds, Window, WindowBackgroundAppearance,
    WindowBounds, WindowOptions, point, px, size,
};
use shell::Shell;
use zeron_ui::icons;
use zeron_ui::theme::{AccentColor, Appearance, Theme};
use zeron_ui::typography::{self, UiFontFamily, UiFontSize};

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
/// both appearances can be checked without touching the machine's OS setting.
/// The agent has no Screen Recording grant and the fork's in-process capture is
/// not available on macOS, so evidence is the `MONO_DESK_SELFTEST` report plus
/// the owner's visual check — never a claim of pixels.
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
/// foundation and shell chrome (appearance, tokens, registered families, icon
/// asset, native window background, glass/tone values and the ported titlebar
/// numbers) and quits. This is the machine-checkable half of the phase check:
/// the agent has no Screen Recording grant, so pixels are the owner's check and
/// these assertions are the agent's.
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
        "selftest fonts momo={} hanken={} google_sans_code={} xanh_mono={} effective={} fixed={}",
        has("Momo Trust Sans"),
        has("Hanken Grotesk"),
        has("Google Sans Code"),
        has("Xanh Mono"),
        theme.font_sans,
        theme.font_sans_fixed,
    );
    eprintln!("selftest icon_sun_svg={icon:?}");
    eprintln!(
        "selftest window_background={:?} frost={} rem={}",
        theme.window_background_appearance(),
        theme.is_frost(),
        typography::font_size(cx).pixels(),
    );
    // Shell chrome: the root surface is theme.glass() over the blurred window,
    // and the sidebar stands on zeron's wash(0.05) tone.
    eprintln!(
        "selftest shell glass={:?} is_glass={} tone={:?} border={:?} sidebar={}",
        theme.glass(),
        theme.is_glass(),
        zeron_ui::theme::wash(0.05),
        theme.border,
        shell::SIDEBAR_WIDTH,
    );
    eprintln!(
        "selftest chrome titlebar={} top_pad={} cluster_start={} cluster_pad={} spacer={} clearance={}",
        Theme::TITLEBAR_HEIGHT,
        Theme::TITLEBAR_TOP_PAD,
        zeron_ui::chrome::titlebar_cluster_start(false),
        zeron_ui::chrome::TITLEBAR_CLUSTER_PAD,
        zeron_ui::chrome::titlebar_spacer_width(
            cfg!(target_os = "macos"),
            false,
            zeron_ui::chrome::TITLEBAR_CLUSTER_PAD
        ),
        zeron_ui::chrome::cluster_clearance(true, false, 0, 16.0),
    );
    // Settings section list: the nav renders from this list alone.
    let sections: Vec<_> = zeron_ui::settings::SettingsSection::ALL
        .iter()
        .map(|section| section.label())
        .collect();
    eprintln!(
        "selftest nav sections={} labels={} group_break={} selected={}",
        sections.len(),
        sections.join("|"),
        zeron_ui::settings::SettingsSection::FinanceUi.label(),
        zeron_ui::settings::SettingsSection::default().label(),
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
                cx.new(Shell::new)
            })
            .expect("failed to open window");

            if std::env::var_os("MONO_DESK_SELFTEST").is_some() {
                selftest_and_quit(cx);
            }
        });
}
