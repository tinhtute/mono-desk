//! The settings pages, built on zeron's page scaffolding
//! (`vendor/zeron-ui/src/settings/widgets.rs`) and zeron's own select
//! primitive (`vendor/zeron-ui/src/settings/select.rs`).
//!
//! Phase `settings-surface` landed General and Notifications (T1), the
//! Profile/Finance UI/Reset/About pages (T2), zeron's own Appearance page (T3)
//! and the sidebar search (T4). The page *content* is mono-desk's own (i18n
//! strings, `AppSettings`, persistence); the page structure — `page_column`,
//! `page_header`, `section`, `section_card`, `card_row`, `row_title`,
//! `meta_line`, `toggle_switch`, `select` — is zeron's, ported verbatim, so the
//! pages read as the same product surface as the vendored chrome.
//!
//! Page state (open select menus, scroll offsets) lives on the page views,
//! like zeron's page views; writes go straight into the `AppSettings` global
//! and through [`persist_now`], and the affected view repaints itself.
//!
//! [`apply_appearance`] is the single funnel that turns the persisted
//! appearance/accent/surface/font-size values into zeron's theme (and the
//! native window background). It replaces zeron's own `appearance::set_*`
//! helpers, which write into zeron's settings service that this app does not
//! vendor.

mod about;
mod appearance;
mod background;
mod finance;
mod general;
mod notifications;
mod profile;
mod reset;
pub(crate) mod text_field;
pub(crate) mod theme_library;
mod wallpaper;
mod wallpaper_colors;

pub(crate) use about::AboutPage;
pub(crate) use appearance::AppearancePage;
pub(crate) use finance::FinancePage;
pub(crate) use general::GeneralPage;
pub(crate) use notifications::NotificationsPage;
pub(crate) use profile::ProfilePage;
pub(crate) use reset::ResetPage;

use gpui::{
    AnyElement, App, Hsla, IntoElement, ParentElement as _, SharedString, Styled as _, Window, div,
    px,
};

use zeron_theme::{AccentPreset, AccentSelection, Color, SurfacePreference, ThemeSelection};
use zeron_ui::motion::ReduceMotion;
use zeron_ui::settings::widgets;
use zeron_ui::theme::{Appearance as ThemeAppearance, Theme};
use zeron_ui::typography::{self, UiFontFamily, UiFontSize};

use crate::app_settings::{self, AppSettings};
use crate::persist;

/// The write-through path every settings control uses: mirror the live global
/// into the persisted struct and save it. `persist::save` is synchronous and
/// already atomic + guarded (see its module docs), so a page can call this
/// from a click handler without owning any save machinery.
pub(crate) fn persist_now(cx: &mut App) {
    let stored = AppSettings::global(cx).to_stored();
    persist::save(&stored);
}

/// zeron's settings row: title + muted meta on the left, the control on the
/// right, split from the previous row by the card's inset hairline. Mirrors
/// the row shapes in zeron's own pages (e.g. `settings/appearance.rs`'s
/// motion rows) on top of the ported `card_row`/`row_title`/`meta_line`.
pub(crate) fn control_row(
    theme: &Theme,
    first: bool,
    title: impl Into<SharedString>,
    meta: impl Into<SharedString>,
    control: impl IntoElement,
) -> gpui::Div {
    widgets::card_row(theme, first)
        .child(
            div()
                .flex_1()
                .min_w(px(160.0))
                .child(widgets::row_title(theme, title))
                .child(widgets::meta_line(
                    theme,
                    vec![div().child(meta.into()).into_any_element() as AnyElement],
                )),
        )
        .child(control)
}

/// Converge the OS autostart entry to the persisted launch-at-login value
/// (pre-port R5): call after any write that can change that field. `persist`
/// owns the marker guard, so a foreign file is never touched.
pub(crate) fn apply_launch_at_login(cx: &App) {
    persist::autostart_converge(AppSettings::global(cx).launch_at_login);
}

/// Dev review hook (`MONO_DESK_APPEARANCE=dark|light`, the same one `main.rs`
/// documents): pins the palette without touching the OS setting, ahead of the
/// persisted choice, so both appearances can be checked headlessly.
fn pinned_appearance() -> Option<ThemeAppearance> {
    match std::env::var("MONO_DESK_APPEARANCE").ok()?.as_str() {
        "dark" => Some(ThemeAppearance::Dark),
        "light" => Some(ThemeAppearance::Light),
        _ => None,
    }
}

/// Resolve the persisted appearance choice: zeron's theme only has Light and
/// Dark, so `System` asks the window (zeron's settings service does the same
/// through its `appearance::mode` accessor).
pub(crate) fn theme_appearance(
    choice: app_settings::Appearance,
    window: &Window,
) -> ThemeAppearance {
    choice_appearance(choice).unwrap_or_else(|| ThemeAppearance::from_window(window.appearance()))
}

/// The appearance a pinned choice resolves to; `None` for `System`, which
/// depends on the live window and is resolved by [`theme_appearance`].
fn choice_appearance(choice: app_settings::Appearance) -> Option<ThemeAppearance> {
    match choice {
        app_settings::Appearance::Light => Some(ThemeAppearance::Light),
        app_settings::Appearance::Dark => Some(ThemeAppearance::Dark),
        app_settings::Appearance::System => None,
    }
}

/// The zeron accent selection matching the persisted `accent` hex: the theme's
/// own accent when nothing is stored, else the preset whose authored color for
/// the current appearance equals the stored value. The persisted file keeps
/// the old `#RRGGBB` shape (NG4); a color from the pre-port palette that has no
/// preset simply falls back to the theme accent.
pub(crate) fn accent_selection(
    stored: Option<Hsla>,
    appearance: ThemeAppearance,
) -> AccentSelection {
    let Some(stored) = stored else {
        return AccentSelection::ThemeDefault;
    };
    AccentPreset::ALL
        .into_iter()
        .find(|preset| {
            let authored = zeron_ui::theme::model_color(preset.color(model_appearance(appearance)));
            hue_close(authored, stored)
        })
        .map(AccentSelection::Preset)
        .unwrap_or(AccentSelection::ThemeDefault)
}

fn hue_close(a: Hsla, b: Hsla) -> bool {
    (a.h - b.h).abs() < 0.004 && (a.s - b.s).abs() < 0.02 && (a.l - b.l).abs() < 0.02
}

/// The value to persist for a picked accent selection: the preset's authored
/// color for the current appearance, or `None` for the theme's own accent.
pub(crate) fn accent_stored(
    selection: AccentSelection,
    appearance: ThemeAppearance,
) -> Option<Hsla> {
    match selection {
        AccentSelection::ThemeDefault => None,
        AccentSelection::Preset(preset) => Some(zeron_ui::theme::model_color(
            preset.color(model_appearance(appearance)),
        )),
    }
}

/// zeron's domain appearance for color math; the UI theme has its own
/// `Appearance` enum because it also converts from the OS window appearance.
pub(crate) fn model_appearance(appearance: ThemeAppearance) -> zeron_theme::Appearance {
    match appearance {
        ThemeAppearance::Light => zeron_theme::Appearance::Light,
        ThemeAppearance::Dark => zeron_theme::Appearance::Dark,
    }
}

/// zeron's surface policy for mono-desk's persisted glass flags: solid when
/// translucency is off, else zeron's frosted (translucent + blurred) material.
/// mono-desk's pre-port "transparent without blur" state has no zeron
/// equivalent and reads as frosted.
pub(crate) fn surface_preference(translucent: bool) -> SurfacePreference {
    if translucent {
        SurfacePreference::Frosted
    } else {
        SurfacePreference::Opaque
    }
}

/// The built-in registry's variant id for `appearance`: the fallback when a
/// persisted id is empty (the built-in ids are `zeron-light`/`zeron-dark`).
pub(crate) fn builtin_variant_id(appearance: ThemeAppearance) -> &'static str {
    if appearance.is_dark() {
        "zeron-dark"
    } else {
        "zeron-light"
    }
}

/// The registry variant id to install for `appearance`, from the persisted
/// per-appearance selection. An empty id (a hand-built state) falls back to the
/// built-in variant; an unknown id is tolerated here because the registry's
/// `for_selection` resolves or falls back at install time.
pub(crate) fn variant_id(settings: &AppSettings, appearance: ThemeAppearance) -> &str {
    let persisted = if appearance.is_dark() {
        &settings.theme_dark
    } else {
        &settings.theme_light
    };
    if persisted.is_empty() {
        builtin_variant_id(appearance)
    } else {
        persisted
    }
}

/// The persisted theme selection, shaped for
/// [`theme_library::reconcile`]/[`Theme::install_selection_with_color`].
pub(crate) fn theme_selection(settings: &AppSettings) -> ThemeSelection {
    ThemeSelection {
        light: settings.theme_light.clone(),
        dark: settings.theme_dark.clone(),
    }
}

/// Reconcile the persisted variant ids against the active registry (which
/// includes the custom theme library loaded at boot) and persist any correction
/// back into `AppSettings`. Returns whether the stored selection changed.
///
/// Must run after [`theme_library::init`] and before the first theme install,
/// so a selection that names a removed custom variant is healed before it can
/// reach the painter.
pub(crate) fn reconcile_persisted_selection(cx: &mut App) -> bool {
    let mut selection = theme_selection(AppSettings::global(cx));
    if !theme_library::reconcile(&mut selection) {
        return false;
    }
    {
        let settings = AppSettings::global_mut(cx);
        settings.theme_light = selection.light;
        settings.theme_dark = selection.dark;
    }
    // A healed selection is a user-visible correction: keep the file truthful.
    persist_now(cx);
    true
}

/// The wallpaper-derived tint a theme install should apply: `Some` only while
/// "Match wallpaper colors" is on and a managed new-thread background exists
/// (`wallpaper_colors::active`'s gate).
pub(crate) fn wallpaper_tint(settings: &AppSettings) -> Option<Color> {
    wallpaper_colors::active(settings)
}

/// Everything one appearance install needs, resolved purely from the persisted
/// state so it can be tested without a window.
struct AppearancePlan {
    appearance: ThemeAppearance,
    variant_id: String,
    accent: AccentSelection,
    surface: SurfacePreference,
    wallpaper_color: Option<Color>,
}

fn appearance_plan(settings: &AppSettings, appearance: ThemeAppearance) -> AppearancePlan {
    AppearancePlan {
        appearance,
        variant_id: variant_id(settings, appearance).to_string(),
        accent: accent_selection(settings.accent, appearance),
        surface: surface_preference(settings.background_translucent),
        wallpaper_color: wallpaper_tint(settings),
    }
}

/// Install one resolved plan through zeron's full selection path (variant id +
/// accent + surface + wallpaper tint). The vendored install reads the effective
/// typography out of the typography state, so callers must apply families/sizes
/// first.
fn install_plan(plan: &AppearancePlan, cx: &mut App) {
    Theme::install_selection_with_color(
        plan.appearance,
        &plan.variant_id,
        plan.accent,
        plan.surface,
        plan.wallpaper_color,
        cx,
    );
}

/// The appearance for the pre-window boot install: the review pin, else the
/// persisted Light/Dark choice, else the OS window appearance (`System`). Using
/// the persisted choice means the first painted frame is already the user's
/// appearance; the window-attached [`apply_appearance`] re-resolves it with the
/// real window (and handles OS changes).
pub(crate) fn boot_appearance(cx: &App) -> ThemeAppearance {
    pinned_appearance().unwrap_or_else(|| {
        choice_appearance(AppSettings::global(cx).appearance)
            .unwrap_or_else(|| ThemeAppearance::from_window(cx.window_appearance()))
    })
}

/// The boot install, before the first window exists: the typography state must
/// already be initialized by [`typography_seed`] + `typography::init` and the
/// selection reconciled. The window-attached [`apply_appearance`] re-runs this
/// with the window's actual appearance.
pub(crate) fn install_appearance(appearance: ThemeAppearance, cx: &mut App) {
    let plan = appearance_plan(AppSettings::global(cx), appearance);
    install_plan(&plan, cx);
    cx.refresh_windows();
}

/// The persisted typography values the boot path seeds the vendored typography
/// state with. Pure so the persisted-word → type mappings are testable without
/// a window.
pub(crate) struct TypographySeed {
    pub(crate) ui_family: UiFontFamily,
    pub(crate) ui_size: UiFontSize,
    pub(crate) terminal_family: UiFontFamily,
    pub(crate) terminal_size: f32,
    pub(crate) code_family: UiFontFamily,
    pub(crate) code_size: f32,
}

/// Seed typography from the persisted families/sizes: bundled families or
/// `installed:<name>` from the file, the UI size snapped onto zeron's discrete
/// catalog, and the terminal/code pixel sizes (already clamped by the codec).
pub(crate) fn typography_seed(settings: &AppSettings) -> TypographySeed {
    TypographySeed {
        ui_family: settings.ui_font_family.clone(),
        ui_size: ui_size(settings.font_size as f32),
        terminal_family: settings.terminal_font_family.clone(),
        terminal_size: settings.terminal_font_size as f32,
        code_family: settings.code_font_family.clone(),
        code_size: settings.code_font_size as f32,
    }
}

/// The persisted motion values the boot path installs zeron's motion runtime
/// with: the reduced-motion preference word and the pause-in-background switch.
pub(crate) fn motion_seed(settings: &AppSettings) -> (ReduceMotion, bool) {
    (settings.reduce_motion, settings.pause_in_background)
}

/// Apply the live `AppSettings` to zeron's theme and the native window:
/// appearance mode (System resolves against the window), accent preset, surface
/// material and the persisted variant id, then re-point the window background
/// (blurred for frosted surfaces). Every Appearance-page control funnels here.
///
/// Typography is applied *before* the theme: [`Theme::install_selection_with_color`]
/// reads the effective families/sizes out of the vendored typography state, so
/// installing the theme first (the pre-T2 order) left the installed `Theme`'s
/// font fields one change stale.
pub(crate) fn apply_appearance(window: &mut Window, cx: &mut App) {
    let appearance = pinned_appearance()
        .unwrap_or_else(|| theme_appearance(AppSettings::global(cx).appearance, window));
    let seed = typography_seed(AppSettings::global(cx));
    typography::set_family(seed.ui_family, cx);
    typography::set_terminal_family(seed.terminal_family, cx);
    typography::set_code_family(seed.code_family, cx);
    typography::set_terminal_font_size(seed.terminal_size, cx);
    typography::set_code_font_size(seed.code_size, cx);
    typography::set_font_size(seed.ui_size, window, cx);
    // `typography::set_font_size` only writes the window rem when its state
    // value changes. At boot the state is seeded from these same persisted
    // settings (see `main.rs`'s boot order), so that setter early-returns and
    // the window would keep GPUI's 16px default while every `ui_rems` site
    // expects the persisted size. Upstream sets the rem explicitly when the
    // window opens (`crates/ui/src/lib.rs:490`), so mirror that here: the
    // funnel always writes the rem, whatever the typography state did.
    window.set_rem_size(px(seed.ui_size.pixels()));

    let plan = appearance_plan(AppSettings::global(cx), appearance);
    install_plan(&plan, cx);
    window.set_background_appearance(Theme::of(cx).window_background_appearance());
    cx.refresh_windows();
}

/// Snap a persisted pixel size onto zeron's discrete UI size catalog (the
/// pre-port app had a continuous 8–32 slider; zeron's page uses a select).
pub(crate) fn ui_size(pixels: f32) -> UiFontSize {
    let mut best = UiFontSize::ALL[0];
    for size in UiFontSize::ALL {
        if (size.pixels() - pixels).abs() < (best.pixels() - pixels).abs() {
            best = size;
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use zeron_ui::theme::Appearance as ThemeAppearance;

    #[test]
    fn accent_presets_round_trip_through_the_persisted_hex() {
        for preset in AccentPreset::ALL {
            for appearance in [ThemeAppearance::Dark, ThemeAppearance::Light] {
                let selection = AccentSelection::Preset(preset);
                let stored = accent_stored(selection, appearance)
                    .expect("a preset stores its authored color");
                assert_eq!(accent_selection(Some(stored), appearance), selection);
            }
        }
        assert_eq!(
            accent_selection(None, ThemeAppearance::Dark),
            AccentSelection::ThemeDefault
        );
        assert_eq!(
            accent_stored(AccentSelection::ThemeDefault, ThemeAppearance::Dark),
            None
        );
    }

    #[test]
    fn unmatched_pre_port_accent_falls_back_to_the_theme_accent() {
        let teal: Hsla = gpui::rgb(0x124b68).into();
        assert_eq!(
            accent_selection(Some(teal), ThemeAppearance::Dark),
            AccentSelection::ThemeDefault
        );
    }

    #[test]
    fn glass_flags_map_onto_zeron_surface_policies() {
        assert_eq!(surface_preference(false), SurfacePreference::Opaque);
        assert_eq!(surface_preference(true), SurfacePreference::Frosted);
    }

    #[test]
    fn persisted_font_sizes_snap_to_the_catalog() {
        assert_eq!(ui_size(16.0).pixels(), 16.0);
        // The pre-port slider persisted 8–32; zeron's catalog is 12–20, so the
        // stored extremes snap onto the catalog floor/ceiling.
        assert_eq!(ui_size(8.0).pixels(), 12.0);
        assert_eq!(ui_size(32.0).pixels(), 20.0);
        // A tie snaps down, matching `UiFontSize::normalized`.
        assert_eq!(ui_size(17.0).pixels(), 16.0);
    }

    #[test]
    fn concrete_choices_map_and_system_defers_to_the_window() {
        assert_eq!(
            choice_appearance(app_settings::Appearance::Light),
            Some(ThemeAppearance::Light)
        );
        assert_eq!(
            choice_appearance(app_settings::Appearance::Dark),
            Some(ThemeAppearance::Dark)
        );
        // `System` has no pure answer: `theme_appearance` resolves it from the
        // window (and the `MONO_DESK_APPEARANCE` pin can override it).
        assert_eq!(choice_appearance(app_settings::Appearance::System), None);
    }

    /// The install variant id comes from the persisted per-appearance choice;
    /// an empty id falls back to the built-in registry id so the installer
    /// never receives "".
    #[test]
    fn variant_ids_follow_the_persisted_selection_with_builtin_fallback() {
        let mut settings = AppSettings::default();
        assert_eq!(variant_id(&settings, ThemeAppearance::Light), "zeron-light");
        assert_eq!(variant_id(&settings, ThemeAppearance::Dark), "zeron-dark");
        assert_eq!(
            theme_selection(&settings),
            ThemeSelection {
                light: "zeron-light".into(),
                dark: "zeron-dark".into(),
            }
        );

        settings.theme_light = "family:custom-light".into();
        settings.theme_dark = "family:custom-dark".into();
        assert_eq!(
            variant_id(&settings, ThemeAppearance::Light),
            "family:custom-light"
        );
        assert_eq!(
            variant_id(&settings, ThemeAppearance::Dark),
            "family:custom-dark"
        );

        // A hand-built state with empty ids (T1's codec never produces one)
        // still installs a real variant.
        settings.theme_light.clear();
        settings.theme_dark.clear();
        assert_eq!(variant_id(&settings, ThemeAppearance::Light), "zeron-light");
        assert_eq!(variant_id(&settings, ThemeAppearance::Dark), "zeron-dark");
    }

    /// The wallpaper tint reaches the install only while the toggle is on and
    /// a managed background with a sampled color exists; every other state
    /// installs the plain selection.
    #[test]
    fn appearance_plan_gates_the_wallpaper_tint() {
        use crate::app_settings::{NewThreadBackground, NewThreadBackgroundAdjustment};
        use std::path::PathBuf;

        let mut settings = AppSettings::default();
        // No artwork yet.
        assert_eq!(
            appearance_plan(&settings, ThemeAppearance::Dark).wallpaper_color,
            None
        );

        settings.new_thread_background = Some(NewThreadBackground {
            path: PathBuf::from("/managed/new-thread-background.png"),
            name: "picked.png".into(),
            adjustment: NewThreadBackgroundAdjustment::default(),
        });
        // Toggle still off.
        assert_eq!(
            appearance_plan(&settings, ThemeAppearance::Dark).wallpaper_color,
            None
        );

        settings.wallpaper_theme_colors = true;
        // Toggle on but no sampled color yet.
        assert_eq!(
            appearance_plan(&settings, ThemeAppearance::Dark).wallpaper_color,
            None
        );

        let sampled = Color::rgb(30, 130, 220);
        settings.wallpaper_color = Some(sampled);
        assert_eq!(
            appearance_plan(&settings, ThemeAppearance::Dark).wallpaper_color,
            Some(sampled)
        );
        // The plan keeps the persisted variant behind the install.
        assert_eq!(
            appearance_plan(&settings, ThemeAppearance::Dark).variant_id,
            "zeron-dark"
        );

        // Turning the toggle off drops the tint again.
        settings.wallpaper_theme_colors = false;
        assert_eq!(
            appearance_plan(&settings, ThemeAppearance::Dark).wallpaper_color,
            None
        );
    }

    /// Boot typography seeds from the persisted words: `installed:` families
    /// survive, the UI size snaps onto zeron's catalog, and the terminal/code
    /// pixel sizes come from the file.
    #[test]
    fn typography_seed_uses_persisted_words_and_catalog() {
        use crate::persist::StoredSettings;

        let stored = StoredSettings {
            ui_font_family: "installed:Inter".into(),
            terminal_font_family: "system".into(),
            code_font_family: "xanh-mono".into(),
            font_size: 17.0,
            terminal_font_size: 15.0,
            code_font_size: 11.0,
            ..StoredSettings::default()
        };
        let seed = typography_seed(&AppSettings::from_stored(stored));
        assert_eq!(seed.ui_family, UiFontFamily::Installed("Inter".into()));
        assert_eq!(seed.terminal_family, UiFontFamily::System);
        assert_eq!(seed.code_family, UiFontFamily::XanhMono);
        // 17 px snaps down to zeron's 16 px catalog entry.
        assert_eq!(seed.ui_size.pixels(), 16.0);
        assert_eq!(seed.terminal_size, 15.0);
        assert_eq!(seed.code_size, 11.0);

        // An unknown persisted word loads as the key's bundled default.
        let junk = StoredSettings {
            ui_font_family: "future-font".into(),
            terminal_font_family: "installed:".into(),
            code_font_family: String::new(),
            ..StoredSettings::default()
        };
        let seed = typography_seed(&AppSettings::from_stored(junk));
        assert_eq!(seed.ui_family, UiFontFamily::MomoTrustSans);
        assert_eq!(seed.terminal_family, UiFontFamily::XanhMono);
        assert_eq!(seed.code_family, UiFontFamily::GoogleSansCode);
    }

    /// Boot motion seeds from the persisted reduced-motion word and pause
    /// switch; an unknown word follows the OS.
    #[test]
    fn motion_seed_uses_persisted_words() {
        use crate::persist::StoredSettings;

        let stored = StoredSettings {
            reduce_motion: "off".into(),
            pause_in_background: true,
            ..StoredSettings::default()
        };
        assert_eq!(
            motion_seed(&AppSettings::from_stored(stored)),
            (ReduceMotion::Off, true)
        );

        let junk = StoredSettings {
            reduce_motion: "sometimes".into(),
            ..StoredSettings::default()
        };
        assert_eq!(
            motion_seed(&AppSettings::from_stored(junk)),
            (ReduceMotion::System, false)
        );
    }
}
