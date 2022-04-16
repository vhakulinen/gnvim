#!/bin/bash
set -xe

nvim --api-info | cargo run --bin apigen functions > nvim-rs/src/gen.rs;
nvim --api-info | cargo run --bin apigen uievents > nvim-rs/src/types/gen.rs

cargo fmt
