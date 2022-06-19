pub static LOG_DOMAIN: &str = "gnvim";

#[macro_export]
macro_rules! spawn_local {
    ($body:expr) => {
        glib::MainContext::default().spawn_local($body)
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
    ($format:literal $(,$arg:expr)* $(,)?) => {
        $crate::log!(
            gtk::glib::LogLevel::Warning,
            $format,
            $($arg),*
        )
    };
}
