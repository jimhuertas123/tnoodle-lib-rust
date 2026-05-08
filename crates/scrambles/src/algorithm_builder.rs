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
    _original_state: S,
    /// Tracks the "raw" state without canonicalization, so we can detect rotations.
    un_normalized_state: S,
    total_cost: i32,
    merging_mode: MergingMode,
}

struct IndexAndMove {
    index: usize,
    mv: Option<String>,
}

impl<S: PuzzleState> AlgorithmBuilder<S> {
    pub fn new(merging_mode: MergingMode, original_state: S) -> Self {
        let states = vec![original_state.clone()];
        Self {
            moves: Vec::new(),
            states,
            _original_state: original_state.clone(),
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

    fn find_best_index_for_move(
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
                return Ok(IndexAndMove {
                    index: 0,
                    mv: None,
                });
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
        // One of state()'s successors must be newNormalizedState.
        let canonical_move = match canonical_move {
            Some(m) => m,
            None => {
                // This shouldn't happen, but if it does just append
                return Ok(IndexAndMove {
                    index: self.moves.len(),
                    mv: Some(mv.to_string()),
                });
            }
        };

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
            // New move
            old_cost_move = 0;
            let mv = iam.mv.as_ref().unwrap();
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
