#!/bin/bash
set -xe

nvim --api-info | cargo run --bin apigen functions | sponge nvim-rs/src/gen.rs;
nvim --api-info | cargo run --bin apigen uievents | sponge nvim-rs/src/types/gen.rs

cargo fmt
