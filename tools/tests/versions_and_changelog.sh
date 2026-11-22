#!/usr/bin/env bash
# Proves that every crate is at the version of the tnoodle-lib release it mirrors, that
# CHANGELOG.md names each public item this release removed or changed (read from the diff
# against main), and that every entry names its Java counterpart in parentheses.
set -uo pipefail

repo="$(cd "$(dirname "$0")/../.." && pwd)"
version="0.20.0"
fail() { echo "FAIL: $1"; exit 1; }
cd "$repo" || fail "no repo"

for manifest in crates/*/Cargo.toml; do
    grep -q "^version = \"$version\"$" "$manifest" || fail "$manifest is not at $version"
done

changelog="CHANGELOG.md"
[ -f "$changelog" ] || fail "CHANGELOG.md is missing"
grep -q "^## $version$" "$changelog" || fail "CHANGELOG.md has no $version section"

# Every bullet of the release section names a Java counterpart: a backticked name in parentheses.
missing_java=$(perl -0ne '
    /^## '"$version"'\n(.*?)(?=^## |\z)/ms;
    for my $bullet (split /\n(?=- )/, $1) {
        next unless $bullet =~ /^- /;
        (my $flat = $bullet) =~ s/\s+/ /g;
        print "$flat\n" unless $flat =~ /\([^()]*`[^`]+`[^()]*\)/;
    }
' "$changelog")
[ -z "$missing_java" ] || fail "entry names no Java counterpart: $missing_java"

# A public signature removed from a source file and found nowhere in the tree after
# normalising whitespace and digit separators was removed or changed: CHANGELOG names it.
normalise() { perl -0777 -pe 's/\s+//g; s/\b(0x[0-9A-Fa-f_]+|\d[\d_]*)/($1 =~ s#_##gr)/ge; s/,\)/)/g'; }
base="$(git merge-base main HEAD)" || fail "no merge base with main"
tree="$(cat crates/*/src/*.rs crates/*/src/*/*.rs 2>/dev/null | normalise)"
status=0
while IFS= read -r line; do
    sig="$(printf '%s' "${line#-}" | normalise)"
    case "$tree" in *"$sig"*) continue ;; esac
    name="$(printf '%s' "$line" | perl -ne 'print $2 if /pub (fn|struct|enum|const|static|type|trait|mod) (\w+)/')"
    [ -n "$name" ] || continue
    if ! grep -q "$name" "$changelog"; then
        echo "FAIL: CHANGELOG.md does not mention changed public item $name (${line#-})"
        status=1
    fi
done < <(git diff "$base" -- crates | perl -ne 'print if /^-\s*pub (fn|struct|enum|const|static|type|trait|mod) /')
[ "$status" -eq 0 ] || exit 1
echo "versions_and_changelog: PASS"
