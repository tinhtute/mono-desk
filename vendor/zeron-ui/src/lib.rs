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
//! (`docs/plans/active/zeron-stack-port.md`); the chrome and settings surfaces
//! consume the rest in later phases.
//!
//! Licensing: zeron is MIT (see `LICENSE`); the icon set is Solar Icons by 480
//! Design (CC BY 4.0); the GPUI fork is Apache-2.0. See
//! `THIRD_PARTY_NOTICES.md` at the repository root.

// The port is incremental: this phase lands the visual foundation, while the
// functions that only the later chrome/settings phases call stay unused for now.
#![allow(dead_code)]
// Vendored upstream source. Clippy's evolving style lints (derivable impls,
// argument counts, collapsible branches) are upstream's to fix: patching them
// here would diverge from the pinned rev and complicate re-sync. First-party
// code under `src/` is still linted with `-D warnings`.
#![allow(clippy::all)]


pub mod chrome;
pub mod frost;
pub(crate) mod glass;
pub mod highlight;
pub mod icons;
pub mod loader_math;
pub mod motion;
pub mod theme;
pub mod typography;

pub use theme::{AccentColor, Appearance, Theme};
