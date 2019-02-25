use std::sync::Arc;

use glib;
use gtk;
use gtk::prelude::*;

use thread_guard::ThreadGuard;
use nvim_bridge::CompletionItem;
use ui::popupmenu::CompletionItemWidgetWrap;
use ui::color::Color;

struct State {
    items: Vec<CompletionItemWidgetWrap>,
    items_to_load: Vec<CompletionItem>,

    source_id: Option<glib::SourceId>,

    once_loaded: Option<Box<Fn(&Vec<CompletionItemWidgetWrap>)>>,

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
            items: vec!(),
            items_to_load: vec!(),
            once_loaded: None,
            source_id: None,
            list,
            css_provider
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

    pub fn set_items(&mut self, items: Vec<CompletionItem>, icon_fg: Color) {
        let mut state = self.state.borrow_mut();
        state.clear();

        state.items_to_load = items;

        let state_ref = self.state.clone();
        let source_id = glib::idle_add(move || {

            let mut state = state_ref.borrow_mut();

            // Load the rows in patches so we avoid renderes of "half height"
            // completion menus.
            for _ in 0..40 {
                if state.items_to_load.len() == 0 {
                    state.source_id = None;

                    if let Some(mut cb) = state.once_loaded.take() {
                        cb(&state.items);
                    }

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

    pub fn once_loaded<F>(&mut self, f: F)
        where F: Fn(&Vec<CompletionItemWidgetWrap>) + 'static {

        let mut state = self.state.borrow_mut();
        if state.source_id.is_some() {
            state.once_loaded = Some(Box::new(f));
        } else {
            f(&state.items);
        }
    }
}
