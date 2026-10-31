//! The Appearance page's Color scheme section: the three mode cards, the
//! Light/Dark theme selectors over the active registry (with palette
//! previews), the accent row and the "Match wallpaper colors" row.
//!
//! Ported from zeron's `crates/ui/src/settings/appearance.rs` (MIT, rev
//! 8b7afb3): mode cards :3562-3589 and :4238-4251, theme selectors
//! :2144-2203 and :3591-3614, accent row :3616-3680, wallpaper-colors row
//! :3682-3704, helpers `accent_helper` :1534-1539 and `palette_preview`
//! :1692-1705 (the latter lives in [`super::preview`] with the other preview
//! helpers).
//!
//! Adaptations: zeron reads its appearance service (`appearance::mode/th...`)
//! and writes through `appearance::set_*`; mono-desk reads the `AppSettings`
//! global and persists/handles through the page view. Search anchors
//! (`search_anchor`) are not ported — the settings-search index lands with its
//! own phase. Row labels and helper sentences come from `crate::i18n::tr`.

use gpui::prelude::*;
use gpui::{AnyElement, Context, Role, SharedString, Toggled, Window, div, px};

use zeron_theme::{
    AccentPreset, AccentSelection, SurfacePreference, ThemeRegistry, ThemeSelection,
};
use zeron_ui::icons;
use zeron_ui::settings::select::{SelectOption, SelectState, select};
use zeron_ui::settings::widgets;
use zeron_ui::theme::{Appearance as ThemeAppearance, Theme};

use crate::app_settings::Appearance as ModeChoice;
use crate::i18n::tr;

use super::AppearancePage;
use super::preview::{self, palette_preview};

/// zeron's mode order (`AppearanceMode::ALL`, `crates/ui/src/appearance.rs:65`):
/// System first, matching the app's follow-the-OS default.
pub(super) const MODES: [ModeChoice; 3] = [ModeChoice::System, ModeChoice::Light, ModeChoice::Dark];

/// The Color scheme section's resolved state for one frame: the persisted
/// values the rows read, snapshotted from the global by the page's render.
pub(super) struct ColorState {
    pub(super) mode: ModeChoice,
    pub(super) accent: AccentSelection,
    pub(super) surface: SurfacePreference,
    pub(super) themes: ThemeSelection,
    pub(super) wallpaper_colors: bool,
}

/// zeron names the mode card/selection ids from `AppearanceMode::label()`
/// (`appearance-System`, `appearance-mode-System-selection`,
/// `crates/ui/src/settings/appearance.rs:3566-3580`). Those words are
/// locale-independent, unlike the persisted `system`/`light`/`dark` words.
fn mode_id_word(mode: ModeChoice) -> &'static str {
    match mode {
        ModeChoice::System => "System",
        ModeChoice::Light => "Light",
        ModeChoice::Dark => "Dark",
    }
}

/// zeron `AppearanceMode::icon` (`crates/ui/src/appearance.rs:51-58`).
fn mode_icon(mode: ModeChoice) -> &'static str {
    match mode {
        ModeChoice::System => icons::MONITOR,
        ModeChoice::Light => icons::SUN,
        ModeChoice::Dark => icons::MOON,
    }
}

/// The accent choices in zeron's order: the theme's own accent first, then the
/// seven presets (upstream :3616-3617 and `accent_choices` :3925-3956).
fn accent_choices() -> Vec<AccentSelection> {
    let mut choices = vec![AccentSelection::ThemeDefault];
    choices.extend(AccentPreset::ALL.into_iter().map(AccentSelection::Preset));
    choices
}

/// The accent row's meta line: zeron's `accent_helper`, localized (upstream
/// :1534-1539); when wallpaper colors are enabled zeron swaps in the notice
/// (upstream :3663-3665).
fn accent_helper(accent: AccentSelection, lang: &str) -> &'static str {
    match accent {
        AccentSelection::ThemeDefault => tr("appearance.accent.default", lang),
        AccentSelection::Preset(preset) => preset.label(),
    }
}

/// The aria label for one accent swatch: the preset's own name (zeron keeps
/// color names untranslated), or the localized "Theme default" word.
fn accent_aria(selection: AccentSelection, lang: &str) -> &'static str {
    match selection {
        AccentSelection::ThemeDefault => tr("appearance.accent.default", lang),
        AccentSelection::Preset(preset) => preset.label(),
    }
}

/// The registry's variant ids and names for one appearance, plus the select
/// index of the persisted id. The index falls back to the first variant when a
/// stored id is unknown; the boot reconcile heals those before the first
/// renderer pass (see `settings::reconcile_persisted_selection`).
fn registry_variants(
    appearance: ThemeAppearance,
    themes: &ThemeSelection,
) -> (Vec<(String, String)>, usize) {
    let registry = ThemeRegistry::active();
    let selected_id = themes
        .variant_id(super::super::model_appearance(appearance))
        .to_owned();
    let variants: Vec<(String, String)> = registry
        .variants_for(super::super::model_appearance(appearance))
        .map(|variant| (variant.id.clone(), variant.name.clone()))
        .collect();
    let selected = variants
        .iter()
        .position(|(id, _)| *id == selected_id)
        .unwrap_or_default();
    (variants, selected)
}

/// One Light/Dark theme selector (upstream `render_theme_selector`,
/// :2144-2203): every active-registry variant for that appearance, each option
/// led by its palette preview, width 218 / menu 260 / themed heading.
fn theme_selector(
    appearance: ThemeAppearance,
    state: &SelectState,
    theme: &Theme,
    themes: &ThemeSelection,
    surface: SurfacePreference,
    lang: &str,
    cx: &mut Context<AppearancePage>,
) -> AnyElement {
    let (variants, selected) = registry_variants(appearance, themes);
    let options = variants.iter().map(|(id, name)| {
        let sample = Theme::for_selection(appearance, id, AccentSelection::ThemeDefault, surface);
        SelectOption::new(SharedString::from(name.clone()))
            .leading(move || palette_preview(&sample).into_any_element())
    });
    let ids: Vec<String> = variants.iter().map(|(id, _)| id.clone()).collect();
    let (slug, label, heading) = if appearance.is_light() {
        (
            "light",
            tr("appearance.light_theme", lang),
            tr("appearance.light_themes", lang),
        )
    } else {
        (
            "dark",
            tr("appearance.dark_theme", lang),
            tr("appearance.dark_themes", lang),
        )
    };
    select(
        format!("{slug}-theme-selector"),
        label,
        theme,
        move |page: &mut AppearancePage| match appearance {
            ThemeAppearance::Light => &mut page.light_theme,
            ThemeAppearance::Dark => &mut page.dark_theme,
        },
    )
    .options(options, selected)
    .width(218.0)
    .menu_width(260.0)
    .heading(heading)
    .on_select(move |page: &mut AppearancePage, ix, window, cx| {
        if let Some(id) = ids.get(ix) {
            page.set_theme_variant(appearance, SharedString::from(id.clone()), window, cx);
        }
    })
    .render(state, cx)
    .into_any_element()
}

/// The mode cards (upstream :3562-3589): one `option_card` per mode, with the
/// resolved theme preview, the retargetable selection fade and button roles.
pub(super) fn mode_cards(
    theme: &Theme,
    state: &ColorState,
    lang: &str,
    window: &mut Window,
    cx: &mut Context<AppearancePage>,
) -> Vec<AnyElement> {
    let reduced_motion = zeron_ui::motion::reduced_motion(cx);
    MODES
        .into_iter()
        .map(|choice| {
            let selected = choice == state.mode;
            let selection_t = widgets::tab_selection_t(
                window,
                format!("appearance-mode-{}-selection", mode_id_word(choice)),
                selected,
                reduced_motion,
            );
            widgets::option_card(
                theme,
                mode_icon(choice),
                tr(choice.label_key(), lang),
                selected,
                selection_t,
                preview::preview(choice, &state.themes, state.accent, state.surface),
            )
            .id(SharedString::from(format!(
                "appearance-{}",
                mode_id_word(choice)
            )))
            .tab_index(0)
            .role(Role::Button)
            .focus_visible(|style| style.border_2().border_color(theme.accent).opacity(1.0))
            .on_click(cx.listener(move |page, _, window, cx| page.set_mode(choice, window, cx)))
            .into_any_element()
        })
        .collect()
}

/// The color card's rows (upstream :3591-3704): Light theme, Dark theme,
/// Accent color, Match wallpaper colors.
pub(super) fn color_rows(
    page: &mut AppearancePage,
    theme: &Theme,
    state: &ColorState,
    lang: &str,
    cx: &mut Context<AppearancePage>,
) -> Vec<AnyElement> {
    let mut rows: Vec<AnyElement> = Vec::with_capacity(4);
    for (index, appearance) in [ThemeAppearance::Light, ThemeAppearance::Dark]
        .into_iter()
        .enumerate()
    {
        let (label, settings) = if appearance.is_light() {
            (tr("appearance.light_theme", lang), &page.light_theme)
        } else {
            (tr("appearance.dark_theme", lang), &page.dark_theme)
        };
        let selector = theme_selector(
            appearance,
            settings,
            theme,
            &state.themes,
            state.surface,
            lang,
            cx,
        );
        rows.push(
            widgets::card_row(theme, index == 0)
                .child(
                    div()
                        .flex_1()
                        .min_w(px(160.0))
                        .child(widgets::row_title(theme, label)),
                )
                .child(selector)
                .into_any_element(),
        );
    }

    let accent_controls = accent_choices()
        .into_iter()
        .map(|selection| {
            let selected = selection == state.accent;
            widgets::accent_swatch(theme, selection, selected)
                .tab_index(0)
                .role(Role::Button)
                .aria_label(accent_aria(selection, lang))
                .focus_visible(|style| style.border_2().border_color(theme.accent).opacity(1.0))
                .on_click(
                    cx.listener(move |page, _, window, cx| page.set_accent(selection, window, cx)),
                )
        })
        .collect::<Vec<_>>();
    let accent_meta: SharedString = if state.wallpaper_colors {
        SharedString::from(tr("appearance.accent.wallpaper_notice", lang))
    } else {
        SharedString::from(accent_helper(state.accent, lang))
    };
    rows.push(
        widgets::card_row(theme, false)
            .child(
                div()
                    .flex_1()
                    .min_w(px(160.0))
                    .child(widgets::row_title(theme, tr("appearance.accent", lang)))
                    .child(widgets::meta_line(
                        theme,
                        vec![div().child(accent_meta).into_any_element()],
                    )),
            )
            .child(
                div()
                    .max_w_full()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap(px(8.0))
                    .children(accent_controls),
            )
            .into_any_element(),
    );

    let match_wallpaper = state.wallpaper_colors;
    rows.push(
        widgets::card_row(theme, false)
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .child(widgets::row_title(
                        theme,
                        tr("appearance.wallpaper_colors", lang),
                    ))
                    .child(widgets::meta_line(
                        theme,
                        vec![
                            div()
                                .child(tr("appearance.wallpaper_colors.desc", lang))
                                .into_any_element(),
                        ],
                    )),
            )
            .child(
                widgets::toggle_switch(theme, match_wallpaper, "wallpaper-theme-colors")
                    .id("wallpaper-theme-colors-toggle")
                    .tab_index(0)
                    .role(Role::Switch)
                    .aria_label(tr("appearance.wallpaper_colors.aria", lang))
                    .focus_visible(|style| style.border_2().border_color(theme.accent))
                    .cursor_pointer()
                    .aria_toggled(if match_wallpaper {
                        Toggled::True
                    } else {
                        Toggled::False
                    })
                    .on_click(cx.listener(move |page, _, window, cx| {
                        page.set_wallpaper_colors(!match_wallpaper, window, cx)
                    })),
            )
            .into_any_element(),
    );

    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_mode_gets_a_card_and_a_glyph() {
        assert_eq!(MODES.len(), 3);
        for mode in MODES {
            assert!(!mode_icon(mode).is_empty());
            assert!(tr(mode.label_key(), "en") != mode.label_key());
        }
    }

    #[test]
    fn accent_choices_start_with_the_theme_accent() {
        let choices = accent_choices();
        assert_eq!(choices.len(), AccentPreset::ALL.len() + 1);
        assert_eq!(choices[0], AccentSelection::ThemeDefault);
        assert_eq!(choices[1], AccentSelection::Preset(AccentPreset::ALL[0]));
    }

    #[test]
    fn the_registry_lists_both_appearances_by_variant_id() {
        let registry = ThemeRegistry::active();
        for appearance in [ThemeAppearance::Light, ThemeAppearance::Dark] {
            let model = super::super::super::model_appearance(appearance);
            assert!(
                registry.variants_for(model).next().is_some(),
                "no registry variant for {appearance:?}"
            );
        }
        let themes = ThemeSelection::default();
        for appearance in [ThemeAppearance::Light, ThemeAppearance::Dark] {
            let (variants, selected) = registry_variants(appearance, &themes);
            assert!(!variants.is_empty());
            assert_eq!(
                variants[selected].0,
                themes.variant_id(super::super::super::model_appearance(appearance))
            );
        }
    }

    #[test]
    fn accent_helper_names_the_theme_default_and_each_preset() {
        // R2 acceptance: the accent row's dynamic meta distinguishes the
        // theme's own accent from a picked preset in every locale, without
        // emitting the raw i18n key.
        for lang in ["en", "vi", "zh-CN", "ja"] {
            let theme_default = accent_helper(AccentSelection::ThemeDefault, lang);
            assert_ne!(theme_default, "appearance.accent.default");
            assert!(!theme_default.is_empty());
            for preset in AccentPreset::ALL {
                let selection = AccentSelection::Preset(preset);
                assert_eq!(accent_helper(selection, lang), preset.label());
                assert_eq!(accent_aria(selection, lang), preset.label());
            }
            assert_eq!(
                accent_aria(AccentSelection::ThemeDefault, lang),
                theme_default
            );
        }
    }
}
