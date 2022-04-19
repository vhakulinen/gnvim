#[macro_export]
macro_rules! nvim_unlock {
    ($nvim:ident) => {
        $nvim.lock().await.as_mut().expect("nvim not set")
    };
}

#[macro_export]
macro_rules! spawn_local {
    ($body:expr) => {
        glib::MainContext::default().spawn_local($body)
    };
}
