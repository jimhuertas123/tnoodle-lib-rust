use crate::puzzle::{PuzzleError, PuzzleState, PuzzleStateAndGenerator};

// ---------------------------------------------------------------------------
// MergingMode
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergingMode {
    /// Blindly append moves. "R R" stays unmodified.
    NoMerging,
    /// Most aggressive merging. Merges and cancels redundant moves
    /// using canonical (normalized) state comparisons.
    /// "R R" becomes "R2", "L Rw" becomes "L2", "F x U" becomes "F2".
    CanonicalizeMoves,
}

// ---------------------------------------------------------------------------
// AlgorithmBuilder
// ---------------------------------------------------------------------------

/// Builds a move sequence with optional move merging/cancellation.
///
/// Port of Java's `AlgorithmBuilder`.
pub struct AlgorithmBuilder<S: PuzzleState> {
    moves: Vec<String>,
    /// `states[i]` = state after applying `moves[0..i]`.
    /// `states[0]` = original state.
    states: Vec<S>,
    original_state: S,
    /// Tracks the "raw" state without canonicalization, so we can detect rotations.
    un_normalized_state: S,
    total_cost: i32,
    merging_mode: MergingMode,
}

/// Where a move would go: `index < len` means it merges with (`mv` is the
/// merged move) or cancels (`mv` is `None`) the move at `index`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexAndMove {
    pub index: usize,
    pub mv: Option<String>,
}

impl<S: PuzzleState> AlgorithmBuilder<S> {
    pub fn new(merging_mode: MergingMode, original_state: S) -> Self {
        let states = vec![original_state.clone()];
        Self {
            moves: Vec::new(),
            states,
            original_state: original_state.clone(),
            un_normalized_state: original_state,
            total_cost: 0,
            merging_mode,
        }
    }

    pub fn state(&self) -> &S {
        self.states.last().unwrap()
    }

    pub fn total_cost(&self) -> i32 {
        self.total_cost
    }

    pub fn is_redundant(&self, mv: &str) -> bool {
        match self.find_best_index_for_move(mv, MergingMode::CanonicalizeMoves) {
            Ok(iam) => iam.index < self.moves.len() || iam.mv.is_none(),
            Err(_) => true,
        }
    }

    pub fn find_best_index_for_move(
        &self,
        mv: &str,
        merging_mode: MergingMode,
    ) -> Result<IndexAndMove, PuzzleError> {
        if merging_mode == MergingMode::NoMerging {
            return Ok(IndexAndMove {
                index: self.moves.len(),
                mv: Some(mv.to_string()),
            });
        }

        let new_un_normalized = self.un_normalized_state.apply(mv)?;

        if new_un_normalized.equals_normalized(&self.un_normalized_state) {
            // Move is just a rotation
            if merging_mode == MergingMode::CanonicalizeMoves {
                return Ok(IndexAndMove { index: 0, mv: None });
            }
        }

        let new_normalized = new_un_normalized.get_normalized();

        let successors = self.state().canonical_moves_by_state();
        let mut canonical_move: Option<String> = None;
        for (ps, name) in &successors {
            if ps.equals_normalized(&new_normalized) {
                canonical_move = Some(name.clone());
                break;
            }
        }
        // One of state()'s successors must be newNormalizedState. Java only
        // asserts this and then fails on the null move, so report it the same
        // way instead of appending a move that is not canonical.
        let canonical_move = canonical_move
            .ok_or_else(|| PuzzleError::InvalidMove(format!("no canonical move matches {mv}")))?;

        if merging_mode == MergingMode::CanonicalizeMoves {
            // Walk backwards looking for a move that doesn't commute
            for last_move_index in (0..self.moves.len()).rev() {
                let last_move = &self.moves[last_move_index];
                let state_before = &self.states[last_move_index];

                if !state_before.moves_commute(last_move, &canonical_move) {
                    break;
                }

                let state_after = &self.states[last_move_index + 1];
                let state_after_both = match state_after.apply(&canonical_move) {
                    Ok(s) => s,
                    Err(_) => break,
                };

                if state_before.equals_normalized(&state_after_both) {
                    // Move cancels with last_move
                    return Ok(IndexAndMove {
                        index: last_move_index,
                        mv: None,
                    });
                }

                // Check if it merges
                let before_successors = state_before.canonical_moves_by_state();
                for (ps, name) in &before_successors {
                    if ps.equals_normalized(&state_after_both) {
                        return Ok(IndexAndMove {
                            index: last_move_index,
                            mv: Some(name.clone()),
                        });
                    }
                }
            }
        }

        Ok(IndexAndMove {
            index: self.moves.len(),
            mv: Some(canonical_move),
        })
    }

    pub fn append_move(&mut self, new_move: &str) -> Result<(), PuzzleError> {
        let iam = self.find_best_index_for_move(new_move, self.merging_mode)?;

        let old_cost_move: i32;
        let new_cost_move: i32;

        if iam.index < self.moves.len() {
            // Redundant move: merge or cancel
            old_cost_move = self.states[iam.index].get_move_cost(&self.moves[iam.index]);
            if let Some(ref merged_move) = iam.mv {
                // Merge
                self.moves[iam.index] = merged_move.clone();
                new_cost_move = self.states[iam.index].get_move_cost(merged_move);
            } else {
                // Cancel
                self.moves.remove(iam.index);
                self.states.remove(iam.index + 1);
                new_cost_move = 0;
            }
        } else {
            // New move. A rotation with no earlier move to fold into has
            // nothing to append; Java rejects it as an unrecognized turn.
            old_cost_move = 0;
            let mv = iam
                .mv
                .as_ref()
                .ok_or_else(|| PuzzleError::InvalidMove(format!("unrecognized turn {new_move}")))?;
            new_cost_move = self.states.last().unwrap().get_move_cost(mv);
            self.moves.push(mv.clone());
            self.states.push(self.states[0].clone()); // placeholder
        }

        self.total_cost += new_cost_move - old_cost_move;

        // Rebuild states from iam.index+1 onward
        for i in (iam.index + 1)..self.states.len() {
            let prev = self.states[i - 1].clone();
            let mv = &self.moves[i - 1];
            self.states[i] = prev.apply(mv).expect("replayed move should be valid");
        }

        self.un_normalized_state = self.un_normalized_state.apply(new_move)?;
        debug_assert_eq!(self.states.len(), self.moves.len() + 1);
        Ok(())
    }

    /// Remove the move at `index`, replaying the remaining moves from the
    /// original state. Port of Java's `popMove`.
    pub fn pop_move(&mut self, index: usize) -> String {
        let mut moves = std::mem::take(&mut self.moves);
        let popped = moves.remove(index);
        let original = self.original_state.clone();
        self.states = vec![original.clone()];
        self.un_normalized_state = original;
        self.total_cost = 0;
        for mv in &moves {
            self.append_move(mv).expect("replayed move should be valid");
        }
        popped
    }

    pub fn append_algorithm(&mut self, algorithm: &str) -> Result<(), PuzzleError> {
        for mv in crate::puzzle::split_algorithm(algorithm) {
            self.append_move(mv)?;
        }
        Ok(())
    }

    pub fn into_state_and_generator(self) -> PuzzleStateAndGenerator<S> {
        let state = self.states.into_iter().last().unwrap();
        let generator = self.moves.join(" ");
        PuzzleStateAndGenerator { state, generator }
    }

    pub fn to_string(&self) -> String {
        self.moves.join(" ")
    }
}

impl<S: PuzzleState> std::fmt::Display for AlgorithmBuilder<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.moves.join(" "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::puzzle::{Puzzle, solve_in};
    use crate::puzzles::cube_puzzle::{CubePuzzle, CubeState};
    use rand::Rng;

    fn canonicalize(size: usize, alg: &str) -> String {
        let mut ab = AlgorithmBuilder::new(MergingMode::CanonicalizeMoves, CubeState::solved(size));
        ab.append_algorithm(alg).unwrap();
        ab.to_string()
    }

    #[test]
    fn merges_commuting_and_rotation_equivalent_moves() {
        // Port of HugeScrambleTest.testAlgorithmBuilder (cube parts).
        assert_eq!(canonicalize(4, "Rw Lw").split_whitespace().count(), 1);
        assert_eq!(canonicalize(3, "R L R"), "R2 L");
        assert_eq!(canonicalize(3, "F' B F'"), "F2 B");
        assert_eq!(canonicalize(3, "U D U'"), "D");
        assert_eq!(canonicalize(3, "R R'"), "");
        assert_eq!(canonicalize(5, "R Rw R"), "R2 Rw");

        let mut ab = AlgorithmBuilder::new(MergingMode::NoMerging, CubeState::solved(5));
        ab.append_algorithm("U R 4Rw'").unwrap();
        assert_eq!(ab.to_string(), "U R 4Rw'");
    }

    #[test]
    fn canonical_algorithm_is_unchanged() {
        // From HugeScrambleTest.testCubeNormalization.
        let alg = "D2 U' L2 B2 F2 D B2 U' B2 F D' F U' R F2 L2 D' B D F'";
        assert_eq!(canonicalize(3, alg), alg);
    }

    #[test]
    fn rotations_behave_like_java() {
        // Java folds a rotation into the first move, and rejects one with
        // no move before it ("Unrecognized turn null") instead of panicking.
        assert_eq!(canonicalize(3, "R x"), "");
        let mut ab = AlgorithmBuilder::new(MergingMode::CanonicalizeMoves, CubeState::solved(3));
        assert!(ab.append_algorithm("x R").is_err());
    }

    #[test]
    fn repeating_any_move_is_redundant() {
        // Port of AlgorithmBuilderTest.testRedundantMoves (cube puzzles).
        for size in 2..=7 {
            let solved = CubeState::solved(size);
            for mv in solved.successors_by_name().keys() {
                let mut ab = AlgorithmBuilder::new(MergingMode::NoMerging, solved.clone());
                ab.append_move(mv).unwrap();
                assert!(ab.is_redundant(mv), "{mv} twice on {size}x{size}");
            }
        }
    }

    #[test]
    fn solve_in_finds_short_solutions() {
        // Port of HugeScrambleTest.testSolveIn / testTwosSolver (cube puzzles).
        let mut rng = rand::rng();
        for size in 2..=4 {
            let puzzle = CubePuzzle::new(size);
            assert_eq!(
                solve_in(&puzzle, &puzzle.solved_state(), 0).as_deref(),
                Some("")
            );
            for _ in 0..10 {
                let mut state = puzzle.solved_state();
                for _ in 0..4 {
                    let successors: Vec<CubeState> =
                        state.successors_by_name().into_values().collect();
                    state = successors[rng.random_range(0..successors.len())].clone();
                }
                let solution = solve_in(&puzzle, &state, 4).expect("solvable in 4");
                assert!(
                    puzzle.is_solved(&state.apply_algorithm(&solution).unwrap()),
                    "{solution}"
                );
            }
        }
        let twos = CubePuzzle::new(2);
        let state = twos.solved_state().apply_algorithm("R2 B2 F2").unwrap();
        let solution = solve_in(&twos, &state, 1).expect("R2 B2 F2 is one move on 2x2");
        assert!(twos.is_solved(&state.apply_algorithm(&solution).unwrap()));
    }
}
