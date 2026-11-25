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
