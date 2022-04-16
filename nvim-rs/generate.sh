#!/bin/bash
set -xe

nvim --api-info | cargo run --bin apigen functions > src/gen.rs;
nvim --api-info | cargo run --bin apigen uievents > src/types/gen.rs

cargo fmt
