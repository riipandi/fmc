[private]
app_bin := "fmc"

[doc('Build the OpenTUI native library without video (cached per version)')]
[group('development')]
native *args:
    @scripts/opentui-native.sh {{ args }}

[doc('Run fmc, e.g. `just run ~/notes`')]
[group('development')]
[no-exit-message]
[positional-arguments]
run *args: native
    @cargo run -q -- "$@"

[doc('Build the release binary')]
[group('development')]
build *args: native
    @cargo build --release {{ args }}
    @ls -lh target/release/{{ app_bin }} | awk '{printf "{{ BOLD + GREEN }}    %s{{ NORMAL }} %s (%s)\n", "Output", $5, $NF}'

[doc('Run the release binary')]
[group('development')]
[no-exit-message]
[positional-arguments]
start *args:
    @target/release/{{ app_bin }} "$@"

[doc('Create the demo git repository, default to tmp/demo-repo')]
[group('development')]
demo dir="tmp/demo-repo":
    @scripts/demo-repo.sh {{ dir }}

[doc('Update Cargo dependencies')]
[group('development')]
deps *args:
    @cargo update {{ args }}

[doc('Install fmc to ~/.local/bin: `just install` (release -> fmc) or `just install debug` (-> fmc-debug)')]
[group('development')]
install profile="release": native
    #!/usr/bin/env bash
    set -euo pipefail
    case "{{ profile }}" in
        release) cargo build --release; src=target/release/{{ app_bin }}; name={{ app_bin }} ;;
        debug) cargo build; src=target/debug/{{ app_bin }}; name={{ app_bin }}-debug ;;
        *) echo "unknown profile: {{ profile }} (expected release or debug)" >&2; exit 2 ;;
    esac
    dir="${INSTALL_DIR:-$HOME/.local/bin}"
    mkdir -p "$dir"
    # Remove first: overwriting a signed binary in place makes macOS kill the next launch.
    rm -f "$dir/$name"
    cp "$src" "$dir/$name"
    printf '{{ BOLD + GREEN }}%12s{{ NORMAL }} %s [%s]\n' "Installed" "$dir/$name" "{{ profile }}"
