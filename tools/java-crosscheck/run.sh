#!/usr/bin/env bash
# Differential test: run the same inputs through this Rust port and through
# the original Java tnoodle-lib, and report every answer that differs.
#
#   tools/java-crosscheck/run.sh [seed] [scale]
#
# Needs a JDK (17+) and git. TNOODLE_LIB may point at an existing checkout;
# otherwise the pinned commit below is cloned into the cache directory.
set -euo pipefail

SEED="${1:-1}"
SCALE="${2:-1}"
TNOODLE_COMMIT=d01a947d9f028f38085cda9b0507a9cf3d3f38a8
SLF4J_VERSION=2.0.17

here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
cache="${XDG_CACHE_HOME:-$HOME/.cache}/tnoodle-crosscheck"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
mkdir -p "$cache"

lib="${TNOODLE_LIB:-$cache/tnoodle-lib}"
if [ ! -d "$lib/.git" ]; then
    git clone -q https://github.com/thewca/tnoodle-lib "$lib"
fi
if [ -z "${TNOODLE_LIB:-}" ]; then
    git -C "$lib" fetch -q origin "$TNOODLE_COMMIT" 2>/dev/null || true
    git -C "$lib" checkout -q "$TNOODLE_COMMIT"
fi
echo "java: tnoodle-lib $(git -C "$lib" rev-parse --short HEAD)"

slf4j="$HOME/.m2/repository/org/slf4j/slf4j-api/$SLF4J_VERSION/slf4j-api-$SLF4J_VERSION.jar"
if [ ! -f "$slf4j" ]; then
    slf4j="$cache/slf4j-api-$SLF4J_VERSION.jar"
    [ -f "$slf4j" ] || curl -sSfL -o "$slf4j" \
        "https://repo1.maven.org/maven2/org/slf4j/slf4j-api/$SLF4J_VERSION/slf4j-api-$SLF4J_VERSION.jar"
fi

find "$lib"/{scrambles,min2phase,threephase,sq12phase,svglite,fto3phase}/src/main/java \
    "$here/stubs" "$here/src" -name '*.java' > "$work/sources.txt"
javac -nowarn -encoding UTF-8 -cp "$slf4j" -d "$work/classes" @"$work/sources.txt" 2>&1 \
    | grep -v '^Note:' || true

echo "rust: $(git -C "$repo" rev-parse --short HEAD)$(git -C "$repo" diff --quiet || echo ' + uncommitted changes')"
cargo run -q --release --manifest-path "$repo/Cargo.toml" -p tnoodle-scrambles \
    --example crosscheck -- "$SEED" "$SCALE" > "$work/rust_cases.tsv"
echo "cases: $(wc -l < "$work/rust_cases.tsv" | tr -d ' ') (seed $SEED, scale $SCALE)"
echo

java -cp "$work/classes:$slf4j" CrossCheck "$work/rust_cases.tsv" 2> >(grep -v '^SLF4J' >&2)
