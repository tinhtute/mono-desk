//! Durable custom-theme library and the bridge into the active runtime
//! registry.
//!
//! Ported from zeron's `crates/ui/src/theme_library.rs` (MIT, Copyright 2026
//! Wing, pinned rev 8b7afb3) in phase `state-persistence` T3. The domain model
//! (`zeron_theme::{CustomThemeLibrary, CustomThemeEntry, InstallMode,
//! ThemeRegistry, replace_custom_families}`) is vendored verbatim; this module
//! keeps upstream's behavior — the global [`ThemeLibraryState`], the
//! load/activate path, the clone-mutate-save-activate mutation pattern and its
//! failure guarantees — and adapts only the app-integration seams:
//!
//! - Upstream reads and writes its `crate::appearance` settings service after
//!   every mutation (`appearance::themes`/`set_theme`/`apply_registry_change`).
//!   mono-desk has no such service, so those writes are gone: [`reconcile`]
//!   is the pure replacement — the boot/appearance layer builds a
//!   [`ThemeSelection`] from the persisted `AppSettings::theme_light`/
//!   `theme_dark`, calls it, persists any correction, then funnels through
//!   `settings::apply_appearance`. Callers that mutate the library must run
//!   that sequence themselves; the mutation helpers only persist and activate
//!   the library.
//! - [`init`] takes the data directory from the caller (`persist::config_dir()`
//!   at boot) instead of reading an app-level global.
//! - Upstream logs the load failure with `tracing::warn!`; the app carries no
//!   tracing subscriber, so the warning is written to stderr and also kept on
//!   the state for the Appearance page ([`load_warning`]).

use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result, anyhow};
use gpui::{App, Global};
use zeron_theme::vscode::SourceCompilation;
use zeron_theme::{
    CustomThemeEntry, CustomThemeLibrary, InstallMode, ThemeRegistry, ThemeSelection,
    replace_custom_families,
};

pub(crate) struct ThemeLibraryState {
    pub(crate) data_dir: PathBuf,
    pub(crate) library: CustomThemeLibrary,
    pub(crate) load_warning: Option<String>,
}

impl Global for ThemeLibraryState {}

/// Install the theme library and its runtime families. Called once at boot,
/// before the first theme install (the reconciled selection must see the
/// custom variants through [`ThemeRegistry::active`]).
pub(crate) fn init(data_dir: impl Into<PathBuf>, cx: &mut App) {
    let data_dir = data_dir.into();
    let (library, load_warning) = load_state(&data_dir);
    library.install_runtime();
    cx.set_global(ThemeLibraryState {
        data_dir,
        library,
        load_warning,
    });
}

/// Read the durable library, degrading a corrupt/foreign file to an empty
/// library plus the user-visible warning (upstream's `init` branch).
fn load_state(data_dir: &Path) -> (CustomThemeLibrary, Option<String>) {
    match CustomThemeLibrary::load(data_dir) {
        Ok(library) => (library, None),
        Err(error) => {
            eprintln!("could not load custom theme library: {error}");
            (CustomThemeLibrary::default(), Some(error.to_string()))
        }
    }
}

pub(crate) fn entries(cx: &App) -> Vec<CustomThemeEntry> {
    cx.try_global::<ThemeLibraryState>()
        .map(|state| state.library.entries.clone())
        .unwrap_or_default()
}

pub(crate) fn load_warning(cx: &App) -> Option<String> {
    cx.try_global::<ThemeLibraryState>()
        .and_then(|state| state.load_warning.clone())
}

pub(crate) fn compile(
    path: &Path,
    family_id: &str,
    family_name: &str,
) -> Result<SourceCompilation> {
    CustomThemeLibrary::compile(path, family_id, family_name)
}

pub(crate) fn install(
    compilation: SourceCompilation,
    selected_variant_ids: &[String],
    mode: InstallMode,
    cx: &mut App,
) -> Result<String> {
    if !cx.has_global::<ThemeLibraryState>() {
        return Err(anyhow!("custom theme library is not initialized"));
    }
    let state = cx.global_mut::<ThemeLibraryState>();
    let mut next = state.library.clone();
    let id = next.install(compilation, selected_variant_ids, mode)?;
    persist_and_activate(state, next)?;
    Ok(id)
}

pub(crate) fn reload(id: &str, cx: &mut App) -> Result<()> {
    if !cx.has_global::<ThemeLibraryState>() {
        return Err(anyhow!("custom theme library is not initialized"));
    }
    let state = cx.global_mut::<ThemeLibraryState>();
    let mut next = state.library.clone();
    let result = next.reload(id);
    // Reload failures update the durable warning while deliberately preserving
    // the last known good family.
    persist_and_activate(state, next)?;
    result
}

pub(crate) fn unlink(id: &str, cx: &mut App) -> Result<()> {
    mutate(cx, |library| library.unlink(id))
}

/// Upstream exposes both duplicate modes; zeron's Appearance page only offers
/// "Duplicate as editable" (`appearance.rs:3321-3475`), so the snapshot variant
/// stays as API parity with the vendored bridge and is covered by its test.
#[allow(dead_code)]
pub(crate) fn duplicate_as_snapshot(id: &str, cx: &mut App) -> Result<String> {
    if !cx.has_global::<ThemeLibraryState>() {
        return Err(anyhow!("custom theme library is not initialized"));
    }
    let state = cx.global_mut::<ThemeLibraryState>();
    let mut next = state.library.clone();
    let duplicate = next.duplicate_as_snapshot(id)?;
    persist_and_activate(state, next)?;
    Ok(duplicate)
}

pub(crate) fn duplicate_as_editable(id: &str, cx: &mut App) -> Result<String> {
    if !cx.has_global::<ThemeLibraryState>() {
        return Err(anyhow!("custom theme library is not initialized"));
    }
    let state = cx.global_mut::<ThemeLibraryState>();
    let mut next = state.library.clone();
    let duplicate = next.duplicate_as_editable(id, &state.data_dir)?;
    persist_and_activate(state, next)?;
    Ok(duplicate)
}

pub(crate) fn remove(id: &str, cx: &mut App) -> Result<()> {
    mutate(cx, |library| {
        library
            .remove(id)
            .then_some(())
            .ok_or_else(|| anyhow!("unknown custom theme `{id}`"))
    })
}

pub(crate) fn reveal(id: &str, cx: &App) -> Result<()> {
    let path = cx
        .try_global::<ThemeLibraryState>()
        .and_then(|state| state.library.entry(id))
        .and_then(|entry| entry.source.path())
        .ok_or_else(|| anyhow!("theme source has no location to reveal"))?;
    cx.reveal_path(path);
    Ok(())
}

fn mutate<T>(
    cx: &mut App,
    operation: impl FnOnce(&mut CustomThemeLibrary) -> Result<T>,
) -> Result<T> {
    if !cx.has_global::<ThemeLibraryState>() {
        return Err(anyhow!("custom theme library is not initialized"));
    }
    let state = cx.global_mut::<ThemeLibraryState>();
    let mut next = state.library.clone();
    let output = operation(&mut next)?;
    persist_and_activate(state, next)?;
    Ok(output)
}

fn persist_and_activate(state: &mut ThemeLibraryState, next: CustomThemeLibrary) -> Result<()> {
    next.save(&state.data_dir)
        .context("could not save custom theme library")?;
    replace_custom_families(
        next.entries
            .iter()
            .map(|entry| entry.family.clone())
            .collect(),
    );
    state.library = next;
    state.load_warning = None;
    Ok(())
}

/// Fall a selected variant id that vanished from the active registry (e.g. the
/// custom family that owned it was removed or failed to reload) back to the
/// first variant for its appearance.
///
/// Returns whether the selection changed. This replaces upstream's
/// `reconcile_and_refresh`: the caller persists the corrected ids and
/// re-applies the appearance (`settings::apply_appearance`), which owns the
/// installed theme and the window background.
pub(crate) fn reconcile(selected: &mut ThemeSelection) -> bool {
    let registry = ThemeRegistry::active();
    let mut changed = false;
    for appearance in zeron_theme::Appearance::ALL {
        let selected_id = selected.variant_id(appearance);
        if registry.variant(selected_id).is_none()
            && let Some(fallback) = registry.variants_for(appearance).next()
        {
            selected.set_variant(appearance, fallback.id.clone());
            changed = true;
        }
    }
    changed
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::Mutex;

    use super::*;
    use zeron_theme::{
        Appearance, CustomThemeSource, CustomThemeStatus, ThemeFamily, ThemeSelection,
        replace_custom_families,
    };

    /// The runtime custom-family registry is process-global, so the tests that
    /// mutate it serialize and reset it afterwards; other crate tests must not
    /// observe a partially installed fixture family.
    static REGISTRY_LOCK: Mutex<()> = Mutex::new(());

    fn write_theme_fixture(dir: &Path, background: &str) -> PathBuf {
        let path = dir.join("fixture.json");
        fs::write(
            &path,
            format!(
                r##"{{
  "name": "Fixture",
  "type": "dark",
  "colors": {{ "editor.background": "{background}", "foreground": "#eeeeee" }}
}}"##
            ),
        )
        .unwrap();
        path
    }

    fn compile_fixture(path: &Path) -> SourceCompilation {
        CustomThemeLibrary::compile(path, "fixture-family", "Fixture Family")
            .expect("the fixture compiles")
    }

    fn selected_variant_ids(compilation: &SourceCompilation) -> Vec<String> {
        compilation
            .family
            .variants
            .iter()
            .map(|variant| variant.id.clone())
            .collect()
    }

    fn install_fixture(library: &mut CustomThemeLibrary, dir: &Path, mode: InstallMode) -> String {
        let path = write_theme_fixture(dir, "#101010");
        let compilation = compile_fixture(&path);
        let selected = selected_variant_ids(&compilation);
        library.install(compilation, &selected, mode).unwrap()
    }

    #[test]
    fn compile_minimal_vscode_theme_yields_its_variant_id() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_theme_fixture(dir.path(), "#101010");

        let compilation = compile_fixture(&path);

        assert_eq!(compilation.family.id, "fixture-family");
        assert_eq!(compilation.family.name, "Fixture Family");
        assert_eq!(compilation.family.variants.len(), 1);
        let variant = &compilation.family.variants[0];
        assert_eq!(variant.id, "fixture-family");
        assert_eq!(variant.family_id, "fixture-family");
        assert_eq!(variant.name, "Fixture");
        assert_eq!(variant.appearance, Appearance::Dark);
        assert_eq!(
            variant.colors.background,
            "#101010".parse::<zeron_theme::Color>().unwrap()
        );
    }

    #[test]
    fn linked_install_reload_and_remove_round_trip_through_the_library_file() {
        let _guard = REGISTRY_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();

        let mut library = CustomThemeLibrary::default();
        let id = install_fixture(&mut library, dir.path(), InstallMode::Link);
        library.save(dir.path()).unwrap();

        // The install is durable and the linked source survives a load.
        let mut loaded = CustomThemeLibrary::load(dir.path()).unwrap();
        let entry = loaded.entry(&id).expect("the installed entry persisted");
        assert!(entry.source.is_linked());

        // Reload recompiles the linked source after it changed on disk.
        fs::write(
            dir.path().join("fixture.json"),
            r##"{"name":"Fixture","type":"dark","colors":{"editor.background":"#202020","foreground":"#e0e0e0"}}"##,
        )
        .unwrap();
        loaded.reload(&id).unwrap();
        assert_eq!(
            loaded.entry(&id).unwrap().family.variants[0]
                .colors
                .background,
            "#202020".parse::<zeron_theme::Color>().unwrap()
        );
        loaded.save(dir.path()).unwrap();

        // Remove persists too: a fresh load sees an empty library.
        assert!(loaded.remove(&id));
        loaded.save(dir.path()).unwrap();
        assert!(
            CustomThemeLibrary::load(dir.path())
                .unwrap()
                .entries
                .is_empty()
        );
        replace_custom_families(Vec::new());
    }

    #[test]
    fn successful_persist_activates_the_next_library_and_clears_the_warning() {
        let _guard = REGISTRY_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let mut library = CustomThemeLibrary::default();
        install_fixture(&mut library, dir.path(), InstallMode::Snapshot);
        let mut state = ThemeLibraryState {
            data_dir: dir.path().to_path_buf(),
            library: CustomThemeLibrary::default(),
            load_warning: Some("previous warning".into()),
        };

        persist_and_activate(&mut state, library.clone()).unwrap();

        assert_eq!(state.library, library);
        assert!(state.load_warning.is_none());
        assert!(
            CustomThemeLibrary::load(dir.path())
                .unwrap()
                .entry("fixture-family")
                .is_some()
        );
        assert!(
            ThemeRegistry::active().variant("fixture-family").is_some(),
            "the runtime registry exposes the activated family"
        );
        replace_custom_families(Vec::new());
    }

    #[test]
    fn failed_persistence_leaves_library_state_unchanged() {
        let dir = tempfile::tempdir().unwrap();
        let blocking_file = dir.path().join("not-a-directory");
        fs::write(&blocking_file, "occupied").unwrap();
        let original = CustomThemeLibrary::default();
        let mut state = ThemeLibraryState {
            data_dir: blocking_file,
            library: original.clone(),
            load_warning: Some("previous warning".into()),
        };
        let mut next = original.clone();
        next.entries.push(CustomThemeEntry {
            id: "unpersisted".into(),
            name: "Unpersisted".into(),
            source: CustomThemeSource::ImportedSnapshot {
                imported_from: None,
            },
            family: ThemeFamily {
                id: "unpersisted".into(),
                name: "Unpersisted".into(),
                variants: Vec::new(),
            },
            reports: Default::default(),
            selected_variant_ids: Vec::new(),
            status: CustomThemeStatus::Ready,
        });

        assert!(persist_and_activate(&mut state, next).is_err());
        assert_eq!(state.library, original);
        assert_eq!(state.load_warning.as_deref(), Some("previous warning"));
    }

    #[test]
    fn reconcile_falls_back_a_vanished_variant_and_leaves_a_present_one_alone() {
        let mut selection = ThemeSelection::default();
        let fallback_light = ThemeRegistry::builtin()
            .variants_for(Appearance::Light)
            .next()
            .expect("the built-in registry has a light variant")
            .id
            .clone();
        let fallback_dark = ThemeRegistry::builtin()
            .variants_for(Appearance::Dark)
            .next()
            .expect("the built-in registry has a dark variant")
            .id
            .clone();

        // Both built-in selections are present: nothing to reconcile.
        assert!(!reconcile(&mut selection));

        // A vanished light selection falls back; the present dark one is kept.
        selection.light = "vanished-custom-light".into();
        selection.dark = fallback_dark.clone();
        assert!(reconcile(&mut selection));
        assert_eq!(selection.light, fallback_light);
        assert_eq!(selection.dark, fallback_dark);

        // Both selections are present now: idempotent.
        assert!(!reconcile(&mut selection));
    }

    #[test]
    fn corrupt_library_file_surfaces_a_load_warning() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(
            load_state(dir.path()),
            (CustomThemeLibrary::default(), None)
        );

        fs::write(CustomThemeLibrary::path(dir.path()), "not a theme library").unwrap();
        let (library, warning) = load_state(dir.path());
        assert!(library.entries.is_empty());
        assert!(
            warning.is_some(),
            "a corrupt library degrades to empty plus a warning"
        );
    }
}
