# LESSONS - auto-maintained by scripts/lessons.py

> Machine-owned. Do NOT hand-edit. Changes are overwritten on the next `lessons.py` write.
> Canonical state lives in `.specs/lessons.json`. Edit lessons only via the script.
> promote_threshold=2 distinct features · window_days=45 · quarantine_threshold=2

## Confirmed (load these at Plan/Checks)

Corroborated across multiple features. Safe to apply as guidance.

_none_

## Candidates (under observation - do NOT load as guidance yet)

Seen once or not yet corroborated. Tracked, not trusted.

### L-001 - When porting a Java path that throws, decide the Rust outcome explicitly and test the error input; a retry loop around a Result can turn Java's exception into a hang.
- signal: `ac_gap` · recurrence: 1 feature(s) · scope: `scrambles` · harmful: 0
- features: mirror-cubes
- evidence: C26 (scrambles)
- last seen: 2026-11-22T05:53:12Z

### L-002 - Every limit an AC names (min and max, time and probes) needs its own assertion; one budget mode's test does not cover the other.
- signal: `ac_gap` · recurrence: 1 feature(s) · scope: `min2phase` · harmful: 0
- features: mirror-cubes
- evidence: C27 (min2phase)
- last seen: 2026-11-22T05:53:12Z

### L-003 - Derive a changelog guard's item list from the public-API diff, not from names picked by hand.
- signal: `weak_test` · recurrence: 1 feature(s) · scope: `tools` · harmful: 0
- features: mirror-cubes
- evidence: C23 (tools)
- last seen: 2026-11-22T05:53:12Z

### L-004 - Pin each enumerated error code with a fixed input; random corruptions reach rare codes only by chance.
- signal: `weak_test` · recurrence: 1 feature(s) · scope: `crosscheck` · harmful: 0
- features: mirror-cubes
- evidence: C12 (crosscheck)
- last seen: 2026-11-22T05:53:12Z

## Quarantined (failed when applied - ignore)

A confirmed lesson that recurred alongside failure. Kept for the maintainer to review.

_none_
