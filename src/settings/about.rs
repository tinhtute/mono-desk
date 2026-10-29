//! The About page (`settings-surface` T2): zeron's page scaffolding with
//! mono-desk's own content — the product blurb, version and third-party
//! credits.
//!
//! Content is ported from the pre-port page on `wip/gpui-kit-shell`
//! (`src/main.rs`): the tagline, the Info/Licenses/Changelog panels and the
//! `status.version` line. zeron's page kit has no tab bar for settings pages
//! (its `TabBar` is the document strip of the chat route, out of scope), so the
//! three panels are zeron `section`s in the same order and keep the old tab
//! labels as their section labels.
//!
//! The page is read-only: no interactive control, so no role/aria/tab-index
//! metadata is needed here.

use gpui::prelude::*;
use gpui::{Context, IntoElement, Render, ScrollHandle, Window, div, px};

use zeron_ui::settings::widgets;
use zeron_ui::theme::Theme;

use crate::app_settings::AppSettings;
use crate::i18n::tr;

/// The bundled third-party notices, in the pre-port page's order.
const LICENSES: [&str; 3] = ["about.licenses.0", "about.licenses.1", "about.licenses.2"];

pub struct AboutPage {
    /// The page's scroll offset (zeron's pages own their scroll handle).
    scroll: ScrollHandle,
}

impl AboutPage {
    pub fn new() -> Self {
        Self {
            scroll: ScrollHandle::new(),
        }
    }
}

impl Render for AboutPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::of(cx).clone();
        let lang = AppSettings::global(cx).language.clone();

        // Info: the app version with the stack description under it.
        let info_card = widgets::section_card(&theme).mt_0().child(
            widgets::card_row(&theme, true)
                .child(
                    div()
                        .flex_1()
                        .min_w(px(160.0))
                        .child(widgets::row_title(&theme, tr("status.version", &lang)))
                        .child(widgets::meta_line(
                            &theme,
                            vec![div().child(tr("about.info.body", &lang)).into_any_element()],
                        )),
                )
                .child(widgets::badge(&theme, env!("CARGO_PKG_VERSION"))),
        );

        // Licenses: the bundled components, one quiet row each.
        let mut licenses_card = widgets::section_card(&theme).mt_0();
        for (ix, key) in LICENSES.into_iter().enumerate() {
            licenses_card = licenses_card.child(
                widgets::card_row(&theme, ix == 0).child(
                    div()
                        .flex_1()
                        .min_w(px(160.0))
                        .child(widgets::row_title(&theme, tr(key, &lang))),
                ),
            );
        }

        // Changelog: the release highlight as a single row.
        let changelog_card = widgets::section_card(&theme).mt_0().child(
            widgets::card_row(&theme, true).child(
                div()
                    .flex_1()
                    .min_w(px(160.0))
                    .child(widgets::row_title(&theme, tr("about.changelog.0", &lang))),
            ),
        );

        div()
            .id("about-settings-page")
            .size_full()
            .overflow_y_scroll()
            .track_scroll(&self.scroll)
            .child(
                widgets::page_column()
                    .child(widgets::page_header(&theme, tr("page.about", &lang), None))
                    .child(widgets::page_subtitle(&theme, tr("about.tagline", &lang)))
                    .child(widgets::section(
                        &theme,
                        tr("about.tab.info", &lang),
                        info_card,
                    ))
                    .child(widgets::section(
                        &theme,
                        tr("about.tab.licenses", &lang),
                        licenses_card,
                    ))
                    .child(widgets::section(
                        &theme,
                        tr("about.tab.changelog", &lang),
                        changelog_card,
                    )),
            )
    }
}
