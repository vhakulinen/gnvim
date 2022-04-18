mod gen;
mod manual;

pub mod uievents {
    pub use super::gen::*;
    pub use super::manual::*;
}

pub use uievents::UiEvent;

pub fn decode_redraw_params(params: rmpv::Value) -> Result<Vec<UiEvent>, rmpv::ext::Error> {
    match params {
        rmpv::Value::Array(params) => params
            .into_iter()
            .map(rmpv::ext::from_value::<uievents::UiEvent>)
            .collect(),
        params => Err(rmpv::ext::Error::Syntax(format!(
            "Invalid params type: {:?}",
            params
        ))),
    }
}
