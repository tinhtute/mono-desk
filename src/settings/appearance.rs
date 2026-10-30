//! The Appearance page (`settings-surface` T3): zeron's own custom Appearance
//! page (`crates/ui/src/settings/appearance.rs`, rev 8b7afb3) rebuilt on the
//! vendored kit — the three `option_card` mode cards with their miniature
//! previews, the light/dark theme-variant rows, the `accent_swatch` grid, the
//! "Glass" surface select, the motion row and the fonts/layout row.
//!
//! Dropped from upstream, per the port plan's NG2: the wallpaper folder and
//! background-image rows, the theme library/import rows, the cloud-only rows
//! and the settings-search anchors. Reduced for this app's persisted keys: the
//! "Fonts and layout" section keeps only the interface font size (zeron's
//! UI/terminal/code font pickers, transcript width and math-size rows have no
//! `AppSettings` field, NG4), and the motion section keeps only "Reduce
//! motion". Both reductions are deliberate and recorded in the plan's R4
//! amendment. Kept structurally: the page is the same column of
//! sections/rows/cards as upstream's render (appearance.rs:4238-4273), so a
//! visual diff against zeron's page reads as the same page.
//!
//! Wiring differs from upstream on purpose: zeron's `appearance::set_*` helpers
//! write into zeron's settings service, which mono-desk does not vendor. Every
//! control instead writes the `AppSettings` global, persists it, and funnels
//! through [`crate::settings::apply_appearance`] — the single theme path also
//! used by the shell's footer toggle and the window's OS-appearance sync.
//!
//! Not persisted (no `AppSettings` key, NG4): the reduce-motion preference and
//! the per-appearance theme-variant choice. The reduce-motion control drives
//! zeron's motion runtime live; the variant rows show the registry's single
//! built-in variant per appearance (the theme library is not ported).

use gpui::prelude::*;
use gpui::{
    AnyElement, Context, Hsla, IntoElement, Render, Role, ScrollHandle, SharedString, Window, div,
    px, relative,
};

use zeron_theme::{
    AccentPreset, AccentSelection, SurfacePreference, SurfaceTreatment, ThemeRegistry,
};
use zeron_ui::icons;
use zeron_ui::motion::{self, ReduceMotion};
use zeron_ui::settings::select::{SelectOption, SelectState, select};
use zeron_ui::settings::widgets::{self, OPTION_CARD_RADIUS};
use zeron_ui::theme::{Appearance as ThemeAppearance, Theme};
use zeron_ui::typography::UiFontSize;

use crate::app_settings::{AppSettings, Appearance as ModeChoice};
use crate::i18n::tr;
use crate::settings::{accent_selection, accent_stored};

/// zeron's mode order (`AppearanceMode::ALL`): System first, matching the app's
/// follow-the-OS default.
const MODES: [ModeChoice; 3] = [ModeChoice::System, ModeChoice::Light, ModeChoice::Dark];

pub struct AppearancePage {
    /// Open state of the two theme-variant selects (one per appearance).
    light_theme: SelectState,
    dark_theme: SelectState,
    /// Open state of the "Glass" surface select.
    surface: SelectState,
    /// Open state of the interface-font-size select.
    font_size: SelectState,
    /// Open state of the reduce-motion select.
    motion: SelectState,
    /// The page's scroll offset (zeron's pages own their scroll handle).
    scroll: ScrollHandle,
}

impl AppearancePage {
    pub fn new() -> Self {
        Self {
            light_theme: SelectState::default(),
            dark_theme: SelectState::default(),
            surface: SelectState::default(),
            font_size: SelectState::default(),
            motion: SelectState::default(),
            scroll: ScrollHandle::new(),
        }
    }

    fn set_mode(&mut self, mode: ModeChoice, window: &mut Window, cx: &mut Context<Self>) {
        AppSettings::global_mut(cx).appearance = mode;
        super::persist_now(cx);
        super::apply_appearance(window, cx);
        cx.notify();
    }

    fn set_accent(
        &mut self,
        selection: AccentSelection,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let appearance = Theme::of(cx).appearance;
        let stored = accent_stored(selection, appearance);
        AppSettings::global_mut(cx).accent = stored;
        super::persist_now(cx);
        super::apply_appearance(window, cx);
        cx.notify();
    }

    /// The persisted glass flags for a surface choice. `ThemeDefault` cannot be
    /// stored (mono-desk persists the flags, not zeron's preference enum), so
    /// it stores the treatment the current theme actually resolved to.
    fn surface_flags(preference: SurfacePreference, resolved: SurfaceTreatment) -> (bool, bool) {
        match preference {
            SurfacePreference::Frosted => (true, true),
            SurfacePreference::Opaque => (false, false),
            SurfacePreference::ThemeDefault => match resolved {
                SurfaceTreatment::Frosted => (true, true),
                SurfaceTreatment::Opaque => (false, false),
            },
        }
    }

    fn set_surface(&mut self, ix: usize, window: &mut Window, cx: &mut Context<Self>) {
        let preference = SurfacePreference::ALL[ix];
        let (translucent, blur) = Self::surface_flags(preference, Theme::of(cx).surface_treatment);
        {
            let settings = AppSettings::global_mut(cx);
            settings.background_translucent = translucent;
            settings.background_blur = blur;
        }
        super::persist_now(cx);
        super::apply_appearance(window, cx);
        cx.notify();
    }

    fn set_font_size(&mut self, ix: usize, window: &mut Window, cx: &mut Context<Self>) {
        let size = UiFontSize::ALL[ix];
        AppSettings::global_mut(cx).font_size = size.pixels() as f64;
        super::persist_now(cx);
        super::apply_appearance(window, cx);
        cx.notify();
    }

    fn set_reduced_motion(&mut self, ix: usize, cx: &mut Context<Self>) {
        motion::set_preference(ReduceMotion::ALL[ix], cx);
        cx.notify();
    }

    /// Re-apply the current selection after a theme-variant pick. The vendored
    /// registry has exactly one built-in variant per appearance, so this keeps
    /// the page's selects live instead of inert; imported variants would slot
    /// in here when the theme library lands.
    fn reapply_selection(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        super::apply_appearance(window, cx);
        cx.notify();
    }
}

fn mode_icon(mode: ModeChoice) -> &'static str {
    match mode {
        ModeChoice::System => icons::MONITOR,
        ModeChoice::Light => icons::SUN,
        ModeChoice::Dark => icons::MOON,
    }
}

/// The "Glass" select's label for a surface preference (zeron's
/// `surface_label`, localized from the i18n table).
fn surface_label(preference: SurfacePreference, lang: &str) -> &'static str {
    match preference {
        SurfacePreference::ThemeDefault => tr("appearance.surface.default", lang),
        SurfacePreference::Frosted => tr("appearance.surface.frosted", lang),
        SurfacePreference::Opaque => tr("appearance.surface.opaque", lang),
    }
}

/// The meta line under the "Glass" row: zeron's `surface_helper` names the
/// selected material and, for the inherited choice, the resolved one.
fn surface_helper(
    preference: SurfacePreference,
    resolved: SurfaceTreatment,
    lang: &str,
) -> SharedString {
    match preference {
        SurfacePreference::ThemeDefault => SharedString::from(format!(
            "{} · {}",
            tr("appearance.surface.default", lang),
            match resolved {
                SurfaceTreatment::Frosted => tr("appearance.surface.frosted", lang),
                SurfaceTreatment::Opaque => tr("appearance.surface.opaque", lang),
            }
        )),
        other => SharedString::from(surface_label(other, lang)),
    }
}

fn bar(fraction: f32, tone: Hsla) -> gpui::Div {
    div()
        .h(px(5.0))
        .w(relative(fraction))
        .rounded(px(3.0))
        .bg(tone)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Corners {
    All,
    Left,
    Right,
}

/// zeron's miniature window mock (appearance.rs:1589-1637): a sidebar column of
/// text bars beside a bordered content pane, painted from a resolved theme so a
/// card previews the appearance it selects.
fn miniature(theme: &Theme, corners: Corners) -> AnyElement {
    let line = theme.text.opacity(0.22);
    let strong = theme.text.opacity(0.34);
    let r = px(OPTION_CARD_RADIUS);
    let root = div().size_full().flex().flex_row().bg(theme.surface);
    let root = match corners {
        Corners::All => root.rounded(r),
        Corners::Left => root.rounded_tl(r).rounded_bl(r),
        Corners::Right => root.rounded_tr(r).rounded_br(r),
    };
    root.child(
        div()
            .w(px(44.0))
            .h_full()
            .flex_none()
            .overflow_hidden()
            .flex()
            .flex_col()
            .gap(px(7.0))
            .px(px(8.0))
            .pt(px(14.0))
            .child(bar(0.70, strong))
            .child(bar(1.0, line))
            .child(bar(0.85, line))
            .child(bar(1.0, line)),
    )
    .child(
        div()
            .flex_1()
            .min_w_0()
            .my(px(8.0))
            .mr(px(8.0))
            .rounded(px(6.0))
            .border_1()
            .border_color(theme.border)
            .bg(theme.bg)
            .overflow_hidden()
            .flex()
            .flex_col()
            .gap(px(7.0))
            .p(px(10.0))
            .child(bar(0.62, strong))
            .child(bar(0.88, line))
            .child(bar(0.76, line))
            .child(bar(0.52, line)),
    )
    .into_any_element()
}

/// The resolved theme for one appearance, from the built-in registry
/// (`ThemeSelection::default()`'s ids), so previews honor the chosen accent and
/// surface just like upstream's `miniature_split` (appearance.rs:1638-1665).
fn preview_theme(
    appearance: ThemeAppearance,
    accent: AccentSelection,
    surface: SurfacePreference,
) -> Theme {
    Theme::for_selection(
        appearance,
        super::builtin_variant_id(appearance),
        accent,
        surface,
    )
}

fn mode_preview(
    mode: ModeChoice,
    accent: AccentSelection,
    surface: SurfacePreference,
) -> AnyElement {
    match mode {
        ModeChoice::Light => miniature(
            &preview_theme(ThemeAppearance::Light, accent, surface),
            Corners::All,
        ),
        ModeChoice::Dark => miniature(
            &preview_theme(ThemeAppearance::Dark, accent, surface),
            Corners::All,
        ),
        ModeChoice::System => div()
            .size_full()
            .flex()
            .flex_row()
            .child(div().w_1_2().h_full().overflow_hidden().child(miniature(
                &preview_theme(ThemeAppearance::Light, accent, surface),
                Corners::Left,
            )))
            .child(div().w_1_2().h_full().overflow_hidden().child(miniature(
                &preview_theme(ThemeAppearance::Dark, accent, surface),
                Corners::Right,
            )))
            .into_any_element(),
    }
}

/// The accent choices in zeron's order: the theme's own accent first, then the
/// seven presets (`appearance.rs:3925-3956`).
fn accent_choices() -> Vec<AccentSelection> {
    let mut choices = vec![AccentSelection::ThemeDefault];
    choices.extend(AccentPreset::ALL.into_iter().map(AccentSelection::Preset));
    choices
}

impl Render for AppearancePage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::of(cx).for_settings_surface();
        let lang = AppSettings::global(cx).language.clone();
        let settings = AppSettings::global(cx);
        let mode = settings.appearance;
        let stored_accent = settings.accent;
        let translucent = settings.background_translucent;
        let font_size = settings.font_size as f32;
        let surface_preference = super::surface_preference(translucent);
        let current_appearance = Theme::of(cx).appearance;
        let accent = accent_selection(stored_accent, current_appearance);
        let reduced_motion = motion::reduced_motion(cx);

        // ---- Color scheme: the three mode cards -------------------------
        let cards = MODES
            .into_iter()
            .map(|choice| {
                let selected = choice == mode;
                let selection_t = widgets::tab_selection_t(
                    window,
                    format!("appearance-mode-{}-selection", choice.as_str()),
                    selected,
                    reduced_motion,
                );
                widgets::option_card(
                    &theme,
                    mode_icon(choice),
                    tr(choice.label_key(), &lang),
                    selected,
                    selection_t,
                    mode_preview(choice, accent, surface_preference),
                )
                .id(SharedString::from(format!(
                    "appearance-{}",
                    choice.as_str()
                )))
                .tab_index(0)
                .role(Role::Button)
                .focus_visible(|style| style.border_2().border_color(theme.accent).opacity(1.0))
                .on_click(cx.listener(move |page, _, window, cx| page.set_mode(choice, window, cx)))
            })
            .collect::<Vec<_>>();

        // ---- Theme-variant rows (one select per appearance) -------------
        let registry = ThemeRegistry::builtin();
        let variants_for = |appearance: zeron_theme::Appearance| {
            registry
                .variants_for(appearance)
                .map(|variant| SharedString::from(variant.name.clone()))
                .collect::<Vec<_>>()
        };
        let light_variants = variants_for(zeron_theme::Appearance::Light);
        let dark_variants = variants_for(zeron_theme::Appearance::Dark);
        let light_theme = select(
            "appearance-light-theme",
            tr("appearance.light_theme", &lang),
            &theme,
            |page: &mut Self| &mut page.light_theme,
        )
        .options(
            light_variants
                .iter()
                .map(|name| SelectOption::new(name.clone())),
            0,
        )
        .width(168.0)
        .on_select(|page: &mut Self, _, window, cx| page.reapply_selection(window, cx))
        .render(&self.light_theme, cx);
        let dark_theme = select(
            "appearance-dark-theme",
            tr("appearance.dark_theme", &lang),
            &theme,
            |page: &mut Self| &mut page.dark_theme,
        )
        .options(
            dark_variants
                .iter()
                .map(|name| SelectOption::new(name.clone())),
            0,
        )
        .width(168.0)
        .on_select(|page: &mut Self, _, window, cx| page.reapply_selection(window, cx))
        .render(&self.dark_theme, cx);

        // ---- Accent row -------------------------------------------------
        let swatches = accent_choices()
            .into_iter()
            .map(|selection| {
                let selected = selection == accent;
                widgets::accent_swatch(&theme, selection, selected)
                    .tab_index(0)
                    .role(Role::Button)
                    .focus_visible(|style| style.border_2().border_color(theme.accent).opacity(1.0))
                    .on_click(cx.listener(move |page, _, window, cx| {
                        page.set_accent(selection, window, cx)
                    }))
            })
            .collect::<Vec<_>>();

        let color_rows = widgets::section_card(&theme)
            .mt_0()
            .child(
                widgets::card_row(&theme, true)
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(160.0))
                            .child(widgets::row_title(
                                &theme,
                                tr("appearance.light_theme", &lang),
                            ))
                            .child(widgets::meta_line(
                                &theme,
                                vec![
                                    div()
                                        .child(tr("appearance.light_theme", &lang))
                                        .into_any_element(),
                                ],
                            )),
                    )
                    .child(light_theme),
            )
            .child(
                widgets::card_row(&theme, false)
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(160.0))
                            .child(widgets::row_title(
                                &theme,
                                tr("appearance.dark_theme", &lang),
                            ))
                            .child(widgets::meta_line(
                                &theme,
                                vec![
                                    div()
                                        .child(tr("appearance.dark_theme", &lang))
                                        .into_any_element(),
                                ],
                            )),
                    )
                    .child(dark_theme),
            )
            .child(
                widgets::card_row(&theme, false)
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(160.0))
                            .child(widgets::row_title(&theme, tr("appearance.accent", &lang)))
                            .child(widgets::meta_line(
                                &theme,
                                vec![
                                    div()
                                        .child(tr("appearance.accent.desc", &lang))
                                        .into_any_element(),
                                ],
                            )),
                    )
                    .child(
                        div()
                            .max_w_full()
                            .flex()
                            .flex_wrap()
                            .items_center()
                            .gap(px(8.0))
                            .children(swatches),
                    ),
            );

        // ---- Material and background: the "Glass" select ----------------
        let surface_ix = SurfacePreference::ALL
            .into_iter()
            .position(|preference| preference == surface_preference)
            .unwrap_or_default();
        let surface = select(
            "appearance-surface",
            tr("appearance.glass", &lang),
            &theme,
            |page: &mut Self| &mut page.surface,
        )
        .options(
            SurfacePreference::ALL
                .into_iter()
                .map(|preference| SelectOption::new(surface_label(preference, &lang))),
            surface_ix,
        )
        .width(168.0)
        .on_select(|page: &mut Self, ix, window, cx| page.set_surface(ix, window, cx))
        .render(&self.surface, cx);
        let material_rows = widgets::section_card(&theme).mt_0().child(
            widgets::card_row(&theme, true)
                .child(
                    div()
                        .flex_1()
                        .min_w(px(160.0))
                        .child(widgets::row_title(&theme, tr("appearance.glass", &lang)))
                        .child(widgets::meta_line(
                            &theme,
                            vec![
                                div()
                                    .child(surface_helper(
                                        surface_preference,
                                        Theme::of(cx).surface_treatment,
                                        &lang,
                                    ))
                                    .into_any_element(),
                            ],
                        )),
                )
                .child(surface),
        );

        // ---- Motion: reduce-motion preference (zeron's select row) ------
        let motion_ix = ReduceMotion::ALL
            .into_iter()
            .position(|preference| preference == motion::preference(cx))
            .unwrap_or_default();
        let reduced_motion_control = select(
            "appearance-reduce-motion",
            tr("appearance.motion.reduce", &lang),
            &theme,
            |page: &mut Self| &mut page.motion,
        )
        .options(
            ReduceMotion::ALL
                .into_iter()
                .map(|preference| SelectOption::new(preference.label())),
            motion_ix,
        )
        .width(168.0)
        .on_select(|page: &mut Self, ix, _window, cx| page.set_reduced_motion(ix, cx))
        .render(&self.motion, cx);
        let motion_rows = widgets::section_card(&theme).mt_0().child(
            widgets::card_row(&theme, true)
                .child(
                    div()
                        .flex_1()
                        .min_w(px(160.0))
                        .child(widgets::row_title(
                            &theme,
                            tr("appearance.motion.reduce", &lang),
                        ))
                        .child(widgets::meta_line(
                            &theme,
                            vec![
                                div()
                                    .child(tr("appearance.motion.reduce.desc", &lang))
                                    .into_any_element(),
                            ],
                        )),
                )
                .child(reduced_motion_control),
        );

        // ---- Fonts and layout: interface font size ---------------------
        let size_ix = UiFontSize::ALL
            .into_iter()
            .position(|size| (size.pixels() - font_size).abs() < 0.01)
            .unwrap_or_else(|| {
                UiFontSize::ALL
                    .into_iter()
                    .position(|size| size == super::ui_size(font_size))
                    .unwrap_or_default()
            });
        let font_size_control = select(
            "appearance-font-size",
            tr("item.font_size", &lang),
            &theme,
            |page: &mut Self| &mut page.font_size,
        )
        .options(
            UiFontSize::ALL
                .into_iter()
                .map(|size| SelectOption::new(size.label())),
            size_ix,
        )
        .width(168.0)
        .on_select(|page: &mut Self, ix, window, cx| page.set_font_size(ix, window, cx))
        .render(&self.font_size, cx);
        let font_rows = widgets::section_card(&theme).mt_0().child(
            widgets::card_row(&theme, true)
                .child(
                    div()
                        .flex_1()
                        .min_w(px(160.0))
                        .child(widgets::row_title(&theme, tr("item.font_size", &lang)))
                        .child(widgets::meta_line(
                            &theme,
                            vec![
                                div()
                                    .child(tr("item.font_size.desc", &lang))
                                    .into_any_element(),
                            ],
                        )),
                )
                .child(font_size_control),
        );

        div()
            .id("appearance-settings-page")
            .size_full()
            .overflow_y_scroll()
            .track_scroll(&self.scroll)
            .child(
                widgets::page_column()
                    .child(widgets::page_header(
                        &theme,
                        tr("page.appearance", &lang),
                        None,
                    ))
                    .child(
                        widgets::section(
                            &theme,
                            tr("appearance.scheme", &lang),
                            widgets::option_card_row().children(cards).block(),
                        )
                        .mt(px(24.0))
                        .gap(px(12.0)),
                    )
                    .child(color_rows.mt(px(16.0)))
                    .child(widgets::section(
                        &theme,
                        tr("appearance.material", &lang),
                        material_rows,
                    ))
                    .child(widgets::section(
                        &theme,
                        tr("appearance.motion", &lang),
                        motion_rows,
                    ))
                    .child(widgets::section(
                        &theme,
                        tr("appearance.fonts", &lang),
                        font_rows,
                    )),
            )
    }
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
    fn surface_select_lists_zerons_three_policies() {
        assert_eq!(SurfacePreference::ALL.len(), 3);
        for preference in SurfacePreference::ALL {
            assert!(!surface_label(preference, "en").is_empty());
        }
        assert_eq!(
            surface_label(SurfacePreference::Opaque, "en"),
            "Solid surfaces"
        );
    }

    #[test]
    fn theme_default_glass_stores_the_resolved_treatment() {
        assert_eq!(
            AppearancePage::surface_flags(SurfacePreference::Frosted, SurfaceTreatment::Opaque),
            (true, true)
        );
        assert_eq!(
            AppearancePage::surface_flags(SurfacePreference::Opaque, SurfaceTreatment::Frosted),
            (false, false)
        );
        assert_eq!(
            AppearancePage::surface_flags(
                SurfacePreference::ThemeDefault,
                SurfaceTreatment::Frosted
            ),
            (true, true)
        );
        assert_eq!(
            AppearancePage::surface_flags(
                SurfacePreference::ThemeDefault,
                SurfaceTreatment::Opaque
            ),
            (false, false)
        );
    }

    #[test]
    fn accent_choices_start_with_the_theme_accent() {
        let choices = accent_choices();
        assert_eq!(choices.len(), AccentPreset::ALL.len() + 1);
        assert_eq!(choices[0], AccentSelection::ThemeDefault);
        assert_eq!(choices[1], AccentSelection::Preset(AccentPreset::ALL[0]));
    }
}
