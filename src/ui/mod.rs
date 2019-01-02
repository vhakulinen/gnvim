#[macro_export]
macro_rules! add_css_provider {
    ($provider:expr, $($widget:expr),*) => (
        {
            $(
                $widget
                    .get_style_context()
                    .unwrap()
                    .add_provider($provider,
                                  gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
            )*
        }
    );
}

mod cmdline;
pub mod color;
mod common;
mod font;
mod grid;
mod cursor_tooltip;
mod popupmenu;
mod tabline;
mod ui;
mod wildmenu;
pub use self::ui::UI;
