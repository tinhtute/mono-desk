//! Pure font-picker model for the Fonts and layout section (phase
//! `fonts-layout`): `FontKind`, the terminal/code size ladders, search
//! filtering and highlight stepping.
//!
//! Ported from zeron's `crates/ui/src/settings/appearance.rs` (MIT, rev
//! 8b7afb3): `FontKind` and its accessors at :129-296 (the
//! `AppearanceSettingsEvent` beside them is not ported — mono-desk has no open
//! editors to notify), the stepping/filtering helpers at :1445-1524.
//!
//! Adaptations: zeron's `SettingId` anchors are not vendored (the settings
//! search index lands with its own phase), and mono-desk's bundled mono is
//! Google Sans Code, so [`fallback_selection`] names it where upstream named
//! GeistMono. Labels/descriptions resolve through [`crate::i18n::tr`] (zeron is
//! English-only); sizes stay numeric labels in every locale.
//!
//! The module is pure except for the `&App`/`&mut App` typography accessors,
//! so everything else is testable headlessly.

use gpui::{App, SharedString};

use zeron_ui::popover;
use zeron_ui::typography::{self, FontAvailability, UiFontFamily, UiFontSize};

use crate::i18n::tr;

/// The three independently configurable font slots. Interface and code/diff
/// draw from the whole catalog — proportional faces are legal there. The
/// terminal draws from the fixed-width subset only (upstream :129-137).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum FontKind {
    Ui,
    Terminal,
    Code,
}

impl FontKind {
    pub(super) const ALL: [Self; 3] = [Self::Ui, Self::Terminal, Self::Code];

    pub(super) fn slug(self) -> &'static str {
        match self {
            Self::Ui => "interface",
            Self::Terminal => "terminal",
            Self::Code => "code",
        }
    }

    /// The row label's i18n key.
    pub(super) fn label_key(self) -> &'static str {
        match self {
            Self::Ui => "appearance.fonts.interface",
            Self::Terminal => "appearance.fonts.terminal",
            Self::Code => "appearance.fonts.code",
        }
    }

    pub(super) fn label(self, lang: &str) -> &'static str {
        tr(self.label_key(), lang)
    }

    /// The muted meta line under the row title.
    pub(super) fn description(self, lang: &str) -> &'static str {
        match self {
            Self::Ui => tr("appearance.fonts.interface.desc", lang),
            Self::Terminal => tr("appearance.fonts.terminal.desc", lang),
            Self::Code => tr("appearance.fonts.code.desc", lang),
        }
    }

    /// The size select's aria label.
    pub(super) fn size_label(self, lang: &str) -> &'static str {
        match self {
            Self::Ui => tr("appearance.fonts.interface.size", lang),
            Self::Terminal => tr("appearance.fonts.terminal.size", lang),
            Self::Code => tr("appearance.fonts.code.size", lang),
        }
    }

    /// The catalog this slot may pick from. Only the terminal narrows: its
    /// renderer positions cursor, selection, and hit-testing on an `m`-wide
    /// cell grid, which a proportional family silently breaks (upstream
    /// :180-188).
    pub(super) fn choices_for(self, availability: &FontAvailability) -> &[UiFontFamily] {
        match self {
            Self::Terminal => availability.fixed_width_choices(),
            _ => availability.choices(),
        }
    }

    pub(super) fn is_available_for(
        self,
        availability: &FontAvailability,
        family: &UiFontFamily,
    ) -> bool {
        match self {
            Self::Terminal => availability.is_fixed_width_available(family),
            _ => availability.is_available(family),
        }
    }

    pub(super) fn requested(self, cx: &App) -> UiFontFamily {
        match self {
            Self::Ui => typography::requested(cx),
            Self::Terminal => typography::terminal_requested(cx),
            Self::Code => typography::code_requested(cx),
        }
    }

    pub(super) fn effective(self, cx: &App) -> UiFontFamily {
        match self {
            Self::Ui => typography::effective(cx),
            Self::Terminal => typography::terminal_effective(cx),
            Self::Code => typography::code_effective(cx),
        }
    }

    /// Apply this slot's family. The vendored setters only update the
    /// typography state; the caller persists and reinstalls the theme
    /// (`crate::settings::apply_appearance`), which upstream did inside its
    /// settings service.
    pub(super) fn apply_family(self, family: UiFontFamily, cx: &mut App) -> bool {
        match self {
            Self::Ui => typography::set_family(family, cx),
            Self::Terminal => typography::set_terminal_family(family, cx),
            Self::Code => typography::set_code_family(family, cx),
        }
    }

    pub(super) fn pixel_size(self, cx: &App) -> f32 {
        match self {
            Self::Ui => typography::font_size(cx).pixels(),
            Self::Terminal => typography::terminal_font_size(cx),
            Self::Code => typography::code_font_size(cx),
        }
    }

    /// Labels for this kind's size ladder, in ladder order. Both ladders read
    /// as plain pixel values, so all three dropdowns look the same (upstream
    /// :229-239).
    pub(super) fn size_labels(self) -> Vec<SharedString> {
        match self {
            Self::Ui => UiFontSize::ALL.iter().map(|size| size.label()).collect(),
            _ => MONO_FONT_SIZES
                .iter()
                .map(|size| SharedString::from(format_px(*size)))
                .collect(),
        }
    }

    pub(super) fn size_count(self) -> usize {
        match self {
            Self::Ui => UiFontSize::ALL.len(),
            _ => MONO_FONT_SIZES.len(),
        }
    }

    /// Ladder position of the committed size. Terminal and code sizes are
    /// stored as free pixels (older settings, hand-edited files), so they snap
    /// to the nearest rung rather than falling off the list (upstream
    /// :248-259).
    pub(super) fn size_ix(self, cx: &App) -> usize {
        match self {
            Self::Ui => UiFontSize::ALL
                .iter()
                .position(|size| *size == typography::font_size(cx))
                .unwrap_or_default(),
            _ => nearest_mono_ix(self.pixel_size(cx)),
        }
    }
}

/// Pixel ladder behind the terminal and code size dropdowns. Both defaults
/// (terminal 13, code 12.5) are rungs, so today's rendering is reachable
/// (upstream :262-264).
pub(super) const MONO_FONT_SIZES: [f32; 10] =
    [10.0, 11.0, 12.0, 12.5, 13.0, 14.0, 15.0, 16.0, 18.0, 20.0];

/// "Math size" choices, in percent of the text around a formula (upstream
/// :266-267); consumed by the layout rows owner.
pub(super) const MATH_SIZES: [u16; 6] = [90, 100, 110, 120, 135, 150];

/// Ladder position of a stored math size; a hand-edited value in range but
/// off the ladder snaps to the nearest rung (upstream :269-278).
pub(super) fn nearest_math_size_ix(size: u16) -> usize {
    MATH_SIZES
        .iter()
        .enumerate()
        .min_by_key(|(_, rung)| rung.abs_diff(size))
        .map(|(ix, _)| ix)
        .unwrap_or_default()
}

pub(super) fn nearest_mono_ix(size: f32) -> usize {
    MONO_FONT_SIZES
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| (**a - size).abs().total_cmp(&(**b - size).abs()))
        .map(|(ix, _)| ix)
        .unwrap_or_default()
}

/// Step the highlight one row, skipping unavailable families and clamping at
/// the ends (upstream :1445-1468).
pub(super) fn step_font(
    current: &UiFontFamily,
    delta: isize,
    choices: &[UiFontFamily],
    kind: FontKind,
    availability: &FontAvailability,
) -> UiFontFamily {
    if choices.is_empty() {
        return current.clone();
    }
    let current = choices
        .iter()
        .position(|family| family == current)
        .unwrap_or_default() as isize;
    let mut ix = current + delta.signum();
    while (0..choices.len() as isize).contains(&ix) {
        let candidate = &choices[ix as usize];
        if kind.is_available_for(availability, candidate) {
            return candidate.clone();
        }
        ix += delta.signum();
    }
    choices[current as usize].clone()
}

/// Narrow and rank families by a typed query: prefix matches first, then
/// substring matches, catalog order preserved within each rank. An empty query
/// keeps the catalog untouched, so bundled entries still sort first (upstream
/// :1470-1482).
pub(super) fn filter_families(query: &str, choices: &[UiFontFamily]) -> Vec<UiFontFamily> {
    if query.trim().is_empty() {
        return choices.to_vec();
    }
    let labels: Vec<&str> = choices.iter().map(UiFontFamily::label).collect();
    popover::filter_indices(query, &labels)
        .into_iter()
        .map(|ix| choices[ix].clone())
        .collect()
}

/// The first selectable family in `choices`, else the kind's fallback
/// (upstream :1484-1494).
pub(super) fn first_available(
    choices: &[UiFontFamily],
    kind: FontKind,
    availability: &FontAvailability,
) -> UiFontFamily {
    choices
        .iter()
        .find(|family| kind.is_available_for(availability, family))
        .cloned()
        .unwrap_or_else(|| fallback_selection(kind))
}

/// The last selectable family in `choices`, else the kind's fallback
/// (upstream :1496-1507).
pub(super) fn last_available(
    choices: &[UiFontFamily],
    kind: FontKind,
    availability: &FontAvailability,
) -> UiFontFamily {
    choices
        .iter()
        .rev()
        .find(|family| kind.is_available_for(availability, family))
        .cloned()
        .unwrap_or_else(|| fallback_selection(kind))
}

/// Highlight target when a kind's catalog offers nothing: System UI is
/// proportional, so the terminal cannot land there — it falls back to the
/// bundled mono (upstream used GeistMono, :1509-1516).
pub(super) fn fallback_selection(kind: FontKind) -> UiFontFamily {
    match kind {
        FontKind::Terminal => UiFontFamily::GoogleSansCode,
        _ => UiFontFamily::System,
    }
}

/// Format a ladder rung as a pixel label: whole rungs drop the decimal
/// (upstream :1518-1524).
pub(super) fn format_px(size: f32) -> String {
    if size.fract().abs() < f32::EPSILON {
        format!("{size:.0} px")
    } else {
        format!("{size:.1} px")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bundled() -> Vec<UiFontFamily> {
        vec![
            UiFontFamily::MomoTrustSans,
            UiFontFamily::HankenGrotesk,
            UiFontFamily::GoogleSansCode,
            UiFontFamily::System,
        ]
    }

    #[test]
    fn every_kind_has_distinct_labels_and_slugs() {
        let mut slugs = Vec::new();
        let mut labels = Vec::new();
        for kind in FontKind::ALL {
            assert!(!kind.slug().is_empty());
            assert!(!kind.label("en").is_empty());
            assert!(!kind.description("en").is_empty());
            assert!(!kind.size_label("en").is_empty());
            assert!(kind.label("en") != kind.label_key());
            slugs.push(kind.slug());
            labels.push(kind.label("en"));
        }
        slugs.sort_unstable();
        slugs.dedup();
        assert_eq!(slugs.len(), FontKind::ALL.len());
        labels.sort_unstable();
        labels.dedup();
        assert_eq!(labels.len(), FontKind::ALL.len());
    }

    #[test]
    fn only_the_terminal_narrows_to_fixed_width() {
        // `FontAvailability::default()` has only System UI available and no
        // fixed-width catalog: the terminal must see the narrow list.
        let availability = FontAvailability::default();
        assert_eq!(
            FontKind::Ui.choices_for(&availability).len(),
            availability.choices().len()
        );
        assert_eq!(
            FontKind::Code.choices_for(&availability).len(),
            availability.choices().len()
        );
        assert!(FontKind::Terminal.choices_for(&availability).is_empty());
        assert!(!FontKind::Terminal.is_available_for(&availability, &UiFontFamily::System));
    }

    #[test]
    fn step_font_skips_unavailable_rows_and_clamps() {
        // Default availability: only System UI is selectable.
        let availability = FontAvailability::default();
        let choices = bundled();
        let stepped = step_font(
            &UiFontFamily::MomoTrustSans,
            1,
            &choices,
            FontKind::Ui,
            &availability,
        );
        assert_eq!(stepped, UiFontFamily::System);
        // Past the end it clamps on the current row instead of wrapping.
        assert_eq!(
            step_font(
                &UiFontFamily::System,
                1,
                &choices,
                FontKind::Ui,
                &availability
            ),
            UiFontFamily::System
        );
        assert_eq!(
            step_font(
                &UiFontFamily::System,
                -1,
                &choices,
                FontKind::Ui,
                &availability
            ),
            UiFontFamily::System
        );
        // An empty catalog (terminal under default availability) keeps the
        // current family.
        assert_eq!(
            step_font(
                &UiFontFamily::GoogleSansCode,
                1,
                &[],
                FontKind::Terminal,
                &availability
            ),
            UiFontFamily::GoogleSansCode
        );
    }

    #[test]
    fn first_and_last_available_skip_unavailable_and_fall_back() {
        let availability = FontAvailability::default();
        let choices = bundled();
        assert_eq!(
            first_available(&choices, FontKind::Ui, &availability),
            UiFontFamily::System
        );
        assert_eq!(
            last_available(&choices, FontKind::Ui, &availability),
            UiFontFamily::System
        );
        assert_eq!(
            first_available(&[], FontKind::Terminal, &availability),
            UiFontFamily::GoogleSansCode
        );
        // Proportional System UI is never a terminal fallback.
        assert_eq!(
            fallback_selection(FontKind::Terminal),
            UiFontFamily::GoogleSansCode
        );
        assert_eq!(fallback_selection(FontKind::Ui), UiFontFamily::System);
        assert_eq!(fallback_selection(FontKind::Code), UiFontFamily::System);
    }

    #[test]
    fn filter_families_ranks_prefix_before_substring_and_keeps_empty_query() {
        let choices = vec![
            UiFontFamily::MomoTrustSans,
            UiFontFamily::HankenGrotesk,
            UiFontFamily::GoogleSansCode,
            UiFontFamily::System,
            UiFontFamily::Installed("IBM Plex Mono".into()),
            UiFontFamily::Installed("Plex Sans".into()),
        ];
        // Empty/whitespace query leaves catalog order untouched.
        assert_eq!(filter_families("", &choices), choices);
        assert_eq!(filter_families("   ", &choices), choices);
        // "plex": only the two device-local Plex families match, in catalog
        // order (the query is a substring of both).
        let plex = filter_families("plex", &choices);
        assert_eq!(plex.len(), 2);
        assert!(plex.iter().all(|family| family.label().contains("Plex")));
        // Prefix matches rank before substring ones.
        let s = filter_families("s", &choices);
        assert_eq!(s[0], UiFontFamily::System);
        assert!(s.contains(&UiFontFamily::GoogleSansCode));
        assert!(s.contains(&UiFontFamily::MomoTrustSans));
    }

    #[test]
    fn format_px_drops_a_whole_pixel_decimal_only() {
        assert_eq!(format_px(13.0), "13 px");
        assert_eq!(format_px(12.5), "12.5 px");
        assert_eq!(format_px(10.0), "10 px");
        assert_eq!(format_px(20.0), "20 px");
    }

    #[test]
    fn size_ladders_snap_off_ladder_values_to_the_nearest_rung() {
        assert_eq!(nearest_mono_ix(13.0), 4);
        assert_eq!(nearest_mono_ix(12.4), 3);
        assert_eq!(nearest_mono_ix(9.0), 0);
        assert_eq!(nearest_mono_ix(24.0), MONO_FONT_SIZES.len() - 1);
        assert_eq!(nearest_math_size_ix(100), 1);
        assert_eq!(nearest_math_size_ix(91), 0);
        assert_eq!(nearest_math_size_ix(149), MATH_SIZES.len() - 1);
        assert_eq!(MATH_SIZES, [90, 100, 110, 120, 135, 150]);
        assert_eq!(MONO_FONT_SIZES[3], 12.5);
    }

    #[test]
    fn size_labels_cover_each_ladder_in_order() {
        assert_eq!(FontKind::Ui.size_count(), UiFontSize::ALL.len());
        assert_eq!(FontKind::Terminal.size_count(), MONO_FONT_SIZES.len());
        assert_eq!(FontKind::Code.size_count(), MONO_FONT_SIZES.len());
        assert_eq!(FontKind::Ui.size_labels().len(), UiFontSize::ALL.len());
        assert_eq!(
            FontKind::Terminal.size_labels()[3],
            SharedString::from("12.5 px")
        );
        assert_eq!(FontKind::Code.size_labels()[4], SharedString::from("13 px"));
    }
}
