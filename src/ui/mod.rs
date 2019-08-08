#[macro_export]
macro_rules! add_css_provider {
    ($provider:expr, $($widget:expr),*) => (
        {
            $(
                $widget
                    .get_style_context()
                    .add_provider($provider,
                                  gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
            )*
        }
    );
}

// Make moving clones into closures more convenient.
// Sources from https://github.com/gtk-rs/examples/blob/e17372b1c65788b022ff152fff37d392d0f31e87/src/bin/treeview.rs#L20-L36
#[macro_export]
macro_rules! clone {
    (@param _) => ( _ );
    (@param $x:ident) => ( $x );
    ($($n:ident),+ => move || $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move || $body
        }
    );
    ($($n:ident),+ => move |$($p:tt),+| $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move |$(clone!(@param $p),)+| $body
        }
    );
}

mod cmdline;
pub mod color;
mod common;
mod cursor_tooltip;
mod font;
mod grid;
mod popupmenu;
mod tabline;
mod ui;
mod wildmenu;
pub use self::ui::UI;
