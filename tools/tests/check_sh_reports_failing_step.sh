#!/usr/bin/env bash
# Proves that tools/check.sh reports a formatting difference as `fmt FAIL`,
# still runs the later steps, exits non-zero and names `fmt` in its last line.
# It runs check.sh in a throwaway worktree of HEAD with one misformatted line.
set -uo pipefail

repo="$(cd "$(dirname "$0")/../.." && pwd)"
tree="$(mktemp -d)/tree"
git -C "$repo" worktree add -q --detach "$tree" HEAD || exit 1
trap 'git -C "$repo" worktree remove --force "$tree"' EXIT

printf '\npub fn fmt_probe( ) { }\n' >> "$tree/crates/svglite/src/lib.rs"
out=$(CARGO_TARGET_DIR="$repo/target/check-sh-test" "$tree/tools/check.sh" 2>&1)
status=$?
echo "$out"

fail() { echo "FAIL: $1"; exit 1; }
[ $status -ne 0 ] || fail "exit status was 0"
grep -q '^fmt FAIL' <<<"$out" || fail "no 'fmt FAIL' line"
for step in clippy debug-tests release-tests crosscheck; do
    grep -Eq "^$step (PASS|FAIL)" <<<"$out" || fail "step $step was not reported"
done
tail -n 1 <<<"$out" | grep -q 'fmt' || fail "last line does not name fmt"
echo "check_sh_reports_failing_step: PASS"
