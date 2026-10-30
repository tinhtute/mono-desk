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
mod finance;
mod general;
mod notifications;
mod profile;
mod reset;
pub(crate) mod text_field;

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

use zeron_theme::{AccentPreset, AccentSelection, SurfacePreference};
use zeron_ui::settings::widgets;
use zeron_ui::theme::{Appearance as ThemeAppearance, Theme};
use zeron_ui::typography::{self, UiFontSize};

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

/// Apply the live `AppSettings` to zeron's theme and the native window:
/// appearance mode (System resolves against the window), accent preset, surface
/// material and the rem-scaling font size, then re-point the window background
/// (blurred for frosted surfaces). Every Appearance-page control funnels here.
pub(crate) fn apply_appearance(window: &mut Window, cx: &mut App) {
    let settings = AppSettings::global(cx);
    let appearance =
        pinned_appearance().unwrap_or_else(|| theme_appearance(settings.appearance, window));
    let accent = accent_selection(settings.accent, appearance);
    let surface = surface_preference(settings.background_translucent);
    let font_size = ui_size(settings.font_size as f32);
    Theme::install_selection(
        appearance,
        builtin_variant_id(appearance),
        accent,
        surface,
        cx,
    );
    typography::set_font_size(font_size, window, cx);
    window.set_background_appearance(Theme::of(cx).window_background_appearance());
    cx.refresh_windows();
}

/// The built-in registry's variant id for `appearance`. mono-desk vendors
/// zeron's built-in variants only (the theme library/import machinery is out of
/// scope), and `Theme::for_selection` falls back to exactly these ids.
pub(crate) fn builtin_variant_id(appearance: ThemeAppearance) -> &'static str {
    if appearance.is_dark() {
        "zeron-dark"
    } else {
        "zeron-light"
    }
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
}
