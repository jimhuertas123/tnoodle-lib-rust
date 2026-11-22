#!/usr/bin/env bash
# Proves that the cross-check fails on a difference PARITY.md does not list,
# and reports a listed one as `expected` without failing. The fixture case
# canonicalizes "R R" and claims Rust answered "R R"; Java answers "R2".
set -uo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
cases="$here/selftest/wrong_canon.tsv"

out=$(CROSSCHECK_CASES="$cases" PARITY_FILE="$here/selftest/parity_empty.md" "$here/run.sh" 2>&1)
status=$?
if [ $status -eq 0 ] || ! grep -Eq '^canon +1 +1  FAIL$' <<<"$out"; then
    echo "FAIL: an unlisted difference must fail (exit $status)"; echo "$out"; exit 1
fi

out=$(CROSSCHECK_CASES="$cases" PARITY_FILE="$here/selftest/parity_lists_canon.md" "$here/run.sh" 2>&1)
status=$?
if [ $status -ne 0 ] || ! grep -Eq '^canon +1 +1  expected$' <<<"$out"; then
    echo "FAIL: a listed difference must be expected and pass (exit $status)"; echo "$out"; exit 1
fi

echo "selftest: PASS"
