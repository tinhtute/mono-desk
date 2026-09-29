//! Status chip — the pill from the DevDsgn reference
//! (`x.com/DevDsgn/status/2104504689962430510`; spec and the three renders in
//! `~/quaqua/runs/ref-devdsgn/`).
//!
//! Shape: `rounded-full`, no border, no shadow. The background is a tint of the
//! tone's hue and the icon and label share one ink of that same hue, at medium
//! weight. Light mode uses the reference's light tints; dark mode uses its
//! *pastel* tints (desaturated background, lightened ink) rather than the
//! reference's vivid dark set.
//!
//! The reference's light ink sits below WCAG AA on its own tint for seven of the
//! eight tones (orange 2.3:1, teal 2.1:1, …), so the light inks here are
//! darkened along their own hue until they clear 4.5:1. The pastel inks clear it
//! as published. [`tests::every_tone_clears_wcag_aa_in_both_modes`] holds both
//! palettes to that floor.
//!
//! Glyphs are filled Lucide shapes: the reference draws solid icons, and GPUI
//! paints an SVG as a single-colour alpha mask, so an outline glyph cannot be
//! thickened into one. Each asset fills the shape and knocks its detail out with
//! `fill-rule="evenodd"` — one path per file, because the rule applies per fill,
//! not across sibling paths.

use gpui_kit::component::{
    ActiveTheme, Icon, Sizable, Size, StyledExt, ThemeMode, h_flex, label::Label,
};
use gpui_kit::{
    App, IntoElement, ParentElement as _, RenderOnce, SharedString, Styled as _, Window, rgb,
};
use rust_i18n::t;

/// A tone's resolved pill colors, as opaque `0xRRGGBB`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ToneColors {
    /// Pill background.
    bg: u32,
    /// Icon and label ink.
    fg: u32,
}

/// The glyph a tone draws.
///
/// Named after the Lucide icon each asset is derived from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatusIcon {
    Clock,
    Calendar,
    ChartPie,
    Flame,
    BadgeCheck,
    Timer,
    OctagonPause,
    FileSearch,
    /// Release tag — not a tone default; used by the About page's version chip.
    Tag,
}

impl StatusIcon {
    fn svg(self) -> &'static [u8] {
        match self {
            Self::Clock => include_bytes!("../assets/icons/status/clock.svg"),
            Self::Calendar => include_bytes!("../assets/icons/status/calendar.svg"),
            Self::ChartPie => include_bytes!("../assets/icons/status/chart-pie.svg"),
            Self::Flame => include_bytes!("../assets/icons/status/flame.svg"),
            Self::BadgeCheck => include_bytes!("../assets/icons/status/badge-check.svg"),
            Self::Timer => include_bytes!("../assets/icons/status/timer.svg"),
            Self::OctagonPause => include_bytes!("../assets/icons/status/octagon-pause.svg"),
            Self::FileSearch => include_bytes!("../assets/icons/status/file-search.svg"),
            Self::Tag => include_bytes!("../assets/icons/status/tag.svg"),
        }
    }
}

/// The eight statuses the reference chip set covers.
///
/// The app renders three of them — `Ready`/`Paused` on the on/off rows and
/// `Scheduled` on the version chip — but the enum stays the full reference set
/// rather than trimmed to today's call sites, so the component can render any
/// of the eight.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatusTone {
    Queued,
    Scheduled,
    InProgress,
    HighPriority,
    Ready,
    Waiting,
    Paused,
    Reviewing,
}

impl StatusTone {
    /// Every tone, in reference order.
    ///
    /// The palette test walks this list, so a tone cannot be added without a
    /// contrast check; the app itself renders a subset.
    #[allow(dead_code)]
    pub const ALL: [Self; 8] = [
        Self::Queued,
        Self::Scheduled,
        Self::InProgress,
        Self::HighPriority,
        Self::Ready,
        Self::Waiting,
        Self::Paused,
        Self::Reviewing,
    ];

    /// The tone's own status name, as a `locales/app.yml` key.
    fn label_key(self) -> &'static str {
        match self {
            Self::Queued => "Status.queued",
            Self::Scheduled => "Status.scheduled",
            Self::InProgress => "Status.in_progress",
            Self::HighPriority => "Status.high_priority",
            Self::Ready => "Status.ready",
            Self::Waiting => "Status.waiting",
            Self::Paused => "Status.paused",
            Self::Reviewing => "Status.reviewing",
        }
    }

    /// The glyph the reference pairs with this tone.
    fn icon(self) -> StatusIcon {
        match self {
            Self::Queued => StatusIcon::Clock,
            Self::Scheduled => StatusIcon::Calendar,
            Self::InProgress => StatusIcon::ChartPie,
            Self::HighPriority => StatusIcon::Flame,
            Self::Ready => StatusIcon::BadgeCheck,
            Self::Waiting => StatusIcon::Timer,
            Self::Paused => StatusIcon::OctagonPause,
            Self::Reviewing => StatusIcon::FileSearch,
        }
    }

    /// Pill colors for `mode`.
    fn colors(self, mode: ThemeMode) -> ToneColors {
        if mode.is_dark() {
            self.pastel()
        } else {
            self.light()
        }
    }

    /// The reference's light tints, with ink darkened to clear WCAG AA.
    ///
    /// Backgrounds are the reference values verbatim. Inks are the reference
    /// values scaled toward black along their own hue (hue and saturation held)
    /// until the pair clears 4.5:1 — the smallest such scale per tone, so
    /// `Queued` keeps its published ink and the rest darken by 18–34%.
    fn light(self) -> ToneColors {
        match self {
            Self::Queued => ToneColors {
                bg: 0xf2e9fe,
                fg: 0x8f24ff,
            },
            Self::Scheduled => ToneColors {
                bg: 0xe9f3ff,
                fg: 0x006ad9,
            },
            Self::InProgress => ToneColors {
                bg: 0xfef2e6,
                fg: 0xae5600,
            },
            Self::HighPriority => ToneColors {
                bg: 0xfdedee,
                fg: 0xd80737,
            },
            Self::Ready => ToneColors {
                bg: 0xe7f7f6,
                fg: 0x017c69,
            },
            Self::Waiting => ToneColors {
                bg: 0xfaecfd,
                fg: 0xb800d6,
            },
            Self::Paused => ToneColors {
                bg: 0xf9f3e5,
                fg: 0x976300,
            },
            Self::Reviewing => ToneColors {
                bg: 0xe4f6f8,
                fg: 0x01769a,
            },
        }
    }

    /// The reference's pastel tints, used as published for dark mode.
    fn pastel(self) -> ToneColors {
        match self {
            Self::Queued => ToneColors {
                bg: 0x2b1e3c,
                fg: 0xba97f5,
            },
            Self::Scheduled => ToneColors {
                bg: 0x192b3f,
                fg: 0x90c4ff,
            },
            Self::InProgress => ToneColors {
                bg: 0x3a2919,
                fg: 0xffc16a,
            },
            Self::HighPriority => ToneColors {
                bg: 0x3c1e28,
                fg: 0xfe94a8,
            },
            Self::Ready => ToneColors {
                bg: 0x19342f,
                fg: 0x83dfc6,
            },
            Self::Waiting => ToneColors {
                bg: 0x34203c,
                fg: 0xeb9af9,
            },
            Self::Paused => ToneColors {
                bg: 0x352d1a,
                fg: 0xf6d167,
            },
            Self::Reviewing => ToneColors {
                bg: 0x17343a,
                fg: 0x79daea,
            },
        }
    }
}

/// A status pill: tinted `rounded-full` background, same-hue filled icon and
/// medium-weight label, no border and no shadow.
#[derive(IntoElement)]
pub struct StatusChip {
    tone: StatusTone,
    label: SharedString,
    icon: StatusIcon,
}

impl StatusChip {
    /// Chip for `tone`, labelled with the tone's own translated status name.
    pub fn new(tone: StatusTone) -> Self {
        Self::labelled(tone, t!(tone.label_key()).to_string())
    }

    /// Chip for `tone` with a caller-supplied, already-translated label.
    pub fn labelled(tone: StatusTone, label: impl Into<SharedString>) -> Self {
        Self {
            tone,
            label: label.into(),
            icon: tone.icon(),
        }
    }

    /// Replace the tone's default glyph.
    pub fn icon(mut self, icon: StatusIcon) -> Self {
        self.icon = icon;
        self
    }
}

impl RenderOnce for StatusChip {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = self.tone.colors(cx.theme().mode);
        // The reference draws its icon at 1.6× the label; 1.25rem keeps the
        // glyph dominant while tracking the Font Size setting, which the label
        // follows through `text_sm`.
        let icon_size = window.rem_size() * 1.25;

        h_flex()
            .gap_1p5()
            .rounded_full()
            .bg(rgb(colors.bg))
            .px_2()
            .py_1()
            // The icon inherits this; `Label` pins the theme foreground of its
            // own accord, so it is given the ink again below.
            .text_color(rgb(colors.fg))
            .font_medium()
            .child(
                Icon::default()
                    .data(self.icon.svg())
                    .with_size(Size::Size(icon_size)),
            )
            .child(Label::new(self.label).text_sm().text_color(rgb(colors.fg)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// WCAG 2.1 relative luminance of an opaque sRGB color.
    fn relative_luminance(hex: u32) -> f32 {
        let channel = |shift: u32| {
            let c = ((hex >> shift) & 0xff) as f32 / 255.;
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * channel(16) + 0.7152 * channel(8) + 0.0722 * channel(0)
    }

    /// WCAG 2.1 contrast ratio between two opaque sRGB colors.
    fn contrast_ratio(a: u32, b: u32) -> f32 {
        let (la, lb) = (relative_luminance(a), relative_luminance(b));
        let (hi, lo) = if la > lb { (la, lb) } else { (lb, la) };
        (hi + 0.05) / (lo + 0.05)
    }

    /// Every tone's ink must clear WCAG AA on its own tint, in both modes.
    #[test]
    fn every_tone_clears_wcag_aa_in_both_modes() {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            for tone in StatusTone::ALL {
                let colors = tone.colors(mode);
                let ratio = contrast_ratio(colors.fg, colors.bg);
                assert!(
                    ratio >= 4.5,
                    "{tone:?} in {mode:?}: fg {:#08x} on bg {:#08x} is {ratio:.2}:1, below 4.5:1",
                    colors.fg,
                    colors.bg,
                );
            }
        }
    }

    /// A tone whose asset is missing or empty renders as a blank gap, which no
    /// other check would catch.
    #[test]
    fn every_glyph_is_a_non_empty_svg() {
        for icon in [
            StatusIcon::Clock,
            StatusIcon::Calendar,
            StatusIcon::ChartPie,
            StatusIcon::Flame,
            StatusIcon::BadgeCheck,
            StatusIcon::Timer,
            StatusIcon::OctagonPause,
            StatusIcon::FileSearch,
            StatusIcon::Tag,
        ] {
            let svg = icon.svg();
            assert!(
                svg.starts_with(b"<svg") && svg.windows(6).any(|w| w == b"</svg>"),
                "{icon:?} is not a complete SVG document",
            );
            assert!(
                svg.windows(6).any(|w| w == b"<path "),
                "{icon:?} has no path geometry",
            );
        }
    }
}
