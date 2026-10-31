//! Theme import + theme mapping review dialogs (phase `theme-library`),
//! ported from zeron's `crates/ui/src/settings/appearance.rs` :2205-2746
//! (import dialog), :3267-3319 (mapping review) and the shared preview/report
//! helpers at :1715-1812 (`import_scene_preview`) and :1814-1881
//! (`report_panel`).
//!
//! Adaptations from upstream:
//! - zeron's `ComposerInput` is not vendored; the source field is the app's
//!   [`TextField`] (created lazily on open, one entity per dialog), observed so
//!   page state (footer enabled state) repaints while typing.
//! - zeron resolves a `Theme` from the *compiled* variant with
//!   `Theme::from_variant` (crate-private in the vendored UI crate). The
//!   import previews therefore paint from the variant's own model colors
//!   ([`VariantPalette`]) — same semantic roles, no registry install needed.
//! - zeron's `appearance::apply_registry_change` has no app counterpart:
//!   a successful install runs `super::super::library::after_mutation`, which
//!   heals the persisted variant ids and funnels through
//!   `settings::apply_appearance`.
//! - Upstream's page-level `dismiss_on_escape` lives in its Settings shell;
//!   here the modal card handles Escape itself (focus is inside the dialog).
//!
//! Dialog copy is localized through `crate::i18n::tr`; theme names, variant
//! names, paths and compiler diagnostics are runtime data.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use gpui::prelude::*;
use gpui::{
    AnyElement, Context, Entity, FocusHandle, Focusable, Hsla, IntoElement, KeyDownEvent,
    PathPromptOptions, Role, SharedString, Subscription, Toggled, Window, div, px,
};

use zeron_theme::vscode::{ImportReport, SourceCompilation};
use zeron_theme::{InstallMode, ThemeVariant};
use zeron_ui::popover;
use zeron_ui::settings::widgets;
use zeron_ui::theme::{Theme, model_color};
use zeron_ui::typography;

use crate::app_settings::AppSettings;
use crate::i18n::tr;
use crate::settings::text_field::TextField;
use crate::settings::theme_library;

use super::super::AppearancePage;

/// Import + review dialog view state (`ImportDialog`, upstream :32-42; the
/// review entry id lives on `library::State`).
pub(crate) struct State {
    open: bool,
    input: Option<Entity<TextField>>,
    /// Repaints the page (and so the footer's Analyze/Import state) while the
    /// source field changes.
    _events: Option<Subscription>,
    focus: Option<FocusHandle>,
    focus_pending: bool,
    mode: InstallMode,
    compilation: Option<SourceCompilation>,
    /// The trimmed source string the compilation came from, so editing the
    /// field clears stale results (upstream listened to `ComposerInputEvent`).
    compiled_from: Option<String>,
    selected: HashSet<String>,
    review_variant: Option<String>,
    error: Option<SharedString>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            open: false,
            input: None,
            _events: None,
            focus: None,
            focus_pending: false,
            mode: InstallMode::Snapshot,
            compilation: None,
            compiled_from: None,
            selected: HashSet::new(),
            review_variant: None,
            error: None,
        }
    }
}

/// Render the import or review modal when open (upstream :2205-2746,
/// :3267-3319). The import dialog wins over the mapping review, matching the
/// modal stack order (`dialogs/mod.rs`).
pub(super) fn render(
    page: &mut AppearancePage,
    theme: &Theme,
    window: &mut Window,
    cx: &mut Context<AppearancePage>,
) -> Option<AnyElement> {
    if page.dialogs.library.open
        && let Some(import) = render_import_dialog(page, theme, window, cx)
    {
        return Some(import);
    }
    render_review_dialog(page, theme, window, cx)
}

/// zeron's modal mouse-up hook is unmapped for the library dialogs: both
impl State {
    /// Open a fresh import dialog (upstream `open_import`, :1022-1077). The
    /// text field entity needs `cx`, which the page constructor does not have,
    /// so it is created here. The `dialogs::library` module itself is private
    /// to the Lead-owned `dialogs` module, so the page opens the dialog
    /// through this method instead of a module path.
    pub(crate) fn open(&mut self, cx: &mut Context<AppearancePage>) {
        let lang = AppSettings::global(cx).language.clone();
        let placeholder = tr("appearance.library.import.source_placeholder", &lang);
        let input = cx.new(|cx| TextField::new("theme-import-source", placeholder, cx));
        let events = cx.observe(&input, |_page, _field, cx| cx.notify());
        *self = State {
            open: true,
            input: Some(input),
            _events: Some(events),
            focus: Some(cx.focus_handle()),
            focus_pending: true,
            ..State::default()
        };
    }
}

/// Close the import dialog (upstream clears `import_dialog`, :2631-2636).
fn close_import(page: &mut AppearancePage, cx: &mut Context<AppearancePage>) {
    page.dialogs.library = State::default();
    cx.notify();
}

fn render_import_dialog(
    page: &mut AppearancePage,
    theme: &Theme,
    window: &mut Window,
    cx: &mut Context<AppearancePage>,
) -> Option<AnyElement> {
    let input = page.dialogs.library.input.clone()?;
    let focus = page.dialogs.library.focus.clone()?;
    let lang = AppSettings::global(cx).language.clone();

    // Editing the source invalidates the previous compile (upstream's
    // `ComposerInputEvent::Edited` branch).
    let source = input.read(cx).value().trim().to_owned();
    if page.dialogs.library.compilation.is_some()
        && page.dialogs.library.compiled_from.as_deref() != Some(source.as_str())
    {
        page.dialogs.library.compilation = None;
        page.dialogs.library.selected.clear();
        page.dialogs.library.review_variant = None;
        page.dialogs.library.error = None;
    }

    if page.dialogs.library.focus_pending {
        let input_focus = input.read(cx).focus_handle(cx);
        window.focus(&input_focus, cx);
        page.dialogs.library.focus_pending = false;
    }

    let mode = page.dialogs.library.mode;
    let compilation = page.dialogs.library.compilation.clone();
    let selected = page.dialogs.library.selected.clone();
    let review_variant = page.dialogs.library.review_variant.clone();
    let error = page.dialogs.library.error.clone();
    let ready = compilation.is_some() && !selected.is_empty();

    let header = div()
        .px(px(20.0))
        .pt(px(18.0))
        .pb(px(16.0))
        .flex()
        .items_start()
        .gap(px(16.0))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .child(popover::dialog_title(
                    theme,
                    tr("appearance.library.import.title", &lang),
                ))
                .child(
                    popover::dialog_body(theme, tr("appearance.library.import.body", &lang))
                        .mt(px(4.0)),
                ),
        )
        .child(
            compact_action(
                theme,
                tr("appearance.library.import.close", &lang),
                "theme-import-close",
            )
            .w(px(28.0))
            .h(px(28.0))
            .px(px(0.0))
            .py(px(0.0))
            .justify_center()
            .aria_label(tr("appearance.library.import.close", &lang))
            .on_click(cx.listener(|page, _, _, cx| close_import(page, cx)))
            .child(
                zeron_ui::icons::icon(zeron_ui::icons::CLOSE)
                    .size(px(12.0))
                    .text_color(theme.text_muted),
            ),
        );

    let mut main = div()
        .id("theme-import-main")
        .max_h(px(520.0))
        .overflow_y_scroll()
        .px(px(20.0))
        .pb(px(18.0))
        .flex()
        .flex_col()
        .child(section_label(
            theme,
            tr("appearance.library.import.source", &lang),
        ))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(
                    popover::dialog_field(input.clone().into_any_element())
                        .flex_1()
                        .min_w_0()
                        .h(px(36.0))
                        .py(px(0.0))
                        .line_height(px(18.0))
                        .flex()
                        .items_center(),
                )
                .child(
                    compact_action(
                        theme,
                        tr("appearance.library.import.browse", &lang),
                        "theme-import-browse",
                    )
                    .h(px(36.0))
                    .px(px(12.0))
                    .flex_none()
                    .aria_label(tr("appearance.library.import.browse", &lang))
                    .on_click(cx.listener(|_page, _, _, cx| choose_import_source(cx))),
                ),
        )
        .child(
            div()
                .mt(px(16.0))
                .child(section_label(
                    theme,
                    tr("appearance.library.import.keep_updated", &lang),
                ))
                .child(
                    div()
                        .flex()
                        .gap(px(8.0))
                        .child(mode_control(
                            theme,
                            tr("appearance.library.import.copy", &lang),
                            tr("appearance.library.import.copy.desc", &lang),
                            InstallMode::Snapshot,
                            mode,
                            cx,
                        ))
                        .child(mode_control(
                            theme,
                            tr("appearance.library.import.link", &lang),
                            tr("appearance.library.import.link.desc", &lang),
                            InstallMode::Link,
                            mode,
                            cx,
                        )),
                ),
        );

    if let Some(ref compilation) = compilation {
        let variants = compilation.family.variants.len();
        main = main.child(
            div()
                .mt(px(18.0))
                .pt(px(16.0))
                .border_t_1()
                .border_color(theme.border.opacity(0.6))
                .flex()
                .items_baseline()
                .justify_between()
                .child(
                    section_label(theme, tr("appearance.library.import.detected", &lang))
                        .mb(px(0.0)),
                )
                .child(
                    div()
                        .text_size(typography::ui_rems(10.5))
                        .text_color(theme.text_muted)
                        .child(remote_variant_count(variants, &lang)),
                ),
        );

        for variant in &compilation.family.variants {
            let variant_id = variant.id.clone();
            let selected_now = selected.contains(&variant_id);
            let review_open = review_variant.as_deref() == Some(variant_id.as_str());
            let appearance = if variant.appearance.is_dark() {
                tr("appearance.dark", &lang)
            } else {
                tr("appearance.light", &lang)
            };
            let report = compilation.reports.get(&variant.id);

            let mut row = div()
                .id(SharedString::from(format!("theme-import-row-{variant_id}")))
                .mt(px(8.0))
                .p(px(11.0))
                .rounded(px(10.0))
                .border_1()
                .border_color(if selected_now {
                    theme.accent.opacity(0.7)
                } else {
                    theme.border
                })
                .bg(if selected_now {
                    theme.accent_wash.opacity(0.42)
                } else {
                    theme.surface_raised.opacity(0.22)
                })
                .flex()
                .flex_col()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(9.0))
                        .child(
                            div()
                                .id(SharedString::from(format!(
                                    "theme-import-select-{variant_id}"
                                )))
                                .size(px(18.0))
                                .rounded(px(5.0))
                                .border_1()
                                .border_color(if selected_now {
                                    theme.accent
                                } else {
                                    theme.border_strong
                                })
                                .bg(if selected_now { theme.accent } else { theme.bg })
                                .flex()
                                .items_center()
                                .justify_center()
                                .cursor_pointer()
                                .when(selected_now, |item| {
                                    item.child(
                                        zeron_ui::icons::icon(zeron_ui::icons::CHECK)
                                            .size(px(12.0))
                                            .text_color(theme.on_accent),
                                    )
                                })
                                .tab_index(0)
                                .role(Role::CheckBox)
                                .aria_label(variant.name.clone())
                                .aria_toggled(if selected_now {
                                    Toggled::True
                                } else {
                                    Toggled::False
                                })
                                .focus_visible(|style| {
                                    style.border_2().border_color(theme.accent).opacity(1.0)
                                })
                                .on_click(cx.listener({
                                    let variant_id = variant_id.clone();
                                    move |page, _, _, cx| {
                                        let state = &mut page.dialogs.library;
                                        if !state.selected.remove(&variant_id) {
                                            state.selected.insert(variant_id.clone());
                                        }
                                        cx.notify();
                                    }
                                })),
                        )
                        .child(variant_swatch(theme, variant))
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .child(
                                    div()
                                        .text_size(typography::ui_rems(12.5))
                                        .font_weight(gpui::FontWeight::MEDIUM)
                                        .text_color(theme.text)
                                        .child(SharedString::from(variant.name.clone())),
                                )
                                .child(
                                    div()
                                        .text_size(typography::ui_rems(11.0))
                                        .text_color(theme.text_muted)
                                        .child(appearance),
                                ),
                        )
                        .child(
                            compact_action(
                                theme,
                                if review_open {
                                    tr("appearance.library.import.hide_details", &lang)
                                } else {
                                    tr("appearance.library.import.details", &lang)
                                },
                                SharedString::from(format!("theme-import-review-{variant_id}")),
                            )
                            .aria_label(tr("appearance.library.import.details", &lang))
                            .on_click(cx.listener({
                                let variant_id = variant_id.clone();
                                move |page, _, _, cx| {
                                    let state = &mut page.dialogs.library;
                                    state.review_variant = if state.review_variant.as_deref()
                                        == Some(variant_id.as_str())
                                    {
                                        None
                                    } else {
                                        Some(variant_id.clone())
                                    };
                                    cx.notify();
                                }
                            })),
                        ),
                );
            if review_open {
                row = row
                    .child(
                        div()
                            .mt(px(10.0))
                            .pt(px(10.0))
                            .border_t_1()
                            .border_color(theme.border.opacity(0.6))
                            .child(scene_preview(theme, variant)),
                    )
                    .when_some(report, |row, report| {
                        row.child(report_panel(theme, report, &lang))
                    });
            }
            main = main.child(row);
        }

        for failure in &compilation.failures {
            main = main.child(
                div()
                    .mt(px(8.0))
                    .p(px(10.0))
                    .rounded(px(8.0))
                    .bg(theme.warning.opacity(0.08))
                    .text_size(typography::ui_rems(11.0))
                    .text_color(theme.warning)
                    .child(SharedString::from(format!(
                        "{} {} · {}",
                        failure.name,
                        tr("appearance.library.import.failed", &lang),
                        failure.message
                    ))),
            );
        }
    } else {
        main = main.child(
            div()
                .mt(px(14.0))
                .flex()
                .items_start()
                .gap(px(7.0))
                .text_size(typography::ui_rems(11.0))
                .line_height(px(16.0))
                .text_color(theme.text_muted)
                .child(
                    zeron_ui::icons::icon(zeron_ui::icons::INFO_CIRCLE)
                        .size(px(13.0))
                        .mt(px(1.0))
                        .flex_none(),
                )
                .child(tr("appearance.library.import.empty_hint", &lang)),
        );
    }

    if let Some(error) = error {
        main = main.child(widgets::error_strip(theme, error).mt(px(12.0)));
    }

    let footer = div()
        .border_t_1()
        .border_color(theme.border.opacity(0.6))
        .bg(theme.surface_raised.opacity(0.18))
        .px(px(20.0))
        .py(px(12.0))
        .flex()
        .items_center()
        .justify_end()
        .gap(px(8.0))
        .child(
            compact_action(
                theme,
                tr("appearance.library.import.cancel", &lang),
                "theme-import-cancel",
            )
            .h(px(34.0))
            .px(px(13.0))
            .aria_label(tr("appearance.library.import.cancel", &lang))
            .on_click(cx.listener(|page, _, _, cx| close_import(page, cx))),
        )
        .child(
            widgets::text_action(
                theme,
                widgets::ActionTone::Solid,
                if compilation.is_some() {
                    tr("appearance.library.import.import_selected", &lang)
                } else {
                    tr("appearance.library.import.analyze", &lang)
                },
            )
            .id("theme-import-action")
            .h(px(34.0))
            .px(px(14.0))
            .py(px(0.0))
            .flex()
            .items_center()
            .when(compilation.is_some() && !ready, |button| {
                button.opacity(0.45)
            })
            .when(compilation.is_none() || ready, |button| {
                button
                    .tab_index(0)
                    .role(Role::Button)
                    .aria_label(if compilation.is_some() {
                        tr("appearance.library.import.import_selected", &lang)
                    } else {
                        tr("appearance.library.import.analyze", &lang)
                    })
                    .focus_visible(|style| style.border_2().border_color(theme.accent).opacity(1.0))
                    .on_click(cx.listener(|page, _, window, cx| {
                        if page.dialogs.library.compilation.is_some() {
                            finish_import(page, window, cx);
                        } else {
                            compile_import(page, cx);
                        }
                    }))
            }),
        );

    let card = popover::dialog_card(theme)
        .id("theme-import-card")
        .w(px(600.0))
        .max_h(px(760.0))
        .p(px(0.0))
        .overflow_hidden()
        .track_focus(&focus)
        .on_key_down(cx.listener(|page, event: &KeyDownEvent, window, cx| {
            match popover::classify_key(
                event.keystroke.key.as_str(),
                event.keystroke.modifiers.platform,
                event.keystroke.modifiers.control,
            ) {
                popover::MenuKey::Escape => {
                    cx.stop_propagation();
                    close_import(page, cx);
                }
                popover::MenuKey::Enter | popover::MenuKey::ModEnter => {
                    cx.stop_propagation();
                    if page.dialogs.library.compilation.is_some() {
                        finish_import(page, window, cx);
                    } else {
                        compile_import(page, cx);
                    }
                }
                _ => {}
            }
        }))
        .on_mouse_down_out(cx.listener(|page, _, _, cx| close_import(page, cx)))
        .child(header)
        .child(widgets::scroll_faded("theme-import-main-fade", main))
        .child(footer);

    Some(popover::modal(
        "theme-import-dialog",
        window.viewport_size(),
        card.into_any_element(),
    ))
}

fn mode_control(
    theme: &Theme,
    label: &'static str,
    description: &'static str,
    value: InstallMode,
    mode: InstallMode,
    cx: &mut Context<AppearancePage>,
) -> AnyElement {
    let active = mode == value;
    let slug = if matches!(value, InstallMode::Snapshot) {
        "copy"
    } else {
        "link"
    };
    div()
        .id(SharedString::from(format!("theme-import-mode-{slug}")))
        .flex_1()
        .min_w_0()
        .p(px(10.0))
        .rounded(px(9.0))
        .border_1()
        .border_color(if active { theme.accent } else { theme.border })
        .bg(if active {
            theme.accent_wash
        } else {
            theme.surface_raised.opacity(0.28)
        })
        .cursor_pointer()
        .when(!active, |control| {
            control.hover(|style| style.bg(theme.surface_raised_hover))
        })
        .tab_index(0)
        .role(Role::RadioButton)
        .aria_label(label)
        .aria_toggled(if active {
            Toggled::True
        } else {
            Toggled::False
        })
        .focus_visible(|style| style.border_2().border_color(theme.accent).opacity(1.0))
        .on_click(cx.listener(move |page, _, _, cx| {
            page.dialogs.library.mode = value;
            cx.notify();
        }))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(7.0))
                .child(
                    div()
                        .size(px(16.0))
                        .rounded_full()
                        .border_1()
                        .border_color(if active {
                            theme.accent
                        } else {
                            theme.border_strong
                        })
                        .flex()
                        .items_center()
                        .justify_center()
                        .when(active, |dot| {
                            dot.child(div().size(px(8.0)).rounded_full().bg(theme.accent))
                        }),
                )
                .child(
                    div()
                        .text_size(typography::ui_rems(12.0))
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(if active { theme.text } else { theme.text_muted })
                        .child(label),
                ),
        )
        .child(
            div()
                .mt(px(4.0))
                .ml(px(23.0))
                .text_size(typography::ui_rems(10.5))
                .text_color(theme.text_muted)
                .child(description),
        )
        .into_any_element()
}

fn section_label(theme: &Theme, label: &'static str) -> gpui::Div {
    div()
        .mb(px(7.0))
        .text_size(typography::ui_rems(11.0))
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .text_color(theme.text_muted)
        .child(label)
}

fn remote_variant_count(variants: usize, lang: &str) -> String {
    format!("{variants} {}", variant_word(variants, lang))
}

fn variant_word(variants: usize, lang: &str) -> &'static str {
    tr(
        if variants == 1 {
            "appearance.library.variant"
        } else {
            "appearance.library.variants"
        },
        lang,
    )
}

/// Browse for a theme file/package and compile it (upstream
/// `choose_import_source`, :1109-1140).
fn choose_import_source(cx: &mut Context<AppearancePage>) {
    let lang = AppSettings::global(cx).language.clone();
    let receiver = cx.prompt_for_paths(PathPromptOptions {
        files: true,
        directories: true,
        multiple: false,
        prompt: Some(tr("appearance.library.import.browse_prompt", &lang).into()),
    });
    cx.spawn(async move |page, cx| {
        let Ok(Ok(Some(mut paths))) = receiver.await else {
            return;
        };
        let Some(path) = paths.pop() else {
            return;
        };
        let _ = page.update(cx, |page, cx| {
            if let Some(input) = page.dialogs.library.input.clone() {
                input.update(cx, |input, cx| {
                    input.set_value(path.display().to_string(), cx);
                });
            }
            compile_import(page, cx);
        });
    })
    .detach();
}

fn lang_of(cx: &gpui::App) -> SharedString {
    AppSettings::global(cx).language.clone()
}

/// Compile the source field (upstream `compile_import`, :1078-1107).
fn compile_import(page: &mut AppearancePage, cx: &mut Context<AppearancePage>) {
    let Some(input) = page.dialogs.library.input.clone() else {
        return;
    };
    let source = input.read(cx).value().trim().to_owned();
    if source.is_empty() {
        page.dialogs.library.error =
            Some(tr("appearance.library.import.error.source", &lang_of(cx)).into());
        cx.notify();
        return;
    }
    let path = PathBuf::from(&source);
    let family_name = source_name(&path);
    let family_id = format!("custom-{}", slug(&family_name));
    match theme_library::compile(&path, &family_id, &family_name) {
        Ok(compilation) => {
            page.dialogs.library.selected = compilation
                .family
                .variants
                .iter()
                .map(|variant| variant.id.clone())
                .collect();
            page.dialogs.library.review_variant = None;
            page.dialogs.library.compilation = Some(compilation);
            page.dialogs.library.compiled_from = Some(source);
            page.dialogs.library.error = None;
        }
        Err(error) => page.dialogs.library.error = Some(error.to_string().into()),
    }
    cx.notify();
}

/// Install the selected variants and close the dialog (upstream
/// `finish_import`, :1370-1391).
fn finish_import(page: &mut AppearancePage, window: &mut Window, cx: &mut Context<AppearancePage>) {
    let Some(compilation) = page.dialogs.library.compilation.clone() else {
        return;
    };
    if page.dialogs.library.selected.is_empty() {
        page.dialogs.library.error =
            Some(tr("appearance.library.import.error.selection", &lang_of(cx)).into());
        cx.notify();
        return;
    }
    let selected = page
        .dialogs
        .library
        .selected
        .iter()
        .cloned()
        .collect::<Vec<_>>();
    let mode = page.dialogs.library.mode;
    match theme_library::install(compilation.clone(), &selected, mode, cx) {
        Ok(_) => {
            page.dialogs.library = State::default();
            super::super::library::after_mutation(page, window, cx);
        }
        Err(error) => {
            page.dialogs.library.compilation = Some(compilation);
            let lang = lang_of(cx);
            page.dialogs.library.error = Some(SharedString::from(bridge_error_text(
                &error.to_string(),
                &lang,
            )));
            cx.notify();
        }
    }
}

fn render_review_dialog(
    page: &mut AppearancePage,
    theme: &Theme,
    window: &mut Window,
    cx: &mut Context<AppearancePage>,
) -> Option<AnyElement> {
    let entry_id = page.library.review_entry.clone()?;
    let entry = theme_library::entries(cx)
        .into_iter()
        .find(|entry| entry.id == entry_id)?;
    let lang = AppSettings::global(cx).language.clone();
    let focus = cx.focus_handle();

    let mut card = popover::dialog_card(theme)
        .id("theme-review-card")
        .w(px(660.0))
        .max_h(px(720.0))
        .overflow_y_scroll()
        .track_focus(&focus)
        .on_key_down(cx.listener(|page, event: &KeyDownEvent, _, cx| {
            if matches!(
                popover::classify_key(
                    event.keystroke.key.as_str(),
                    event.keystroke.modifiers.platform,
                    event.keystroke.modifiers.control,
                ),
                popover::MenuKey::Escape
            ) {
                cx.stop_propagation();
                page.library.review_entry = None;
                cx.notify();
            }
        }))
        .on_mouse_down_out(cx.listener(|page, _, _, cx| {
            page.library.review_entry = None;
            cx.notify();
        }))
        .child(popover::dialog_title(
            theme,
            tr("appearance.library.review.title", &lang),
        ))
        .child(
            popover::dialog_body(
                theme,
                format!("{} · {}", entry.name, source_label(&entry.source, &lang)),
            )
            .mt(px(6.0)),
        );

    for variant in &entry.family.variants {
        card = card
            .child(
                div()
                    .mt(px(14.0))
                    .text_size(typography::ui_rems(12.5))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .child(SharedString::from(variant.name.clone())),
            )
            .child(scene_preview(theme, variant));
        if let Some(report) = entry.reports.get(&variant.id) {
            card = card.child(report_panel(theme, report, &lang));
        }
    }

    card = card.child(
        div().mt(px(16.0)).flex().justify_end().child(
            widgets::text_action(
                theme,
                widgets::ActionTone::Solid,
                tr("appearance.library.review.done", &lang),
            )
            .id("theme-review-close")
            .tab_index(0)
            .role(Role::Button)
            .aria_label(tr("appearance.library.review.done", &lang))
            .focus_visible(|style| style.border_2().border_color(theme.accent).opacity(1.0))
            .on_click(cx.listener(|page, _, _, cx| {
                page.library.review_entry = None;
                cx.notify();
            })),
        ),
    );

    Some(popover::modal(
        "theme-review-dialog",
        window.viewport_size(),
        widgets::scroll_faded("theme-review-fade", card).into_any_element(),
    ))
}

/// zeron's `CustomThemeSource::label()` words, localized (upstream :3281).
fn source_label(source: &zeron_theme::CustomThemeSource, lang: &str) -> &'static str {
    match source {
        zeron_theme::CustomThemeSource::ImportedSnapshot { .. } => {
            tr("appearance.library.source.imported", lang)
        }
        zeron_theme::CustomThemeSource::LinkedFile { .. } => {
            tr("appearance.library.source.linked_file", lang)
        }
        zeron_theme::CustomThemeSource::LinkedPackage { .. } => {
            tr("appearance.library.source.linked_package", lang)
        }
        zeron_theme::CustomThemeSource::EditableFile { .. } => {
            tr("appearance.library.source.editable_file", lang)
        }
    }
}

/// The palette strip an import row leads with: zeron's three-band
/// `palette_preview` (:1692-1705) painted from the *compiled* variant's model
/// colors (upstream resolved a `Theme::from_variant` sample, :2399-2403).
fn variant_swatch(theme: &Theme, variant: &ThemeVariant) -> gpui::Div {
    div()
        .flex_none()
        .w(px(30.0))
        .h(px(18.0))
        .rounded(px(5.0))
        .overflow_hidden()
        .border_1()
        .border_color(theme.border)
        .flex()
        .child(div().w_1_3().h_full().bg(model_color(variant.colors.shell)))
        .child(
            div()
                .w_1_3()
                .h_full()
                .bg(model_color(variant.colors.background)),
        )
        .child(
            div()
                .w_1_3()
                .h_full()
                .bg(model_color(variant.accent.primary)),
        )
}

/// The semantic colors one compiled variant paints its scene preview with.
/// zeron builds a whole `Theme` from the variant for this; the vendored
/// `Theme::from_variant` is crate-private, so the preview reads the same
/// model roles directly.
#[derive(Clone, Copy)]
struct VariantPalette {
    background: Hsla,
    shell: Hsla,
    text: Hsla,
    border: Hsla,
    accent_wash: Hsla,
    keyword: Hsla,
    string: Hsla,
    function: Hsla,
    punctuation: Hsla,
    diff_add: Hsla,
    diff_delete: Hsla,
    ansi: [Hsla; 8],
}

fn variant_palette(variant: &ThemeVariant) -> VariantPalette {
    let syntax = |key: &str, fallback: zeron_theme::Color| {
        model_color(variant.syntax.get(key).copied().unwrap_or(fallback))
    };
    let mut ansi = [model_color(variant.terminal.ansi[0]); 8];
    for (slot, color) in ansi.iter_mut().zip(variant.terminal.ansi.iter().take(8)) {
        *slot = model_color(*color);
    }
    VariantPalette {
        background: model_color(variant.colors.background),
        shell: model_color(variant.colors.shell),
        text: model_color(variant.colors.text),
        border: model_color(variant.colors.border),
        accent_wash: model_color(variant.accent.wash),
        keyword: syntax("keyword", variant.accent.primary),
        string: syntax("string", variant.colors.text),
        function: syntax("function", variant.accent.primary),
        punctuation: syntax("punctuation", variant.colors.text_muted),
        diff_add: model_color(variant.colors.diff_add),
        diff_delete: model_color(variant.colors.diff_delete),
        ansi,
    }
}

/// zeron's `import_scene_preview` (:1715-1812): a window mock, a code pane with
/// the variant's syntax colors, and a diff/ANSI strip.
fn scene_preview(theme: &Theme, variant: &ThemeVariant) -> AnyElement {
    let palette = variant_palette(variant);
    div()
        .w_full()
        .h(px(86.0))
        .flex()
        .gap(px(8.0))
        .child(
            div()
                .w(px(152.0))
                .h_full()
                .overflow_hidden()
                .rounded(px(8.0))
                .border_1()
                .border_color(palette.border)
                .child(scene_window_mock(&palette)),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .h_full()
                .rounded(px(8.0))
                .border_1()
                .border_color(palette.border)
                .bg(palette.background)
                .p(px(9.0))
                .flex()
                .flex_col()
                .gap(px(6.0))
                .child(
                    div()
                        .text_size(typography::ui_rems(10.0))
                        .font_family(theme.font_mono.clone())
                        .child(
                            div()
                                .text_color(palette.keyword)
                                .child("fn ")
                                .child(div().text_color(palette.function).child("preview"))
                                .child(div().text_color(palette.punctuation).child("() {")),
                        ),
                )
                .child(
                    div()
                        .text_size(typography::ui_rems(10.0))
                        .font_family(theme.font_mono.clone())
                        .text_color(palette.string)
                        .child("  \"Theme mapping\""),
                )
                .child(
                    div()
                        .mt_auto()
                        .h(px(12.0))
                        .flex()
                        .rounded(px(3.0))
                        .overflow_hidden()
                        .children(
                            palette
                                .ansi
                                .iter()
                                .map(|color| div().flex_1().h_full().bg(*color)),
                        ),
                ),
        )
        .child(
            div()
                .w(px(84.0))
                .h_full()
                .rounded(px(8.0))
                .border_1()
                .border_color(palette.border)
                .bg(palette.shell)
                .p(px(8.0))
                .flex()
                .flex_col()
                .gap(px(6.0))
                .child(
                    div()
                        .h(px(12.0))
                        .rounded(px(3.0))
                        .bg(palette.diff_add.opacity(0.35)),
                )
                .child(
                    div()
                        .h(px(12.0))
                        .rounded(px(3.0))
                        .bg(palette.diff_delete.opacity(0.35)),
                )
                .child(div().h(px(12.0)).rounded(px(3.0)).bg(palette.accent_wash)),
        )
        .into_any_element()
}

/// zeron's miniature window mock inside the scene preview (:1722-1730 uses the
/// page's `miniature`; adapted here so the preview paints the compiled
/// variant's own colors without installing it).
fn scene_window_mock(palette: &VariantPalette) -> gpui::Div {
    let line = palette.text.opacity(0.22);
    let strong = palette.text.opacity(0.34);
    let bar = |fraction: f32, tone: Hsla| {
        div()
            .h(px(5.0))
            .w(gpui::relative(fraction))
            .rounded(px(3.0))
            .bg(tone)
    };
    div()
        .size_full()
        .flex()
        .flex_row()
        .bg(palette.shell)
        .child(
            div()
                .w(px(44.0))
                .h_full()
                .flex_none()
                .flex()
                .flex_col()
                .gap(px(7.0))
                .px(px(8.0))
                .pt(px(14.0))
                .child(bar(0.70, strong))
                .child(bar(1.0, line))
                .child(bar(0.85, line))
                .child(bar(1.0, line)),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .my(px(8.0))
                .mr(px(8.0))
                .rounded(px(6.0))
                .border_1()
                .border_color(palette.border)
                .bg(palette.background)
                .p(px(10.0))
                .flex()
                .flex_col()
                .gap(px(7.0))
                .child(bar(0.62, strong))
                .child(bar(0.88, line))
                .child(bar(0.76, line))
                .child(bar(0.52, line)),
        )
}

/// zeron's import report panel (:1814-1881): summary line plus every
/// adjustment/fallback/warning/validation/unsupported/mapping entry in a
/// faded scroll area.
fn report_panel(theme: &Theme, report: &ImportReport, lang: &str) -> AnyElement {
    let summary = format!(
        "{} {} · {} {} · {} {} · {} {} · {} {} · {} {}",
        report.mappings.len(),
        tr("appearance.library.report.mapped", lang),
        report.adjustments.len(),
        tr("appearance.library.report.adjusted", lang),
        report.fallbacks.len(),
        tr("appearance.library.report.inferred", lang),
        report.dropped.len(),
        tr("appearance.library.report.unsupported", lang),
        report.warnings.len(),
        tr("appearance.library.report.warnings", lang),
        report.validation.len(),
        tr("appearance.library.report.validation", lang),
    );
    let key = format!("theme-report-{}", report.source_hash);
    let area = div()
        .id(SharedString::from(key.clone()))
        .mt(px(8.0))
        .w_full()
        .max_h(px(168.0))
        .overflow_y_scroll()
        .rounded(px(8.0))
        .border_1()
        .border_color(theme.border)
        .bg(theme.surface_raised.opacity(0.35))
        .p(px(10.0))
        .text_size(typography::ui_rems(11.0))
        .line_height(px(16.0))
        .text_color(theme.text_muted)
        .child(
            div()
                .text_color(theme.text)
                .child(SharedString::from(summary)),
        )
        .children(report.adjustments.iter().map(|adjustment| {
            div().mt(px(4.0)).child(SharedString::from(format!(
                "{} · {} {} → {} · {}",
                tr("appearance.library.report.adjusted_prefix", lang),
                adjustment.zeron_role,
                adjustment.original,
                adjustment.resolved,
                adjustment.reason
            )))
        }))
        .children(report.fallbacks.iter().map(|message| {
            div().mt(px(4.0)).child(SharedString::from(format!(
                "{} · {message}",
                tr("appearance.library.report.fallback_prefix", lang)
            )))
        }))
        .children(report.warnings.iter().map(|message| {
            div().mt(px(4.0)).child(SharedString::from(format!(
                "{} · {message}",
                tr("appearance.library.report.warning_prefix", lang)
            )))
        }))
        .children(report.validation.iter().map(|issue| {
            div().mt(px(4.0)).child(SharedString::from(format!(
                "{} · {:?} {:?} · {}",
                tr("appearance.library.report.validation_prefix", lang),
                issue.category,
                issue.severity,
                issue.message
            )))
        }))
        .children(report.dropped.iter().map(|message| {
            div().mt(px(4.0)).child(SharedString::from(format!(
                "{} · {message}",
                tr("appearance.library.report.unsupported_prefix", lang)
            )))
        }))
        .children(report.mappings.iter().map(|mapping| {
            div().mt(px(4.0)).child(SharedString::from(format!(
                "{} ← {}",
                mapping.zeron_role, mapping.vscode_key
            )))
        }));
    widgets::scroll_faded(SharedString::from(key), area).into_any_element()
}

/// A compact filled action with the shared focus ring (upstream
/// `compact_action`, :1707-1713 plus the per-control a11y chain).
fn compact_action(
    theme: &Theme,
    label: &'static str,
    id: impl Into<SharedString>,
) -> gpui::Stateful<gpui::Div> {
    widgets::text_action(theme, widgets::ActionTone::Filled, label)
        .id(id.into())
        .tab_index(0)
        .role(Role::Button)
        .focus_visible(move |style| style.border_2().border_color(theme.accent).opacity(1.0))
}

/// The family display name for a source path: a `package.json` names its
/// directory, anything else its file stem (upstream :1411-1421).
fn source_name(path: &Path) -> String {
    let path = if path.file_name().and_then(|name| name.to_str()) == Some("package.json") {
        path.parent().unwrap_or(path)
    } else {
        path
    };
    path.file_stem()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or("Custom theme")
        .to_owned()
}

/// Stable id slug for an imported family (upstream :1424-1446).
fn slug(value: &str) -> String {
    let mut result = String::new();
    let mut separator = false;
    for character in value.chars().flat_map(char::to_lowercase) {
        if character.is_ascii_alphanumeric() {
            if separator && !result.is_empty() {
                result.push('-');
            }
            result.push(character);
            separator = false;
        } else {
            separator = true;
        }
    }
    if result.is_empty() {
        "theme".into()
    } else {
        result
    }
}

/// Localize the bridge's known save/initialization words on the import path;
/// importer compile diagnostics stay raw (developer-facing, recorded decision).
fn bridge_error_text(error: &str, lang: &str) -> String {
    let key = if error.contains("not initialized") {
        Some("appearance.library.error.not_initialized")
    } else if error.contains("save") || error.contains("write") || error.contains("persist") {
        Some("appearance.library.error.save")
    } else {
        None
    };
    key.map_or_else(|| error.to_owned(), |key| tr(key, lang).to_owned())
}

#[cfg(test)]
mod tests {
    use super::{slug, source_name};
    use std::path::Path;

    #[test]
    fn package_manifests_name_their_directory() {
        assert_eq!(
            source_name(Path::new("/themes/Night Owl/package.json")),
            "Night Owl"
        );
        assert_eq!(source_name(Path::new("/themes/paper.json")), "paper");
        assert_eq!(source_name(Path::new("/")), "Custom theme");
    }

    #[test]
    fn slugs_become_stable_family_ids() {
        assert_eq!(slug("Night Owl"), "night-owl");
        assert_eq!(slug("  My Theme!!  "), "my-theme");
        assert_eq!(slug("日本語"), "theme");
    }

    /// The dialog's compile → selected-variants → install → persist path,
    /// headless: exactly what Analyze theme + Import selected do once the
    /// platform-free parts are in place (the modal itself needs a window).
    #[test]
    fn import_pipeline_compiles_selects_and_persists_a_minimal_theme() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("Night Owl.json");
        std::fs::write(
            &path,
            r##"{"name":"Night Owl","type":"dark","colors":{"editor.background":"#101010","foreground":"#eeeeee"}}"##,
        )
        .unwrap();

        let family_name = source_name(&path);
        let family_id = format!("custom-{}", slug(&family_name));
        let compilation =
            crate::settings::theme_library::compile(&path, &family_id, &family_name).unwrap();
        assert_eq!(family_id, "custom-night-owl");
        assert_eq!(compilation.family.name, "Night Owl");

        // The dialog starts with every compiled variant selected.
        let selected = compilation
            .family
            .variants
            .iter()
            .map(|variant| variant.id.clone())
            .collect::<Vec<_>>();
        assert_eq!(selected, vec!["custom-night-owl".to_string()]);

        let mut library = zeron_theme::CustomThemeLibrary::default();
        let entry_id = library
            .install(compilation, &selected, zeron_theme::InstallMode::Snapshot)
            .unwrap();
        assert_eq!(entry_id, "custom-night-owl");
        library.save(dir.path()).unwrap();

        let persisted = zeron_theme::CustomThemeLibrary::load(dir.path()).unwrap();
        let entry = persisted.entry(&entry_id).expect("the import persisted");
        assert_eq!(entry.family.variants[0].id, entry_id);
        assert_eq!(
            entry.family.variants[0].colors.background,
            "#101010".parse::<zeron_theme::Color>().unwrap()
        );
    }
}
