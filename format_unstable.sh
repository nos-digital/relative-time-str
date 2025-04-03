#!/usr/bin/env bash
cargo fmt -- --config="$(cat rustfmt-unstable.toml | tr '\n' ',' | sed 's/ //g; s/"//g; s/,$//')"
