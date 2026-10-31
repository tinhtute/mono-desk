//! Paint-time crop geometry for the new-thread background artwork: cover-fit
//! with the persisted framing, direct-manipulation pan/zoom math, and the
//! adjusted canvas paint used by the Appearance page's preview.
//!
//! Adapted from zeron's `crates/ui/src/new_thread_background_mask.rs` (MIT,
//! Copyright 2026 Wing, pinned rev 8b7afb3), phase `material-background` T2.
//!
//! Ported verbatim (the pure geometry and paint helpers):
//! [`FittedGeometry`], `fitted_geometry`, `source_geometry`, `pan_adjustment`,
//! `zoom_adjustment_around`, `adjusted_image_paint`, `paint_adjusted`, and
//! upstream's fitted-geometry / pan / zoom / adjusted-paint tests (locally
//! adapted to [`Framing`], see below).
//!
//! Adaptations from upstream (recorded, no behaviour stubs):
//! - The adjustment type: upstream took zeron's `NewThreadBackgroundAdjustment`
//!   from its settings store. That type lives in mono-desk's app crate
//!   (`src/app_settings.rs`, T1), which a vendored crate must not depend on, so
//!   this module owns [`Framing`] — the same three fields and the same clamps
//!   (`focal_x`/`focal_y` in `0..=1`, `zoom` in [`MIN_ZOOM`]..=[`MAX_ZOOM`]),
//!   `0.5/0.5/1.0` default. The app converts at the two call sites and keeps
//!   `NewThreadBackgroundAdjustment` as the persisted shape.
//! - Trimmed: the composer/cutout alpha-mask pipeline (`CUTOUT_REVEAL_OPACITY`,
//!   `SurfaceBounds`, `mask`, masked `paint`, the reveal-pass tests). mono-desk
//!   has no new-thread composer or composer geometry (plan NG4), and the
//!   Appearance preview paints without a mask — `paint_adjusted` is the only
//!   paint entry point the page uses.
//! - `paint_adjusted` and [`Framing`] are `pub` (not upstream's `pub(crate)`):
//!   the app crate is a separate crate from this vendored one.

use std::sync::Arc;

use gpui::{Bounds, Corners, Pixels, Point, RenderImage, Window, point, px, size};

/// zeron's adjustment zoom bounds (upstream `NewThreadBackgroundAdjustment`
/// `MIN_ZOOM`/`MAX_ZOOM`; the app's persisted type uses the same values).
pub const MIN_ZOOM: f32 = 1.0;
pub const MAX_ZOOM: f32 = 4.0;

/// Viewport-relative framing of the background image: normalized focal
/// alignment plus magnification. The vendored mirror of the app's persisted
/// `NewThreadBackgroundAdjustment` (same fields, same clamps).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Framing {
    /// Normalized horizontal alignment: 0 left, 1 right.
    pub focal_x: f32,
    /// Normalized vertical alignment: 0 top, 1 bottom.
    pub focal_y: f32,
    /// Multiplier applied after the image has been scaled to cover the pane.
    pub zoom: f32,
}

impl Default for Framing {
    fn default() -> Self {
        Self {
            focal_x: 0.5,
            focal_y: 0.5,
            zoom: MIN_ZOOM,
        }
    }
}

impl Framing {
    /// Clamp every field; a non-finite field falls back to its default rather
    /// than reaching the renderer (the app's `.normalized()` rule).
    pub fn normalized(mut self) -> Self {
        let defaults = Self::default();
        self.focal_x = clamp_or(self.focal_x, 0.0, 1.0, defaults.focal_x);
        self.focal_y = clamp_or(self.focal_y, 0.0, 1.0, defaults.focal_y);
        self.zoom = clamp_or(self.zoom, MIN_ZOOM, MAX_ZOOM, defaults.zoom);
        self
    }
}

fn clamp_or(value: f32, min: f32, max: f32, fallback: f32) -> f32 {
    if value.is_finite() {
        value.clamp(min, max)
    } else {
        fallback
    }
}

#[derive(Clone, Copy)]
struct FittedGeometry {
    bounds: Bounds<Pixels>,
    width: f32,
    height: f32,
    overflow_x: f32,
    overflow_y: f32,
}

fn fitted_geometry(
    source_width: f32,
    source_height: f32,
    bounds: Bounds<Pixels>,
    framing: Framing,
) -> Option<FittedGeometry> {
    let width = f32::from(bounds.size.width);
    let height = f32::from(bounds.size.height);
    if width <= 0.0
        || height <= 0.0
        || !width.is_finite()
        || !height.is_finite()
        || source_width <= 0.0
        || source_height <= 0.0
        || !source_width.is_finite()
        || !source_height.is_finite()
    {
        return None;
    }

    let framing = framing.normalized();
    let cover = (width / source_width).max(height / source_height);
    let fitted_width = (source_width * cover * framing.zoom).max(width);
    let fitted_height = (source_height * cover * framing.zoom).max(height);
    let overflow_x = (fitted_width - width).max(0.0);
    let overflow_y = (fitted_height - height).max(0.0);
    let fitted = Bounds::new(
        point(
            bounds.left() - px(overflow_x * framing.focal_x),
            bounds.top() - px(overflow_y * framing.focal_y),
        ),
        size(px(fitted_width), px(fitted_height)),
    );
    Some(FittedGeometry {
        bounds: fitted,
        width: fitted_width,
        height: fitted_height,
        overflow_x,
        overflow_y,
    })
}

fn source_geometry(
    source: &RenderImage,
    bounds: Bounds<Pixels>,
    framing: Framing,
) -> Option<FittedGeometry> {
    let source_size = source.size(0);
    fitted_geometry(
        source_size.width.0 as f32,
        source_size.height.0 as f32,
        bounds,
        framing,
    )
}

/// Translate a direct-manipulation drag into viewport-independent framing.
pub fn pan_adjustment(
    source: &RenderImage,
    bounds: Bounds<Pixels>,
    framing: Framing,
    delta: Point<Pixels>,
) -> Framing {
    let framing = framing.normalized();
    let Some(geometry) = source_geometry(source, bounds, framing) else {
        return framing;
    };
    let mut next = framing;
    if geometry.overflow_x > 0.0 {
        next.focal_x -= f32::from(delta.x) / geometry.overflow_x;
    }
    if geometry.overflow_y > 0.0 {
        next.focal_y -= f32::from(delta.y) / geometry.overflow_y;
    }
    next.normalized()
}

/// Preserve the source pixel beneath `anchor` while changing magnification.
pub fn zoom_adjustment_around(
    source: &RenderImage,
    bounds: Bounds<Pixels>,
    framing: Framing,
    zoom: f32,
    anchor: Point<Pixels>,
) -> Framing {
    let framing = framing.normalized();
    let Some(previous) = source_geometry(source, bounds, framing) else {
        return framing;
    };
    let mut next = Framing { zoom, ..framing }.normalized();
    let Some(fitted) = source_geometry(source, bounds, next) else {
        return framing;
    };

    let source_x = f32::from(anchor.x - previous.bounds.left()) / previous.width;
    let source_y = f32::from(anchor.y - previous.bounds.top()) / previous.height;
    if fitted.overflow_x > 0.0 {
        let desired_left = f32::from(anchor.x) - source_x * fitted.width;
        next.focal_x = (f32::from(bounds.left()) - desired_left) / fitted.overflow_x;
    }
    if fitted.overflow_y > 0.0 {
        let desired_top = f32::from(anchor.y) - source_y * fitted.height;
        next.focal_y = (f32::from(bounds.top()) - desired_top) / fitted.overflow_y;
    }
    next.normalized()
}

fn adjusted_image_paint(
    source: &RenderImage,
    bounds: Bounds<Pixels>,
    framing: Framing,
    corner_radii: Corners<Pixels>,
) -> Option<(Bounds<Pixels>, Bounds<Pixels>, Corners<Pixels>)> {
    let fitted = source_geometry(source, bounds, framing)?;
    Some((bounds, fitted.bounds, corner_radii))
}

/// Paint the exact responsive crop the persisted framing describes, without a
/// composer mask. Appearance uses this for a faithful adjustment preview.
pub fn paint_adjusted(
    source: Arc<RenderImage>,
    bounds: Bounds<Pixels>,
    framing: Framing,
    corner_radii: Corners<Pixels>,
    window: &mut Window,
) {
    let Some((visible, fitted, corner_radii)) =
        adjusted_image_paint(&source, bounds, framing, corner_radii)
    else {
        return;
    };
    let _ = window.paint_image_fitted_masked(visible, fitted, corner_radii, source, 0, false, None);
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{point, px};

    fn hero(width: f32, height: f32) -> Bounds<Pixels> {
        Bounds::new(point(px(40.0), px(24.0)), size(px(width), px(height)))
    }

    fn fitted(source: (f32, f32), viewport: Bounds<Pixels>, framing: Framing) -> FittedGeometry {
        fitted_geometry(source.0, source.1, viewport, framing).unwrap()
    }

    fn assert_near(actual: Pixels, expected: Pixels) {
        assert!(
            (f32::from(actual - expected)).abs() < 0.001,
            "expected {expected:?}, got {actual:?}"
        );
    }

    fn source(width: u32, height: u32) -> RenderImage {
        RenderImage::new([image::Frame::new(image::RgbaImage::from_pixel(
            width,
            height,
            image::Rgba([79, 151, 233, 180]),
        ))])
    }

    #[test]
    fn default_framing_is_the_existing_centered_cover_crop() {
        let viewport = hero(1000.0, 500.0);
        let crop = fitted((1600.0, 900.0), viewport, Framing::default());
        assert_eq!(crop.bounds.size.width, px(1000.0));
        assert_eq!(crop.bounds.size.height, px(562.5));
        assert_eq!(crop.bounds.left(), viewport.left());
        assert_eq!(crop.bounds.top(), viewport.top() - px(31.25));
        assert_eq!(crop.bounds.center(), viewport.center());
    }

    #[test]
    fn focal_extremes_align_the_overflowing_image_edges() {
        let viewport = hero(1000.0, 500.0);
        let start = fitted(
            (1600.0, 900.0),
            viewport,
            Framing {
                focal_x: 0.0,
                focal_y: 0.0,
                zoom: 2.0,
            },
        );
        assert_eq!(start.bounds.left(), viewport.left());
        assert_eq!(start.bounds.top(), viewport.top());

        let end = fitted(
            (1600.0, 900.0),
            viewport,
            Framing {
                focal_x: 1.0,
                focal_y: 1.0,
                zoom: 2.0,
            },
        );
        assert_near(end.bounds.right(), viewport.right());
        assert_near(end.bounds.bottom(), viewport.bottom());
    }

    #[test]
    fn zoom_multiplies_cover_before_positioning() {
        let viewport = hero(1000.0, 500.0);
        let base = fitted((1600.0, 900.0), viewport, Framing::default());
        let zoomed = fitted(
            (1600.0, 900.0),
            viewport,
            Framing {
                zoom: 2.0,
                ..Framing::default()
            },
        );
        assert_near(zoomed.bounds.size.width, base.bounds.size.width * 2.0);
        assert_near(zoomed.bounds.size.height, base.bounds.size.height * 2.0);
        assert_eq!(zoomed.bounds.center(), viewport.center());
    }

    #[test]
    fn resizing_preserves_the_normalized_focal_alignment() {
        let framing = Framing {
            focal_x: 0.23,
            focal_y: 0.71,
            zoom: 2.2,
        };
        for viewport in [
            hero(420.0, 300.0),
            hero(960.0, 420.0),
            hero(1200.0, 760.0),
        ] {
            let crop = fitted((1600.0, 900.0), viewport, framing);
            let x = f32::from(viewport.left() - crop.bounds.left()) / crop.overflow_x;
            let y = f32::from(viewport.top() - crop.bounds.top()) / crop.overflow_y;
            assert!((x - framing.focal_x).abs() < 0.0001);
            assert!((y - framing.focal_y).abs() < 0.0001);
        }
    }

    #[test]
    fn every_supported_crop_covers_the_entire_viewport() {
        for source in [(900.0, 1600.0), (1600.0, 900.0), (1000.0, 1000.0)] {
            for viewport in [
                hero(360.0, 760.0),
                hero(1000.0, 500.0),
                hero(640.0, 640.0),
            ] {
                for focal_x in [0.0, 0.37, 1.0] {
                    for focal_y in [0.0, 0.61, 1.0] {
                        for zoom in [1.0, 1.8, MAX_ZOOM] {
                            let crop = fitted(
                                source,
                                viewport,
                                Framing {
                                    focal_x,
                                    focal_y,
                                    zoom,
                                },
                            );
                            assert!(crop.bounds.left() <= viewport.left());
                            assert!(crop.bounds.top() <= viewport.top());
                            assert!(crop.bounds.right() >= viewport.right());
                            assert!(crop.bounds.bottom() >= viewport.bottom());
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn pan_adjustment_tracks_direct_pointer_movement() {
        let source = source(16, 9);
        let viewport = hero(100.0, 50.0);
        let framing = Framing {
            zoom: 2.0,
            ..Framing::default()
        };
        let before = source_geometry(&source, viewport, framing).unwrap();
        let next = pan_adjustment(&source, viewport, framing, point(px(10.0), px(-5.0)));
        let after = source_geometry(&source, viewport, next).unwrap();
        assert_near(after.bounds.left(), before.bounds.left() + px(10.0));
        assert_near(after.bounds.top(), before.bounds.top() - px(5.0));
    }

    #[test]
    fn pointer_anchored_zoom_keeps_the_same_source_pixel_underneath() {
        let source = source(16, 9);
        let viewport = hero(100.0, 50.0);
        let framing = Framing {
            zoom: 1.5,
            ..Framing::default()
        };
        let anchor = point(viewport.left() + px(60.0), viewport.top() + px(30.0));
        let before = source_geometry(&source, viewport, framing).unwrap();
        let source_x = f32::from(anchor.x - before.bounds.left()) / before.width;
        let source_y = f32::from(anchor.y - before.bounds.top()) / before.height;

        let next = zoom_adjustment_around(&source, viewport, framing, 2.4, anchor);
        let after = source_geometry(&source, viewport, next).unwrap();
        let next_source_x = f32::from(anchor.x - after.bounds.left()) / after.width;
        let next_source_y = f32::from(anchor.y - after.bounds.top()) / after.height;
        assert!((next_source_x - source_x).abs() < 0.0001);
        assert!((next_source_y - source_y).abs() < 0.0001);
    }

    #[test]
    fn preview_paint_keeps_rounded_viewport_corners_when_panning_and_zooming() {
        let source = source(16, 9);
        let viewport = hero(100.0, 50.0);
        let rounded = Corners::all(px(11.0));
        let initial = Framing::default();
        let zoomed = zoom_adjustment_around(&source, viewport, initial, 2.0, viewport.center());
        let panned = pan_adjustment(&source, viewport, zoomed, point(px(10.0), px(-5.0)));

        for framing in [initial, zoomed, panned] {
            let (visible, fitted, corners) =
                adjusted_image_paint(&source, viewport, framing, rounded).unwrap();
            let (hero_visible, hero_fitted, hero_corners) =
                adjusted_image_paint(&source, viewport, framing, Corners::all(px(0.0))).unwrap();
            assert_eq!(visible, viewport);
            assert_eq!(corners, rounded);
            assert_eq!(hero_corners, Corners::all(px(0.0)));
            assert_eq!((visible, fitted), (hero_visible, hero_fitted));
            assert_eq!(
                fitted,
                source_geometry(&source, viewport, framing).unwrap().bounds
            );
        }
    }

    #[test]
    fn normalized_framing_clamps_non_finite_and_out_of_range_values() {
        let framing = Framing {
            focal_x: f32::NAN,
            focal_y: 5.0,
            zoom: -1.0,
        }
        .normalized();
        assert_eq!(framing.focal_x, 0.5);
        assert_eq!(framing.focal_y, 1.0);
        assert_eq!(framing.zoom, MIN_ZOOM);
    }
}
