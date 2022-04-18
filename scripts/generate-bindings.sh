#!/bin/bash
set -xe

nvim --api-info | cargo run --bin apigen functions | sponge lib/nvim-rs/src/gen.rs
nvim --api-info | cargo run --bin apigen uievents | sponge lib/nvim-rs/src/types/gen.rs

cargo fmt
