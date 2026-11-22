# Changelog

## 0.20.0

Mirrors [tnoodle-lib](https://github.com/thewca/tnoodle-lib) `v0.20.0`: a seed gives the same
scrambles as tnoodle, and the 3x3 solver returns tnoodle's exact text. `PARITY.md` lists every
intentional difference.

### Added

- `tnoodle-random`: `JavaRandom`, Sun's `SHA1PRNG` with the `java.util.Random` methods
  (`SecureRandom.getInstance("SHA1PRNG", "SUN")`).
- `tnoodle_scrambles::generate_scrambles` (`Puzzle.generateScrambles(int)`),
  `generate_seeded_scramble` (`Puzzle.generateSeededScramble(String)`),
  `generate_seeded_scrambles` (`Puzzle.generateSeededScrambles(String, int)`),
  `generate_wca_scramble` (`Puzzle.generateWcaScramble(Random)`), `generate_scramble_with`,
  `ScrambleOptions` and `SearchBudget`.
- `tnoodle_min2phase::search::SearchWca` (`SearchWCA`), `Verbose` (the `verbose` flags),
  `SearchError` (`"Error 1"` to `"Error 9"`), `SearchBudget`, and `Search::next`,
  `Search::number_of_probes`, `Search::length` (`next`, `numberOfProbes`, `length`).
- `tnoodle_min2phase::tools::random_last_layer`, `random_last_slot`, `random_zb_last_layer`,
  `random_corner_of_last_layer`, `random_edge_of_last_layer`, `random_cross_solved`,
  `random_edge_solved`, `random_corner_solved`, `super_flip`, `from_scramble_moves`
  (the `Tools` methods of the same names) and `InvalidMove`.
- `tnoodle_scrambles::puzzle::choose` (`Puzzle.choose`) and
  `PuzzleState::scramble_successor_names`.

### Changed (breaking)

- `generate_scramble(PuzzleType, &mut impl Rng)` is now `generate_scramble(PuzzleType)`
  (`Puzzle.generateScramble()`); randomness is always a `JavaRandom`.
- `Puzzle::generate_random_moves` and `Puzzle::generate_wca_scramble` take `&mut JavaRandom`
  (`Puzzle.generateRandomMoves(Random)`, `Puzzle.generateWcaScramble(Random)`); random turns are
  chosen like `Puzzle.choose` over the successors in Java's order. `puzzle::choose_from_keys` is
  removed; use `puzzle::choose` (`Puzzle.choose`).
- `ScrambleCacher::new` takes a closure over `&mut JavaRandom` (`ScrambleCacher`).
- Every `puzzles::*::generate_scramble*` function takes `&mut JavaRandom`; the 3x3 ones also take
  a `SearchBudget` (`ThreeByThreeCubePuzzle.generateRandomMoves`).
- `tnoodle_min2phase::search::Search::solution` returns `Result<String, SearchError>` with Java's
  text and takes `Verbose` (`Search.solution`). `SearchResult` and
  `Search::solution_with_restrictions` are removed; use `SearchWca::solution`
  (`SearchWCA.solution`).
- `tnoodle_min2phase::tools::random_cube` returns facelets drawn from a `JavaRandom`
  (`Tools.randomCube(Random)`); `tools::random_state` and `tools::parse_facelets` are removed;
  `tools::from_scramble` returns `Result` (`Tools.fromScramble(String)`).
- `tnoodle_min2phase::coord_cube` is rebuilt as tnoodle configures it (`USE_TWIST_FLIP_PRUN`,
  `USE_COMBP_PRUN` and `USE_CONJ_PRUN` on): `N_COMB` is 140, `CoordTables` gains
  `twist_flip_prun`, `CoordCube::set_with_prun` takes `&mut self` and returns `bool`, and
  `CoordCube` gains `calc_pruning` and `do_move_prun_conj` (`CoordCube`).
- `tnoodle_min2phase::phase2` is removed: phase 2 uses Java's symmetry-reduced tables.
- `tnoodle_threephase::search::Search::random_state`, `tools::random_cube` and the piece
  constructors take `&mut JavaRandom` (`Search.randomState(Random)`); the phase-1 candidate queue
  and the `Center1` symmetry choice follow Java's `PriorityQueue` and `getsym`.
- `tnoodle_sq12phase::full_cube::FullCube::random_cube` takes `&mut JavaRandom`
  (`FullCube.randomCube(Random)`).
- `tnoodle_solvers::{two_by_two, skewb, pyraminx}::random_state` take `&mut JavaRandom`
  (`TwoByTwoSolver`, `SkewbSolver`, `PyraminxSolver` `randomState(Random)`).
- Every crate is versioned `0.20.0`, after the tnoodle-lib release it mirrors.
