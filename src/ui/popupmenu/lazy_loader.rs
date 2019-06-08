use std::sync::Arc;

use glib;
use gtk;
use gtk::prelude::*;

use nvim_bridge::CompletionItem;
use thread_guard::ThreadGuard;
use ui::color::Color;
use ui::popupmenu::CompletionItemWidgetWrap;

struct State {
    items: Vec<CompletionItemWidgetWrap>,
    items_to_load: Vec<CompletionItem>,

    source_id: Option<glib::SourceId>,

    /// Once we're loaded some (or all) data, this closure gets called if
    /// one exists. The first value in the tuple can is indication on the
    /// number of items needed before calling the closure.
    once_loaded: Option<(Option<i32>, Box<Fn(&Vec<CompletionItemWidgetWrap>)>)>,

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
            items: vec![],
            items_to_load: vec![],
            once_loaded: None,
            source_id: None,
            list,
            css_provider,
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

    pub fn set_items(
        &mut self,
        items: Vec<CompletionItem>,
        icon_fg: Color,
        size: f64,
    ) {
        let mut state = self.state.borrow_mut();
        state.clear();

        state.items_to_load = items;

        // Check if CompletionItems have known kinds
        let mut know_items_kinds = false;
        for item in state.items_to_load.iter() {
            use ui::popupmenu::get_icon_pixbuf;
            match get_icon_pixbuf(&item.kind, &icon_fg, size, know_items_kinds) {
                Ok(_) => know_items_kinds = true,
                Err(_) => continue,
            }
        }


        let state_ref = self.state.clone();
        let source_id = glib::idle_add(move || {
            let mut state = state_ref.borrow_mut();

            // Load the rows in patches so we avoid renders of "half height"
            // completion menus.
            for _ in 0..40 {
                if state.items_to_load.len() == 0 {
                    state.source_id = None;

                    if let Some((_, cb)) = state.once_loaded.take() {
                        cb(&state.items);
                    }

                    return Continue(false);
                }

                let item = state.items_to_load.remove(0);
                let widget = CompletionItemWidgetWrap::create(
                    item,
                    &state.css_provider,
                    &icon_fg,
                    size,
                    know_items_kinds,
                );
                state.list.add(&widget.row);
                widget.row.show_all();
                state.items.push(widget);
            }

            // Check if we have a eager closure to be called.
            if let Some((i, cb)) = state.once_loaded.take() {
                if let Some(i) = i {
                    if state.items.len() >= i as usize {
                        cb(&state.items);
                    } else {
                        // Not ready yet, put the items back.
                        state.once_loaded = Some((Some(i), cb));
                    }
                } else {
                    // Not ready yet, put the items back.
                    state.once_loaded = Some((i, cb));
                }
            }

            Continue(true)
        });

        state.source_id = Some(source_id);

        state.list.show_all();
    }

    /// Calls `f` once `i` mount if items (or all items) are loaded.
    /// Only one callback can exists at a time (e.g. when we are loading
    /// items). If all items are already loaded, `f` is called immediately.
    pub fn once_loaded<F>(&mut self, i: Option<i32>, f: F)
    where
        F: Fn(&Vec<CompletionItemWidgetWrap>) + 'static,
    {
        let mut state = self.state.borrow_mut();
        if state.source_id.is_some() {
            state.once_loaded = Some((i, Box::new(f)));
        } else {
            f(&state.items);
        }
    }
}
