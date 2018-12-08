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

mod grid;
mod ui;
mod popupmenu;
mod tabline;
mod cmdline;
mod wildmenu;
pub mod color;
pub use self::ui::UI;
