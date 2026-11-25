#!/bin/sh
# Builds the OpenTUI native library without video (no FFmpeg, so no LGPL code in fmc's binary)
# and links it at .cache/opentui-native/<version>, where .cargo/config.toml points OPENTUI_LIB_DIR.
# The build is cached per version in ~/.cache/fmc/opentui-native/<version>; a second run is a no-op.
# Usage: scripts/opentui-native.sh [version]   (default: the version .cargo/config.toml expects)
set -eu

VERSION=${1:-0.9.9}
ZIG_VERSION=0.17.1
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
CACHE="${XDG_CACHE_HOME:-$HOME/.cache}/fmc/opentui-native/$VERSION"
LINK="$ROOT/.cache/opentui-native/$VERSION"

link() {
    mkdir -p "$(dirname "$LINK")"
    ln -sfn "$CACHE" "$LINK"
}

if [ -f "$CACHE/libopentui.a" ] && [ -f "$CACHE/opentui.h" ]; then
    link
    exit 0
fi

case "$(uname -s)-$(uname -m)" in
    Darwin-arm64) ;;
    *) echo "error: only macOS aarch64 is supported for now" >&2; exit 1 ;;
esac

command -v zig >/dev/null || { echo "error: zig $ZIG_VERSION is required (brew install zig)" >&2; exit 1; }
found=$(zig version)
[ "$found" = "$ZIG_VERSION" ] || { echo "error: zig $ZIG_VERSION is required, found $found" >&2; exit 1; }

WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT

echo "Fetching OpenTUI v$VERSION source"
curl -fsSL "https://codeload.github.com/anomalyco/opentui/tar.gz/refs/tags/v$VERSION" \
    | tar -xz -C "$WORK" --strip-components=1 "opentui-$VERSION/packages/native"

NATIVE="$WORK/packages/native"
sh "$NATIVE/scripts/prepare-zig-deps.sh"

echo "Building libopentui (video off); this takes a few minutes"
(cd "$NATIVE" && zig build -Doptimize=ReleaseFast -Dvideo=false)

OUT="$NATIVE/lib/aarch64-macos"
[ -f "$OUT/libopentui.a" ] || { echo "error: $OUT/libopentui.a was not built" >&2; exit 1; }

# The license exception in deny.toml holds only while the archive has no FFmpeg code.
if nm "$OUT/libopentui.a" 2>/dev/null | grep -qE ' [TtDd] _?(avcodec|avformat|swscale|swresample)_'; then
    echo "error: libopentui.a contains FFmpeg symbols" >&2
    exit 1
fi

mkdir -p "$CACHE.tmp"
# Only the static archive and the header: without a dylib the build cannot fall back to dynamic linking.
cp "$OUT/libopentui.a" "$OUT/opentui.h" "$CACHE.tmp/"
rm -rf "$CACHE"
mv "$CACHE.tmp" "$CACHE"
link
echo "Installed $CACHE"
