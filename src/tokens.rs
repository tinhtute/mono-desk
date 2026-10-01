//! The app's elevation snapshot.
//!
//! beUI gives raised surfaces a shadow scale (`sm`/`md`/`lg`) that the
//! vendored overlays paint from their own hardcoded values. The app-owned
//! raised surface — the active sidebar pill — needs the same scale, so it is
//! resolved here from the live theme with [`Theme::resolve_semantic_config`]
//! (non-mutating) and stored as a [`Global`].
//!
//! `theme.shadow` stays `false`: the legacy flag gates the vendored
//! surface-control shadows, and `apply_semantic_config*` (which would flip
//! it) is never called. The snapshot is re-resolved by
//! [`crate::theme::apply_overrides`] after every theme rebuild and every
//! font-size change, so a mode switch cannot restore stale values.

use gpui_kit::component::{SemanticShadowConfig, SemanticThemeConfig, ShadowTokens, Theme};
use gpui_kit::{App, BoxShadow, Global, hsla, px};

/// Ink every layer of the app's elevation scale carries — the same 10% black
/// the vendored overlays spend (`SURFACE_SHADOW_INK`).
const ELEVATION_INK: f32 = 0.1;

/// The app's elevation scale, resolved from the live theme.
///
/// Non-color and mode-independent: the ink is fixed black, so light and dark
/// resolve to the same values.
pub struct UiTokens {
    pub shadow: ShadowTokens,
}

impl Global for UiTokens {}

impl UiTokens {
    /// Resolve the app's elevation scale from `theme` without mutating it.
    pub fn resolve(theme: &Theme) -> Self {
        Self {
            shadow: theme.resolve_semantic_config(&elevation_config()).shadow,
        }
    }

    /// Re-resolve the snapshot from the live theme and store it.
    pub fn sync(cx: &mut App) {
        let tokens = Self::resolve(Theme::global(cx));
        cx.set_global(tokens);
    }
}

/// The app's elevation scale as a semantic config: only the three shadow
/// tiers are set, so resolving it can never write a colour, radius, spacing,
/// or typography token.
///
/// The values are the vendored scale (shadcn/Tailwind, blur halved for GPUI's
/// sigma): `sm` = `raised_shadow`, `md` = `popover_shadow` without its ring,
/// `lg` = `toast_shadow`.
fn elevation_config() -> SemanticThemeConfig {
    let ink = hsla(0., 0., 0., ELEVATION_INK);
    SemanticThemeConfig {
        shadow: SemanticShadowConfig {
            sm: Some(vec![
                BoxShadow::new(px(0.), px(1.), ink).blur_radius(px(1.5)),
                BoxShadow::new(px(0.), px(1.), ink)
                    .blur_radius(px(1.))
                    .spread_radius(px(-1.)),
            ]),
            md: Some(vec![
                BoxShadow::new(px(0.), px(4.), ink)
                    .blur_radius(px(3.))
                    .spread_radius(px(-1.)),
                BoxShadow::new(px(0.), px(2.), ink)
                    .blur_radius(px(2.))
                    .spread_radius(px(-2.)),
            ]),
            lg: Some(vec![
                BoxShadow::new(px(0.), px(10.), ink)
                    .blur_radius(px(7.5))
                    .spread_radius(px(-3.)),
                BoxShadow::new(px(0.), px(4.), ink)
                    .blur_radius(px(3.))
                    .spread_radius(px(-4.)),
            ]),
        },
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::{UiTokens, elevation_config};
    use gpui_kit::component::{Theme, ThemeMode, ThemeSet};
    use gpui_kit::{BoxShadow, hsla, px};

    /// The app's own palette, resolved the way `Theme::change` resolves it.
    fn mono_theme(mode: ThemeMode) -> Theme {
        let set: ThemeSet = serde_json::from_str(crate::theme::MONO_THEMES)
            .expect("embedded mono.json must deserialize");
        let config = set
            .themes
            .into_iter()
            .find(|theme| theme.mode == mode)
            .expect("both modes are present in the embedded theme set");
        let mut theme = Theme::default();
        theme.apply_config(&Rc::new(config));
        theme
    }

    /// Resolving the elevation config must never write a colour: every
    /// `SemanticColorConfig` field stays `None`.
    #[test]
    fn elevation_config_carries_no_colors() {
        let colors =
            serde_json::to_value(&elevation_config().colors).expect("colors must serialize");
        let colors = colors
            .as_object()
            .expect("the color config must serialize as an object");
        assert!(
            !colors.is_empty(),
            "the color config must have fields to check"
        );
        for (field, value) in colors {
            assert!(value.is_null(), "colors.{field} must stay unset");
        }
    }

    /// The resolved scale is the vendored one (shadcn/Tailwind, blur halved
    /// for GPUI's sigma): `sm` = `raised_shadow`, `md` = `popover_shadow`
    /// without its ring, `lg` = `toast_shadow`, all at 10% black ink.
    #[test]
    fn resolve_matches_the_vendored_elevation_scale() {
        let ink = hsla(0., 0., 0., 0.1);
        let layer = |x: f32, y: f32, blur: f32, spread: f32| {
            BoxShadow::new(px(x), px(y), ink)
                .blur_radius(px(blur))
                .spread_radius(px(spread))
        };

        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let tokens = UiTokens::resolve(&mono_theme(mode));
            assert_eq!(
                tokens.shadow.sm,
                vec![layer(0., 1., 1.5, 0.), layer(0., 1., 1., -1.)],
                "{}: shadow.sm",
                mode.name()
            );
            assert_eq!(
                tokens.shadow.md,
                vec![layer(0., 4., 3., -1.), layer(0., 2., 2., -2.)],
                "{}: shadow.md",
                mode.name()
            );
            assert_eq!(
                tokens.shadow.lg,
                vec![layer(0., 10., 7.5, -3.), layer(0., 4., 3., -4.)],
                "{}: shadow.lg",
                mode.name()
            );
        }
    }

    /// The snapshot is mode-independent (fixed black ink) and resolving it
    /// never mutates the theme: `theme.shadow` stays false and every colour
    /// token is unchanged.
    #[test]
    fn resolve_is_mode_independent_and_non_mutating() {
        let light = mono_theme(ThemeMode::Light);
        let dark = mono_theme(ThemeMode::Dark);

        let light_tokens = UiTokens::resolve(&light);
        let dark_tokens = UiTokens::resolve(&dark);
        assert_eq!(light_tokens.shadow, dark_tokens.shadow);

        for theme in [&light, &dark] {
            let before = theme.color_tokens();
            let _ = UiTokens::resolve(theme);
            assert!(!theme.shadow, "resolve must not flip `theme.shadow`");
            assert_eq!(
                theme.color_tokens(),
                before,
                "resolve must not write colours"
            );
        }
    }

    /// A font-size override changes the rem base, not the elevation scale:
    /// the snapshot is equal before and after, and the base returns to 14.
    #[test]
    fn font_overrides_leave_the_snapshot_unchanged() {
        let mut theme = mono_theme(ThemeMode::Light);
        let before = UiTokens::resolve(&theme);

        crate::theme::write_overrides(&mut theme, 16.0);
        assert_eq!(theme.font_size, px(16.));
        assert_eq!(UiTokens::resolve(&theme).shadow, before.shadow);

        crate::theme::write_overrides(&mut theme, 14.0);
        assert_eq!(theme.font_size, px(14.));
        assert_eq!(UiTokens::resolve(&theme).shadow, before.shadow);
    }
}
