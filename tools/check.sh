#!/usr/bin/env bash
# The local gate before every push (no CI, see .specs/STATE.md AD-007):
# formatting, clippy, debug and release tests, and the Java cross-check.
# Every step runs even after a failure; the exit status is 0 only when all pass.
set -uo pipefail

repo="$(cd "$(dirname "$0")/.." && pwd)"
cd "$repo" || exit 1

steps=(fmt clippy debug-tests release-tests crosscheck)
failed=()

run_step() {
    local name="$1"
    shift
    if "$@" > "$repo/target/check-$name.log" 2>&1; then
        echo "$name PASS"
    else
        echo "$name FAIL (log: target/check-$name.log)"
        failed+=("$name")
    fi
}

mkdir -p "$repo/target"
for step in "${steps[@]}"; do
    case "$step" in
        fmt) run_step fmt cargo fmt --check ;;
        clippy) run_step clippy cargo clippy --workspace --all-targets -q ;;
        debug-tests) run_step debug-tests cargo test --workspace ;;
        release-tests) run_step release-tests cargo test --workspace --release ;;
        crosscheck) run_step crosscheck tools/java-crosscheck/run.sh ;;
    esac
done

if [ ${#failed[@]} -eq 0 ]; then
    echo "check.sh: PASS"
else
    echo "check.sh: FAIL (${failed[*]})"
    exit 1
fi
