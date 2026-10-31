//! Vendored subset of zeron's UI source (MIT, Copyright 2026 Wing).
//!
//! Source: <https://github.com/zeronsh/zeron>, `crates/ui` @ 8b7afb3. The
//! modules here are adapted copies, not drop-in upstream updates: upstream they
//! import zeron's domain crates (`zeron_syntax`, `zeron_proto`, its persisted
//! `settings` store, the `gpui_base` fork) and paint a chat/session workspace.
//! This port keeps the layout/animation constants, token math and widget call
//! shape and replaces the domain imports with the local ones documented in each
//! file header.
//!
//! Ported in the plan's `theme-foundation` phase
//! (`docs/plans/completed/zeron-stack-port.md`); the chrome and settings
//! surfaces consume the rest, and `docs/plans/active/appearance-parity.md`
//! extends this kit for zeron's Appearance page (scroll rail, installed-font
//! catalog, wallpaper tint).
//!
//! Licensing: zeron is MIT (see `LICENSE`); the icon set is Solar Icons by 480
//! Design (CC BY 4.0); the GPUI fork is Apache-2.0. See
//! `THIRD_PARTY_NOTICES.md` at the repository root.

// A few upstream helpers remain unused here (the theme-library UI bridge will
// live in `src/settings/theme_library.rs` from phase `state-persistence` T3,
// and zeron's settings-search metadata is not ported), which keeps the
// vendored crate warning-free without patching upstream style rules.
#![allow(dead_code)]
// Vendored upstream source. Clippy's evolving style lints (derivable impls,
// argument counts, collapsible branches) are upstream's to fix: patching them
// here would diverge from the pinned rev and complicate re-sync. First-party
// code under `src/` is still linted with `-D warnings`.
#![allow(clippy::all)]


pub mod chrome;
pub mod edge_fade;
pub mod frost;
pub(crate) mod glass;
pub mod highlight;
pub mod icons;
pub mod loader_math;
pub mod motion;
pub mod new_thread_background_effects;
pub mod new_thread_background_mask;
pub mod popover;
pub mod settings;
pub mod theme;
pub mod typography;

pub use theme::{AccentColor, Appearance, Theme};
