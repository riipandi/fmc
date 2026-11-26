#!/usr/bin/env -S just --justfile
# ^ A shebang isn't required, but allows a justfile to be executed
#   like a script, with `./justfile test`, for example.

set dotenv-load

import 'scripts/task-development.just'
import 'scripts/task-quality.just'
import 'scripts/task-utilities.just'

# Cargo runs through kache (github.com/kunobi-ninja/kache) unless KACHE is false, 0, or no.
kache_off := if lowercase(env("KACHE", "true")) =~ '^(false|0|no)$' { "1" } else { "" }
# An empty RUSTC_WRAPPER also overrides `build.rustc-wrapper` from a user-level ~/.cargo/config.toml.
export RUSTC_WRAPPER := if kache_off == "1" { "" } else { "kache" }
# Makes any kache shim a user-level config still reaches (`HOST_CC = "kache cc"`) pass straight through.
export KACHE_DISABLED := if kache_off == "1" { "1" } else { env("KACHE_DISABLED", "") }

[private]
default:
    @just --list --unsorted

[private]
kache:
    @[ -z "$RUSTC_WRAPPER" ] || command -v kache >/dev/null || { echo 'error: kache is not installed. Install it with `cargo install kache --locked`, or run `KACHE=false just ...` for plain cargo.' >&2; exit 1; }
