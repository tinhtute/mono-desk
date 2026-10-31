//! Wallpaper-folder domain: scan a folder for supported artwork and pick one
//! at random, persisting the folder and the chosen source.
//!
//! Ported/adapted from zeron's `crates/ui/src/settings/wallpaper.rs` (MIT,
//! Copyright 2026 Wing, pinned rev 8b7afb3) — the folder scan in `choose()` and
//! the folder/source writes from `crates/ui/src/settings/appearance.rs`
//! (`choose_wallpaper_folder`, `random_wallpaper`). The plan's
//! `state-persistence` phase (T4) only needs the testable domain helpers; the
//! Appearance page wires them in the `material-background` phase.
//!
//! Adaptations from upstream (recorded, no behaviour stubs):
//! - **Trim:** no artwork rendering, preload queue, effect generation or
//!   decode-on-pick (`PreloadQueue`, `Candidate`, `take_ready`, `LOOKAHEAD`).
//!   mono-desk paints no wallpaper (plan NG4: no chat/new-thread canvas), so
//!   picking needs only the file path; the picture pipeline lands with the
//!   page that shows it, not with this state layer.
//! - `choose()`'s "random independent rank per candidate" used `uuid::Uuid`
//!   v4 as the chance source; uuid is not a dependency here, so
//!   [`random_index`] derives its seed from `SystemTime` nanos plus a process
//!   counter (xorshift64*). Same intent: uniformly random pick, no RNG crate.
//! - Upstream kept `wallpaper_history` (recent-repeat cooldown). mono-desk's
//!   persisted model has `wallpaper_folder`/`wallpaper_source` but no history
//!   key (T1's frozen key set), so the selection is plain random without the
//!   cooldown; the folder/source writes stay the same.
//! - Errors are zeron's user-facing strings, verbatim except where the message
//!   has no upstream counterpart.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use gpui::App;

use crate::app_settings::AppSettings;

/// Image extensions the `image` dependency in the app manifest can decode
/// (`png`, `jpeg` — both `jpg`/`jpeg` —, `gif`, `webp`, `bmp`, `tiff` — both
/// `tif`/`tiff` —, `ico`). zeron's `choose()` also accepted `tif`; the shared
/// manifest enables `ico` on top of that list.
pub(crate) const SUPPORTED_EXTENSIONS: [&str; 9] = [
    "png", "jpg", "jpeg", "gif", "webp", "bmp", "tif", "tiff", "ico",
];

/// zeron's error when no folder has been chosen yet (also used when the stored
/// folder no longer exists: the user must choose one again in Appearance).
pub(crate) const CHOOSE_FOLDER_ERROR: &str = "Choose a wallpaper folder in Appearance first.";
/// zeron's error when the chosen folder exists but cannot be read.
pub(crate) const READ_FOLDER_ERROR: &str =
    "Unable to read the wallpaper folder. Choose an accessible folder in Appearance.";
/// zeron's error when the chosen folder holds no usable artwork.
pub(crate) const NO_WALLPAPERS_ERROR: &str = "No readable wallpapers found in this folder. Add images such as PNG or JPEG, or choose another folder.";

/// Whether `path` carries an extension the app can decode.
pub(crate) fn is_supported(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            let extension = extension.to_ascii_lowercase();
            SUPPORTED_EXTENSIONS.contains(&extension.as_str())
        })
}

/// Every supported image directly inside `folder`, sorted by path so the scan
/// is deterministic (the random pick happens afterwards).
pub(crate) fn supported_images(folder: &Path) -> Result<Vec<PathBuf>, String> {
    let entries = std::fs::read_dir(folder).map_err(|_| READ_FOLDER_ERROR.to_string())?;
    let mut images = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|_| READ_FOLDER_ERROR.to_string())?;
        let path = entry.path();
        if is_supported(&path) && path.is_file() {
            images.push(path);
        }
    }
    images.sort();
    Ok(images)
}

/// Pick one supported image from `folder` at random.
///
/// A missing (or no longer existing) folder asks the user to choose one;
/// a folder that exists but holds no supported image gets zeron's
/// no-wallpapers message. The pick does not decode the file — decoding belongs
/// to the renderer, so picking stays cheap and testable with placeholder files.
pub(crate) fn pick_random(folder: &Path) -> Result<PathBuf, String> {
    if !folder.is_dir() {
        return Err(CHOOSE_FOLDER_ERROR.to_string());
    }
    let images = supported_images(folder)?;
    if images.is_empty() {
        return Err(NO_WALLPAPERS_ERROR.to_string());
    }
    Ok(images[random_index(images.len())].clone())
}

/// Increments on every pick so two picks in the same nanosecond still differ.
static PICK_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// A uniformly distributed index below `len`; `len` is always non-zero.
///
/// Seed: wall-clock nanoseconds mixed with a per-process sequence number
/// (upstream's chance source was `uuid::Uuid::new_v4`, which is not a
/// dependency here), then xorshift64*.
fn random_index(len: usize) -> usize {
    let sequence = PICK_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_nanos() as u64);
    let mut state = nanos ^ sequence.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
    state ^= state >> 12;
    state ^= state << 25;
    state ^= state >> 27;
    let value = state.wrapping_mul(0x2545_F491_4F6C_DD1D);
    (value % len as u64) as usize
}

/// Persist the wallpaper folder (or clear it) and repaint.
pub(crate) fn set_folder(folder: Option<PathBuf>, cx: &mut App) {
    if apply_folder(AppSettings::global_mut(cx), folder) {
        super::persist_now(cx);
        cx.refresh_windows();
    }
}

/// Remember which file a shuffle/install displayed; device-local pointer only.
pub(crate) fn record_source(source: &Path, cx: &mut App) {
    if apply_source(AppSettings::global_mut(cx), Some(source)) {
        super::persist_now(cx);
    }
}

/// The state half of [`set_folder`], split so the write is unit-testable
/// without a gpui context.
fn apply_folder(settings: &mut AppSettings, folder: Option<PathBuf>) -> bool {
    if settings.wallpaper_folder == folder {
        return false;
    }
    settings.wallpaper_folder = folder;
    true
}

/// The state half of [`record_source`].
fn apply_source(settings: &mut AppSettings, source: Option<&Path>) -> bool {
    let source = source.map(Path::to_path_buf);
    if settings.wallpaper_source == source {
        return false;
    }
    settings.wallpaper_source = source;
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn placeholder(folder: &Path, name: &str) -> PathBuf {
        let path = folder.join(name);
        std::fs::write(&path, b"not decoded by this layer").unwrap();
        path
    }

    #[test]
    fn supported_extensions_match_the_images_the_app_can_decode() {
        for extension in SUPPORTED_EXTENSIONS {
            assert!(is_supported(Path::new(&format!("art.{extension}"))));
            assert!(is_supported(Path::new(&format!(
                "art.{}",
                extension.to_ascii_uppercase()
            ))));
        }
        assert!(!is_supported(Path::new("notes.txt")));
        assert!(!is_supported(Path::new("no-extension")));
    }

    #[test]
    fn scan_keeps_only_supported_files_and_is_sorted() {
        let temp = tempfile::tempdir().unwrap();
        let png = placeholder(temp.path(), "b.png");
        let jpeg = placeholder(temp.path(), "a.JPEG");
        placeholder(temp.path(), "notes.txt");
        std::fs::create_dir(temp.path().join("nested.png")).unwrap();

        let images = supported_images(temp.path()).unwrap();
        assert_eq!(images, vec![jpeg, png]);
    }

    #[test]
    fn random_pick_returns_one_of_the_supported_images() {
        let temp = tempfile::tempdir().unwrap();
        let png = placeholder(temp.path(), "one.png");
        let jpg = placeholder(temp.path(), "two.jpg");
        placeholder(temp.path(), "skip.txt");

        for _ in 0..32 {
            let picked = pick_random(temp.path()).unwrap();
            assert!(picked == png || picked == jpg, "{picked:?}");
        }
    }

    #[test]
    fn missing_folder_reports_zerons_choose_folder_message() {
        let temp = tempfile::tempdir().unwrap();
        let error = pick_random(&temp.path().join("missing")).unwrap_err();
        assert_eq!(error, CHOOSE_FOLDER_ERROR);
    }

    #[test]
    fn folder_without_supported_images_reports_zerons_no_wallpapers_message() {
        let temp = tempfile::tempdir().unwrap();
        assert_eq!(pick_random(temp.path()).unwrap_err(), NO_WALLPAPERS_ERROR);
        placeholder(temp.path(), "notes.txt");
        assert_eq!(pick_random(temp.path()).unwrap_err(), NO_WALLPAPERS_ERROR);
    }

    #[test]
    fn folder_and_source_writes_report_changes_and_mirror_the_settings() {
        let mut settings = AppSettings::default();
        let folder = PathBuf::from("/wallpapers");
        assert!(apply_folder(&mut settings, Some(folder.clone())));
        assert_eq!(settings.wallpaper_folder, Some(folder));
        assert!(!apply_folder(
            &mut settings,
            Some(PathBuf::from("/wallpapers"))
        ));
        assert!(apply_folder(&mut settings, None));
        assert_eq!(settings.wallpaper_folder, None);

        let source = PathBuf::from("/wallpapers/current.png");
        assert!(apply_source(&mut settings, Some(&source)));
        assert_eq!(settings.wallpaper_source, Some(source));
        let same = settings.wallpaper_source.clone();
        assert!(!apply_source(&mut settings, same.as_deref()));
        assert!(apply_source(&mut settings, None));
        assert_eq!(settings.wallpaper_source, None);
    }
}
