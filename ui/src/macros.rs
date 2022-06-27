pub static LOG_DOMAIN: &str = "gnvim";

#[macro_export]
macro_rules! spawn_local {
    ($body:expr) => {
        glib::MainContext::default().spawn_local($body)
    };
}

#[macro_export]
macro_rules! some_or_return {
    ($opt:expr, $msgformat:literal $(,$arg:expr)* $(,)?) => {
        if let Some(some) = $opt {
            some
        } else {
            $crate::warn!($msgformat, $($arg),*);
            return;
        }
    };
}

#[macro_export]
macro_rules! log {
    ($level:expr, $format:literal $(,$arg:expr)* $(,)?) => {
        glib::g_log!(
            $crate::macros::LOG_DOMAIN,
            $level,
            $format,
            $($arg),*
        )
    };
}

#[macro_export]
macro_rules! warn {
    // TODO(ville): It would make sense to display some error to the user here too.
    ($format:literal $(,$arg:expr)* $(,)?) => {
        $crate::log!(
            gtk::glib::LogLevel::Warning,
            $format,
            $($arg),*
        )
    };
}
