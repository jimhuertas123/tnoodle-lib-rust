# mirror-cubes proofs

**HEAD**: 195d8ba
**Tree**: clean
**Shell**: /bin/sh
**Result**: 34 pass, 0 fail

| Check | Proof | Result |
| --- | --- | --- |
| C1 | `tools/check.sh` | pass |
| C2 | `tools/tests/check_sh_reports_failing_step.sh` | pass |
| C3 | `run xcheck / java: tnoodle-lib v0.20.0` | pass |
| C4 | `tools/java-crosscheck/selftest.sh` | pass |
| C5 | `run xcheck / javarandom-next-int-bounded OK` | pass |
| C6 | `run xcheck / javarandom-other-methods OK` | pass |
| C7 | `run rel / java_random::tests::non_positive_bound_is_an_error ... ok` | pass |
| C8 | `run rel / java_random::tests::entropy_generators_differ ... ok` | pass |
| C9 | `run xcheck / min2phase-search OK` | pass |
| C9 | `run xcheck / min2phase-search-optimal OK` | pass |
| C10 | `run xcheck / min2phase-search-wca OK` | pass |
| C11 | `run xcheck / min2phase-next OK` | pass |
| C12 | `run xcheck / min2phase-errors OK` | pass |
| C12 | `run rel / search::tests::invalid_facelets_return_java_codes ... ok` | pass |
| C13 | `run rel / search::tests::exhausted_searches_return_java_codes ... ok` | pass |
| C14 | `run rel / three_by_three::tests::time_budget_searches_at_least_min ... ok` | pass |
| C14 | `run rel / three_by_three::tests::probe_budget_zero_returns_first_solution ... ok` | pass |
| C15 | `run xcheck / min2phase-tools OK` | pass |
| C16 | `run xcheck / min2phase-from-scramble OK` | pass |
| C16 | `run rel / tools::tests::invalid_tokens_are_errors ... ok` | pass |
| C17 | `run rel / three_by_three::tests::average_length_matches_tnoodle ... ok` | pass |
| C18 | `run xcheck / seeded-identical OK` | pass |
| C19 | `run xcheck / seeded-three-family OK` | pass |
| C20 | `run xcheck / seeded-skewb-pyram OK` | pass |
| C21 | `run rel / tests::seeded_scrambles_repeat_for_every_event ... ok` | pass |
| C22 | `JUDGE_COUNT=2000 tools/java-crosscheck/run.sh 1 0` | pass |
| C22 | `cargo test --release -p tnoodle-scrambles --test randomness -- --ignored` | pass |
| C23 | `tools/tests/versions_and_changelog.sh` | pass |
| C24 | `run rel / tests::concurrent_seeded_scrambles_match_sequential ... ok` | pass |
| C25 | `run dbg / test result: ok` | pass |
| C26 | `run rel / three_by_three::tests::exhausted_budget_returns_first_solution_without_extra_draws ... ok` | pass |
| C27 | `run rel / search::tests::exhausted_searches_return_java_codes ... ok` | pass |
| Build | `cargo build --workspace --all-targets` | pass |
| Build | `cargo clippy --workspace --all-targets -q` | pass |

## Output

`Run xcheck` - exit 0

```
sq1       19/24.1/27            17/23.9/29            -
minx      77/77.0/77            77/77.0/77            -
clock     15/15.0/15            15/15.0/15            -
skewb     11/11.0/11            11/11.0/11            -

first differences in svgrot:
  - 2x2 R U U' F F2 R' R F2 F2 F2 F2 F2 F' F' y2 F' z' U2 U' U y2 R R2 R2 R' U' R z2 U' F'
      java: {}
      rust: {12,24=#0000ff, 12,34=#ff0000, 2,24=#ffffff, 2,34=#0000ff, 24,12=#ffffff, 24,2=#00ff00, 24,24=#ff8000, 24,34=#ffffff, 24,46=#0000ff, 24,56=#ff8000, 34,12=#ffff00, 34,2=#ffff00, 34,24=#ff0000, 34,34=#0000ff, 34,46=#ff0000, 34,56=#ffffff, 46,24=#00ff00, 46,34=#ffff00, 56,24=#00ff00, 56,34=#00ff00, 68,24=#ff8000, 68,34=#ff8000, 78,24=#ff0000, 78,34=#ffff00}
  - 2x2 U U' U' U' U2 U2 U' U' U U' F F U U' U U' z F2 F F2 F2 y' F' R' F2 U U' U' U2 U
      java: {}
      rust: {12,24=#00ff00, 12,34=#00ff00, 2,24=#ff8000, 2,34=#0000ff, 24,12=#ff8000, 24,2=#0000ff, 24,24=#ffffff, 24,34=#ffff00, 24,46=#ff8000, 24,56=#ff0000, 34,12=#ff8000, 34,2=#ff0000, 34,24=#ffffff, 34,34=#ff0000, 34,46=#00ff00, 34,56=#ffff00, 46,24=#0000ff, 46,34=#ffff00, 56,24=#ffffff, 56,34=#ff0000, 68,24=#00ff00, 68,34=#0000ff, 78,24=#ffff00, 78,34=#ffffff}
  - 2x2 R' y' U2 U U2 U' U2 U U2 F R2 R2 R F2 F F2 R2 F R R' R R' R' U2 R2 R2 R' U2 U' U2
      java: {}
      rust: {12,24=#0000ff, 12,34=#00ff00, 2,24=#0000ff, 2,34=#ffff00, 24,12=#ff0000, 24,2=#ff0000, 24,24=#ffffff, 24,34=#ff8000, 24,46=#ffffff, 24,56=#00ff00, 34,12=#ff8000, 34,2=#00ff00, 34,24=#0000ff, 34,34=#ffffff, 34,46=#0000ff, 34,56=#00ff00, 46,24=#ffff00, 46,34=#ff8000, 56,24=#ff0000, 56,34=#ffff00, 68,24=#ffffff, 68,34=#ff8000, 78,24=#ffff00, 78,34=#ff0000}
```

`Run rel` - exit 0

```
     Running unittests src/bin/demo.rs (target/release/deps/demo-754cf38fb4406a2a)
     Running unittests src/bin/demo-svg.rs (target/release/deps/demo_svg-696575377599c901)
     Running tests/randomness.rs (target/release/deps/randomness-a512e6173f56df2b)
     Running tests/scramble_properties.rs (target/release/deps/scramble_properties-f8a2ef8f9465019f)
     Running unittests src/lib.rs (target/release/deps/tnoodle_solvers-41dbbf72afb7b255)
     Running unittests src/lib.rs (target/release/deps/tnoodle_sq12phase-c25f2942add92ee0)
     Running unittests src/lib.rs (target/release/deps/tnoodle_svglite-b7b6ea2c788b39c9)
     Running unittests src/lib.rs (target/release/deps/tnoodle_threephase-ef3d75bf34a5d00e)
   Doc-tests tnoodle_min2phase
   Doc-tests tnoodle_random
   Doc-tests tnoodle_scrambles
   Doc-tests tnoodle_solvers
   Doc-tests tnoodle_sq12phase
   Doc-tests tnoodle_svglite
   Doc-tests tnoodle_threephase
```

`Run dbg` - exit 0

```
     Running unittests src/bin/demo.rs (target/debug/deps/demo-49000a924417ab73)
     Running unittests src/bin/demo-svg.rs (target/debug/deps/demo_svg-d62a4cb0cf956a3c)
     Running tests/randomness.rs (target/debug/deps/randomness-ed34c4744b1fea7a)
     Running tests/scramble_properties.rs (target/debug/deps/scramble_properties-784b5e44c99bda8f)
     Running unittests src/lib.rs (target/debug/deps/tnoodle_solvers-8d7771ed67a3087e)
     Running unittests src/lib.rs (target/debug/deps/tnoodle_sq12phase-ed577daba7d63e41)
     Running unittests src/lib.rs (target/debug/deps/tnoodle_svglite-65375139b8d0f246)
     Running unittests src/lib.rs (target/debug/deps/tnoodle_threephase-79a0a6eecc69010a)
   Doc-tests tnoodle_min2phase
   Doc-tests tnoodle_random
   Doc-tests tnoodle_scrambles
   Doc-tests tnoodle_solvers
   Doc-tests tnoodle_sq12phase
   Doc-tests tnoodle_svglite
   Doc-tests tnoodle_threephase
```

`tools/check.sh` - exit 0

```
fmt PASS
clippy PASS
debug-tests PASS
release-tests PASS
crosscheck PASS
check.sh: PASS
```

`tools/tests/check_sh_reports_failing_step.sh` - exit 0

```
fmt FAIL (log: target/check-fmt.log)
clippy PASS
debug-tests PASS
release-tests PASS
crosscheck PASS
check.sh: FAIL (fmt)
check_sh_reports_failing_step: PASS
```

`tools/java-crosscheck/selftest.sh` - exit 0

```
selftest: PASS
```

`JUDGE_COUNT=2000 tools/java-crosscheck/run.sh 1 0` - exit 0

```
333       15/18.6/20            16/18.6/20            -
444       38/44.2/47            41/44.3/47            -
444fast   40/40.0/40            40/40.0/40            -
555       60/60.0/60            60/60.0/60            -
666       80/80.0/80            80/80.0/80            -
777       100/100.0/100         100/100.0/100         -
333ni     14/20.2/22            17/20.5/22            -
444ni     41/45.8/49            42/45.9/49            -
555ni     60/61.5/62            60/61.5/62            -
333fm     21/24.8/27            23/24.7/27            -
pyram     11/13.7/15            11/13.6/15            -
sq1       15/23.9/29            19/24.0/29            [(-2,6), (-5,6), (1,6), (6,2), (6,5)]
minx      77/77.0/77            77/77.0/77            -
clock     15/15.0/15            15/15.0/15            -
skewb     11/11.0/11            11/11.0/11            -
```

`cargo test --release -p tnoodle-scrambles --test randomness -- --ignored` - exit 0

```
running 1 test
test three_by_three_states_are_uniformly_random ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 3.15s

    Finished `release` profile [optimized] target(s) in 0.05s
     Running tests/randomness.rs (target/release/deps/randomness-a512e6173f56df2b)
```

`tools/tests/versions_and_changelog.sh` - exit 0

```
versions_and_changelog: PASS
```
