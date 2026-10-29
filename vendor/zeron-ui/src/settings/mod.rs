//! The settings surface: section model and zeron's settings widgets.
//!
//! Adapted from zeron's `crates/ui/src/shell.rs` (`SettingsSection`) and
//! `crates/ui/src/settings/widgets.rs` (MIT, Copyright 2026 Wing). mono-desk's
//! section list is its page list, while the *shape* (label + icon rows,
//! spacing-only nav groups, tab roles/aria) and the widget code stay zeron's.
//!
//! Phase `chrome` T2 ports the section-list row and its selection animation
//! ([`widgets::section_tab`], [`widgets::tab_selection_t`]). The page
//! scaffolding (`page_column`, `page_header`, `section`, `section_card`,
//! cards, toggles, selects) lands with the `settings-surface` phase.


pub mod menu;
pub mod select;
pub mod widgets;

use crate::icons;

/// mono-desk's settings sections, in nav order. zeron's own enum has thirteen
/// sections (Devices, Cloud, Billing, Providers, Accounts, Appearance, Files,
/// Notifications, Voice, Shortcuts, General, Appshots, Archived); mono-desk
/// keeps its pages plus `Appearance` and zeron's row/label/group conventions.
/// `Appearance` is zeron's *custom* appearance page (`settings/appearance.rs`),
/// which the `settings-surface` phase fills in — the row is already here so the
/// nav never disagrees with the page list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsSection {
    #[default]
    General,
    /// zeron's custom appearance page (owner directive R8).
    Appearance,
    Notifications,
    Profile,
    /// The Finance UI demo page — first row of the workspace group.
    FinanceUi,
    Reset,
    About,
}

impl SettingsSection {
    pub const ALL: [Self; 7] = [
        Self::General,
        Self::Appearance,
        Self::Notifications,
        Self::Profile,
        Self::FinanceUi,
        Self::Reset,
        Self::About,
    ];

    /// Nav groups are separated by spacing alone: preferences, then workspace
    /// data (zeron `SettingsSection::starts_nav_group`).
    pub fn starts_nav_group(self) -> bool {
        matches!(self, Self::FinanceUi)
    }

    /// Sidebar and page-header label (zeron's `SettingsSection::label`).
    pub fn label(self) -> &'static str {
        match self {
            Self::General => "General",
            Self::Appearance => "Appearance",
            Self::Notifications => "Notifications",
            Self::Profile => "Profile",
            Self::FinanceUi => "Finance UI",
            Self::Reset => "Reset",
            Self::About => "About",
        }
    }

    /// The section's glyph, in the nav and on its page header (zeron's
    /// `SettingsSection::icon`).
    pub fn icon(self) -> &'static str {
        match self {
            Self::General => icons::SETTINGS,
            Self::Appearance => icons::TUNING,
            Self::Notifications => icons::BELL,
            Self::Profile => icons::USER,
            Self::FinanceUi => icons::WALLET,
            Self::Reset => icons::RESTART,
            Self::About => icons::INFO_CIRCLE,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sections_are_in_nav_order_with_one_group_break() {
        // The workspace group starts at Finance UI and every row has a label
        // and a glyph: the nav renders from this list alone.
        // zeron orders General before Appearance; mono-desk keeps that order.
        assert_eq!(SettingsSection::ALL[0], SettingsSection::General);
        assert_eq!(SettingsSection::ALL[1], SettingsSection::Appearance);
        let breaks: Vec<_> = SettingsSection::ALL
            .into_iter()
            .filter(|section| section.starts_nav_group())
            .collect();
        assert_eq!(breaks, vec![SettingsSection::FinanceUi]);
        assert_eq!(SettingsSection::ALL.len(), 7);
        assert_eq!(SettingsSection::default(), SettingsSection::General);
        for section in SettingsSection::ALL {
            assert!(!section.label().is_empty());
            assert!(section.icon().ends_with(".svg"), "{}", section.label());
        }
    }
}
