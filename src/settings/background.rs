//! The new-thread background image: validate a chosen file, copy it into the
//! app's managed directory, remember its framing/effect, and delete only files
//! this setting owns.
//!
//! Ported/adapted from zeron's `crates/ui/src/settings.rs` (MIT, Copyright
//! 2026 Wing, pinned rev 8b7afb3) — `install_new_thread_composer_background`,
//! `prepare_background_file`, `commit_background`,
//! `remove_new_thread_composer_background`,
//! `set_new_thread_background_adjustment`, `set_new_thread_background_effect`
//! (upstream lines ~588-790) and the `new_thread_background_image::decode`
//! contract. The typed state (`NewThreadBackground`,
//! `NewThreadBackgroundAdjustment`, `NewThreadBackgroundEffect`) is T1's model
//! in `src/app_settings.rs`; this module mutates it and persists, it does not
//! redefine it.
//!
//! Adaptations from upstream (recorded, no behaviour stubs):
//! - The data directory is `persist::config_dir()` (upstream read zeron's
//!   settings store); the managed subdirectory keeps zeron's name,
//!   [`NEW_THREAD_BACKGROUND_DIR`].
//! - The unique managed file name cannot use `uuid::Uuid` (not a dependency):
//!   it is a [`DefaultHasher`] content hash plus a process sequence number and
//!   `SystemTime` nanoseconds, so replacing the image never reuses a name a
//!   stale image cache could still hold.
//! - No `attachments::StagedAttachment` staging layer (zeron's attachment
//!   pipeline is not vendored): the file is read once, validated from those
//!   exact bytes, written to a temp file and renamed into place, mirroring
//!   zeron's "validate the exact bytes that will be saved" rule.
//! - No wallpaper history / preload queue: mono-desk's persisted model has no
//!   history key (T1's frozen key set). The install still records
//!   `wallpaper_source` and samples `wallpaper_color`, as zeron's commit did.
//! - Error strings are zeron's, with the app's own name in the two
//!   "restart" messages (no zeron identity, plan NG5).
//! - The save gate is upstream's (`settings.rs:690-702`, `:717-726`): the
//!   persisted pointer is saved explicitly, and a failed save restores the
//!   previous state and discards the staged copy instead of deleting the old
//!   managed file. `super::persist_now` is only used by the wording/effect
//!   setters, which never delete files.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use gpui::App;
use zeron_theme::Color;

use super::wallpaper;
use super::wallpaper_colors;
use crate::app_settings::{
    AppSettings, NewThreadBackground, NewThreadBackgroundAdjustment, NewThreadBackgroundEffect,
};
use crate::persist;

/// zeron's managed directory under the app's data directory.
pub(crate) const NEW_THREAD_BACKGROUND_DIR: &str = "new-thread-backgrounds";

/// zeron's decode failure message (upstream `settings.rs:606`).
pub(crate) const ERROR_UNSUPPORTED: &str =
    "This background image is unsupported or damaged. Choose a valid image such as PNG or JPEG.";
/// zeron's copy failure message (upstream `settings.rs:637`).
pub(crate) const ERROR_SAVE: &str =
    "Unable to save the image. Check folder permissions and try again.";
/// zeron's missing-data-directory message on install, with this app's name.
pub(crate) const ERROR_NO_DATA_DIR: &str =
    "Unable to save the image. Restart mono-desk and try again.";
/// zeron's missing-data-directory message on remove, with this app's name.
pub(crate) const ERROR_REMOVE_NO_DATA_DIR: &str =
    "Unable to remove the image. Restart mono-desk and try again.";
/// zeron's failed-save message on remove (upstream `settings.rs:720`).
pub(crate) const ERROR_REMOVE_SAVE: &str =
    "Unable to remove the image. Check folder permissions and try again.";

/// The managed background directory inside the app's data directory.
pub(crate) fn backgrounds_dir() -> Option<PathBuf> {
    persist::config_dir().map(|dir| dir.join(NEW_THREAD_BACKGROUND_DIR))
}

/// Install (or replace) the new-thread background from `source`.
///
/// The file is read and decoded before anything is persisted; the managed copy
/// is written before the setting points at it; the persisted pointer is saved
/// before the previous managed file is retired. If that save fails, the
/// previous state is restored and the staged copy is discarded — the old file
/// is never deleted while disk still points at it (zeron's commit order).
pub(crate) fn install(source: &Path, cx: &mut App) -> Result<(), String> {
    let backgrounds = backgrounds_dir().ok_or_else(|| ERROR_NO_DATA_DIR.to_string())?;
    let (background, color) = stage(source, &backgrounds)?;
    let previous = PreviousState::capture(AppSettings::global(cx));
    let staged_path = background.path.clone();
    {
        let settings = AppSettings::global_mut(cx);
        settings.wallpaper_source = Some(source.to_path_buf());
        settings.wallpaper_color = color;
        settings.new_thread_background = Some(background);
    }
    let saved = persist::save(&AppSettings::global(cx).to_stored());
    commit_install(
        AppSettings::global_mut(cx),
        previous,
        &staged_path,
        &backgrounds,
        saved,
    )?;
    cx.refresh_windows();
    Ok(())
}

/// Remove the installed background: clear the state, save, then delete the
/// managed copy (never a hand-edited or legacy path outside the directory).
/// A failed save restores the previous state and keeps the file, so disk and
/// memory never disagree and no still-referenced image is deleted.
pub(crate) fn remove(cx: &mut App) -> Result<(), String> {
    let previous = PreviousState::capture(AppSettings::global(cx));
    if previous.background.is_none() {
        return Ok(());
    }
    let backgrounds = backgrounds_dir().ok_or_else(|| ERROR_REMOVE_NO_DATA_DIR.to_string())?;
    {
        let settings = AppSettings::global_mut(cx);
        settings.new_thread_background = None;
        settings.wallpaper_source = None;
        settings.wallpaper_color = None;
    }
    let saved = persist::save(&AppSettings::global(cx).to_stored());
    commit_remove(AppSettings::global_mut(cx), previous, &backgrounds, saved)?;
    cx.refresh_windows();
    Ok(())
}

/// The background state an install/remove replaces, kept so a failed save can
/// restore memory exactly (upstream built `next` and persisted it before
/// `replace(next, ...)` for the same reason).
#[derive(Clone)]
struct PreviousState {
    background: Option<NewThreadBackground>,
    source: Option<PathBuf>,
    color: Option<Color>,
}

impl PreviousState {
    fn capture(settings: &AppSettings) -> Self {
        Self {
            background: settings.new_thread_background.clone(),
            source: settings.wallpaper_source.clone(),
            color: settings.wallpaper_color,
        }
    }

    fn restore(self, settings: &mut AppSettings) {
        settings.new_thread_background = self.background;
        settings.wallpaper_source = self.source;
        settings.wallpaper_color = self.color;
    }
}

/// The save gate for [`install`], with the save result passed in so the
/// failure path is unit-testable headlessly. On a failed save: restore the
/// previous state and discard the staged managed copy (it is unreferenced);
/// never touch the file the previous state pointed at. On success the replaced
/// file is retired now that the pointer on disk is durable.
fn commit_install(
    settings: &mut AppSettings,
    previous: PreviousState,
    staged_path: &Path,
    backgrounds_dir: &Path,
    saved: bool,
) -> Result<(), String> {
    if !saved {
        previous.restore(settings);
        remove_managed(staged_path, backgrounds_dir);
        return Err(ERROR_SAVE.to_string());
    }
    if let Some(previous) = &previous.background {
        remove_managed(&previous.path, backgrounds_dir);
    }
    Ok(())
}

/// The save gate for [`remove`]; on a failed save the installed background
/// stays in memory and on disk, and the error names the failing operation.
fn commit_remove(
    settings: &mut AppSettings,
    previous: PreviousState,
    backgrounds_dir: &Path,
    saved: bool,
) -> Result<(), String> {
    if !saved {
        previous.restore(settings);
        return Err(ERROR_REMOVE_SAVE.to_string());
    }
    if let Some(previous) = &previous.background {
        remove_managed(&previous.path, backgrounds_dir);
    }
    Ok(())
}

/// Save the framing for the installed background (clamped by
/// [`NewThreadBackgroundAdjustment::normalized`]); no-op without a background.
pub(crate) fn set_adjustment(adjustment: NewThreadBackgroundAdjustment, cx: &mut App) -> bool {
    let changed = apply_adjustment(AppSettings::global_mut(cx), adjustment);
    if changed {
        super::persist_now(cx);
        cx.refresh_windows();
    }
    changed
}

/// Save the background effect; repaints only when the value changed.
pub(crate) fn set_effect(effect: NewThreadBackgroundEffect, cx: &mut App) -> bool {
    let changed = apply_effect(AppSettings::global_mut(cx), effect);
    if changed {
        super::persist_now(cx);
        cx.refresh_windows();
    }
    changed
}

/// Copy and validate `source` into `backgrounds_dir`, returning the new
/// background value and its sampled color. Pure filesystem work, split out of
/// [`install`] so the managed-copy behaviour is testable without a gpui
/// context.
fn stage(
    source: &Path,
    backgrounds_dir: &Path,
) -> Result<(NewThreadBackground, Option<Color>), String> {
    let bytes = std::fs::read(source).map_err(|_| ERROR_UNSUPPORTED.to_string())?;
    let image =
        wallpaper_colors::decode_image(&bytes).ok_or_else(|| ERROR_UNSUPPORTED.to_string())?;
    let color = wallpaper_colors::color_of(&image);
    std::fs::create_dir_all(backgrounds_dir).map_err(|_| ERROR_SAVE.to_string())?;
    let extension = image_extension(source);
    let destination = backgrounds_dir.join(unique_file_name(&bytes, &extension));
    write_atomically(&destination, &bytes)?;
    let name = source
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| format!("background.{extension}"));
    Ok((
        NewThreadBackground {
            path: destination,
            name,
            adjustment: NewThreadBackgroundAdjustment::default(),
        },
        color,
    ))
}

/// The managed file's extension: the source's own when it is one the app can
/// decode (keeps zeron's "name the copy after the picked file" behaviour),
/// else `png`, because the bytes are a valid image regardless of a misleading
/// source suffix.
fn image_extension(source: &Path) -> String {
    source
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .filter(|extension| wallpaper::SUPPORTED_EXTENSIONS.contains(&extension.as_str()))
        .unwrap_or_else(|| "png".to_string())
}

/// Increments on every install, so two installs in the same nanosecond cannot
/// collide.
static INSTALL_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// A unique managed name: content hash + process sequence + wall-clock
/// nanoseconds, with the image extension. zeron used a random v4 uuid; this
/// keeps the same property (a replaced image never reuses a name) without a
/// new dependency.
fn unique_file_name(bytes: &[u8], extension: &str) -> String {
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    let content_hash = hasher.finish();
    let sequence = INSTALL_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_nanos());
    format!("new-thread-background-{content_hash:016x}-{sequence}-{nanos}.{extension}")
}

/// Write to a sibling temp file and rename into place; cleans the temp file up
/// when either step fails.
fn write_atomically(destination: &Path, bytes: &[u8]) -> Result<(), String> {
    let temporary = destination.with_extension("tmp");
    if std::fs::write(&temporary, bytes)
        .and_then(|()| std::fs::rename(&temporary, destination))
        .is_err()
    {
        let _ = std::fs::remove_file(&temporary);
        return Err(ERROR_SAVE.to_string());
    }
    Ok(())
}

/// Delete `path` only when it sits directly inside the managed directory —
/// zeron's guard against removing an arbitrary legacy or hand-edited path.
/// Returns whether a file was removed.
fn remove_managed(path: &Path, backgrounds_dir: &Path) -> bool {
    if path.parent() != Some(backgrounds_dir) {
        return false;
    }
    std::fs::remove_file(path).is_ok()
}

/// The state half of [`set_adjustment`], split for unit tests.
fn apply_adjustment(settings: &mut AppSettings, adjustment: NewThreadBackgroundAdjustment) -> bool {
    let Some(background) = settings.new_thread_background.as_mut() else {
        return false;
    };
    let normalized = adjustment.normalized();
    if background.adjustment == normalized {
        return false;
    }
    background.adjustment = normalized;
    true
}

/// The state half of [`set_effect`], split for unit tests.
fn apply_effect(settings: &mut AppSettings, effect: NewThreadBackgroundEffect) -> bool {
    if settings.new_thread_background_effect == effect {
        return false;
    }
    settings.new_thread_background_effect = effect;
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_png(path: &Path, rgba: [u8; 4]) {
        image::RgbaImage::from_pixel(4, 4, image::Rgba(rgba))
            .save(path)
            .unwrap();
    }

    fn installed_background(path: &str) -> NewThreadBackground {
        NewThreadBackground {
            path: PathBuf::from(path),
            name: "background.png".into(),
            adjustment: NewThreadBackgroundAdjustment::default(),
        }
    }

    #[test]
    fn valid_images_are_copied_into_the_managed_directory_and_sampled() {
        let temp = tempfile::tempdir().unwrap();
        let managed = temp.path().join(NEW_THREAD_BACKGROUND_DIR);
        let source = temp.path().join("picked.png");
        write_png(&source, [30, 130, 220, 255]);

        let (background, color) = stage(&source, &managed).unwrap();
        assert_eq!(background.name, "picked.png");
        assert_eq!(background.path.parent(), Some(managed.as_path()));
        assert_eq!(
            background.adjustment,
            NewThreadBackgroundAdjustment::default()
        );
        assert!(background.path.is_file());
        assert_eq!(
            std::fs::read(&background.path).unwrap(),
            std::fs::read(&source).unwrap()
        );
        assert_eq!(
            color.map(|color| (color.r, color.g, color.b)),
            Some((30, 130, 220))
        );
    }

    #[test]
    fn damaged_images_are_rejected_with_zerons_message() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("damaged.png");
        std::fs::write(&source, b"not an image").unwrap();
        let Err(error) = stage(&source, &temp.path().join(NEW_THREAD_BACKGROUND_DIR)) else {
            panic!("damaged bytes must not be installed");
        };
        assert_eq!(error, ERROR_UNSUPPORTED);
    }

    #[test]
    fn managed_deletion_never_touches_paths_outside_the_directory() {
        let temp = tempfile::tempdir().unwrap();
        let managed = temp.path().join(NEW_THREAD_BACKGROUND_DIR);
        let nested_dir = managed.join("nested");
        std::fs::create_dir_all(&nested_dir).unwrap();
        let inside = managed.join("new-thread-background-inside.png");
        let nested = nested_dir.join("new-thread-background-nested.png");
        let outside = temp.path().join("keep.png");
        for path in [&inside, &nested, &outside] {
            std::fs::write(path, b"x").unwrap();
        }

        assert!(remove_managed(&inside, &managed));
        assert!(!inside.exists());
        assert!(!remove_managed(&outside, &managed));
        assert!(outside.is_file());
        assert!(!remove_managed(&nested, &managed));
        assert!(nested.is_file());
    }

    #[test]
    fn repeated_installs_of_the_same_bytes_use_distinct_managed_names() {
        let temp = tempfile::tempdir().unwrap();
        let managed = temp.path().join(NEW_THREAD_BACKGROUND_DIR);
        let source = temp.path().join("picked.png");
        write_png(&source, [30, 130, 220, 255]);

        let (first, _) = stage(&source, &managed).unwrap();
        let (second, _) = stage(&source, &managed).unwrap();
        assert_ne!(first.path, second.path);
        assert!(first.path.is_file() && second.path.is_file());
    }

    #[test]
    fn adjustment_is_normalized_and_ignored_without_a_background() {
        let mut settings = AppSettings::default();
        assert!(!apply_adjustment(
            &mut settings,
            NewThreadBackgroundAdjustment {
                focal_x: 0.2,
                focal_y: 0.2,
                zoom: 2.0,
            }
        ));

        settings.new_thread_background = Some(installed_background("/managed/bg.png"));
        assert!(apply_adjustment(
            &mut settings,
            NewThreadBackgroundAdjustment {
                focal_x: f32::NAN,
                focal_y: 5.0,
                zoom: -1.0,
            }
        ));
        let adjustment = settings.new_thread_background.as_ref().unwrap().adjustment;
        assert_eq!(adjustment.focal_x, 0.5);
        assert_eq!(adjustment.focal_y, 1.0);
        assert_eq!(adjustment.zoom, 1.0);
        assert!(!apply_adjustment(&mut settings, adjustment));
    }

    #[test]
    fn effect_writes_report_whether_they_changed_the_setting() {
        let mut settings = AppSettings::default();
        assert!(apply_effect(
            &mut settings,
            NewThreadBackgroundEffect::Dither
        ));
        assert_eq!(
            settings.new_thread_background_effect,
            NewThreadBackgroundEffect::Dither
        );
        assert!(!apply_effect(
            &mut settings,
            NewThreadBackgroundEffect::Dither
        ));
    }

    /// An installed background with its managed file on disk, plus the source
    /// and sampled color the install recorded.
    fn installed_state(managed: &Path) -> (AppSettings, NewThreadBackground) {
        std::fs::create_dir_all(managed).unwrap();
        let previous_path = managed.join("new-thread-background-previous.png");
        std::fs::write(&previous_path, b"previous").unwrap();
        let previous = NewThreadBackground {
            path: previous_path,
            name: "previous.png".into(),
            adjustment: NewThreadBackgroundAdjustment {
                focal_x: 0.2,
                focal_y: 0.8,
                zoom: 2.0,
            },
        };
        let settings = AppSettings {
            new_thread_background: Some(previous.clone()),
            wallpaper_source: Some(PathBuf::from("/wallpapers/previous.png")),
            wallpaper_color: Some(Color::rgb(10, 20, 30)),
            ..AppSettings::default()
        };
        (settings, previous)
    }

    #[test]
    fn failed_install_save_restores_the_previous_state_and_discards_the_staged_copy() {
        let temp = tempfile::tempdir().unwrap();
        let managed = temp.path().join(NEW_THREAD_BACKGROUND_DIR);
        let source = temp.path().join("picked.png");
        write_png(&source, [30, 130, 220, 255]);
        let (mut settings, previous) = installed_state(&managed);
        let previous_state = PreviousState::capture(&settings);

        // The in-memory install write happens before the save attempt.
        let (staged, color) = stage(&source, &managed).unwrap();
        let staged_path = staged.path.clone();
        settings.new_thread_background = Some(staged);
        settings.wallpaper_source = Some(source);
        settings.wallpaper_color = color;

        let error = commit_install(&mut settings, previous_state, &staged_path, &managed, false)
            .unwrap_err();
        assert_eq!(error, ERROR_SAVE);
        assert_eq!(settings.new_thread_background, Some(previous.clone()));
        assert_eq!(
            settings.wallpaper_source,
            Some(PathBuf::from("/wallpapers/previous.png"))
        );
        assert_eq!(
            settings
                .wallpaper_color
                .map(|color| (color.r, color.g, color.b)),
            Some((10, 20, 30))
        );
        assert!(!staged_path.exists(), "staged copy must be discarded");
        assert!(previous.path.is_file(), "old managed file must survive");
    }

    #[test]
    fn successful_install_save_retires_the_replaced_managed_copy() {
        let temp = tempfile::tempdir().unwrap();
        let managed = temp.path().join(NEW_THREAD_BACKGROUND_DIR);
        let source = temp.path().join("picked.png");
        write_png(&source, [30, 130, 220, 255]);
        let (mut settings, previous) = installed_state(&managed);
        let previous_state = PreviousState::capture(&settings);

        let (staged, color) = stage(&source, &managed).unwrap();
        let staged_path = staged.path.clone();
        settings.new_thread_background = Some(staged.clone());
        settings.wallpaper_source = Some(source);
        settings.wallpaper_color = color;

        commit_install(&mut settings, previous_state, &staged_path, &managed, true).unwrap();
        assert_eq!(settings.new_thread_background, Some(staged.clone()));
        assert!(staged_path.is_file());
        assert!(
            !previous.path.exists(),
            "replaced managed copy retires after a durable save"
        );
    }

    #[test]
    fn failed_remove_save_restores_the_installed_background_and_keeps_the_file() {
        let temp = tempfile::tempdir().unwrap();
        let managed = temp.path().join(NEW_THREAD_BACKGROUND_DIR);
        let (mut settings, previous) = installed_state(&managed);
        let previous_state = PreviousState::capture(&settings);

        // The in-memory remove write happens before the save attempt.
        settings.new_thread_background = None;
        settings.wallpaper_source = None;
        settings.wallpaper_color = None;

        let error = commit_remove(&mut settings, previous_state, &managed, false).unwrap_err();
        assert_eq!(error, ERROR_REMOVE_SAVE);
        assert_eq!(settings.new_thread_background, Some(previous.clone()));
        assert_eq!(
            settings.wallpaper_source,
            Some(PathBuf::from("/wallpapers/previous.png"))
        );
        assert_eq!(
            settings
                .wallpaper_color
                .map(|color| (color.r, color.g, color.b)),
            Some((10, 20, 30))
        );
        assert!(previous.path.is_file(), "managed file must survive");
    }

    #[test]
    fn successful_remove_save_deletes_only_the_managed_copy() {
        let temp = tempfile::tempdir().unwrap();
        let managed = temp.path().join(NEW_THREAD_BACKGROUND_DIR);
        let (mut settings, previous) = installed_state(&managed);
        let previous_state = PreviousState::capture(&settings);
        settings.new_thread_background = None;
        settings.wallpaper_source = None;
        settings.wallpaper_color = None;

        commit_remove(&mut settings, previous_state, &managed, true).unwrap();
        assert_eq!(settings.new_thread_background, None);
        assert!(!previous.path.exists(), "removed copy is deleted");
    }
}
