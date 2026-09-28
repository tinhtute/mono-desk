use std::ops::Range;
use std::rc::Rc;

use crate::{
    IconName, Sizable, Size, StyledExt,
    group_box::GroupBoxVariant,
    h_resizable,
    input::{Input, InputState},
    resizable_panel,
    setting::{SettingGroup, SettingPage},
    sidebar::{Sidebar, SidebarCollapsible, SidebarMenu, SidebarMenuItem},
    v_flex,
};
use gpui::{
    AnyElement, App, AppContext as _, Axis, ElementId, Entity, IntoElement, ParentElement as _,
    Pixels, RenderOnce, StyleRefinement, Styled, Window, container_query, div,
    prelude::FluentBuilder as _, px, relative,
};
use rust_i18n::t;

const STACKED_LAYOUT_MAX_WIDTH: Pixels = px(480.);

/// The settings structure containing multiple pages for app settings.
///
/// The hierarchy of settings is as follows:
///
/// ```ignore
/// Settings
///   SettingPage     <- The single active page displayed
///     SettingGroup
///       SettingItem
///         Label
///         SettingField (e.g., Switch, Dropdown, Input)
/// ```
#[derive(IntoElement)]
pub struct Settings {
    id: ElementId,
    pages: Vec<SettingPage>,
    group_variant: GroupBoxVariant,
    size: Size,
    sidebar_width: Pixels,
    sidebar_size_range: Range<Pixels>,
    /// Style refinement applied to active sidebar menu items.
    active_item_style: StyleRefinement,
    /// Style refinement applied to the content panel.
    content_panel_style: StyleRefinement,
    sidebar_style: StyleRefinement,
    default_selected_index: SelectIndex,
    header_style: StyleRefinement,
    /// Custom content rendered above the sidebar search input.
    sidebar_header: Option<AnyElement>,
    /// Custom content rendered at the bottom of the sidebar.
    sidebar_footer: Option<AnyElement>,
    /// Collapse the sidebar to icon width.
    sidebar_collapsed: bool,
    /// Whether to paint the resize handle between the panels.
    show_handle: bool,
}

impl Settings {
    /// Create a new settings with the given ID.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            pages: vec![],
            group_variant: GroupBoxVariant::default(),
            size: Size::default(),
            sidebar_width: px(250.0),
            sidebar_size_range: px(160.0)..px(360.0),
            active_item_style: StyleRefinement::default(),
            content_panel_style: StyleRefinement::default(),
            sidebar_style: StyleRefinement::default(),
            default_selected_index: SelectIndex::default(),
            header_style: StyleRefinement::default(),
            sidebar_header: None,
            sidebar_footer: None,
            sidebar_collapsed: false,
            show_handle: true,
        }
    }

    /// Set the width of the sidebar, default is `250px`.
    pub fn sidebar_width(mut self, width: impl Into<Pixels>) -> Self {
        self.sidebar_width = width.into();
        self
    }

    /// Set the resize range of the sidebar, default is `160px..360px`.
    pub fn sidebar_size_range(mut self, range: impl Into<Range<Pixels>>) -> Self {
        self.sidebar_size_range = range.into();
        self
    }

    /// Add a page to the settings.
    pub fn page(mut self, page: SettingPage) -> Self {
        self.pages.push(page);
        self
    }

    /// Add pages to the settings.
    pub fn pages(mut self, pages: impl IntoIterator<Item = SettingPage>) -> Self {
        self.pages.extend(pages);
        self
    }

    /// Set the default variant for all setting groups.
    ///
    /// All setting groups will use this variant unless overridden individually.
    pub fn with_group_variant(mut self, variant: GroupBoxVariant) -> Self {
        self.group_variant = variant;
        self
    }

    /// Set the style refinement for the sidebar.
    pub fn sidebar_style(mut self, style: &StyleRefinement) -> Self {
        self.sidebar_style = style.clone();
        self
    }

    /// Set the style refinement applied to active sidebar menu items.
    ///
    /// Applied after the built-in active styling, so it wins over the
    /// default accent background.
    pub fn active_item_style(mut self, style: &StyleRefinement) -> Self {
        self.active_item_style = style.clone();
        self
    }

    /// Set the style refinement for the content panel.
    pub fn content_panel_style(mut self, style: &StyleRefinement) -> Self {
        self.content_panel_style = style.clone();
        self
    }

    /// Set the default index of the page to be selected.
    pub fn default_selected_index(mut self, index: SelectIndex) -> Self {
        self.default_selected_index = index;
        self
    }

    /// Set the style refinement for the header.
    pub fn header_style(mut self, style: &StyleRefinement) -> Self {
        self.header_style = style.clone();
        self
    }

    /// Set custom content rendered above the sidebar search input.
    pub fn sidebar_header(mut self, header: impl IntoElement) -> Self {
        self.sidebar_header = Some(header.into_any_element());
        self
    }

    /// Set custom content rendered at the bottom of the sidebar.
    pub fn sidebar_footer(mut self, footer: impl IntoElement) -> Self {
        self.sidebar_footer = Some(footer.into_any_element());
        self
    }

    /// Collapse the sidebar to icon width.
    ///
    /// When collapsed the header (brand and search) is hidden and menu
    /// items render as centered icons.
    pub fn sidebar_collapsed(mut self, collapsed: bool) -> Self {
        self.sidebar_collapsed = collapsed;
        self
    }

    /// Whether to paint the resize handle between the panels, default is `true`.
    ///
    /// When `false`, the handle keeps its hit area and drag behavior but
    /// paints nothing.
    pub fn show_handle(mut self, show: bool) -> Self {
        self.show_handle = show;
        self
    }

    fn filtered_pages(&self, query: &str, cx: &App) -> Vec<SettingPage> {
        self.pages
            .iter()
            .filter_map(|page| {
                let filtered_groups: Vec<SettingGroup> = page
                    .groups
                    .iter()
                    .filter_map(|group| {
                        let mut group = group.clone();
                        group.items = group
                            .items
                            .iter()
                            .filter(|item| item.is_match(&query, cx))
                            .cloned()
                            .collect();
                        if group.items.is_empty() {
                            None
                        } else {
                            Some(group)
                        }
                    })
                    .collect();
                let mut page = page.clone();
                page.groups = filtered_groups;
                if page.groups.is_empty() {
                    None
                } else {
                    Some(page)
                }
            })
            .collect()
    }

    fn render_active_page(
        &self,
        state: &Entity<SettingsState>,
        pages: &Vec<SettingPage>,
        options: &RenderOptions,
        window: &mut Window,
        cx: &mut App,
    ) -> gpui::AnyElement {
        let selected_index = state.read(cx).selected_index;

        for (ix, page) in pages.into_iter().enumerate() {
            if selected_index.page_ix == ix {
                return page
                    .render(ix, state, &options, window, cx)
                    .into_any_element();
            }
        }

        return div().into_any_element();
    }

    fn render_sidebar(
        &mut self,
        state: &Entity<SettingsState>,
        pages: &Vec<SettingPage>,
        _: &mut Window,
        cx: &mut App,
    ) -> impl IntoElement {
        let selected_index = state.read(cx).selected_index;
        let search_input = state.read(cx).search_input.clone();

        Sidebar::new("settings-sidebar")
            .w(relative(1.))
            .border_0()
            .refine_style(&self.sidebar_style)
            .collapsible(SidebarCollapsible::Icon)
            .collapsed(self.sidebar_collapsed)
            .when(!self.sidebar_collapsed, |this| {
                this.header(
                    v_flex()
                        .w_full()
                        .gap_2()
                        .when_some(self.sidebar_header.take(), |this, header| {
                            this.child(header)
                        })
                        .child(
                            div()
                                .w_full()
                                .refine_style(&self.header_style)
                                .child(Input::new(&search_input).prefix(IconName::Search)),
                        ),
                )
            })
            .when_some(self.sidebar_footer.take(), |this, footer| {
                this.footer(footer)
            })
            .child(
                SidebarMenu::new().children(pages.iter().enumerate().map(|(page_ix, page)| {
                    let is_page_active =
                        selected_index.page_ix == page_ix && selected_index.group_ix.is_none();
                    SidebarMenuItem::new(page.title.clone())
                        .click_to_open(true)
                        .when_some(page.icon.clone(), |this, icon| this.icon(icon))
                        .default_open(page.default_open)
                        .active(is_page_active)
                        .active_style(self.active_item_style.clone())
                        .on_click({
                            let state = state.clone();
                            move |_, _, cx| {
                                state.update(cx, |state, cx| {
                                    state.selected_index = SelectIndex {
                                        page_ix,
                                        ..Default::default()
                                    };
                                    cx.notify();
                                })
                            }
                        })
                        .when(page.groups.len() > 1, |this| {
                            this.children(
                                page.groups
                                    .iter()
                                    .filter(|g| g.title.is_some())
                                    .enumerate()
                                    .map(|(group_ix, group)| {
                                        let is_active = selected_index.page_ix == page_ix
                                            && selected_index.group_ix == Some(group_ix);
                                        let title = group.title.clone().unwrap_or_default();

                                        SidebarMenuItem::new(title)
                                            .when_some(group.icon.clone(), |this, icon| {
                                                this.icon(icon)
                                            })
                                            .active(is_active)
                                            .active_style(self.active_item_style.clone())
                                            .on_click({
                                                let state = state.clone();
                                                move |_, _, cx| {
                                                    state.update(cx, |state, cx| {
                                                        state.selected_index = SelectIndex {
                                                            page_ix,
                                                            group_ix: Some(group_ix),
                                                        };
                                                        state.deferred_scroll_group_ix =
                                                            Some(group_ix);
                                                        cx.notify();
                                                    })
                                                }
                                            })
                                    }),
                            )
                        })
                })),
            )
    }
}

impl Sizable for Settings {
    fn with_size(mut self, size: impl Into<Size>) -> Self {
        self.size = size.into();
        self
    }
}

pub(super) struct SettingsState {
    pub(super) selected_index: SelectIndex,
    /// If set, defer scrolling to this group index after rendering.
    pub(super) deferred_scroll_group_ix: Option<usize>,
    pub(super) search_input: Entity<InputState>,
}

/// Options for rendering setting item.
///
/// The fields are private and reached through the methods below, so that a new
/// one can be added without breaking the item renderers. The setters take
/// `self` by value, so a nested renderer narrows a copy of its parent options:
///
/// ```ignore
/// item.render_item(&options.with_item_ix(item_ix), window, cx)
/// ```
#[derive(Clone, Copy)]
pub struct RenderOptions {
    page_ix: usize,
    group_ix: usize,
    item_ix: usize,
    size: Size,
    group_variant: GroupBoxVariant,
    layout: Axis,
    disabled: bool,
}

impl RenderOptions {
    pub fn new() -> Self {
        Self {
            page_ix: 0,
            group_ix: 0,
            item_ix: 0,
            size: Size::default(),
            group_variant: GroupBoxVariant::default(),
            layout: Axis::Horizontal,
            disabled: false,
        }
    }

    pub fn with_page_ix(mut self, page_ix: usize) -> Self {
        self.page_ix = page_ix;
        self
    }

    pub fn with_group_ix(mut self, group_ix: usize) -> Self {
        self.group_ix = group_ix;
        self
    }

    pub fn with_item_ix(mut self, item_ix: usize) -> Self {
        self.item_ix = item_ix;
        self
    }

    pub fn with_size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    pub fn with_group_variant(mut self, group_variant: GroupBoxVariant) -> Self {
        self.group_variant = group_variant;
        self
    }

    pub fn with_layout(mut self, layout: Axis) -> Self {
        self.layout = layout;
        self
    }

    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn page_ix(&self) -> usize {
        self.page_ix
    }

    pub fn group_ix(&self) -> usize {
        self.group_ix
    }

    pub fn item_ix(&self) -> usize {
        self.item_ix
    }

    pub fn size(&self) -> Size {
        self.size
    }

    pub fn group_variant(&self) -> GroupBoxVariant {
        self.group_variant
    }

    pub fn layout(&self) -> Axis {
        self.layout
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Default)]
pub struct SelectIndex {
    pub page_ix: usize,
    pub group_ix: Option<usize>,
}

impl RenderOnce for Settings {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = window.use_keyed_state(self.id.clone(), cx, |window, cx| {
            let search_input = cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder(t!("Settings.search_placeholder"))
                    .default_value("")
            });

            SettingsState {
                search_input,
                selected_index: self.default_selected_index,
                deferred_scroll_group_ix: None,
            }
        });

        let query = state.read(cx).search_input.read(cx).value();
        let filtered_pages = self.filtered_pages(&query, cx);
        let options = RenderOptions::new()
            .with_size(self.size)
            .with_group_variant(self.group_variant);
        let sidebar_size_range = self.sidebar_size_range.clone();
        let sidebar = self
            .render_sidebar(&state, &filtered_pages, window, cx)
            .into_any_element();

        h_resizable(self.id.clone())
            .when(!self.show_handle, |this| {
                this.with_handle_appearance(Rc::new(|_, _, _| {
                    Some(div().into_any_element())
                }))
            })
            .child(
                resizable_panel()
                    .size(self.sidebar_width)
                    .size_range(sidebar_size_range)
                    .child(sidebar),
            )
            .child(
                resizable_panel()
                    .refine_style(&self.content_panel_style)
                    .child(container_query(move |size, window, cx| {
                        let options =
                            options.with_layout(if size.width <= STACKED_LAYOUT_MAX_WIDTH {
                                Axis::Vertical
                            } else {
                                Axis::Horizontal
                            });
                        self.render_active_page(&state, &filtered_pages, &options, window, cx)
                    })),
            )
    }
}
