use std::sync::Arc;

use glib;
use gtk;
use gtk::prelude::*;

use thread_guard::ThreadGuard;
use nvim_bridge::CompletionItem;
use ui::popupmenu::CompletionItemWidgetWrap;
use ui::popupmenu::get_icon_pixbuf;
use ui::color::Color;

struct State {
    selected: i32,
    items: Vec<CompletionItemWidgetWrap>,
    items_to_load: Vec<CompletionItem>,

    source_id: Option<glib::SourceId>,

    list: gtk::ListBox,
    css_provider: gtk::CssProvider,
}

impl State {
    fn clear(&mut self) {
        if let Some(source_id) = self.source_id.take() {
            glib::source::source_remove(source_id);
        }

        while let Some(item) = self.items.pop() {
            item.row.destroy();
        }
    }

    fn new(list: gtk::ListBox, css_provider: gtk::CssProvider) -> Self {
        State {
            selected: -1,
            items: vec!(),
            items_to_load: vec!(),
            source_id: None,
            list,
            css_provider
        }
    }

    fn ensure_selected(&self) {
        if self.selected < 0 {
            self.list.unselect_all();
            return
        }

        if let Some(item) = self.items.get(self.selected as usize) {
            self.list.select_row(&item.row);
            item.row.grab_focus();
        }
    }
}

pub struct LazyLoader {
    state: Arc<ThreadGuard<State>>,
}

impl LazyLoader {
    pub fn new(list: gtk::ListBox, css_provider: gtk::CssProvider) -> Self {
        Self {
            state: Arc::new(ThreadGuard::new(State::new(list, css_provider))),
        }
    }

    pub fn get_selected(&self) -> i32 {
        self.state.borrow().selected
    }

    pub fn with_selected_item<F>(&self, f: F)
        where F: FnOnce(Option<&CompletionItemWidgetWrap>) {

        let state = self.state.borrow();
        let widget = if state.selected >= 0 {
            state.items.get(state.selected as usize)
        } else {
            None
        };

        f(widget);
    }

    pub fn len(&self) -> usize {
        let state = self.state.borrow();
        state.items.len()
    }

    pub fn set_items(&mut self, items: Vec<CompletionItem>, icon_fg: Color) {
        let mut state = self.state.borrow_mut();
        state.clear();

        state.items_to_load = items;
        state.selected = -1;

        let state_ref = self.state.clone();
        let source_id = glib::idle_add(move || {

            let mut state = state_ref.borrow_mut();

            // Load the rows in patches so we avoid renderes of "half height"
            // completion menus.
            for _ in 0..40 {
                if state.items_to_load.len() == 0 {
                    state.source_id = None;
                    state.ensure_selected();
                    return Continue(false)
                }

                let item = state.items_to_load.remove(0);
                let widget = CompletionItemWidgetWrap::create(item, &state.css_provider, &icon_fg);
                state.list.add(&widget.row);
                widget.row.show_all();
                state.items.push(widget);
            }

            Continue(true)
        });

        state.source_id = Some(source_id);

        state.list.show_all();
    }

    pub fn select(&mut self, item_num: i32) {
        let mut state = self.state.borrow_mut();
        state.selected = item_num;
        state.ensure_selected();
    }
}
