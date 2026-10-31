//! Material and background section: the Glass surface select, the new-thread
//! background image row (with its actions), the conditional Background effect
//! select and the Wallpaper folder row, in zeron's slot between the color card
//! and Motion.
//!
//! Ported from zeron's `crates/ui/src/settings/appearance.rs` (MIT, Copyright
//! 2026 Wing, pinned rev 8b7afb3): the Glass row + `surface_helper` :1568-1580
//! and the row control :3626-3660, the new-thread background row :3730-3846,
//! the wallpaper folder row :3848-3898, and the conditional background-effect
//! row :3900-3957.
//!
//! Wiring differs from upstream on purpose: zeron's rows call
//! `crate::settings::*` handlers and its `appearance::set_*` service; mono-desk
//! writes the `AppSettings` global and funnels appearance changes through
//! `settings::apply_appearance`, and the background/wallpaper domain helpers
//! live in `settings::background` / `settings::wallpaper` (state-persistence
//! T4). File dialogs use `cx.prompt_for_paths`, like the other settings pages.
//!
//! Adaptations/trims from upstream (recorded):
//! - Glass maps onto mono-desk's two persisted booleans
//!   (`background_translucent` + `background_blur`) instead of zeron's
//!   `SurfacePreference`, whose third `ThemeDefault` value the pre-port
//!   settings file cannot express (NG4: keys are frozen). The row offers the
//!   two explicit materials (Frosted/Opaque); the helper line keeps zeron's
//!   `surface_helper` copy through i18n.
//! - The row's error renders inline with `widgets::error_strip` instead of
//!   zeron's toast queue (mono-desk vendors no toast module); the message is
//!   the `settings::background` error string.
//! - Shuffle picks a wallpaper and adopts it as the new-thread background —
//!   zeron's `random_wallpaper` does the same through its preload queue —
//!   because mono-desk paints no chat hero (plan NG4), so there is nothing
//!   else a wallpaper could feed.
//! - The wallpaper shortcut label is a local constant of zeron's default
//!   binding (`mod-u` → `⌘U`); mono-desk persists no keymap.
//! - No `search_anchor` markers: mono-desk's settings search is the sidebar
//!   section filter, not zeron's per-setting search index.
//! - The background adjustment dialog (phase 4 T4) is opened through
//!   [`super::dialogs::background`], filled in the same task.

use gpui::prelude::*;
use gpui::{
    AnyElement, Context, IntoElement, ObjectFit, PathPromptOptions, Role, SharedString, Window,
    div, img, px,
};
use zeron_theme::SurfacePreference;
use zeron_ui::settings::select::{SelectOption, SelectState, select};
use zeron_ui::settings::widgets;
use zeron_ui::theme::Theme;

use crate::app_settings::{AppSettings, NewThreadBackgroundEffect};
use crate::i18n::tr;

use super::AppearancePage;

/// The two materials the persisted glass flags can express (see module doc).
const SURFACE_OPTIONS: [SurfacePreference; 2] =
    [SurfacePreference::Frosted, SurfacePreference::Opaque];

/// zeron's default random-wallpaper binding, `mod-u`, rendered for macOS
/// (`KeymapConfig::random_wallpaper` default; no keymap is persisted here).
const WALLPAPER_SHORTCUT: &str = "⌘U";

/// View state the Material rows own (upstream `surface_select` :324 plus the
/// background-effect select; the background action buttons add no state).
#[derive(Default)]
pub(super) struct State {
    pub(super) surface: SelectState,
    pub(super) effect: SelectState,
    /// The last background/wallpaper action's user-facing error, shown inline
    /// beneath the card (upstream kept it in `background_error` for its toast).
    error: Option<SharedString>,
}

impl State {
    fn set_error(&mut self, error: Option<String>) {
        self.error = error.map(SharedString::from);
    }
}

/// The "Material and background" section, in zeron's column slot between the
/// color card and Motion (upstream :4257-4261).
pub(super) fn section(
    page: &mut AppearancePage,
    theme: &Theme,
    lang: &str,
    _window: &mut Window,
    cx: &mut Context<AppearancePage>,
) -> AnyElement {
    let (translucent, background, effect, folder) = {
        let settings = AppSettings::global(cx);
        (
            settings.background_translucent,
            settings.new_thread_background.clone(),
            settings.new_thread_background_effect,
            settings.wallpaper_folder.clone(),
        )
    };
    let background_available = background
        .as_ref()
        .is_some_and(|background| background.path.is_file());

    // ---- Glass ----------------------------------------------------------
    let current_surface = if translucent {
        SurfacePreference::Frosted
    } else {
        SurfacePreference::Opaque
    };
    let surface_selected = SURFACE_OPTIONS
        .iter()
        .position(|surface| *surface == current_surface)
        .unwrap_or(1);
    let surface_control = select(
        "appearance-surface",
        tr("appearance.glass", lang),
        theme,
        |page: &mut AppearancePage| &mut page.material.surface,
    )
    .options(
        SURFACE_OPTIONS
            .into_iter()
            .map(|surface| SelectOption::new(tr(surface_option_key(surface), lang))),
        surface_selected,
    )
    .width(148.0)
    .on_select(|page, ix, window, cx| page.set_surface_row(ix, window, cx))
    .render(&page.material.surface, cx);
    let glass_row = widgets::card_row(theme, true)
        .child(
            div()
                .flex_1()
                .min_w(px(160.0))
                .child(widgets::row_title(theme, tr("appearance.glass", lang)))
                .child(widgets::meta_line(
                    theme,
                    vec![
                        div()
                            .child(surface_helper(current_surface, lang))
                            .into_any_element(),
                    ],
                )),
        )
        .child(surface_control)
        .into_any_element();

    // ---- New thread background -----------------------------------------
    let background_tile: Option<AnyElement> = background
        .as_ref()
        .filter(|_| background_available)
        .map(|background| {
            div()
                .flex_none()
                .size(px(36.0))
                .rounded(px(10.0))
                .overflow_hidden()
                .border_1()
                .border_color(zeron_ui::theme::hairline(0.10))
                .child(
                    img(background.path.clone())
                        .size(px(34.0))
                        .rounded(px(9.0))
                        .object_fit(ObjectFit::Cover),
                )
                .into_any_element()
        });
    let background_meta: Vec<AnyElement> = match background.as_ref() {
        Some(background) if background_available => {
            vec![div().child(background.name.clone()).into_any_element()]
        }
        Some(_) => vec![
            div()
                .child(tr("appearance.background.unavailable", lang))
                .into_any_element(),
            div()
                .child(tr("appearance.background.unavailable.desc", lang))
                .into_any_element(),
        ],
        None => vec![
            div()
                .child(tr("appearance.background.no_image", lang))
                .into_any_element(),
        ],
    };
    let mut background_actions = div()
        .max_w_full()
        .flex()
        .flex_wrap()
        .items_center()
        .gap(px(8.0));
    if background.is_some() {
        if background_available {
            background_actions = background_actions.child(
                action(
                    theme,
                    tr("appearance.background.adjust", lang),
                    "new-thread-background-adjust",
                )
                .on_click(cx.listener(|page, _, window, cx| {
                    page.open_background_adjustment(window, cx);
                })),
            );
        }
        background_actions = background_actions
            .child(
                action(
                    theme,
                    tr("appearance.background.replace", lang),
                    "new-thread-background-replace",
                )
                .on_click(cx.listener(|page, _, window, cx| page.choose_background(window, cx))),
            )
            .child(
                action(
                    theme,
                    tr("appearance.background.remove", lang),
                    "new-thread-background-remove",
                )
                .text_color(theme.danger)
                .on_click(cx.listener(|page, _, window, cx| page.remove_background(window, cx))),
            );
    } else {
        background_actions = background_actions.child(
            action(
                theme,
                tr("appearance.background.choose", lang),
                "new-thread-background-choose",
            )
            .on_click(cx.listener(|page, _, window, cx| page.choose_background(window, cx))),
        );
    }
    let background_row = widgets::card_row(theme, false)
        .children(background_tile)
        .child(
            div()
                .flex_1()
                .min_w(px(160.0))
                .child(widgets::row_title(theme, tr("appearance.background", lang)))
                .child(widgets::meta_line(theme, background_meta)),
        )
        .child(background_actions)
        .into_any_element();

    // ---- Wallpaper folder ----------------------------------------------
    let folder_label = folder
        .as_ref()
        .map(|folder| folder.display().to_string())
        .unwrap_or_else(|| tr("appearance.wallpaper_folder.empty", lang).to_string());
    let folder_meta =
        tr("appearance.wallpaper_folder.meta", lang).replace("{shortcut}", WALLPAPER_SHORTCUT);
    let mut folder_actions = div().flex().gap(px(8.0)).child(
        action(
            theme,
            tr("appearance.wallpaper_folder.choose", lang),
            "wallpaper-folder-choose",
        )
        .on_click(cx.listener(|page, _, window, cx| page.choose_wallpaper_folder(window, cx))),
    );
    if folder.is_some() {
        folder_actions = folder_actions.child(
            action(
                theme,
                tr("appearance.wallpaper_folder.shuffle", lang),
                "wallpaper-shuffle",
            )
            .on_click(cx.listener(|page, _, window, cx| page.shuffle_wallpaper(window, cx))),
        );
    }
    let wallpaper_row = widgets::card_row(theme, false)
        .child(
            div()
                .flex_1()
                .min_w(px(160.0))
                .child(widgets::row_title(
                    theme,
                    tr("appearance.wallpaper_folder", lang),
                ))
                .child(widgets::meta_line(
                    theme,
                    vec![
                        div()
                            .child(format!("{folder_label} · {folder_meta}"))
                            .into_any_element(),
                    ],
                )),
        )
        .child(folder_actions)
        .into_any_element();

    let mut rows: Vec<AnyElement> = vec![glass_row, background_row, wallpaper_row];

    // ---- Conditional Background effect ---------------------------------
    if background_available {
        let effect_selected = NewThreadBackgroundEffect::ALL
            .iter()
            .position(|candidate| *candidate == effect)
            .unwrap_or_default();
        let effect_control = select(
            "new-thread-background-effect",
            tr("appearance.background.effect", lang),
            theme,
            |page: &mut AppearancePage| &mut page.material.effect,
        )
        .options(
            NewThreadBackgroundEffect::ALL
                .into_iter()
                .map(|effect| SelectOption::new(tr(effect_label_key(effect), lang))),
            effect_selected,
        )
        .width(128.0)
        .on_select(|page, ix, _, cx| page.set_effect_row(ix, cx))
        .render(&page.material.effect, cx);
        rows.push(
            widgets::card_row(theme, false)
                .child(
                    div()
                        .flex_1()
                        .min_w(px(160.0))
                        .child(widgets::row_title(
                            theme,
                            tr("appearance.background.effect", lang),
                        ))
                        .child(widgets::meta_line(
                            theme,
                            vec![
                                div()
                                    .child(tr(effect_description_key(effect), lang))
                                    .into_any_element(),
                            ],
                        )),
                )
                .child(effect_control)
                .into_any_element(),
        );
    }

    // ---- Inline action error (upstream's toast slot) -------------------
    let mut block = widgets::section_card(theme).mt_0().children(rows);
    if let Some(error) = page.material.error.clone() {
        block = block.child(
            div()
                .px(px(16.0))
                .pb(px(12.0))
                .child(widgets::error_strip(theme, action_error_text(&error, lang))),
        );
    }
    widgets::section(theme, tr("appearance.material", lang), block).into_any_element()
}

/// Localize a settings-domain action error. `settings::background` and
/// `settings::wallpaper` keep zeron's English `Result<_, String>` words for
/// their domain checks; the page maps those known words through i18n and falls
/// back to the raw message for unexpected I/O diagnostics (Log decision: those
/// are developer-facing edge cases, not page copy).
fn action_error_text(error: &str, lang: &str) -> String {
    use crate::settings::{background, wallpaper};
    let key = match error {
        background::ERROR_UNSUPPORTED => Some("appearance.background.error.unsupported"),
        background::ERROR_SAVE => Some("appearance.background.error.save_permissions"),
        background::ERROR_NO_DATA_DIR => Some("appearance.background.error.save_restart"),
        background::ERROR_REMOVE_NO_DATA_DIR => Some("appearance.background.error.remove_restart"),
        background::ERROR_REMOVE_SAVE => Some("appearance.background.error.remove_permissions"),
        wallpaper::CHOOSE_FOLDER_ERROR => Some("appearance.wallpaper_folder.error.choose"),
        wallpaper::READ_FOLDER_ERROR => Some("appearance.wallpaper_folder.error.unreadable"),
        wallpaper::NO_WALLPAPERS_ERROR => Some("appearance.wallpaper_folder.error.empty"),
        _ => None,
    };
    key.map_or_else(|| error.to_owned(), |key| tr(key, lang).to_owned())
}

/// zeron's `surface_helper` (`crates/ui/src/settings/appearance.rs:1568-1580`)
/// for the two materials this app's persisted booleans can express
/// (`ThemeDefault` is not reachable from the Glass row here).
fn surface_helper(surface: SurfacePreference, lang: &str) -> &'static str {
    match surface {
        SurfacePreference::Frosted => tr("appearance.surface.frosted", lang),
        SurfacePreference::Opaque => tr("appearance.surface.opaque", lang),
        SurfacePreference::ThemeDefault => tr("appearance.surface.helper.default_frosted", lang),
    }
}

/// zeron's `compact_action`: a filled text action with a stable debug id.
fn action(
    theme: &Theme,
    label: impl Into<SharedString>,
    id: &'static str,
) -> gpui::Stateful<gpui::Div> {
    widgets::text_action(theme, widgets::ActionTone::Filled, label)
        .id(id)
        .tab_index(0)
        .role(Role::Button)
        .focus_visible(|style| style.border_2().border_color(theme.accent).opacity(1.0))
}

fn surface_option_key(surface: SurfacePreference) -> &'static str {
    match surface {
        SurfacePreference::Frosted => "appearance.surface.option.frosted",
        SurfacePreference::Opaque => "appearance.surface.option.opaque",
        SurfacePreference::ThemeDefault => "appearance.surface.default",
    }
}

fn effect_label_key(effect: NewThreadBackgroundEffect) -> &'static str {
    match effect {
        NewThreadBackgroundEffect::None => "appearance.background.effect.none",
        NewThreadBackgroundEffect::Dither => "appearance.background.effect.dither",
        NewThreadBackgroundEffect::Ascii => "appearance.background.effect.ascii",
        NewThreadBackgroundEffect::Halftone => "appearance.background.effect.halftone",
        NewThreadBackgroundEffect::Scanlines => "appearance.background.effect.scanlines",
    }
}

fn effect_description_key(effect: NewThreadBackgroundEffect) -> &'static str {
    match effect {
        NewThreadBackgroundEffect::None => "appearance.background.effect.none.desc",
        NewThreadBackgroundEffect::Dither => "appearance.background.effect.dither.desc",
        NewThreadBackgroundEffect::Ascii => "appearance.background.effect.ascii.desc",
        NewThreadBackgroundEffect::Halftone => "appearance.background.effect.halftone.desc",
        NewThreadBackgroundEffect::Scanlines => "appearance.background.effect.scanlines.desc",
    }
}

impl AppearancePage {
    /// Persist the chosen glass material (zeron `appearance::set_surface`):
    /// the pre-port model stores two booleans, so Frosted is
    /// `translucent + blur` and Opaque clears both.
    fn set_surface_row(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(surface) = SURFACE_OPTIONS.get(index) else {
            return;
        };
        let translucent = matches!(surface, SurfacePreference::Frosted);
        {
            let settings = AppSettings::global_mut(cx);
            settings.background_translucent = translucent;
            settings.background_blur = translucent;
        }
        crate::settings::persist_now(cx);
        crate::settings::apply_appearance(window, cx);
        cx.notify();
    }

    /// Persist the selected background effect.
    fn set_effect_row(&mut self, index: usize, cx: &mut Context<Self>) {
        let Some(effect) = NewThreadBackgroundEffect::ALL.get(index).copied() else {
            return;
        };
        if crate::settings::background::set_effect(effect, cx) {
            cx.notify();
        }
    }

    /// Open the background adjustment dialog (upstream
    /// `open_background_adjustment`).
    fn open_background_adjustment(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.dialogs.background.open(window, cx);
        cx.notify();
    }

    /// Choose a background image and install it (upstream
    /// `choose_new_thread_background`).
    fn choose_background(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.material.error = None;
        let language = AppSettings::global(cx).language.clone();
        let receiver = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some(tr("appearance.background.choose", &language).into()),
        });
        let window_handle = window.window_handle();
        cx.spawn(async move |page, cx| {
            let path = match receiver.await {
                Ok(Ok(Some(mut paths))) => paths.pop(),
                _ => None,
            };
            let Some(path) = path else {
                return;
            };
            let installed = page.update(cx, |page, cx| {
                page.material
                    .set_error(crate::settings::background::install(&path, cx).err());
                page.material.error.is_none()
            });
            if matches!(installed, Ok(true)) {
                let _ = window_handle.update(cx, |_, window, cx| {
                    crate::settings::apply_appearance(window, cx);
                });
            }
            let _ = page.update(cx, |_, cx| cx.notify());
        })
        .detach();
    }

    /// Remove the installed background (upstream
    /// `remove_new_thread_background`).
    fn remove_background(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.material
            .set_error(crate::settings::background::remove(cx).err());
        if self.material.error.is_none() {
            crate::settings::apply_appearance(window, cx);
        }
        cx.notify();
    }

    /// Choose the wallpaper folder and immediately shuffle from it (upstream
    /// `choose_wallpaper_folder` calls `random_wallpaper`).
    fn choose_wallpaper_folder(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.material.error = None;
        let language = AppSettings::global(cx).language.clone();
        let receiver = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some(tr("appearance.wallpaper_folder.choose", &language).into()),
        });
        let window_handle = window.window_handle();
        cx.spawn(async move |page, cx| {
            let path = match receiver.await {
                Ok(Ok(Some(mut paths))) => paths.pop(),
                _ => None,
            };
            let Some(path) = path else {
                return;
            };
            let shuffled = page.update(cx, |page, cx| {
                crate::settings::wallpaper::set_folder(Some(path), cx);
                page.shuffle_wallpaper_now(cx)
            });
            if matches!(shuffled, Ok(true)) {
                let _ = window_handle.update(cx, |_, window, cx| {
                    crate::settings::apply_appearance(window, cx);
                });
            }
            let _ = page.update(cx, |_, cx| cx.notify());
        })
        .detach();
    }

    /// Shuffle one image from the persisted wallpaper folder (upstream
    /// `random_wallpaper`): pick it and adopt it as the artwork.
    fn shuffle_wallpaper(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.material.error = None;
        if self.shuffle_wallpaper_now(cx) {
            crate::settings::apply_appearance(window, cx);
        }
        cx.notify();
    }

    /// The state-only half of [`Self::shuffle_wallpaper`]; returns whether the
    /// installed background changed.
    fn shuffle_wallpaper_now(&mut self, cx: &mut Context<Self>) -> bool {
        let Some(folder) = AppSettings::global(cx).wallpaper_folder.clone() else {
            self.material.error = Some(
                tr(
                    "appearance.wallpaper_folder.empty",
                    &AppSettings::global(cx).language,
                )
                .into(),
            );
            return false;
        };
        let path = match crate::settings::wallpaper::pick_random(&folder) {
            Ok(path) => path,
            Err(error) => {
                self.material.set_error(Some(error));
                return false;
            }
        };
        crate::settings::wallpaper::record_source(&path, cx);
        match crate::settings::background::install(&path, cx) {
            Ok(()) => true,
            Err(error) => {
                self.material.set_error(Some(error));
                false
            }
        }
    }
}
