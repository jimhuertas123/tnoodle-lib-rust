#!/usr/bin/env bash
# Proves that every crate is at the version of the tnoodle-lib release it mirrors,
# and that CHANGELOG.md names each public item this release removed or changed.
set -uo pipefail

repo="$(cd "$(dirname "$0")/../.." && pwd)"
version="0.20.0"
fail() { echo "FAIL: $1"; exit 1; }

for manifest in "$repo"/crates/*/Cargo.toml; do
    grep -q "^version = \"$version\"$" "$manifest" || fail "$manifest is not at $version"
done

changelog="$repo/CHANGELOG.md"
[ -f "$changelog" ] || fail "CHANGELOG.md is missing"
grep -q "^## $version$" "$changelog" || fail "CHANGELOG.md has no $version section"
for item in generate_scramble generate_seeded_scramble generate_wca_scramble \
    generate_random_moves ScrambleCacher::new SearchResult solution_with_restrictions \
    tools::random_cube tools::random_state tools::parse_facelets tools::from_scramble \
    set_with_prun N_COMB phase2 random_state FullCube::random_cube choose_from_keys; do
    grep -q "$item" "$changelog" || fail "CHANGELOG.md does not mention $item"
done
echo "versions_and_changelog: PASS"
