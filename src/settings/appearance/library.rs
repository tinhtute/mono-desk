//! Theme library card (phase `theme-library`): the unlabeled card after
//! Motion with the "Theme library"/"Add theme" row, Imported/Linked groups,
//! per-entry status rows and their actions, ported from zeron's
//! `crates/ui/src/settings/appearance.rs` :3321-3543 (entry row :3321-3475,
//! row assembly :3476-3543) and its page-column injection at :4267-4281.
//!
//! Wiring differs from upstream only where the app has no appearance service:
//! each action calls `crate::settings::theme_library`, then [`after_mutation`]
//! heals the persisted variant selection
//! (`settings::reconcile_persisted_selection`) and funnels through
//! `settings::apply_appearance` — upstream's `apply_registry_change`.
//!
//! All user-visible copy is localized; the source labels replace upstream's
//! runtime `CustomThemeSource::label()` words so the row copy follows the page
//! language. Paths, theme names, variant names and library error messages are
//! runtime data and stay as-is.

use gpui::prelude::*;
use gpui::{AnyElement, App, Context, IntoElement, Role, SharedString, Window, div, px};

use zeron_theme::{CustomThemeEntry, CustomThemeSource, CustomThemeStatus};
use zeron_ui::settings::widgets;
use zeron_ui::theme::Theme;
use zeron_ui::typography;

use crate::i18n::tr;
use crate::settings::theme_library;

use super::AppearancePage;

/// View state the library card owns (`library_error`/`review_entry`, upstream
/// :343-344; the import dialog state lives in `dialogs::library`).
#[derive(Default)]
pub(super) struct State {
    pub(super) error: Option<SharedString>,
    pub(super) review_entry: Option<String>,
}

/// The unlabeled theme-library card (upstream :4267-4271, `.mt(32)`), with the
/// load/per-action warning under it (upstream :4272-4280).
pub(super) fn section(
    _page: &mut AppearancePage,
    theme: &Theme,
    lang: &str,
    _window: &mut Window,
    cx: &mut Context<AppearancePage>,
) -> AnyElement {
    let mut rows: Vec<AnyElement> = vec![header_row(theme, lang, cx)];

    let entries = theme_library::entries(cx);
    let (linked, imported): (Vec<_>, Vec<_>) = entries
        .into_iter()
        .partition(|entry| entry.source.is_linked());

    if !imported.is_empty() {
        rows.push(group_label(theme, tr("appearance.library.imported", lang)));
        rows.extend(
            imported
                .into_iter()
                .map(|entry| entry_row(entry, theme, lang, cx)),
        );
    }
    if !linked.is_empty() {
        rows.push(group_label(theme, tr("appearance.library.linked", lang)));
        rows.extend(
            linked
                .into_iter()
                .map(|entry| entry_row(entry, theme, lang, cx)),
        );
    }

    widgets::section_card(theme)
        .mt(px(32.0))
        .children(rows)
        .into_any_element()
}

/// The library warning line: upstream renders it as the page-column tail
/// **after** the Fonts and layout section (`crates/ui/src/settings/appearance.rs
/// :4272-4280`), so the page assembles it there rather than under the card.
pub(super) fn warning(
    page: &mut AppearancePage,
    theme: &Theme,
    lang: &str,
    cx: &mut Context<AppearancePage>,
) -> Option<AnyElement> {
    let warning = page
        .library
        .error
        .as_deref()
        .map(|error| SharedString::from(library_error_text(error, lang)))
        .or_else(|| theme_library::load_warning(cx).map(SharedString::from))?;
    Some(
        div()
            .mt(px(8.0))
            .text_size(typography::ui_rems(11.5))
            .text_color(theme.warning)
            .child(warning)
            .into_any_element(),
    )
}

/// The card's first row: "Theme library" and the "Add theme" action
/// (upstream :3485-3503).
fn header_row(theme: &Theme, lang: &str, cx: &mut Context<AppearancePage>) -> AnyElement {
    widgets::card_row(theme, true)
        .child(
            div()
                .flex_1()
                .min_w(px(160.0))
                .child(widgets::row_title(theme, tr("appearance.library", lang))),
        )
        .child(
            widgets::text_action(
                theme,
                widgets::ActionTone::Solid,
                tr("appearance.library.add", lang),
            )
            .id("theme-library-add")
            .tab_index(0)
            .role(Role::Button)
            .aria_label(tr("appearance.library.add", lang))
            .focus_visible(|style| style.border_2().border_color(theme.accent).opacity(1.0))
            .on_click(cx.listener(|page, _, _, cx| {
                page.dialogs.library.open(cx);
                cx.notify();
            })),
        )
        .into_any_element()
}

/// A group label row ("Imported" / "Linked", upstream :3504-3527).
fn group_label(theme: &Theme, label: &'static str) -> AnyElement {
    div()
        .mx(px(16.0))
        .pt(px(12.0))
        .pb(px(4.0))
        .border_t_1()
        .border_color(widgets::row_divider(theme))
        .text_size(typography::ui_rems(12.0))
        .text_color(theme.text_muted)
        .child(label)
        .into_any_element()
}

/// One library entry row: name + status line on the left, the six actions on
/// the right (upstream :3321-3475).
fn entry_row(
    entry: CustomThemeEntry,
    theme: &Theme,
    lang: &str,
    cx: &mut Context<AppearancePage>,
) -> AnyElement {
    let id = entry.id.clone();
    let linked = entry.source.is_linked();
    let warning = matches!(entry.status, CustomThemeStatus::Warning { .. });
    let status = status_line(&entry, lang);

    let mut actions = div().flex_none().flex().items_center().gap(px(2.0));

    if linked {
        let action_id = id.clone();
        actions = actions.child(
            action(
                theme,
                tr("appearance.library.reload", lang),
                format!("theme-reload-{id}"),
            )
            .on_click(cx.listener(move |page, _, window, cx| {
                run(page, window, cx, |cx| theme_library::reload(&action_id, cx));
            })),
        );
    }

    let action_id = id.clone();
    actions = actions.child(
        action(
            theme,
            tr("appearance.library.reveal", lang),
            format!("theme-reveal-{id}"),
        )
        .on_click(cx.listener(move |page, _, _, cx| {
            if let Err(error) = theme_library::reveal(&action_id, cx) {
                page.library.error = Some(error.to_string().into());
                cx.notify();
            }
        })),
    );

    let action_id = id.clone();
    actions = actions.child(
        action(
            theme,
            tr("appearance.library.review", lang),
            format!("theme-review-{id}"),
        )
        .on_click(cx.listener(move |page, _, _, cx| {
            page.library.review_entry = Some(action_id.clone());
            cx.notify();
        })),
    );

    let action_id = id.clone();
    actions = actions.child(
        action(
            theme,
            tr("appearance.library.duplicate", lang),
            format!("theme-duplicate-{id}"),
        )
        .on_click(cx.listener(move |page, _, window, cx| {
            run(page, window, cx, |cx| {
                theme_library::duplicate_as_editable(&action_id, cx).map(|_| ())
            });
        })),
    );

    if linked {
        let action_id = id.clone();
        actions = actions.child(
            action(
                theme,
                tr("appearance.library.unlink", lang),
                format!("theme-unlink-{id}"),
            )
            .on_click(cx.listener(move |page, _, window, cx| {
                run(page, window, cx, |cx| theme_library::unlink(&action_id, cx));
            })),
        );
    }

    let action_id = id.clone();
    actions = actions.child(
        action(
            theme,
            tr("appearance.library.remove", lang),
            format!("theme-remove-{id}"),
        )
        .text_color(theme.danger)
        .on_click(cx.listener(move |page, _, window, cx| {
            run(page, window, cx, |cx| theme_library::remove(&action_id, cx));
        })),
    );

    widgets::card_row(theme, false)
        .child(
            div()
                .flex_1()
                .min_w(px(160.0))
                .child(widgets::row_title(theme, entry.name))
                .child(
                    div()
                        .truncate()
                        .text_size(typography::ui_rems(11.0))
                        .text_color(if warning {
                            theme.warning
                        } else {
                            theme.text_muted
                        })
                        .child(status),
                ),
        )
        .child(actions)
        .into_any_element()
}

/// The status line under an entry's name (upstream :3330-3345): localized
/// source kind, variant count, and the source path (or that it is a
/// self-contained snapshot); a warning entry names the last-known-good family
/// instead.
fn status_line(entry: &CustomThemeEntry, lang: &str) -> SharedString {
    if let CustomThemeStatus::Warning { message } = &entry.status {
        return format!(
            "{} · {}",
            tr("appearance.library.status.last_known_good", lang),
            message
        )
        .into();
    }
    let source = source_label(&entry.source, lang);
    let variants = entry.family.variants.len();
    let count = format!(
        "{} {}",
        variants,
        tr(
            if variants == 1 {
                "appearance.library.variant"
            } else {
                "appearance.library.variants"
            },
            lang
        )
    );
    let location = entry
        .source
        .path()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| tr("appearance.library.source.snapshot", lang).to_owned());
    format!("{source} · {count} · {location}").into()
}

/// zeron's runtime `CustomThemeSource::label()` words, localized
/// (upstream :3334).
fn source_label(source: &CustomThemeSource, lang: &str) -> &'static str {
    match source {
        CustomThemeSource::ImportedSnapshot { .. } => {
            tr("appearance.library.source.imported", lang)
        }
        CustomThemeSource::LinkedFile { .. } => tr("appearance.library.source.linked_file", lang),
        CustomThemeSource::LinkedPackage { .. } => {
            tr("appearance.library.source.linked_package", lang)
        }
        CustomThemeSource::EditableFile { .. } => {
            tr("appearance.library.source.editable_file", lang)
        }
    }
}

/// A row action in zeron's filled compact style with the shared focus ring
/// (upstream `compact_action`, :1707-1713 plus the per-action a11y chain).
fn action(
    theme: &Theme,
    label: &'static str,
    id: impl Into<SharedString>,
) -> gpui::Stateful<gpui::Div> {
    widgets::text_action(theme, widgets::ActionTone::Filled, label)
        .id(id.into())
        .tab_index(0)
        .role(Role::Button)
        .aria_label(label)
        .focus_visible(move |style| style.border_2().border_color(theme.accent).opacity(1.0))
}

/// Run one library mutation, then re-install the appearance. Failures are
/// Localize the bridge's known error words; unexpected library diagnostics
/// stay as the raw message (Log decision, same policy as the material rows).
fn library_error_text(error: &str, lang: &str) -> String {
    let key = if error.contains("not initialized") {
        Some("appearance.library.error.not_initialized")
    } else if error.contains("no location") || error.contains("has no path") {
        Some("appearance.library.error.no_location")
    } else if error.contains("save") || error.contains("write") || error.contains("persist") {
        Some("appearance.library.error.save")
    } else {
        None
    };
    key.map_or_else(|| error.to_owned(), |key| tr(key, lang).to_owned())
}

/// surfaced on the card, matching upstream's `library_error`.
fn run(
    page: &mut AppearancePage,
    window: &mut Window,
    cx: &mut Context<AppearancePage>,
    operation: impl FnOnce(&mut App) -> anyhow::Result<()>,
) {
    match operation(cx) {
        Ok(()) => after_mutation(page, window, cx),
        Err(error) => {
            page.library.error = Some(error.to_string().into());
            cx.notify();
        }
    }
}

/// Upstream's `reconcile_and_refresh`/`apply_registry_change`: heal the
/// persisted variant ids against the freshly activated registry, persist the
/// correction, and funnel the reinstall through the one appearance path.
pub(crate) fn after_mutation(
    page: &mut AppearancePage,
    window: &mut Window,
    cx: &mut Context<AppearancePage>,
) {
    page.library.error = None;
    crate::settings::reconcile_persisted_selection(cx);
    crate::settings::apply_appearance(window, cx);
    cx.notify();
}
