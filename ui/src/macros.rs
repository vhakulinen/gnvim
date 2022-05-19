#[macro_export]
macro_rules! spawn_local {
    ($body:expr) => {
        glib::MainContext::default().spawn_local($body)
    };
}
