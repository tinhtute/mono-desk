//! The settings pages, built on zeron's page scaffolding
//! (`vendor/zeron-ui/src/settings/widgets.rs`) and zeron's own select
//! primitive (`vendor/zeron-ui/src/settings/select.rs`).
//!
//! Phase `settings-surface` T1 lands General and Notifications. The page
//! *content* is mono-desk's own (i18n strings, `AppSettings`, persistence);
//! the page structure — `page_column`, `page_header`, `section`,
//! `section_card`, `card_row`, `row_title`, `meta_line`, `toggle_switch`,
//! `select` — is zeron's, ported verbatim, so the two pages read as the same
//! product surface as the vendored chrome.
//!
//! Page state (open select menus, scroll offsets) lives on the page views,
//! like zeron's page views; writes go straight into the `AppSettings` global
//! and through [`persist_now`], and the affected view repaints itself.

mod general;
mod notifications;

pub(crate) use general::GeneralPage;
pub(crate) use notifications::NotificationsPage;

use gpui::{AnyElement, App, IntoElement, ParentElement as _, SharedString, Styled as _, div, px};

use zeron_ui::settings::widgets;
use zeron_ui::theme::Theme;

use crate::app_settings::AppSettings;
use crate::persist;

/// The write-through path every settings control uses: mirror the live global
/// into the persisted struct and save it. `persist::save` is synchronous and
/// already atomic + guarded (see its module docs), so a page can call this
/// from a click handler without owning any save machinery.
pub(crate) fn persist_now(cx: &mut App) {
    let stored = AppSettings::global(cx).to_stored();
    persist::save(&stored);
}

/// zeron's settings row: title + muted meta on the left, the control on the
/// right, split from the previous row by the card's inset hairline. Mirrors
/// the row shapes in zeron's own pages (e.g. `settings/appearance.rs`'s
/// motion rows) on top of the ported `card_row`/`row_title`/`meta_line`.
pub(crate) fn control_row(
    theme: &Theme,
    first: bool,
    title: impl Into<SharedString>,
    meta: impl Into<SharedString>,
    control: impl IntoElement,
) -> gpui::Div {
    widgets::card_row(theme, first)
        .child(
            div()
                .flex_1()
                .min_w(px(160.0))
                .child(widgets::row_title(theme, title))
                .child(widgets::meta_line(
                    theme,
                    vec![div().child(meta.into()).into_any_element() as AnyElement],
                )),
        )
        .child(control)
}
