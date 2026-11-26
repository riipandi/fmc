[doc('Format all sources')]
[group('quality')]
format:
    @cargo fmt --all
    @if command -v rumdl >/dev/null; then rumdl fmt --silent .; fi

[doc('Check formatting')]
[group('quality')]
format-check:
    @cargo fmt --all --check
    @if command -v rumdl >/dev/null; then rumdl fmt --check --quiet .; fi

[doc('Type-check all targets')]
[group('quality')]
check *args: kache native
    @cargo check --all-targets {{ args }}

[doc('Lint with clippy')]
[group('quality')]
lint *args: kache native
    @cargo clippy --all-targets {{ args }} -- -D warnings

[doc('Run the tests')]
[group('quality')]
test *args: kache native
    @if command -v cargo-nextest >/dev/null; then cargo nextest run --no-tests=pass {{ args }}; else cargo test {{ args }}; fi

[doc('Check dependency licenses against the allowlist')]
[group('quality')]
licenses:
    @cargo deny check licenses

[doc('Write third-party notices to target/THIRD_PARTY_LICENSES.html')]
[group('quality')]
notices:
    @mkdir -p target
    @cargo about generate about.hbs -o target/THIRD_PARTY_LICENSES.html

[doc('Run all quality gates')]
[group('quality')]
verify: format-check check lint test licenses
