//! The Finance UI page (`settings-surface` T2): zeron's page scaffolding hosts
//! mono-desk's own finance showcase — the eight-card canvas cloned from the
//! pre-port app (`src/finance/`), rebuilt on zeron's widgets instead of
//! gpui-component.
//!
//! Placeholder body: the shell wiring lands first; the showcase is filled in
//! by the T2 finance task.

use gpui::prelude::*;
use gpui::{Context, IntoElement, Render, ScrollHandle, Window, div};

use zeron_ui::settings::widgets;
use zeron_ui::theme::Theme;

use crate::app_settings::AppSettings;
use crate::i18n::tr;

pub struct FinancePage {
    scroll: ScrollHandle,
}

impl FinancePage {
    pub fn new() -> Self {
        Self {
            scroll: ScrollHandle::new(),
        }
    }
}

impl Render for FinancePage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::of(cx).clone();
        let lang = AppSettings::global(cx).language.clone();
        div()
            .id("finance-settings-page")
            .size_full()
            .overflow_y_scroll()
            .track_scroll(&self.scroll)
            .child(
                widgets::page_column()
                    .child(widgets::page_header(
                        &theme,
                        tr("page.finance", &lang),
                        None,
                    ))
                    .child(widgets::page_subtitle(
                        &theme,
                        tr("page.finance.desc", &lang),
                    ))
                    .child(crate::finance::page(&lang)),
            )
    }
}
