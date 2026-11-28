# Release and distribution

The fmc release, as `docs/development.md` ("Releasing") and `.github/workflows/release.yml` define it. Re-read both before a release: they win over this file.

## Version

fmc is before 1.0 (`version` in `Cargo.toml`). The bump comes from `CHANGELOG.md` `## [Unreleased]`:

| `[Unreleased]` holds | Bump | Example |
| --- | --- | --- |
| Any `### Added`, `### Changed`, `### Removed`, or `### Deprecated` entry | minor | 0.5.0 → 0.6.0 |
| Only `### Fixed` or `### Security` | patch | 0.6.0 → 0.6.1 |
| A breaking change after 1.0 | major | 1.4.2 → 2.0.0 |

The owner's `patch`, `minor`, or `major` wins. The version is `X.Y.Z`; the tag is `vX.Y.Z`.

## No release when

- No commit since the last tag touches a shipped path (`src/`, `Cargo.toml`, `Cargo.lock`, `build.rs`, `vendor/`, `rust-toolchain.toml`, `.cargo/`, `scripts/opentui-native.sh`).
- The tag `vX.Y.Z` already exists (`git ls-remote --tags origin vX.Y.Z`): the work is already released; report it.
- The owner said `no-release`.

Say which reason applied.

## Steps

1. **Branch.** From the updated default branch: `git switch -c release/vX.Y.Z`.
2. **Version.** Set `version = "X.Y.Z"` under `[package]` in `Cargo.toml` (an edit that asks for permission by design), then `cargo build` so `Cargo.lock` takes it.
3. **Changelog.** Under `## [Unreleased]`, insert `## [X.Y.Z] - YYYY-MM-DD` (today, from `date +%F`) with a one-sentence lead that names the release's main changes, and move the entries under it; leave `## [Unreleased]` empty above. At the bottom: `[Unreleased]: …/compare/vX.Y.Z...HEAD` and a new `[X.Y.Z]: …/compare/v<previous>...vX.Y.Z` line. The workflow publishes this section as the release notes, so it must read on its own.
4. **Gate and commit.** `just verify`, then commit `Cargo.toml`, `Cargo.lock`, `CHANGELOG.md` as `chore(release): X.Y.Z` with the trailer.
5. **PR and merge.** Push, open `chore(release): X.Y.Z` with a one-line body naming the PRs it releases, wait for CI, merge with `--merge --delete-branch` (SKILL.md steps 5 to 8).
6. **Tag.** `git switch <default> && git pull --ff-only`, confirm `grep -m1 '^version' Cargo.toml` shows `X.Y.Z`, then `git tag vX.Y.Z && git push origin vX.Y.Z`.
7. **Workflow.** `gh run list --workflow release --limit 1`, then `gh run watch <id> --exit-status` in the background. It checks the tag against `Cargo.toml` and the changelog section, builds, packages, and publishes the GitHub Release. A failure: read `gh run view <id> --log-failed`; never delete or move a pushed tag without asking the owner.
8. **Assets.** `gh release view vX.Y.Z --json assets` lists `fmc-X.Y.Z-aarch64-apple-darwin.tar.gz` and its `.sha256`.

## Distribution

The channels are the ones `docs/install.md` lists; check it for a channel added since this file was written.

### Release archive

Published by the workflow. Verify in a fresh `mktemp -d`:

```sh
gh release download vX.Y.Z -R riipandi/fmc -D "$dir"
cd "$dir" && shasum -a 256 -c fmc-X.Y.Z-aarch64-apple-darwin.tar.gz.sha256
tar xzf fmc-X.Y.Z-aarch64-apple-darwin.tar.gz && ./fmc-X.Y.Z-aarch64-apple-darwin/fmc --version
```

The archive holds `fmc`, `README.md`, `CHANGELOG.md`, `THIRD_PARTY_LICENSES.html`, and both licences.

### Homebrew tap

`riipandi/homebrew-tap`, file `Formula/fmc.rb`. The release does not update it (no `HOMEBREW_TAP_TOKEN` secret), so the skill does:

1. `gh repo clone riipandi/homebrew-tap "$(mktemp -d)/tap"`.
2. In `Formula/fmc.rb`, set `url` to `https://github.com/riipandi/fmc/releases/download/vX.Y.Z/fmc-X.Y.Z-aarch64-apple-darwin.tar.gz` and `sha256` to the first field of the `.sha256` asset. The formula has no `version` line; the URL carries it.
3. Commit `fmc X.Y.Z` (the tap's history uses that subject) with the trailer, and `git push origin HEAD` to the tap's default branch (the tap takes direct pushes).
4. `brew update` and `brew info riipandi/tap/fmc` shows `stable X.Y.Z`.

Upgrading the owner's own install (`brew upgrade riipandi/tap/fmc`) is the owner's step: list it under Owner check.

## Report

The version, the release URL, the assets verified, each channel's state (archive checksum and `--version`; tap commit and `brew info`), and what is left for the owner.
