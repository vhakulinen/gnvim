pub mod client;
mod gen;
pub mod rpc;
pub mod types;

pub use client::Client;
pub use gen::Neovim as NeovimApi;

pub use async_trait;
// NOTE(ville): re-export serde.
pub use serde;
