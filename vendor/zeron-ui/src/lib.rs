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
//! Licensing: zeron is MIT (see `LICENSE`); the UI glyph set is Phosphor Icons
//! v2.1.1 (MIT, https://phosphoricons.com/) and the brand marks keep zeron's
//! artwork; the GPUI fork is Apache-2.0. See `THIRD_PARTY_NOTICES.md` at the
//! repository root.

// The port is complete for the surfaces mono-desk uses; a few upstream helpers
// remain unused here (theme library, scrollbar rail, wallpaper plumbing) and
// keep the vendored crate warning-free without patching upstream style rules.
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
pub mod popover;
pub mod settings;
pub mod theme;
pub mod typography;

pub use theme::{AccentColor, Appearance, Theme};
