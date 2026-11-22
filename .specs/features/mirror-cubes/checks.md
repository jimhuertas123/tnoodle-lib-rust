# Mirror cubes checks

Build: `cargo build --workspace --all-targets`; `cargo clippy --workspace --all-targets -q`
Run rel: `cargo test --workspace --release`
Run dbg: `cargo test --workspace`
Run xcheck: `tools/java-crosscheck/run.sh 1 1`
Plan: `.specs/features/mirror-cubes/plan.md`

25 checks in 4 slices · 6 one-way doors · 0 open, of which 0 block

## Checks

### S1 - Local gate pinned to v0.20.0

**C1** - `tools/check.sh` runs fmt, clippy, debug tests, release tests and the Java cross-check in that order, prints one `PASS`/`FAIL` line per step, and exits `0` when all pass (AC 1)
Proof: `tools/check.sh`

**C2** - With a formatting difference only, `tools/check.sh` prints `fmt FAIL`, still reports the later steps, exits non-zero and names `fmt` in its last line (AC 1, AC 2)
Proof: `tools/tests/check_sh_reports_failing_step.sh`

**C3** - The cross-check builds tnoodle-lib `v0.20.0` (`d5ddbe6`) by default and prints `java: tnoodle-lib v0.20.0` first (AC 3)
Proof: run xcheck | java: tnoodle-lib v0.20.0

**C4** - A cross-check difference not listed in `PARITY.md` exits non-zero; a listed one is reported `expected` and does not fail (AC 4)
Proof: `tools/java-crosscheck/selftest.sh`

### S2 - Java-identical randomness

**C5** - `JavaRandom::from_seed` gives Java `SHA1PRNG` `nextInt(bound)` values, 1,000 per bound for bounds 1, 2, 3, 12, 40320, 479001600, 2^31-1, for seeds `""`, `"abc"`, a 1,000-byte seed and `"ñandú"` (AC 5)
Proof: run xcheck | javarandom-next-int-bounded OK

**C6** - `nextInt()`, `nextLong()`, `nextBoolean()`, `nextDouble()` and 37-byte `nextBytes`, 1,000 each per seed, equal Java's (AC 6)
Proof: run xcheck | javarandom-other-methods OK

**C7** - `next_int_bounded` with bound `0` or `-5` returns `Err(InvalidBound)` and does not panic (AC 7)
Proof: run rel | java_random::tests::non_positive_bound_is_an_error ... ok

**C8** - Two `JavaRandom::from_entropy()` generators give different first `next_long()` values (AC 8)
Proof: run rel | java_random::tests::entropy_generators_differ ... ok

### S3 - min2phase is tnoodle's solver

**C9** - `Search::solution` equals Java's text byte for byte on 1,000 random states for verbose `0`, `INVERSE_SOLUTION`, `USE_SEPARATOR`, `APPEND_LENGTH`, `USE_SEPARATOR|INVERSE_SOLUTION|APPEND_LENGTH`, with probes `(100000, 0)` and `(100000, 2000)`, and `OPTIMAL_SOLUTION` at maxDepth 13 on states 13 moves from solved (AC 9)
Proof: run xcheck | min2phase-search OK
Proof: run xcheck | min2phase-search-optimal OK

**C10** - `SearchWca::solution` with `probeMin = 0` equals Java's `SearchWCA` for restrictions (`R`,`F`), (`U`,`B`), (`L`,`L`), (`F`,`D`), (none,`F`), (`R`,none), including Java's U/D last-axis check (AC 10)
Proof: run xcheck | min2phase-search-wca OK

**C11** - After a solution, `Search::next` returns Java's next text, and `number_of_probes()` and `length()` equal Java's (AC 11)
Proof: run xcheck | min2phase-next OK

**C12** - Invalid facelets return `SearchError` with Java's code: 53 characters → 1, a flipped edge → 3, a twisted corner → 5, odd parity → 6, and Java's code for 200 corrupted facelet strings (AC 12)
Proof: run xcheck | min2phase-errors OK
Proof: run rel | search::tests::invalid_facelets_return_java_codes ... ok

**C13** - No solution within `max_depth` returns code 7, an exhausted probe maximum returns code 8, and a restriction that is not a face move returns code 9 (AC 13)
Proof: run rel | search::tests::exhausted_searches_return_java_codes ... ok

**C14** - With `SearchBudget::Time { min: 200 ms, .. }` a 333 scramble takes at least 200 ms; with `Probes { min: 0, max: 60000 }` the search returns its first solution (AC 14)
Proof: run rel | three_by_three::tests::time_budget_searches_at_least_min ... ok
Proof: run rel | three_by_three::tests::probe_budget_zero_returns_first_solution ... ok

**C15** - `randomCube`, `randomLastLayer`, `randomLastSlot`, `randomZBLastLayer`, `randomCornerOfLastLayer`, `randomEdgeOfLastLayer`, `randomCrossSolved`, `randomEdgeSolved`, `randomCornerSolved` and `superFlip` give Java's facelets from the same seeded generator, 500 calls each (AC 15)
Proof: run xcheck | min2phase-tools OK

**C16** - `tools::from_scramble` gives Java's facelets for canonical scrambles, and `Err(InvalidMove)` for `R2'`, `R3`, `Rw`, `RU` (AC 16)
Proof: run xcheck | min2phase-from-scramble OK
Proof: run rel | tools::tests::invalid_tokens_are_errors ... ok

**C17** - Over 300 unseeded 333 scrambles with default options the average length is between 18.0 and 19.2 moves (AC 17)
Proof: run rel | three_by_three::tests::average_length_matches_tnoodle ... ok

### S4 - Seeded scrambles equal tnoodle's

**C18** - For seeds `"a"`, `"tnoodle"`, `"2026-11-21 round 1"` and `""`, 20 seeded scrambles each are byte-identical to Java for `222`, `444`, `444fast`, `555`, `666`, `777`, `444ni`, `555ni`, `minx`, `clock`, `sq1` (AC 18, AC 23)
Proof: run xcheck | seeded-identical OK

**C19** - Seeded `333`, `333ni`, `333fm` equal Java's scramble from the same seeded state with `SearchWCA` at `probeMin = 0` (AC 19)
Proof: run xcheck | seeded-three-family OK

**C20** - Seeded `skewb` and `pyram` reach Java's state for the seed, have Java's length, and repeat across two Rust runs (AC 20)
Proof: run xcheck | seeded-skewb-pyram OK

**C21** - Calling `generate_seeded_scramble(type, seed)` twice returns the same string, for all 16 event types (AC 21)
Proof: run rel | tests::seeded_scrambles_repeat_for_every_event ... ok

**C22** - 2,000 unseeded scrambles per event pass Java's judge (valid, not within the minimum distance, canonical for cube events), and the randomness test passes (AC 22)
Proof: `JUDGE_COUNT=2000 tools/java-crosscheck/run.sh 1 0`
Proof: `cargo test --release -p tnoodle-scrambles --test randomness -- --ignored`

**C23** - Every crate is at version `0.20.0`, and `CHANGELOG.md` lists every removed or changed public item with its Java counterpart (AC 24)
Proof: `tools/tests/versions_and_changelog.sh`

**C24** - Seeded scrambles from 8 threads at once equal the same seeds run one after another (sweep: concurrency)
Proof: run rel | tests::concurrent_seeded_scrambles_match_sequential ... ok

**C25** - The debug build passes the same tests, with no overflow panics (AC 1, AD-004)
Proof: run dbg | test result: ok

## Coverage

| Set (size) | Member -> proof | Unproven |
| --- | --- | --- |
| check.sh steps (5) | fmt C1 C2 · clippy C1 · debug tests C1 · release tests C1 · cross-check C1 | - |
| `nextInt(bound)` bounds (7) | 1 C5 · 2 C5 · 3 C5 · 12 C5 · 40320 C5 · 479001600 C5 · 2^31-1 C5 | - |
| seeds for JavaRandom (4) | `""` C5 · `"abc"` C5 · 1,000-byte C5 · `"ñandú"` C5 | - |
| JavaRandom methods (6) | nextInt(bound) C5 · nextInt C6 · nextLong C6 · nextBoolean C6 · nextDouble C6 · nextBytes C6 | - |
| verbose flags (4) | USE_SEPARATOR C9 · INVERSE_SOLUTION C9 · APPEND_LENGTH C9 · OPTIMAL_SOLUTION C9 | - |
| WCA restriction pairs (6) | R,F C10 · U,B C10 · L,L C10 · F,D C10 · none,F C10 · R,none C10 | - |
| Java error codes (9) | 1 C12 · 2 C12 · 3 C12 · 4 C12 · 5 C12 · 6 C12 · 7 C13 · 8 C13 · 9 C13 | - |
| `Tools` generators (10) | randomCube C15 · randomLastLayer C15 · randomLastSlot C15 · randomZBLastLayer C15 · randomCornerOfLastLayer C15 · randomEdgeOfLastLayer C15 · randomCrossSolved C15 · randomEdgeSolved C15 · randomCornerSolved C15 · superFlip C15 | - |
| invalid `from_scramble` tokens (4) | `R2'` C16 · `R3` C16 · `Rw` C16 · `RU` C16 | - |
| events byte-identical seeded (11) | 222 C18 · 444 C18 · 444fast C18 · 555 C18 · 666 C18 · 777 C18 · 444ni C18 · 555ni C18 · minx C18 · clock C18 · sq1 C18 | - |
| events with a PARITY exception (5) | 333 C19 · 333ni C19 · 333fm C19 · skewb C20 · pyram C20 | - |
| seeds for scrambles (4) | `"a"` C18 · `"tnoodle"` C18 · `"2026-11-21 round 1"` C18 · `""` C18 | - |
| all events, seeded repeat (16) | C21, table-driven over all 16 | - |
| all events, unseeded judge (16) | C22, table-driven over all 16 | - |
| budget modes (2) | Time C14 · Probes C14 | - |
| Landing doors (6) | randomness API C18 C21 · JavaRandom C5 C6 · sha1 C5 · min2phase API C9 C12 C13 · PARITY.md C4 · versions C23 | - |

## Swept

- validation: C7, C12, C16
- failure modes: C13, C2
- idempotency: C21
- authorization: n/a - a library with no callers to tell apart; every function is public by design
- concurrency: C24
- data lifecycle: n/a - nothing is stored
- dependency failure: n/a - the only external dependency is OS entropy in `from_entropy`, whose failure Java also cannot recover from; seeded generation never touches it
- state transitions: C11
- observability: n/a - the library has no logging or metrics requirement, as Java

## Handoff

- **Settled mid-build:** user: "nono if you have slow tests then run them, take your time just be transparent if they will take long say it" - run slow proofs during the build and give a time estimate first
- **Dispatched:** none
- **Boundary:** S1 (C1-C4) closed at `d712712`; S2 (C5-C8) at `68633c7`; S3 C9-C13, C15, C16 at `cc9b839` - C14 and C17 wait for S4's `ScrambleOptions`. Workspace formatted once at `1e54ac0` so `cargo fmt --check` can gate.
- **Found mid-build:** the old port built min2phase as Java with `USE_TWIST_FLIP_PRUN`, `USE_COMBP_PRUN` and `USE_CONJ_PRUN` off; tnoodle has them on. `CoordCube` was re-ported with Java's packed table flags, and `Perm2CombP` was missing the `parity * 70` term; the cross-check now hashes all 20 min2phase tables against Java.
- **Abandoned:** none
