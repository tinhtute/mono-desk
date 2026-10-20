use std::borrow::Cow;

use gpui_kit::{AssetSource, Result, SharedString};

const BRAND_MARK: &str = "brand/brand-mark.svg";
/// Rasterized mark: GPUI renders `svg()` as a monochrome alpha mask tinted by
/// the text color, so full-color brand art has to go through `img()`.
const BRAND_MARK_PNG: &str = "brand/brand-mark.png";

/// App asset source: serves the vendored brand mark and delegates every
/// other path (notably `IconName` icons) to the gpui-kit full bundle —
/// the complete Lucide catalog, not just the default component subset.
pub struct AppAssets;

impl AssetSource for AppAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        match path {
            BRAND_MARK => Ok(Some(Cow::Borrowed(
                &include_bytes!("../assets/brand/brand-mark.svg")[..],
            ))),
            BRAND_MARK_PNG => Ok(Some(Cow::Borrowed(
                &include_bytes!("../assets/brand/brand-mark.png")[..],
            ))),
            _ => gpui_kit::assets::AllAssets.load(path),
        }
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut paths = gpui_kit::assets::AllAssets.list(path)?;
        for brand_path in [BRAND_MARK, BRAND_MARK_PNG] {
            if brand_path.starts_with(path) {
                paths.push(brand_path.into());
            }
        }
        Ok(paths)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_kit::assets::{IconName, IconNamed};

    #[test]
    fn brand_mark_loads_and_icons_still_delegate() {
        let source = AppAssets;
        let brand = source
            .load(BRAND_MARK)
            .expect("brand mark load")
            .expect("brand mark present");
        assert!(brand.starts_with(b"<svg"));

        let png = source
            .load(BRAND_MARK_PNG)
            .expect("brand png load")
            .expect("brand png present");
        assert!(png.starts_with(&[0x89, b'P', b'N', b'G']));
        assert!(png.len() > 1024, "full-size mark, not a placeholder");

        // A default-bundle icon delegates byte-for-byte…
        let icon_path = IconNamed::path(IconName::Search);
        let icon = source
            .load(icon_path.as_ref())
            .expect("icon load")
            .expect("icon present");
        let delegate = gpui_kit::assets::AllAssets
            .load(icon_path.as_ref())
            .expect("delegate load")
            .expect("delegate icon present");
        assert_eq!(icon.as_ref(), delegate.as_ref());

        // …and the full catalog covers glyphs the default bundle lacks.
        let plane = source
            .load(IconNamed::path(IconName::Plane).as_ref())
            .expect("full-catalog icon load")
            .expect("full-catalog icon present");
        assert!(plane.starts_with(b"<svg"));
    }
}
