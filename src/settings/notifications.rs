//! The Notifications page (`settings-surface` T1): zeron's page scaffolding
//! with mono-desk's own content — the delivery master switch, the three
//! channel switches and the digest cadence.
//!
//! zeron has no checkbox or radio widget in its settings kit, so the channels
//! are `toggle_switch` rows and the cadence is zeron's `select`; the master
//! switch keeps the card readable but inert while delivery is off (the old
//! page disabled the channels in the same state).

use gpui::prelude::*;
use gpui::{
    AnyElement, Context, IntoElement, Render, Role, ScrollHandle, SharedString, Toggled, Window,
    div, px,
};

use zeron_ui::settings::select::{SelectOption, SelectState, select};
use zeron_ui::settings::widgets;
use zeron_ui::theme::Theme;

use crate::app_settings::{AppSettings, DigestCadence};
use crate::i18n::tr;

/// The three notification channels, in the old page's order.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Channel {
    Email,
    Desktop,
    Digest,
}

impl Channel {
    const ALL: [Self; 3] = [Self::Email, Self::Desktop, Self::Digest];

    fn title_key(self) -> &'static str {
        match self {
            Self::Email => "item.channel_email",
            Self::Desktop => "item.channel_desktop",
            Self::Digest => "item.channel_digest",
        }
    }

    fn get(self, settings: &AppSettings) -> bool {
        match self {
            Self::Email => settings.channel_email,
            Self::Desktop => settings.channel_desktop,
            Self::Digest => settings.channel_digest,
        }
    }

    fn set(self, settings: &mut AppSettings, on: bool) {
        match self {
            Self::Email => settings.channel_email = on,
            Self::Desktop => settings.channel_desktop = on,
            Self::Digest => settings.channel_digest = on,
        }
    }
}

pub struct NotificationsPage {
    /// Open state of the digest cadence select.
    digest: SelectState,
    /// The page's scroll offset (zeron's pages own their scroll handle).
    scroll: ScrollHandle,
}

impl NotificationsPage {
    pub fn new() -> Self {
        Self {
            digest: SelectState::default(),
            scroll: ScrollHandle::new(),
        }
    }

    fn set_delivery(&mut self, cx: &mut Context<Self>) {
        let enabled = AppSettings::global(cx).notifications_enabled;
        AppSettings::global_mut(cx).notifications_enabled = !enabled;
        super::persist_now(cx);
        cx.notify();
    }

    fn set_all_channels(&mut self, on: bool, cx: &mut Context<Self>) {
        let settings = AppSettings::global_mut(cx);
        for channel in Channel::ALL {
            channel.set(settings, on);
        }
        super::persist_now(cx);
        cx.notify();
    }

    fn toggle_channel(&mut self, channel: Channel, cx: &mut Context<Self>) {
        if !AppSettings::global(cx).notifications_enabled {
            return;
        }
        let on = channel.get(AppSettings::global(cx));
        channel.set(AppSettings::global_mut(cx), !on);
        super::persist_now(cx);
        cx.notify();
    }

    fn set_digest(&mut self, ix: usize, cx: &mut Context<Self>) {
        AppSettings::global_mut(cx).digest = DigestCadence::ALL[ix];
        super::persist_now(cx);
        cx.notify();
    }
}

impl Render for NotificationsPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::of(cx).clone();
        let settings = AppSettings::global(cx);
        let lang: SharedString = settings.language.clone();
        let enabled = settings.notifications_enabled;
        let digest_cadence = settings.digest;
        let channel_on = Channel::ALL.map(|channel| channel.get(settings));
        let all_on = channel_on.iter().all(|on| *on);
        let enabled_count = channel_on.iter().filter(|on| **on).count();

        // Delivery: the master switch with the current state as a badge.
        let delivery_control = div()
            .flex()
            .items_center()
            .gap(px(12.0))
            .child(widgets::badge(
                &theme,
                tr(
                    if enabled {
                        "status.enabled"
                    } else {
                        "status.disabled"
                    },
                    &lang,
                ),
            ))
            .child(
                widgets::toggle_switch(&theme, enabled, "notifications-delivery")
                    .id("notifications-delivery-toggle")
                    .tab_index(0)
                    .role(Role::Switch)
                    .aria_label(tr("item.notifications", &lang))
                    .aria_toggled(if enabled {
                        Toggled::True
                    } else {
                        Toggled::False
                    })
                    .focus_visible(|style| style.border_2().border_color(theme.accent))
                    .cursor_pointer()
                    .on_click(cx.listener(|page, _, _, cx| page.set_delivery(cx))),
            );

        let delivery = widgets::section_card(&theme)
            .mt_0()
            .child(super::control_row(
                &theme,
                true,
                tr("item.notifications", &lang),
                tr("item.notifications.desc", &lang),
                delivery_control,
            ));

        // Channels: an "all" switch that reads as the tri-state parent (on only
        // when every channel is on) above the three channel switches.
        let mut channels_card = widgets::section_card(&theme)
            .mt_0()
            .child(super::control_row(
                &theme,
                true,
                tr("item.channel_all", &lang),
                format!("{enabled_count}/{}", Channel::ALL.len()),
                widgets::toggle_switch(&theme, all_on, "notifications-all-channels")
                    .id("notifications-all-channels-toggle")
                    .tab_index(0)
                    .role(Role::Switch)
                    .aria_label(tr("item.channel_all", &lang))
                    .aria_toggled(if all_on {
                        Toggled::True
                    } else {
                        Toggled::False
                    })
                    .focus_visible(|style| style.border_2().border_color(theme.accent))
                    .cursor_pointer()
                    .on_click(cx.listener(|page, _, _, cx| page.set_all_channels(true, cx))),
            ));
        for (ix, channel) in Channel::ALL.into_iter().enumerate() {
            let on = channel_on[ix];
            let row_id: SharedString = format!("notifications-channel-{ix}").into();
            let switch_id: SharedString = format!("notifications-switch-{ix}").into();
            channels_card = channels_card.child(super::control_row(
                &theme,
                false,
                tr(channel.title_key(), &lang),
                tr("item.channels.desc", &lang),
                widgets::toggle_switch(&theme, on, switch_id)
                    .id(row_id)
                    .tab_index(0)
                    .role(Role::Switch)
                    .aria_label(tr(channel.title_key(), &lang))
                    .aria_toggled(if on { Toggled::True } else { Toggled::False })
                    .focus_visible(|style| style.border_2().border_color(theme.accent))
                    .cursor_pointer()
                    .on_click(cx.listener(move |page, _, _, cx| page.toggle_channel(channel, cx))),
            ));
        }
        // The old page greyed the channel card out while delivery is off.
        let channels: AnyElement = div()
            .when(!enabled, |el| el.opacity(0.55))
            .child(channels_card)
            .into_any_element();

        // Digest: zeron's select over the same three stored cadences.
        let digest_ix = DigestCadence::ALL
            .iter()
            .position(|cadence| *cadence == digest_cadence)
            .unwrap_or_default();
        let digest = select(
            "notifications-digest",
            tr("group.digest", &lang),
            &theme,
            |page: &mut Self| &mut page.digest,
        )
        .options(
            DigestCadence::ALL
                .iter()
                .map(|cadence| SelectOption::new(tr(cadence.label_key(), &lang))),
            digest_ix,
        )
        .width(144.0)
        .on_select(|page: &mut Self, ix, _window, cx| page.set_digest(ix, cx))
        .render(&self.digest, cx);
        let digest_card = widgets::section_card(&theme)
            .mt_0()
            .child(super::control_row(
                &theme,
                true,
                tr("group.digest", &lang),
                tr("item.digest.desc", &lang),
                digest,
            ));

        div()
            .id("notifications-settings-page")
            .size_full()
            .overflow_y_scroll()
            .track_scroll(&self.scroll)
            .child(
                widgets::page_column()
                    .child(widgets::page_header(
                        &theme,
                        tr("page.notifications", &lang),
                        None,
                    ))
                    .child(widgets::section(
                        &theme,
                        tr("group.delivery", &lang),
                        delivery,
                    ))
                    .child(widgets::section(
                        &theme,
                        tr("group.channels", &lang),
                        channels,
                    ))
                    .child(widgets::section(
                        &theme,
                        tr("group.digest", &lang),
                        digest_card,
                    )),
            )
    }
}
