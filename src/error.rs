use crate::nvim_gio;

#[derive(Debug)]
pub enum Error {
    Start(nvim_gio::Error),
    Call(Box<nvim_rs::error::CallError>),
    Cairo(gtk::cairo::Error),
    GridDoesNotExist(i64),
    FailedToCreateSurface(),
    GetPangoMetrics(),
}

impl From<nvim_gio::Error> for Error {
    fn from(arg: nvim_gio::Error) -> Self {
        Error::Start(arg)
    }
}

impl From<Box<nvim_rs::error::CallError>> for Error {
    fn from(arg: Box<nvim_rs::error::CallError>) -> Self {
        Error::Call(arg)
    }
}

impl From<gtk::cairo::Error> for Error {
    fn from(arg: gtk::cairo::Error) -> Self {
        Error::Cairo(arg)
    }
}
