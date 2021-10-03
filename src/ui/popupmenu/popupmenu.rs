use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{gdk, glib, pango};
use log::error;

use crate::nvim_bridge::CompletionItem;
use crate::nvim_gio::GioNeovim;
use crate::ui::color::{Highlight, HlDefs, HlGroup};
use crate::ui::common::{
    calc_line_space, get_preferred_horizontal_position,
    get_preferred_vertical_position, spawn_local,
};
use crate::ui::font::{Font, FontUnit};
use crate::ui::grid::GridMetrics;
use crate::ui::popupmenu::get_icon_pixbuf;
use crate::ui::popupmenu::LazyLoader;

/// Maximum height of completion menu.
const MAX_HEIGHT: i32 = 500;
/// Fixed width of completion menu.
const DEFAULT_WIDTH_NO_DETAILS: i32 = 430;
const DEFAULT_WIDTH_WITH_DETAILS: i32 = 660;

#[derive(Default)]
pub struct PmenuColors {
    pub hl: Highlight,
    pub hl_sel: Highlight,
}

struct State {
    selected: i32,

    /// Size available for the popupmenu to use (width and height).
    available_size: Option<gdk::Rectangle>,
    /// Our anchor position where the popupmenu should be "pointing" to.
    anchor: gdk::Rectangle,
    /// Metrics of the base grid. Used when reporting pum bounds to nvim.
    base_metrics: Option<GridMetrics>,

    current_width: i32,

    width_no_details: i32,
    width_with_details: i32,
}

impl State {
    fn new() -> Self {
        State {
            selected: -1,
            available_size: None,
            base_metrics: None,
            anchor: gdk::Rectangle {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
            },

            current_width: DEFAULT_WIDTH_NO_DETAILS,
            width_no_details: DEFAULT_WIDTH_NO_DETAILS,
            width_with_details: DEFAULT_WIDTH_WITH_DETAILS,
        }
    }
}

pub struct Popupmenu {
    /// Box that contains all the "content". This box is placed in side the
    /// layout container.
    box_: gtk::Box,
    /// Top level container.
    layout: gtk::Layout,
    /// Scrolled window that contains the list box that displays all the items.
    scrolled_list: gtk::ScrolledWindow,
    /// Scrolled window that contains the info label for full info view.
    scrolled_info: gtk::ScrolledWindow,
    /// List box that contains all the completion items.
    list: gtk::ListBox,
    /// Style provider for all internal widgets.
    css_provider: gtk::CssProvider,

    /// Flag telling if the info label is shown.
    info_shown: bool,
    /// Label for displaying full info of a completion item.
    info_label: gtk::Label,
    /// Flag telling if the menu label should be shown on inactive items too.
    show_menu_on_all_items: bool,

    state: Rc<RefCell<State>>,
    items: LazyLoader,

    /// Our colors.
    colors: PmenuColors,
    /// Our font.
    font: Font,

    /// Line spacing.
    line_space: i64,
}

impl Popupmenu {
    /// Creates a new popupmenu.
    ///
    /// * `parent` - Overlay where popupmenu is placed. Ideally, this overlay
    ///              is where all the (neovim) grids are drawn.
    /// * `nvim` - Neovim instance. Popupmenu will instruct neovim to act on
    ///            user interaction.
    pub fn new(parent: &gtk::Overlay, nvim: GioNeovim) -> Self {
        let css_provider = gtk::CssProvider::new();

        let info_label = gtk::Label::new(Some(""));
        info_label.set_halign(gtk::Align::Start);
        info_label.set_valign(gtk::Align::Start);
        info_label.set_xalign(0.0);
        info_label.set_line_wrap(true);
        info_label.set_line_wrap_mode(pango::WrapMode::WordChar);
        info_label.set_widget_name("info-label");

        // Because we're setting valign and halign to the info label, we'll
        // need to have some container in between the label and scrolled window.
        // Other wise there'll be some black boxes when scroll bars are needed.
        let info_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        info_box.add(&info_label);

        let scrolled_info = gtk::ScrolledWindow::new(
            None::<&gtk::Adjustment>,
            None::<&gtk::Adjustment>,
        );
        scrolled_info.add(&info_box);
        scrolled_info
            .set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);

        let list = gtk::ListBox::new();
        list.set_valign(gtk::Align::Start);
        list.set_selection_mode(gtk::SelectionMode::Single);

        let scrolled_list = gtk::ScrolledWindow::new(
            None::<&gtk::Adjustment>,
            None::<&gtk::Adjustment>,
        );
        scrolled_list.add(&list);
        scrolled_list
            .set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);

        let box_ = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        box_.pack_start(&scrolled_list, true, true, 0);
        box_.pack_start(&scrolled_info, true, true, 0);
        box_.set_size_request(DEFAULT_WIDTH_NO_DETAILS, MAX_HEIGHT);
        box_.set_homogeneous(true);

        add_css_provider!(
            &css_provider,
            info_label,
            scrolled_info,
            list,
            scrolled_list,
            box_,
            // In gtk 3.18, the list box it self can't have borders,
            // so we'll have to add the border to its parent (which is the
            // viewport that scorlled window adds). This aint perfect,
            // but I didn't any find better solutions.
            scrolled_list.child().unwrap()
        );

        let state = Rc::new(RefCell::new(State::new()));

        // When a row is activated (by mouse click), notify neovim to change
        // the selection to the activated row.
        list.connect_row_activated(clone!(nvim, state => move |_, row| {
            let state = state.borrow_mut();
            let new = row.index();

            let selected = state.selected;

            let op = if new > selected { "<C-n>" } else { "<C-p>" };

            let mut payload = String::new();
            for _ in 0..(new - selected).abs() {
                payload.push_str(op)
            }

            let nvim = nvim.clone();
            spawn_local(async move {
                nvim.input(payload.as_str())
                    .await
                    .unwrap();
            });
        }));

        // On (mouse) button press...
        list.connect_button_press_event(clone!(nvim => move |_, e| {
            // ...check if the button press is double click.
            if e.event_type() == gdk::EventType::DoubleButtonPress {
                // And if so, tell neovim to select the current completion item.
                let nvim = nvim.clone();
                spawn_local(async move {
                    nvim.input("<C-y>")
                        .await
                        .unwrap();
                });
            }

            Inhibit(false)
        }));

        // TODO(ville): Should use gtk::Fixed here.
        let layout = gtk::Layout::new(
            None::<&gtk::Adjustment>,
            None::<&gtk::Adjustment>,
        );
        layout.put(&box_, 0, 0);
        layout.show_all();
        scrolled_info.hide();

        layout.connect_size_allocate(clone!(state => move |_, alloc| {
            let mut state = state.borrow_mut();
            state.available_size = Some(*alloc);
        }));

        let layout_weak = layout.downgrade();
        box_.connect_size_allocate(clone!(state, nvim, layout_weak, scrolled_info, scrolled_list => move |box_, alloc| {
            let layout = upgrade_weak!(layout_weak);
            let state = state.borrow();

            if let Some(area) = state.available_size {
                let pos = state.anchor;

                let (x, width) = get_preferred_horizontal_position(
                    &area,
                    &pos,
                    state.current_width,
                );
                let (y, height) = get_preferred_vertical_position(
                    &area,
                    &pos,
                    alloc.height.min(MAX_HEIGHT),
                );

                layout.move_(box_, x, y);

                box_.set_size_request(width, height);

                if let Some(ref base_metrics) = state.base_metrics {
                    let width = width as f64 / base_metrics.cell_width as f64;
                    let height = height as f64 / base_metrics.cell_height as f64;
                    let col = x as f64 / base_metrics.cell_width as f64;
                    let row = y as f64 / base_metrics.cell_height as f64;

                    let nvim = nvim.clone();
                    spawn_local(async move {
                        if let Err(err) = nvim.ui_pum_set_bounds(width, height, row, col).await {
                            error!("Failed to set pum bounds: {}", err);
                        }
                    });
                }

                // If we moved the popupmenu above the achor position, make
                // sure our contents are aligned to the bottom so there is not
                // cap between the achor and the content it self.
                if y < pos.y {
                    // Use get_child to get the viewport which is between
                    // the scrolled window and the actual widget that is
                    // inside it.
                    scrolled_list
                        .child()
                        .unwrap()
                        .set_valign(gtk::Align::End);
                    scrolled_info
                        .child()
                        .unwrap()
                        .set_valign(gtk::Align::End);
                } else {
                    scrolled_list
                        .child()
                        .unwrap()
                        .set_valign(gtk::Align::Start);
                    scrolled_info
                        .child()
                        .unwrap()
                        .set_valign(gtk::Align::Start);
                }
            }
        }));

        parent.add_overlay(&layout);
        // Hide the layout initially so it wont catch any input events that
        // should go to the girds.
        layout.hide();

        Popupmenu {
            items: LazyLoader::new(list.clone(), css_provider.clone()),
            show_menu_on_all_items: false,
            box_,
            layout,
            css_provider,
            list,
            scrolled_list,
            scrolled_info,
            info_label,
            state,
            info_shown: false,
            colors: PmenuColors::default(),
            font: Font::default(),
            line_space: 0,
        }
    }

    pub fn set_show_menu_on_all_items(&mut self, b: bool) {
        self.show_menu_on_all_items = b;
    }

    #[allow(unused)]
    pub fn is_above_anchor(&self) -> bool {
        self.scrolled_list.child().unwrap().valign() == gtk::Align::End
    }

    pub fn set_base_metrics(&self, metrics: GridMetrics) {
        let mut state = self.state.borrow_mut();
        state.base_metrics = Some(metrics);
    }

    pub fn toggle_show_info(&mut self) {
        {
            let state = self.state.borrow();

            self.info_shown = !self.info_shown;

            let selected = state.selected as usize;
            let info_shown = self.info_shown;
            let info_label = self.info_label.clone();
            self.items.once_loaded(Some(state.selected), move |items| {
                if let Some(item) = items.get(selected) {
                    item.info.set_visible(!info_shown);
                    item.menu.set_visible(!info_shown);

                    if item.item.info.is_empty() {
                        item.info.set_visible(false);
                    }

                    info_label.set_visible(
                        info_shown
                            && item.item.menu.len() + item.item.info.len() > 0,
                    );
                }
            });

            if !self.info_shown {
                let adj = self.scrolled_info.vadjustment();
                adj.set_value(0.0);
                // TODO(ville): There is a bug in GTK+ and some adjustment animations,
                //              where the adjustment's value is set back to upper - page-size
                //              if the user has "overshot" the scrolling. Work around this.
            }

            self.scrolled_info.set_visible(self.info_shown);
        }

        self.ensure_container_width();
    }

    fn ensure_container_width(&mut self) {
        let mut state = self.state.borrow_mut();

        state.current_width = if self.info_shown {
            state.width_with_details
        } else {
            state.width_no_details
        };

        self.box_.set_size_request(state.current_width, MAX_HEIGHT);
    }

    pub fn set_width(&mut self, w: i32) {
        {
            let mut state = self.state.borrow_mut();
            state.width_no_details = w;
        }
        self.ensure_container_width();
    }

    pub fn set_width_details(&mut self, w: i32) {
        {
            let mut state = self.state.borrow_mut();
            state.width_with_details = w;
        }
        self.ensure_container_width();
    }

    /// Hides the popupmenu.
    pub fn hide(&mut self) {
        self.layout.hide();
    }

    /// Shows the popupmenu.
    pub fn show(&mut self) {
        self.layout.show();
        self.box_.check_resize();
    }

    /// Sets the anchor point for popupmenu.
    pub fn set_anchor(&self, rect: gdk::Rectangle) {
        let mut state = self.state.borrow_mut();
        self.layout.move_(&self.box_, rect.x, rect.y + rect.height);
        state.anchor = rect;
    }

    pub fn set_items(&mut self, items: Vec<CompletionItem>, hl_defs: &HlDefs) {
        self.items.set_items(
            items,
            self.colors.hl.foreground.unwrap_or(hl_defs.default_fg),
            self.font.height as f64,
            self.show_menu_on_all_items,
        );

        self.list.show_all();
    }

    pub fn select(&mut self, item_num: i32, hl_defs: &HlDefs) {
        let state = self.state.clone();
        let scrolled_list = self.scrolled_list.clone();
        let fg = self.colors.hl.foreground.unwrap_or(hl_defs.default_fg);
        let fg_sel =
            self.colors.hl_sel.foreground.unwrap_or(hl_defs.default_fg);
        let font_height = self.font.height as f64;
        let list = self.list.clone();
        let info_label = self.info_label.clone();
        let info_shown = self.info_shown;
        let show_kind = self.items.get_show_kind();

        let show_menu_on_all_items = self.show_menu_on_all_items;

        self.items.once_loaded(Some(item_num), move |items| {
            let mut state = state.borrow_mut();

            if let Some(prev) = items.get(state.selected as usize) {
                prev.info.set_visible(false);
                prev.menu.set_visible(show_menu_on_all_items);

                if show_kind {
                    // Update the `kind` icon with default fg color.
                    let buf =
                        get_icon_pixbuf(&prev.item.kind, &fg, font_height);
                    prev.image.set_from_pixbuf(Some(&buf));
                }
            }

            state.selected = item_num;

            if item_num < 0 {
                list.unselect_all();
                info_label.set_text("");
                info_label.hide();

                // If selection is removed, move the scrolled window to the top.
                let adj = scrolled_list.vadjustment();
                glib::idle_add_local(move || {
                    adj.set_value(0.0);
                    Continue(false)
                });

                return;
            }

            if let Some(item) = items.get(state.selected as usize) {
                item.info.set_visible(!info_shown);
                item.menu.set_visible(!info_shown);

                if item.item.info.is_empty() {
                    item.info.set_visible(false);
                }

                item.row.grab_focus();
                list.select_row(Some(&item.row));

                {
                    let id = Rc::new(RefCell::new(None));
                    // Ensure that the row is in the view, but make sure first
                    // that the row it self has allocated itself. It is possible
                    // that when we selected the row and grabbed focus for it
                    // the row it self isn't "ready" to grab focus yet. Hence
                    // this signal handler here to ensure the row is in view.
                    // NOTE(ville): According to some IRC discussions, this
                    // hack wont work on GTK4. Prepare yourself!
                    let list_weak = list.downgrade();
                    let sig_id = item.row.connect_size_allocate(
                        clone!(id, list_weak => move |row, _| {
                            let list = upgrade_weak!(list_weak);
                            ensure_row_visible(&list, &row);

                            let id = id.borrow_mut().take().unwrap();
                            row.disconnect(id);
                        }),
                    );
                    *id.borrow_mut() = Some(sig_id);
                }

                if show_kind {
                    // Update the `kind` icon with "selected" fg color.
                    let buf =
                        get_icon_pixbuf(&item.item.kind, &fg_sel, font_height);
                    item.image.set_from_pixbuf(Some(&buf));
                }

                let newline = if !item.item.menu.is_empty()
                    && !item.item.info.is_empty()
                {
                    "\n"
                } else {
                    ""
                };

                info_label.set_text(&format!(
                    "{}{}{}",
                    item.item.menu, newline, item.item.info
                ));

                let has_info_content =
                    item.item.menu.len() + item.item.info.len() > 0;
                info_label.set_visible(info_shown && has_info_content);
            }
        });
    }

    pub fn set_colors(&mut self, hl_defs: &HlDefs) {
        self.colors = PmenuColors {
            hl: hl_defs
                .get_hl_group(&HlGroup::Pmenu)
                .cloned()
                .unwrap_or_default(),
            hl_sel: hl_defs
                .get_hl_group(&HlGroup::PmenuSel)
                .cloned()
                .unwrap_or_default(),
        };
        self.set_styles(hl_defs);
    }

    pub fn set_line_space(&mut self, space: i64, hl_defs: &HlDefs) {
        self.line_space = space;
        self.set_styles(hl_defs);

        // Set line space to the info_label with pango attrs.
        let attrs = pango::AttrList::new();
        let attr =
            pango::Attribute::new_rise(self.line_space as i32 * pango::SCALE);
        attrs.insert(attr);
        self.info_label.set_attributes(Some(&attrs));
    }

    fn set_styles(&self, hl_defs: &HlDefs) {
        let (above, below) = calc_line_space(self.line_space);

        let css = format!(
            "{font_wild}

            grid, list, row, label {{
                color: #{normal_fg};
                background-color: {normal_bg};
                outline: none;
            }}

            #info-label, list {{
                border: 1px solid #{normal_fg};
            }}

            row {{
                padding-top: {above}px;
                padding-bottom: {below}px;
            }}

            row:selected, row:selected > grid, row:selected > grid > label {{
                color: #{selected_fg};
                background-color: {selected_bg};
            }}

            box {{
            }}
            ",
            font_wild = self.font.as_wild_css(FontUnit::Point),
            normal_fg = self
                .colors
                .hl
                .foreground
                .unwrap_or(hl_defs.default_fg)
                .to_hex(),
            normal_bg = self.colors.hl.apply_blend(
                &self.colors.hl.background.unwrap_or(hl_defs.default_bg)
            ),
            selected_bg = self.colors.hl_sel.apply_blend(
                &self.colors.hl_sel.background.unwrap_or(hl_defs.default_bg)
            ),
            selected_fg = self
                .colors
                .hl_sel
                .foreground
                .unwrap_or(hl_defs.default_fg)
                .to_hex(),
            above = above.max(0),
            below = below.max(0),
        );
        CssProviderExt::load_from_data(&self.css_provider, css.as_bytes())
            .unwrap();
    }

    pub fn set_font(&mut self, font: Font, hl_defs: &HlDefs) {
        self.font = font;
        self.set_styles(hl_defs);
    }
}

fn ensure_row_visible(list: &gtk::ListBox, row: &gtk::ListBoxRow) {
    if let Some(adj) = list.adjustment() {
        let alloc = row.allocation();
        let y = alloc.y;
        let height = alloc.height;

        adj.clamp_page(y.into(), (y + height).into());
    }
}
