#[macro_export]
macro_rules! add_css_provider {
    ($provider:expr, $($widget:expr),*) => (
        {
            $(
                $widget
                    .style_context()
                    .add_provider($provider,
                                  gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
            )*
        }
    );
}

// Make moving clones into closures more convenient.
// Sourced from https://github.com/gtk-rs/examples/blob/e17372b1c65788b022ff152fff37d392d0f31e87/src/bin/treeview.rs#L20-L36
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

// Upgrade weak reference or return.
// Sourced from https://github.com/gtk-rs/examples/blob/e17372b1c65788b022ff152fff37d392d0f31e87/src/bin/gtktest.rs#L36-L48
#[macro_export]
macro_rules! upgrade_weak {
    ($x:ident, $r:expr) => {{
        match $x.upgrade() {
            Some(o) => o,
            None => return $r,
        }
    }};
    ($x:ident) => {
        upgrade_weak!($x, ())
    };
}

mod animation;
mod cmdline;
pub mod color;
mod common;
#[cfg(feature = "libwebkit2gtk")]
mod cursor_tooltip;
mod font;
mod grid;
mod popupmenu;
mod state;
mod tabline;
#[allow(clippy::module_inception)]
mod ui;
mod wildmenu;
mod window;
pub use self::ui::UI;
