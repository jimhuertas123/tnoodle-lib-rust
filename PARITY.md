# Parity with tnoodle-lib v0.20.0

tnoodle-rs mirrors the observable behaviour of [tnoodle-lib](https://github.com/thewca/tnoodle-lib)
`v0.20.0`: the same output where Java is deterministic, the same distribution where it is random.
This file lists every intentional difference. Nothing else may differ.

The Java cross-check (`tools/java-crosscheck/run.sh`) reads the `Check` column: a listed check that
differs is reported as `expected` instead of failing. A row with `-` documents a difference no
cross-check case exercises.

| Check | Java | Rust | Decision |
| --- | --- | --- | --- |
| `svgrot` | `CubePuzzle` refuses to draw a 2x2 scramble containing `y` or `z` rotations (`InvalidScrambleException`) | draws it | open until plan `mirror-images` ports drawing |
| - | `Search.solution` before anything calls `Search.init()` searches with partly built pruning tables (`CoordCube.init(false)`), which can change which equally valid solution it returns | always searches with fully built tables, as Java does after `Search.init()`, which `SearchWCA` (tnoodle's 3x3 scrambles) and threephase (4x4) always call | AD-001 |
| - | seeded `333`, `333ni`, `333fm` search for at least 200 ms, so the scramble for a seed depends on machine speed | seeded 3x3-family scrambles use `probeMin = 0` and are the same on every machine | AD-012 |
| - | the Skewb and Pyraminx solvers order moves with an unseeded `new Random()`, so a seed fixes the state but not the scramble text | the move order is drawn from the seeded stream, so a seed fixes the text | AD-015 |
| - | `Tools.fromScramble` guesses a move for a token that is not one (`R2'` becomes `F`) | returns an error | AD-002 |
| - | errors are `"Error N"` strings, `null` and exceptions | `Result` with typed errors carrying Java's codes; solution text is unchanged | AD-001, AD-016 |
| - | `generateScramble(Random)` accepts any `java.util.Random` | only `JavaRandom` | AD-009 |
