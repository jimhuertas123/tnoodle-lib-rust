# Mirror cubes verification

**Verdict**: FAIL
**Proofs**: `proofs.md` at 70f972c
**Verifier**: independent sub-agent (author != verifier)

## Checks

| Check | Test | Holds |
| --- | --- | --- |
| C1 | `tools/check.sh` (five `run_step` calls in order, exit 1 when `failed` is non-empty) | yes |
| C2 | `tools/tests/check_sh_reports_failing_step.sh` (misformats a line in a worktree, asserts `fmt FAIL`, the 4 later steps reported, non-zero exit, `fmt` in the last line) | yes |
| C3 | xcheck header in `tools/java-crosscheck/run.sh` (`git describe --tags --exact-match` on the checkout pinned to `b7f9bfe`, the commit tag `v0.20.0` points to; `d5ddbe6` is the annotated tag object) | yes |
| C4 | `tools/java-crosscheck/selftest.sh` (`wrong_canon.tsv` against an empty and a listing PARITY file; asserts `FAIL` plus non-zero exit, then `expected` plus exit 0) | yes |
| C5 | xcheck case `jr-bounded` (`print_java_random_cases` x `CrossCheck` `jr-bounded`): 4 seeds x 7 bounds x 1,000 draws, full string compared | yes |
| C6 | xcheck case `jr-other`: 4 seeds x 5 methods x 1,000, doubles as raw bits, bytes as hex | yes |
| C7 | `java_random::tests::non_positive_bound_is_an_error` (asserts `Err(InvalidBound(0))` and `Err(InvalidBound(-5))`) | yes |
| C8 | `java_random::tests::entropy_generators_differ` | yes |
| C9 | xcheck cases `m2p-search` (1,000 states x verbose 0, 2, 1, 4, 7 at `(100000, 0)`, plus verbose 0 at `(100000, 2000)`) and `m2p-opt` (50 states 13 moves from solved, maxDepth 13, verbose 8); Java text compared with `equals` | yes |
| C10 | xcheck case `m2p-wca`: 100 states x the 6 named restriction pairs, Rust `Time{0, 60 s}` against Java `SearchWCA.solution(f, 21, 60000, 0, INVERSE_SOLUTION, first, last)` | yes |
| C11 | xcheck case `m2p-next`: solution then 3 x `next`, each followed by `numberOfProbes/length`, as one string | yes |
| C12 | xcheck case `m2p-error` (200 strings, every 10th truncated, the rest with two random stickers swapped) and `search::tests::invalid_facelets_return_java_codes` (1, 3, 5, 6) | yes - for the claim as worded; codes 2 and 4 depend on the random swaps (see Findings) |
| C13 | `search::tests::exhausted_searches_return_java_codes` (7 at depth 3, 8 at probe max 0, 9 for `X` and `Rw`) | yes |
| C14 | `three_by_three::tests::time_budget_searches_at_least_min` (3 scrambles, each >= 200 ms) and `three_by_three::tests::probe_budget_zero_returns_first_solution` (20 states, `Probes{0, 60000}` equals `Time{0, 60 s}`, which C10 ties to Java) | yes for C14 as worded; AC 14's "code 8 when `max` elapses" has no test (see Findings) |
| C15 | xcheck case `m2p-tools`: 10 generators x 500 calls, same `SHA1PRNG` seed `tools-<name>`, Java `Tools.setRandomSource(r)` | yes |
| C16 | xcheck case `m2p-from-scramble` (200 canonical 25-move scrambles) and `tools::tests::invalid_tokens_are_errors` (`R2'`, `R3`, `Rw`, `RU`, `x` → `Err(InvalidMove(token))`) | yes |
| C17 | `three_by_three::tests::average_length_matches_tnoodle` (300 scrambles, default budget, `18.0..=19.2`) | yes - draws from the fixed seed `average length`, not unseeded, but it is the same 300-sample average |
| C18 | xcheck case `seeded` → `seeded-identical`: Rust `generate_seeded_scrambles(type, seed, 20)` against Java `Puzzle.generateSeededScrambles(seed, 20)`, `Arrays.equals` on all 20, 11 events x 4 seeds, `\n` restored for minx | yes |
| C19 | xcheck case `seeded` → `seeded-three-family`, against `SeededThreeByThree.java`; read beside tnoodle's `ThreeByThreeCubePuzzle`, `NoInspectionThreeByThreeCubePuzzle`, `ThreeByThreeCubeFewestMovesPuzzle` and `Puzzle.generateWcaScramble`: a faithful copy with only `THREE_BY_THREE_TIMEMIN` set to 0 | yes |
| C20 | xcheck case `seeded` → `seeded-skewb-pyram`: two Rust runs equal, then per scramble Java's move count and Java's solver coordinates | yes |
| C21 | `tests::seeded_scrambles_repeat_for_every_event` (all of `PuzzleType::ALL`, seed `repeat`) | yes |
| C22 | `JUDGE_COUNT=2000 run.sh 1 0` → xcheck `scramble` case (`scramble-valid`, `scramble-not-too-close` via `solveIn(min - 1) == null`, `scramble-canonical` with the NI suffix stripped) and `randomness` test `three_by_three_states_are_uniformly_random` | yes |
| C23 | `tools/tests/versions_and_changelog.sh` | no - greps the changelog for a hand-picked list of 17 names; it never checks that an entry names its Java counterpart, or that the list is every changed item (see Findings) |
| C24 | `tests::concurrent_seeded_scrambles_match_sequential` (8 threads x 6 events) | yes |
| C25 | `cargo test --workspace` (debug) `test result: ok` | yes |
| C26 | `three_by_three::tests::exhausted_budget_returns_first_solution_without_extra_draws` (333, 333ni, 333fm x 5 at `Time{0,0}`) | yes - asserts < 5 s, equality with a same-seed first-solution run, and the next `next_long()`; the 70f972c loop never returns, so no `ok` line (Verifier round 2, transcribed by the builder) |
| C27 | `search::tests::exhausted_searches_return_java_codes` (`SearchWca` with `Time{min: 0, max: 0}`) | yes - asserts exactly `Err(8)` at the `SearchWca` layer (Verifier round 2, transcribed by the builder) |

PARITY.md lists only `svgrot` in its `Check` column, and no contract check above prints under that name, so no `expected` entry can hide a mismatch in C3-C22.

## Own examples

- **AC 5/6 (JavaRandom)**, values nobody named, run at 70f972c: seeds `"x"`, bytes `ff 00 80`, and 55-, 56- and 64-byte seeds (the SHA-1 padding boundaries), bounds 5, 7, 2^30, 2^30+1, 65536, 1,000,000,007, 1,000 draws each, through the harness (`CROSSCHECK_CASES`). Result: 30/30 OK. Also one generator interleaving `nextBoolean`, `nextInt(7)`, `nextBytes(0..22)`, `nextLong`, `nextDouble`, `nextInt` 300 times (exercises the `remainder` buffer across mixed widths) against a JDK-only Java program: byte-identical.
- **AC 18/19/20 (seeded scrambles)**, seeds nobody named: `"ñandú"` (non-ASCII; this JDK 17 has `file.encoding=UTF-8`), `"0"` and a 200-character seed, all 16 events x 20 scrambles. Result: `seeded-identical` 33/33, `seeded-three-family` 9/9, `seeded-skewb-pyram` 6/6 OK.
- **AC 12 (error codes)**: 14 hand-made facelet strings (two centers swapped, edge/corner sticker swaps, 55 characters, empty, an unknown colour `X`, solved). All 14 match Java. Two centers swapped gives `Error 2` on both sides.
- **AC 14 (Time budget max)**: `SearchWca` with `Time{min 0, max 0}` returns `Err(8)` (correct). But `generate_scramble_with(Three, &ScrambleOptions{ search_budget: Time{0, 0} })` was still running after 15 s and never returns (see Findings).

Throwaway examples were deleted; the tree is as before.

## Findings

| Finding | Evidence | Fix | Closed |
| --- | --- | --- | --- |
| A 3x3 scramble with a budget too small to find a solution hangs forever, where Java throws. The plan's Surface says `generate_scramble_with` / `generate_wca_scramble` "always return scrambles" | `three_by_three.rs` `generate_random_moves`: `loop { … if let Ok(scramble) = result { break scramble; } }` draws a fresh `random_cube` after every `SearchError`. `generate_scramble_with(PuzzleType::Three, Time{min: 0, max: 0})` was still running after 15 s. Java's `ThreeByThreeCubePuzzle.generateRandomMoves` gets `"Error 8"`, and `appendAlgorithm` throws `RuntimeException`. The retry also draws extra randomness Java never draws, and PARITY.md does not list it | either surface the error (as Java does, or as a `Result`/panic) and list it in PARITY.md, or reject budgets whose `max` cannot find a solution; add a test for `generate_scramble_with` with `max = 0` | `4f39580` - searches the same state to its first solution (PARITY.md row); proven by C26 |
| AC 14's second half ("return code 8 when `max` elapses without one") has no test | C14's tests cover only the 200 ms minimum and `Probes{0,…}` against `Time{0,…}`. C13's code-8 test uses `Search` with probe max 0, not `SearchBudget::Time`. A manual run shows `Time{0, 0}` → `Err(8)`, so the behaviour holds today but nothing guards it | add an assertion that `SearchWca::solution` with `Time{min: 0, max: 0}` returns code 8 | `4f39580` - `exhausted_searches_return_java_codes` asserts `Time{0,0}` gives code 8; proven by C27 |
| C23's test does not hold "with the Java member it now mirrors", nor "every removed or changed item" | `versions_and_changelog.sh` only runs `grep -q "$item"` over 17 names its author chose. An entry with no Java counterpart, or a changed item missing from the list (for example `threephase::search::Search::random_state` or `sq12phase` `FullCube::random_cube` are grepped only as `random_state`/`FullCube::random_cube`, which other lines also match), passes. The CHANGELOG as written does name Java members, so this is about the guard, not the content | check every line under `### Changed (breaking)` / `### Added` for a backticked Java member, and build the item list from the public API diff instead of by hand | `4f39580` - the guard reads changed public items from the diff against main and requires a parenthesised Java name per entry; mutation-checked (old CHANGELOG and a dropped `new_random` both fail); proven by C23 |
| The coverage table says C12 proves Java codes 2 and 4, but no case names them | `m2p-error` makes its 200 cases by random two-sticker swaps, so whether code 2 or code 4 comes up depends on the draws. `invalid_facelets_return_java_codes` asserts only 1, 3, 5 and 6. My center-swap examples produced code 2; I saw no input that produced code 4 | add fixed `m2p-error` rows (or unit asserts checked against Java) for an input Java answers with code 2 and one it answers with code 4 | `4f39580` - fixed `m2p-error` rows for codes 2 and 4 (Java agrees; a planted wrong code fails) and unit asserts; proven by C12 |
