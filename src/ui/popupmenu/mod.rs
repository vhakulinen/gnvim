mod completion_item_widget;
mod lazy_loader;
#[allow(clippy::module_inception)]
mod popupmenu;

use self::completion_item_widget::get_icon_pixbuf;
use self::completion_item_widget::CompletionItemWidgetWrap;
use self::lazy_loader::LazyLoader;
pub use self::popupmenu::Popupmenu;
