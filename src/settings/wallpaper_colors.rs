//! Wallpaper-derived color: sampling the dominant color of an image and the
//! gate that decides when a wallpaper may tint the theme.
//!
//! Ported/adapted from zeron's `crates/ui/src/settings/wallpaper_colors.rs`
//! (MIT, Copyright 2026 Wing, pinned rev 8b7afb3). [`extract`] is verbatim
//! (16³ color bins, saturation weighting, `a < 128` skipped, the thumbnail
//! bounded by the caller).
//!
//! Adaptations from upstream (recorded, no behaviour stubs):
//! - `tint_variant` is **not** duplicated: the vendored theme already carries
//!   zeron's tint (`vendor/zeron-ui/src/theme.rs:1692-1731`) and applies it
//!   when `Theme::install_selection_with_color(...)` receives a wallpaper
//!   color, so this module only produces/validates the color.
//! - [`active`] reads the app's [`AppSettings`] directly (upstream read its
//!   persisted settings store through `current(cx)`); the gate itself is
//!   unchanged — the toggle is on *and* a managed background exists.
//! - [`apply_enabled`] is the smallest seam onto the app: it persists the
//!   toggle and samples the installed background image on the background
//!   executor, then returns the color for the caller to apply. zeron instead
//!   called `appearance::apply(cx)` itself; mono-desk's equivalent needs a
//!   `Window` (`settings::apply_appearance(window, cx)`), which a domain
//!   helper must not own, so the Appearance page awaits the returned task and
//!   re-runs its appearance funnel. `ensure_color`'s separate backfill path is
//!   covered by the same sampling here (install always samples, and enabling
//!   resamples the installed image).
//! - The sampled color and the toggle keep zeron's persisted shape; the
//!   `#RRGGBB` codec lives in `src/app_settings.rs` (T1).

use gpui::{App, Task};
use image::DynamicImage;
use zeron_theme::Color;

use crate::app_settings::AppSettings;

/// Quantized dominant colour, favouring chromatic regions over neutral pixels.
/// Sampling is bounded by the caller; transparent pixels do not influence it.
/// (Verbatim zeron `settings/wallpaper_colors.rs:7-33`.)
pub(crate) fn extract(pixels: impl IntoIterator<Item = [u8; 4]>) -> Option<Color> {
    let mut bins = vec![(0.0_f64, [0.0_f64; 3]); 4096];
    for [r, g, b, a] in pixels {
        if a < 128 {
            continue;
        }
        let high = r.max(g).max(b) as f64;
        let low = r.min(g).min(b) as f64;
        let saturation = (high - low) / high.max(1.0);
        let weight = (0.2 + saturation * saturation) * a as f64 / 255.0;
        let index = ((r as usize >> 4) << 8) | ((g as usize >> 4) << 4) | (b as usize >> 4);
        let (count, channels) = &mut bins[index];
        *count += weight;
        for (sum, channel) in channels.iter_mut().zip([r, g, b]) {
            *sum += channel as f64 * weight;
        }
    }
    let (count, channels) = bins.into_iter().max_by(|a, b| a.0.total_cmp(&b.0))?;
    (count > 0.0).then(|| {
        Color::rgb(
            (channels[0] / count).round() as u8,
            (channels[1] / count).round() as u8,
            (channels[2] / count).round() as u8,
        )
    })
}

/// Decode the exact bytes that were (or will be) copied into the managed
/// background directory; `None` when the format is unsupported or the data is
/// damaged. Same decoder contract as zeron's
/// `new_thread_background_image::decode` (guess the format from the bytes, not
/// from the file extension).
pub(crate) fn decode_image(bytes: &[u8]) -> Option<DynamicImage> {
    image::ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .ok()?
        .decode()
        .ok()
}

/// The dominant color of `image`, sampled from zeron's 64×64 thumbnail.
pub(crate) fn color_of(image: &DynamicImage) -> Option<Color> {
    let thumbnail = image.thumbnail(64, 64).to_rgba8();
    extract(thumbnail.pixels().map(|pixel| pixel.0))
}

/// Decode `bytes` and sample their dominant color; `None` for unreadable data
/// or an image that is entirely transparent.
pub(crate) fn sample_color(bytes: &[u8]) -> Option<Color> {
    color_of(&decode_image(bytes)?)
}

/// The wallpaper color that may tint the theme: `Some` only while the
/// "Match wallpaper colors" toggle is on, a managed background is installed
/// and a color has been sampled. Mirrors zeron's `active(cx)` gate.
pub(crate) fn active(settings: &AppSettings) -> Option<Color> {
    if settings.wallpaper_theme_colors && settings.new_thread_background.is_some() {
        settings.wallpaper_color
    } else {
        None
    }
}

/// Persist the "Match wallpaper colors" toggle and, when enabling it for an
/// installed background, resample that image's color on the background
/// executor. The returned task resolves to the newly active color once the
/// sampled value has been persisted, so the caller can re-run its appearance
/// funnel (theme reinstall); it resolves to `None` when the toggle was turned
/// off or nothing could be sampled.
///
/// The toggle is persisted immediately: it is a user choice, and a slow or
/// failed image read must not lose it.
pub(crate) fn apply_enabled(enabled: bool, cx: &mut App) -> Task<Option<Color>> {
    AppSettings::global_mut(cx).wallpaper_theme_colors = enabled;
    super::persist_now(cx);
    let background = AppSettings::global(cx).new_thread_background.clone();
    let Some(background) = background.filter(|_| enabled) else {
        return cx.spawn(async move |_| None::<Color>);
    };
    cx.spawn(async move |cx| {
        let path = background.path.clone();
        let color = cx
            .background_executor()
            .spawn(async move {
                let bytes = std::fs::read(path).ok()?;
                sample_color(&bytes)
            })
            .await;
        let color = color?;
        cx.update(|cx| {
            // A newer background or a toggled-off switch meanwhile wins.
            let settings = AppSettings::global_mut(cx);
            if !settings.wallpaper_theme_colors {
                return;
            }
            if settings
                .new_thread_background
                .as_ref()
                .map(|current| &current.path)
                != Some(&background.path)
            {
                return;
            }
            settings.wallpaper_color = Some(color);
            super::persist_now(cx);
        });
        Some(color)
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::app_settings::NewThreadBackground;
    use crate::app_settings::NewThreadBackgroundAdjustment;

    #[test]
    fn extraction_ignores_transparency_and_favours_prominent_colour() {
        let mut pixels = vec![[90, 90, 90, 255]; 100];
        pixels.extend(vec![[30, 130, 220, 255]; 80]);
        pixels.extend(vec![[255, 0, 0, 0]; 1000]);
        assert_eq!(
            extract(pixels).map(|color| (color.r, color.g, color.b)),
            Some((30, 130, 220))
        );
        assert_eq!(
            extract([[100, 100, 100, 255]; 10]).map(|color| (color.r, color.g, color.b)),
            Some((100, 100, 100))
        );
        assert!(extract([[255, 0, 0, 0]; 10]).is_none());
    }

    #[test]
    fn active_requires_the_toggle_an_installed_background_and_a_sampled_colour() {
        let mut settings = AppSettings::default();
        assert!(active(&settings).is_none());

        settings.new_thread_background = Some(NewThreadBackground {
            path: PathBuf::from("/managed/new-thread-background.png"),
            name: "picked.png".into(),
            adjustment: NewThreadBackgroundAdjustment::default(),
        });
        settings.wallpaper_color = Some(Color::rgb(30, 130, 220));
        // Toggle still off.
        assert!(active(&settings).is_none());
        settings.wallpaper_theme_colors = true;
        assert_eq!(
            active(&settings).map(|color| (color.r, color.g, color.b)),
            Some((30, 130, 220))
        );
        // No sampled colour yet: nothing to tint with.
        settings.wallpaper_color = None;
        assert!(active(&settings).is_none());
        // Background removed while the toggle is on: no tint either.
        settings.wallpaper_color = Some(Color::rgb(30, 130, 220));
        settings.new_thread_background = None;
        assert!(active(&settings).is_none());
    }

    #[test]
    fn sample_color_decodes_the_thumbnail_dominant_colour() {
        let image = DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(
            4,
            4,
            image::Rgba([30, 130, 220, 255]),
        ));
        let mut bytes = Vec::new();
        image
            .write_to(
                &mut std::io::Cursor::new(&mut bytes),
                image::ImageFormat::Png,
            )
            .unwrap();
        assert_eq!(
            sample_color(&bytes).map(|color| (color.r, color.g, color.b)),
            Some((30, 130, 220))
        );
        assert!(sample_color(b"not an image").is_none());
    }
}
