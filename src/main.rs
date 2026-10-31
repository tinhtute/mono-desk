//! mono-desk — a settings-style desktop demo.
//!
//! UI stack: zeron's GPUI fork (`github.com/zeronsh/zui`, pinned rev). The
//! window chrome, sidebar/settings layout and widget styling are ported from
//! zeron's MIT-licensed source (`github.com/zeronsh/zeron`) into
//! `vendor/zeron-ui/` and adapted to this settings-only app; the theme domain
//! model lives in `vendor/zeron-theme/`. See
//! `docs/plans/active/zeron-stack-port.md` and `THIRD_PARTY_NOTICES.md`.
//!
//! The window runs on zeron's Settings-route shell — the glass root surface
//! over the blurred window background, the 38px titlebar drag strip, the
//! sidebar tone/column, its seven-section list with the roving tab focus, the
//! search field and the footer identity row — and the seven settings pages are
//! built on zeron's vendored page/widget kit. `src/shell.rs` owns the chrome,
//! `src/settings/` the pages, `src/finance/` the Finance UI showcase.

mod app_settings;
mod finance;
mod i18n;
mod persist;
mod settings;
mod shell;

use gpui::{
    App, AppContext as _, AssetSource as _, Bounds, Window, WindowBackgroundAppearance,
    WindowBounds, WindowOptions, point, px, size,
};
use shell::Shell;
use zeron_ui::icons;
use zeron_ui::theme::Theme;
use zeron_ui::typography;

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
    // Shell chrome: the root surface is theme.glass() over the blurred window,
    // and the sidebar stands on zeron's wash(0.05) tone.
    eprintln!(
        "selftest shell glass={:?} is_glass={} tone={:?} border={:?} sidebar_default={}",
        theme.glass(),
        theme.is_glass(),
        zeron_ui::theme::wash(0.05),
        theme.border,
        shell::SIDEBAR_WIDTH,
    );
    eprintln!(
        "selftest chrome titlebar={} top_pad={} cluster_start={} spacer={} clearance={}",
        Theme::TITLEBAR_HEIGHT,
        Theme::TITLEBAR_TOP_PAD,
        zeron_ui::chrome::titlebar_cluster_start(false),
        zeron_ui::chrome::titlebar_spacer_width(cfg!(target_os = "macos"), false, 10.0),
        zeron_ui::chrome::cluster_clearance(true, false, 0, 16.0),
    );
    // Sidebar collapse (the chat shell's titlebar toggle ported onto this
    // shell): the persisted state the shell seeds and the width its column
    // targets. `MONO_DESK_SIDEBAR=collapsed` seeds it without a synthetic ⌘B,
    // so a headless run can prove the collapsed shell path.
    let collapsed = app_settings::AppSettings::global(cx).sidebar_collapsed;
    eprintln!(
        "selftest sidebar collapsed={collapsed} target={} content={}",
        shell::sidebar_target(collapsed),
        shell::SIDEBAR_WIDTH,
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
        shell::section_from_env().unwrap_or_default().label(),
    );
    if let Ok(query) = std::env::var("MONO_DESK_SEARCH") {
        let visible: Vec<&str> =
            shell::sections_for_query(&query, &app_settings::AppSettings::global(cx).language)
                .into_iter()
                .map(|section| section.label())
                .collect();
        eprintln!(
            "selftest search query={} visible={}",
            query,
            visible.join("|")
        );
    }
    // Settings state: the global the pages read and write, with the persisted
    // shape it round-trips through (`settings-surface` T1).
    let settings = app_settings::AppSettings::global(cx);
    eprintln!(
        "selftest settings language={} timezone={} launch={} notifications={} channels={}{}{} digest={:?} display_name={}",
        settings.language,
        settings.timezone,
        settings.launch_at_login,
        settings.notifications_enabled,
        u8::from(settings.channel_email),
        u8::from(settings.channel_desktop),
        u8::from(settings.channel_digest),
        settings.digest,
        settings.display_name,
    );
    // Appearance domain (`state-persistence` T2): the persisted words, the
    // variant the theme actually installed (post-reconcile) and the typography
    // the installed theme carries — the fields the old clobber left stale.
    eprintln!(
        "selftest appearance theme_light={} theme_dark={} variant={} surface={:?} wallpaper={:?} family={} terminal={} code={} terminal_px={} code_px={} reduce_motion={:?} pause={} smooth_scrolling={} wallpaper_theme_colors={} wallpaper_color={:?} background={} effect={} accent_word={}",
        settings.theme_light,
        settings.theme_dark,
        theme.variant_id,
        theme.surface_preference,
        theme.wallpaper_color,
        theme.font_sans,
        theme.font_terminal,
        theme.font_mono,
        theme.terminal_font_size,
        theme.code_font_size,
        settings.reduce_motion,
        settings.pause_in_background,
        settings.smooth_scrolling,
        settings.wallpaper_theme_colors,
        settings
            .wallpaper_color
            .map(|color| (color.r, color.g, color.b)),
        settings.new_thread_background.is_some(),
        settings.new_thread_background_effect.word(),
        settings
            .accent
            .map(app_settings::accent_to_str)
            .unwrap_or_else(|| "theme".to_string()),
    );
    // Theme library (`theme-library` T1-T4): the entry count and load-warning
    // flag prove R5's "selftest prints library entry count" acceptance without
    // needing the page to render.
    let library_entries = settings::theme_library::entries(cx).len();
    let library_warning = settings::theme_library::load_warning(cx).is_some();
    eprintln!(
        "selftest library entries={} warning={}",
        library_entries, library_warning
    );
    // Window rem + native background, printed *after* the last `theme` use so
    // the window handle can be updated (mutable `cx` borrow). The rem marker
    // must read the window value, not typography state: the boot-seeded
    // typography state can equal the persisted size while the window still
    // holds GPUI's 16px default (`state-persistence` gate finding B1). Both are
    // printed so a mismatch is visible here.
    let window_handle = cx.windows().first().cloned();
    let mut window_rem = None;
    if let Some(window) = window_handle {
        window_rem = window
            .update(cx, |_, window, _| f32::from(window.rem_size()))
            .ok();
    }
    let theme = Theme::of(cx);
    eprintln!(
        "selftest window_background={:?} frost={} rem_state={} rem_window={:?}",
        theme.window_background_appearance(),
        theme.is_frost(),
        typography::font_size(cx).pixels(),
        window_rem,
    );
    eprintln!(
        "selftest boot order=fonts_registered>settings_loaded>typography_init>motion_init>theme_library_init>selection_reconciled>appearance_installed"
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
    settings::apply_appearance(window, cx);
}

fn main() {
    gpui_platform::application()
        .with_assets(icons::Assets)
        .run(|cx: &mut App| {
            let selftest = std::env::var_os("MONO_DESK_SELFTEST").is_some();
            let log = |line: String| {
                if selftest {
                    eprintln!("selftest boot {line}");
                }
            };
            // Boot order (`state-persistence` T2): every stage reads only what
            // the previous ones installed.
            //   1. bundled fonts → 2. persisted `AppSettings` global →
            //   3. typography state seeded from it → 4. motion runtime →
            //   5. theme library + its custom variants in the active registry →
            //   6. persisted variant ids reconciled (and persisted if healed) →
            //   7. first theme install from the reconciled selection.
            // The theme install reads the effective typography, so typography
            // must come first; the reconcile needs the library's variants
            // registered before it can judge a persisted id.
            let fonts = typography::register_fonts(cx);
            log(format!(
                "stage=fonts_registered choices={}",
                fonts.choices().len()
            ));

            cx.set_global(app_settings::AppSettings::from_stored(persist::load()));
            log("stage=settings_loaded".to_string());

            let seed = settings::typography_seed(app_settings::AppSettings::global(cx));
            log(format!(
                "stage=typography_init ui={} ui_px={} terminal={} terminal_px={} code={} code_px={}",
                seed.ui_family.label(),
                seed.ui_size.pixels(),
                seed.terminal_family.label(),
                seed.terminal_size,
                seed.code_family.label(),
                seed.code_size,
            ));
            typography::init(
                seed.ui_family,
                seed.ui_size,
                seed.terminal_family,
                seed.terminal_size,
                seed.code_family,
                seed.code_size,
                fonts,
                cx,
            );

            let (reduce_motion, pause_in_background) =
                settings::motion_seed(app_settings::AppSettings::global(cx));
            zeron_ui::motion::init(reduce_motion, pause_in_background, cx);
            log(format!(
                "stage=motion_init reduce_motion={reduce_motion:?} pause_in_background={pause_in_background}"
            ));

            // The custom-theme library lives beside settings.conf. Without a
            // config dir (HOME unset) boot still needs a staging directory: it
            // falls back to the process cwd and the fallback is logged.
            let data_dir = persist::config_dir().unwrap_or_else(|| {
                let cwd = std::env::current_dir().unwrap_or_else(|_| ".".into());
                eprintln!(
                    "boot: no config directory (HOME unset); theme library falls back to {}",
                    cwd.display()
                );
                cwd
            });
            settings::theme_library::init(&data_dir, cx);
            log(format!(
                "stage=theme_library_init data_dir={}",
                data_dir.display()
            ));

            // Heal persisted ids whose custom family vanished (removed or failed
            // to reload) before they can reach the painter; a healed selection
            // is written back so the file stays truthful.
            let reconciled = settings::reconcile_persisted_selection(cx);
            let (theme_light, theme_dark) = {
                let settings = app_settings::AppSettings::global(cx);
                (settings.theme_light.clone(), settings.theme_dark.clone())
            };
            log(format!(
                "stage=selection_reconciled changed={reconciled} light={theme_light} dark={theme_dark}"
            ));

            // First theme install, before the window opens: the persisted
            // appearance/variant selection + wallpaper tint over the typography
            // state seeded above (`MONO_DESK_APPEARANCE` still pins it).
            settings::install_appearance(settings::boot_appearance(cx), cx);
            {
                let theme = Theme::of(cx);
                log(format!(
                    "stage=appearance_installed variant={} surface={:?} wallpaper={:?} font_sans={} font_terminal={}",
                    theme.variant_id,
                    theme.surface_preference,
                    theme.wallpaper_color,
                    theme.font_sans,
                    theme.font_terminal,
                ));
            }

            // The settings text fields' editing bindings, scoped to the
            // `TextField` key context so they cannot shadow the settings
            // selects' arrow-key handling (the zui fork's input example binds
            // these globally; see `src/settings/text_field.rs`).
            cx.bind_keys(settings::text_field::TextField::key_bindings());
            // ⌘F puts the keyboard in the settings search (zeron binds the
            // same shortcut for its settings field).
            cx.bind_keys([gpui::KeyBinding::new(
                "cmd-f",
                shell::shell_actions::FocusSearch,
                None,
            )]);
            // zeron's `mod-b` sidebar toggle: the primary modifier (⌘ on macOS,
            // Ctrl elsewhere — gpui's `secondary`) flips the collapse state;
            // the titlebar button is the pointer path to the same action.
            cx.bind_keys([gpui::KeyBinding::new(
                "secondary-b",
                shell::shell_actions::ToggleSidebar,
                None,
            )]);

            finance::init(cx);
            // Startup converges the OS autostart entry to the persisted
            // launch-at-login choice (pre-port R5); a foreign file is left
            // alone by `persist`'s marker guard.
            persist::autostart_converge(app_settings::AppSettings::global(cx).launch_at_login);

            let window_background = Theme::of(cx).window_background_appearance();
            cx.open_window(window_options(window_background), |window, cx| {
                sync_appearance(window, cx);
                window.observe_window_appearance(sync_appearance).detach();
                cx.new(Shell::new)
            })
            .expect("failed to open window");

            if selftest {
                selftest_and_quit(cx);
            }
        });
}
