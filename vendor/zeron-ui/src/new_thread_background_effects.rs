//! Raster effect treatments for the new-thread background artwork: the
//! source-space decoded luminance/colors plus Dither/ASCII/Halftone/Scanlines
//! renders, cached per (effect, light/dark) and generated off the UI thread.
//!
//! Adapted from zeron's `crates/ui/src/new_thread_background_effects.rs` (MIT,
//! Copyright 2026 Wing, pinned rev 8b7afb3), phase `material-background` T2.
//!
//! Ported verbatim: [`BackgroundLuminance`] with its `raster_image` /
//! `render_image` job cache, `scanline_pixels`, `ascii_pixels`,
//! `halftone_pixels`, `dither_pixels`, `sample_cover`/`cover_index`,
//! `source_from_image`, `decode_source`, the path cache (`background_luminance`)
//! and [`prepare`], plus upstream's pure effect tests.
//!
//! Adaptations from upstream (recorded, no behaviour stubs):
//! - The effect type: upstream used `NewThreadBackgroundEffect` from its
//!   persisted settings model. That type lives in mono-desk's app crate
//!   (`src/app_settings.rs`, T1), which a vendored crate must not depend on, so
//!   this module owns [`BackgroundEffect`] with the same five variants. The app
//!   converts at its call sites.
//! - [`prepare`] is `pub` (not upstream's `pub(super)`): the app crate is a
//!   separate crate from this vendored one and needs the entry point.
//! - The source decoder is inlined here (`image::ImageReader` with a guessed
//!   format) instead of calling zeron's `new_thread_background_image::decode`;
//!   it is the same one-line contract — inspect the exact bytes, not the path.
//! - Trimmed: the composer/hero crossfade pipeline (`Readiness`,
//!   `ArtworkFrame`, `crossfade_mix`, motion `WALLPAPER_CROSSFADE` timing) and
//!   the preload/install bridge (`PreloadedArtwork`, `install`, `color`). Those
//!   serve zeron's chat hero, which mono-desk does not paint (plan NG4); the
//!   Appearance page and its adjustment dialog only need a prepared
//!   `Arc<RenderImage>` for the installed image. The window-drawing tests that
//!   needed gpui's `test-support` feature are not ported (the vendored crate
//!   does not enable it); the pure raster tests are.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use gpui::{Pixels, px};

use crate::theme::{Appearance, Theme};

/// The background effect treatment. Mirrors the app's persisted
/// `NewThreadBackgroundEffect` (same words/variants); see the header.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BackgroundEffect {
    None,
    Dither,
    Ascii,
    Halftone,
    Scanlines,
}

#[derive(Debug)]
struct BackgroundLuminance {
    width: u32,
    height: u32,
    pixels: Box<[u8]>,
    colors: Box<[[u8; 4]]>,
    effects: Mutex<Vec<EffectEntry>>,
}

type EffectEntry = ((BackgroundEffect, bool), Option<Arc<gpui::RenderImage>>);

impl BackgroundLuminance {
    fn raster_image(
        self: &Arc<Self>,
        effect: BackgroundEffect,
        light: bool,
        cx: &mut gpui::App,
    ) -> Option<Arc<gpui::RenderImage>> {
        let mut effects = self.effects.lock().unwrap();
        let key = (
            effect,
            light
                && !matches!(
                    effect,
                    BackgroundEffect::Dither | BackgroundEffect::None
                ),
        );
        if let Some((_, image)) = effects.iter().find(|(cached, _)| *cached == key) {
            return image.clone();
        }
        // None marks the single pending job for this source/effect, not a viewport.
        effects.push((key, None));
        drop(effects);
        let source = self.clone();
        cx.spawn(async move |cx| {
            let worker = source.clone();
            let image = cx
                .background_executor()
                .spawn(async move { worker.render_image(effect, light) })
                .await;
            cx.update(|cx| {
                if let Some((_, ready)) = source
                    .effects
                    .lock()
                    .unwrap()
                    .iter_mut()
                    .find(|(cached, _)| *cached == key)
                {
                    *ready = Some(image);
                }
                cx.refresh_windows();
            });
        })
        .detach();
        None
    }

    fn render_image(&self, effect: BackgroundEffect, light: bool) -> Arc<gpui::RenderImage> {
        let pixels = match effect {
            BackgroundEffect::None => {
                image::RgbaImage::from_fn(self.width, self.height, |x, y| {
                    let [r, g, b, a] = self.colors[(y * self.width + x) as usize];
                    image::Rgba([b, g, r, a])
                })
            }
            BackgroundEffect::Dither => self.dither_pixels(self.width, self.height),
            BackgroundEffect::Halftone => self.halftone_pixels(self.width, self.height, light),
            BackgroundEffect::Ascii => self.ascii_pixels(light),
            BackgroundEffect::Scanlines => self.scanline_pixels(light),
        };
        Arc::new(gpui::RenderImage::new([image::Frame::new(pixels)]))
    }

    fn scanline_pixels(&self, light: bool) -> image::RgbaImage {
        image::RgbaImage::from_fn(self.width, self.height, |x, y| {
            let [r, g, b, a] = self.colors[(y * self.width + x) as usize];
            let gain = if y % 3 == 0 { 0.52 } else { 1.0 };
            let channel = |value: u8| {
                if light {
                    (value as f32 + (255.0 - value as f32) * (1.0 - gain)) as u8
                } else {
                    (value as f32 * gain) as u8
                }
            };
            image::Rgba([channel(b), channel(g), channel(r), a])
        })
    }

    fn ascii_pixels(&self, light: bool) -> image::RgbaImage {
        // Five-column bitmap glyphs, one column/row of spacing. These are
        // artwork pixels rather than thousands of shaped UI text runs.
        const GLYPHS: [[u8; 7]; 10] = [
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 4, 0],
            [0, 4, 0, 0, 4, 0, 0],
            [0, 0, 0, 14, 0, 0, 0],
            [0, 0, 14, 0, 14, 0, 0],
            [0, 4, 4, 31, 4, 4, 0],
            [0, 21, 14, 31, 14, 21, 0],
            [10, 10, 31, 10, 31, 10, 10],
            [17, 2, 4, 4, 8, 16, 17],
            [14, 17, 23, 21, 23, 16, 14],
        ];
        image::RgbaImage::from_fn(self.width, self.height, |x, y| {
            let sx = (x / 6 * 6 + 3).min(self.width - 1);
            let sy = (y / 8 * 8 + 4).min(self.height - 1);
            let sample = (sy * self.width + sx) as usize;
            let ink_density = if light {
                255 - self.pixels[sample]
            } else {
                self.pixels[sample]
            };
            let index = ((ink_density as f32 / 255.0).sqrt() * 9.0) as usize;
            let ink =
                x % 6 < 5 && y % 8 < 7 && GLYPHS[index][y as usize % 8] & (1 << (4 - x % 6)) != 0;
            let [r, g, b, a] = self.colors[(y * self.width + x) as usize];
            let [cr, cg, cb, _] = self.colors[sample];
            let mix = |base: u8, glyph: u8| {
                let paper = if light { 255.0 } else { 0.0 };
                // Keep a colored image beneath the glyph texture in both themes.
                (base as f32 * 0.60
                    + if ink {
                        glyph as f32 * 0.40
                    } else {
                        paper * 0.40
                    }) as u8
            };
            image::Rgba([mix(b, cb), mix(g, cg), mix(r, cr), a])
        })
    }

    fn halftone_pixels(&self, width: u32, height: u32, light: bool) -> image::RgbaImage {
        let bounds = gpui::size(px(width as f32), px(height as f32));
        let paper = if light { 255 } else { 0 };
        let mut pixels =
            image::RgbaImage::from_pixel(width, height, image::Rgba([paper, paper, paper, 255]));
        for y in (0..height).step_by(4) {
            for x in (0..width).step_by(4) {
                let luma = self.sample_cover(bounds, x as f32, y as f32);
                let luma = if light { 255 - luma } else { luma };
                let radius = 2.0 * (0.3 + 0.7 * (luma as f32 / 255.0).sqrt());
                let [r, g, b, a] =
                    self.colors[self.cover_index(bounds, x as f32 + 2.0, y as f32 + 2.0)];
                for dy in 0..4.min(height - y) {
                    for dx in 0..4.min(width - x) {
                        let distance =
                            ((dx as f32 - 1.5).powi(2) + (dy as f32 - 1.5).powi(2)).sqrt();
                        let coverage = (radius + 0.5 - distance).clamp(0.0, 1.0) * a as f32 / 255.0;
                        let [sr, sg, sb, sa] = self.colors[((y + dy) * width + x + dx) as usize];
                        let blend = |source: u8, dot: u8| {
                            (source as f32 * 0.60
                                + (dot as f32 * coverage + paper as f32 * (1.0 - coverage)) * 0.40)
                                as u8
                        };
                        pixels.put_pixel(
                            x + dx,
                            y + dy,
                            image::Rgba([blend(sb, b), blend(sg, g), blend(sr, r), sa]),
                        );
                    }
                }
            }
        }
        pixels
    }

    fn dither_pixels(&self, width: u32, height: u32) -> image::RgbaImage {
        const BAYER: [[u8; 4]; 4] = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]];
        let bounds = gpui::size(px(width as f32), px(height as f32));
        let mut pixels = image::RgbaImage::new(width, height);
        for y in (0..height).step_by(2) {
            for x in (0..width).step_by(2) {
                // Sample/quantize once per dot, not four times per 2x2 cell.
                let index = self.cover_index(bounds, (x + 1) as f32, (y + 1) as f32);
                let [r, g, b, a] = dither_color(
                    self.colors[index],
                    BAYER[y as usize / 2 % 4][x as usize / 2 % 4],
                );
                for dy in 0..2.min(height - y) {
                    for dx in 0..2.min(width - x) {
                        // RenderImage consumes BGRA.
                        pixels.put_pixel(x + dx, y + dy, image::Rgba([b, g, r, a]));
                    }
                }
            }
        }
        pixels
    }

    fn sample_cover(&self, bounds: gpui::Size<Pixels>, x: f32, y: f32) -> u8 {
        self.pixels[self.cover_index(bounds, x, y)]
    }

    fn cover_index(&self, bounds: gpui::Size<Pixels>, x: f32, y: f32) -> usize {
        let width = f32::from(bounds.width).max(1.0);
        let height = f32::from(bounds.height).max(1.0);
        let source_width = self.width as f32;
        let source_height = self.height as f32;
        let scale = (width / source_width).max(height / source_height);
        let visible_width = width / scale;
        let visible_height = height / scale;
        let source_x = ((source_width - visible_width) * 0.5 + x / scale)
            .clamp(0.0, source_width - 1.0) as u32;
        let source_y = ((source_height - visible_height) * 0.5 + y / scale)
            .clamp(0.0, source_height - 1.0) as u32;
        (source_y * self.width + source_x) as usize
    }
}

type Source = Arc<Mutex<Option<Arc<BackgroundLuminance>>>>;
type Cache = Vec<(PathBuf, Source)>;
static CACHE: OnceLock<Mutex<Cache>> = OnceLock::new();

/// Decode the exact bytes the app copied into its managed directory (guess the
/// format from the bytes, not the file extension).
fn decode(bytes: &[u8]) -> Option<image::DynamicImage> {
    image::ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .ok()?
        .decode()
        .ok()
}

fn decode_source(bytes: &[u8]) -> Option<Arc<BackgroundLuminance>> {
    Some(source_from_image(&decode(bytes)?))
}

fn source_from_image(image: &image::DynamicImage) -> Arc<BackgroundLuminance> {
    let proxy = image.thumbnail(2048, 2048);
    let gray = proxy.to_luma8();
    Arc::new(BackgroundLuminance {
        width: gray.width(),
        height: gray.height(),
        pixels: gray.into_raw().into_boxed_slice(),
        colors: proxy.to_rgba8().pixels().map(|pixel| pixel.0).collect(),
        effects: Mutex::new(Vec::new()),
    })
}

fn background_luminance(path: &Path, cx: &mut gpui::App) -> Option<Arc<BackgroundLuminance>> {
    let cache = CACHE.get_or_init(|| Mutex::new(Vec::new()));
    if let Some(source) = cache
        .lock()
        .ok()?
        .iter()
        .find_map(|(key, source)| (key == path).then(|| source.clone()))
    {
        return source.lock().ok()?.clone();
    }
    let pending = Arc::new(Mutex::new(None));
    {
        let mut cache = cache.lock().ok()?;
        cache.push((path.to_path_buf(), pending.clone()));
        if cache.len() > 4 {
            cache.remove(0);
        }
    }
    let path = path.to_path_buf();
    cx.spawn(async move |cx| {
        let source = cx
            .background_executor()
            .spawn(async move {
                let bytes = std::fs::read(path).ok()?;
                decode_source(&bytes)
            })
            .await;
        cx.update(|cx| {
            *pending.lock().unwrap() = source;
            cx.refresh_windows();
        });
    })
    .detach();
    None
}

/// The decoded source for `path` with `effect` applied, cached by
/// (path, effect, appearance). Returns `None` while the source or the effect
/// raster is still being prepared off-thread; the next frame picks it up.
pub fn prepare(
    effect: BackgroundEffect,
    theme: &Theme,
    path: &Path,
    cx: &mut gpui::App,
) -> Option<Arc<gpui::RenderImage>> {
    let light = matches!(theme.appearance, Appearance::Light);
    background_luminance(path, cx).and_then(|source| source.raster_image(effect, light, cx))
}

fn dither_color([r, g, b, a]: [u8; 4], threshold: u8) -> [u8; 4] {
    let peak = r.max(g).max(b) as f32;
    let bright = peak / 255.0 > (threshold as f32 + 0.5) / 16.0;
    let gain = if bright { 255.0 / peak.max(1.0) } else { 0.08 };
    [
        (r as f32 * gain).round() as u8,
        (g as f32 * gain).round() as u8,
        (b as f32 * gain).round() as u8,
        a,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Arc<BackgroundLuminance> {
        Arc::new(BackgroundLuminance {
            width: 60,
            height: 32,
            pixels: vec![128; 1920].into_boxed_slice(),
            colors: vec![[128, 64, 32, 200]; 1920].into_boxed_slice(),
            effects: Mutex::new(Vec::new()),
        })
    }

    #[test]
    fn light_treatments_use_light_paper_without_inverting_source_hues() {
        let source = fixture();
        for (light, dark) in [
            (source.ascii_pixels(true), source.ascii_pixels(false)),
            (source.scanline_pixels(true), source.scanline_pixels(false)),
            (
                source.halftone_pixels(60, 32, true),
                source.halftone_pixels(60, 32, false),
            ),
        ] {
            let brightness = |image: &image::RgbaImage| -> u64 {
                image
                    .pixels()
                    .map(|p| p.0[..3].iter().map(|c| u64::from(*c)).sum::<u64>())
                    .sum()
            };
            assert!(brightness(&light) > brightness(&dark));
            assert_eq!(light.dimensions(), dark.dimensions());
            // Raster output is BGRA; the warm source remains warm on light paper.
            assert!(light.pixels().all(|p| p.0[2] >= p.0[1] && p.0[1] >= p.0[0]));
        }
    }

    #[test]
    fn raster_treatments_preserve_source_dimensions_and_alpha() {
        let source = fixture();
        for image in [
            source.dither_pixels(60, 32),
            source.ascii_pixels(false),
            source.scanline_pixels(false),
            source.ascii_pixels(true),
            source.scanline_pixels(true),
        ] {
            assert_eq!(image.dimensions(), (60, 32));
            assert!(image.pixels().all(|pixel| pixel.0[3] == 200));
        }
        assert_eq!(source.halftone_pixels(60, 32, false).dimensions(), (60, 32));
    }

    #[test]
    fn dithering_quantizes_bright_and_dark_dots_without_dropping_alpha() {
        // Above the threshold the dot is normalized to full brightness.
        assert_eq!(dither_color([128, 64, 32, 180], 0), [255, 128, 64, 180]);
        // At the darkest thresholds a near-black dot stays near-black.
        let dark = dither_color([2, 1, 0, 200], 15);
        assert_eq!([dark[0], dark[1], dark[2]], [0, 0, 0]);
        assert_eq!(dark[3], 200);
    }
}
