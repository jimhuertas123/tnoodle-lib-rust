use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::hash::Hash;

use rand::Rng;
use tnoodle_svglite::{Color, Dimension, Svg};

use crate::algorithm_builder::{AlgorithmBuilder, MergingMode};

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum PuzzleError {
    InvalidMove(String),
    InvalidScramble(String),
}

impl std::fmt::Display for PuzzleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidMove(m) => write!(f, "Invalid move: {m}"),
            Self::InvalidScramble(s) => write!(f, "Invalid scramble: {s}"),
        }
    }
}

impl std::error::Error for PuzzleError {}

// ---------------------------------------------------------------------------
// PuzzleState trait
// ---------------------------------------------------------------------------

/// A state of a puzzle. Must be cloneable, hashable, and comparable for
/// the bidirectional BFS solver to work.
///
/// Port of Java's `Puzzle.PuzzleState` inner abstract class.
pub trait PuzzleState: Clone + Eq + Hash + Send + Sync + Sized {
    /// All legal moves from this state. Keys are move names (no spaces),
    /// values are the resulting states.
    fn successors_by_name(&self) -> HashMap<String, Self>;

    /// Subset of successors used for scramble generation.
    /// Default: canonical moves (deduped by normalized state).
    fn scramble_successors(&self) -> HashMap<String, Self> {
        self.canonical_moves_by_state()
            .into_iter()
            .map(|(state, name)| (name, state))
            .collect()
    }

    /// Canonical (deduplicated) successors: maps state -> move name,
    /// filtering out states that are equivalent under normalization.
    fn canonical_moves_by_state(&self) -> HashMap<Self, String> {
        let successors = self.successors_by_name();
        let mut unique: HashMap<Self, String> = HashMap::new();
        let mut seen_normalized: HashSet<Self> = HashSet::new();
        seen_normalized.insert(self.get_normalized());

        for (name, state) in successors {
            let normalized = state.get_normalized();
            if seen_normalized.insert(normalized) {
                unique.insert(state, name);
            }
        }
        unique
    }

    /// Canonical representative of this state. All rotations of the same
    /// physical state should return the same normalized form.
    fn get_normalized(&self) -> Self {
        self.clone()
    }

    fn is_normalized(&self) -> bool {
        *self == self.get_normalized()
    }

    fn equals_normalized(&self, other: &Self) -> bool {
        self.get_normalized() == other.get_normalized()
    }

    /// Draw this state as an SVG image.
    fn draw_scramble(&self, colors: &HashMap<String, Color>) -> Svg;

    /// Cost of performing a move. Default is 1.
    fn get_move_cost(&self, _mv: &str) -> i32 {
        1
    }

    /// Apply a single move, returning the new state.
    fn apply(&self, mv: &str) -> Result<Self, PuzzleError> {
        self.successors_by_name()
            .remove(mv)
            .ok_or_else(|| PuzzleError::InvalidMove(format!("Unrecognized turn {mv}")))
    }

    /// Apply a space-separated sequence of moves.
    fn apply_algorithm(&self, algorithm: &str) -> Result<Self, PuzzleError> {
        let mut state = self.clone();
        for mv in split_algorithm(algorithm) {
            state = state
                .apply(mv)
                .map_err(|_| PuzzleError::InvalidScramble(algorithm.to_string()))?;
        }
        Ok(state)
    }

    /// Check whether two moves commute on this state.
    fn moves_commute(&self, m1: &str, m2: &str) -> bool {
        let s1 = self.apply(m1).and_then(|s| s.apply(m2));
        let s2 = self.apply(m2).and_then(|s| s.apply(m1));
        match (s1, s2) {
            (Ok(a), Ok(b)) => a == b,
            _ => false,
        }
    }
}

// ---------------------------------------------------------------------------
// Puzzle trait
// ---------------------------------------------------------------------------

/// A twisty puzzle that can generate WCA-standard scrambles.
///
/// Port of Java's `Puzzle` abstract class.
pub trait Puzzle: Send + Sync {
    type State: PuzzleState;

    fn short_name(&self) -> &str;
    fn long_name(&self) -> &str;
    fn default_color_scheme(&self) -> HashMap<String, Color>;
    fn preferred_size(&self) -> Dimension;
    fn solved_state(&self) -> Self::State;
    fn random_move_count(&self) -> i32;

    fn wca_min_scramble_distance(&self) -> i32 {
        2
    }

    fn initialization_status(&self) -> f64 {
        1.0
    }

    /// Generate a random move sequence. Override for solver-based puzzles
    /// (3x3, 4x4, Square-1, etc.) that use their own random state generators.
    ///
    /// Default: builds a random-turn scramble using `AlgorithmBuilder`.
    fn generate_random_moves(
        &self,
        rng: &mut (impl Rng + ?Sized),
    ) -> PuzzleStateAndGenerator<Self::State> {
        let mut ab = AlgorithmBuilder::new(MergingMode::NoMerging, self.solved_state());

        while ab.total_cost() < self.random_move_count() {
            let mut successors = ab.state().scramble_successors();
            loop {
                if successors.is_empty() {
                    break;
                }
                let mv = choose_from_keys(rng, &successors);
                if ab.is_redundant(&mv) {
                    successors.remove(&mv);
                    continue;
                }
                ab.append_move(&mv)
                    .expect("move from successors should be valid");
                break;
            }
        }
        ab.into_state_and_generator()
    }

    /// Generate a single WCA-legal scramble. Rejects scrambles solvable in
    /// fewer than `wca_min_scramble_distance` moves.
    fn generate_wca_scramble(&self, rng: &mut (impl Rng + ?Sized)) -> String {
        loop {
            let psag = self.generate_random_moves(rng);
            let min_dist = self.wca_min_scramble_distance();
            if solve_in(self, &psag.state, min_dist - 1).is_none() {
                return psag.generator;
            }
        }
    }

    fn is_solved(&self, state: &Self::State) -> bool {
        state.equals_normalized(&self.solved_state())
    }

    /// Draw a scramble as SVG.
    fn draw_scramble(
        &self,
        scramble: &str,
        color_scheme: Option<&HashMap<String, Color>>,
    ) -> Result<Svg, PuzzleError> {
        let mut colors = self.default_color_scheme();
        if let Some(scheme) = color_scheme {
            colors.extend(scheme.iter().map(|(k, v)| (k.clone(), *v)));
        }
        let state = self.solved_state().apply_algorithm(scramble)?;
        Ok(state.draw_scramble(&colors))
    }
}

// ---------------------------------------------------------------------------
// PuzzleStateAndGenerator
// ---------------------------------------------------------------------------

/// A puzzle state paired with the scramble string that produced it.
#[derive(Debug, Clone)]
pub struct PuzzleStateAndGenerator<S> {
    pub state: S,
    pub generator: String,
}

// ---------------------------------------------------------------------------
// Bidirectional BFS solver
// ---------------------------------------------------------------------------

/// Find a solution of cost <= `n` from `state` to solved.
/// Returns `None` if no such solution exists.
///
/// Port of Java's `Puzzle.solveIn()`.
pub fn solve_in<P: Puzzle + ?Sized>(puzzle: &P, ps: &P::State, n: i32) -> Option<String> {
    if puzzle.is_solved(ps) {
        return Some(String::new());
    }

    let mut seen_solved: HashMap<P::State, i32> = HashMap::new();
    let mut fringe_solved: SortedBuckets<P::State> = SortedBuckets::new();
    let mut seen_scrambled: HashMap<P::State, i32> = HashMap::new();
    let mut fringe_scrambled: SortedBuckets<P::State> = SortedBuckets::new();

    let mut best_intersection_cost = n + 1;
    let mut best_intersection: Option<P::State> = None;

    let solved_normalized = puzzle.solved_state().get_normalized();
    fringe_solved.add(solved_normalized.clone(), 0);
    seen_solved.insert(solved_normalized, 0);

    let ps_normalized = ps.get_normalized();
    fringe_scrambled.add(ps_normalized.clone(), 0);
    seen_scrambled.insert(ps_normalized, 0);

    let mut fringe_ties = 0u32;

    // Like Java, an empty fringe keeps the last minimum it had (-1 before any).
    let mut min_fringe_scrambled = -1;
    let mut min_fringe_solved = -1;
    while !fringe_solved.is_empty() || !fringe_scrambled.is_empty() {
        if !fringe_scrambled.is_empty() {
            min_fringe_scrambled = fringe_scrambled.smallest_value();
        }
        if !fringe_solved.is_empty() {
            min_fringe_solved = fringe_solved.smallest_value();
        }

        let extend_solved = if fringe_solved.is_empty() || fringe_scrambled.is_empty() {
            !fringe_solved.is_empty()
        } else if min_fringe_solved < min_fringe_scrambled {
            true
        } else if min_fringe_solved > min_fringe_scrambled {
            false
        } else {
            // Java: (fringeTies++) % 2 == 0, so the solved side goes first.
            let extend = fringe_ties % 2 == 0;
            fringe_ties += 1;
            extend
        };

        // We need to borrow both sides mutably, but only one at a time.
        // Use a flag to track which side we're extending.
        let (node, distance, min_comparing) = if extend_solved {
            let node = fringe_solved.pop();
            let distance = *seen_solved.get(&node).unwrap();
            (node, distance, min_fringe_scrambled)
        } else {
            let node = fringe_scrambled.pop();
            let distance = *seen_scrambled.get(&node).unwrap();
            (node, distance, min_fringe_solved)
        };

        // Check intersection
        let other_seen = if extend_solved {
            &seen_scrambled
        } else {
            &seen_solved
        };
        if let Some(&other_dist) = other_seen.get(&node) {
            let cost = other_dist + distance;
            if cost < best_intersection_cost {
                best_intersection_cost = cost;
                best_intersection = Some(node);
            }
            continue;
        }

        let best_possible = distance + min_comparing;
        if best_possible >= best_intersection_cost || distance >= (n + 1) / 2 {
            continue;
        }

        // Expand node
        let moves_by_state = node.canonical_moves_by_state();
        let extending_seen = if extend_solved {
            &mut seen_solved
        } else {
            &mut seen_scrambled
        };
        let extending_fringe = if extend_solved {
            &mut fringe_solved
        } else {
            &mut fringe_scrambled
        };

        for (next_state, move_name) in &moves_by_state {
            let move_cost = node.get_move_cost(move_name);
            let next_distance = distance + move_cost;
            let next_norm = next_state.get_normalized();

            if let Some(&existing) = extending_seen.get(&next_norm) {
                if next_distance >= existing {
                    continue;
                }
            }
            extending_fringe.add(next_norm.clone(), next_distance);
            extending_seen.insert(next_norm, next_distance);
        }
    }

    let best_intersection = best_intersection?;

    // Reconstruct path: bestIntersection -> scrambled
    let dist_from_scrambled = *seen_scrambled.get(&best_intersection).unwrap();
    let mut linked_states: Vec<Option<P::State>> = vec![None; (dist_from_scrambled + 1) as usize];
    linked_states[dist_from_scrambled as usize] = Some(best_intersection.clone());

    {
        let mut state = best_intersection.clone();
        let mut d = dist_from_scrambled;
        while d > 0 {
            let moves = state.canonical_moves_by_state();
            let mut found = false;
            for (next, _) in &moves {
                let next_norm = next.get_normalized();
                if let Some(&nd) = seen_scrambled.get(&next_norm) {
                    if nd < d {
                        state = next_norm;
                        d = nd;
                        linked_states[d as usize] = Some(state.clone());
                        found = true;
                        break;
                    }
                }
            }
            debug_assert!(found);
        }
    }

    // Build solution: scrambled -> bestIntersection -> solved
    let mut solution = AlgorithmBuilder::new(MergingMode::CanonicalizeMoves, ps.clone());
    let mut state = ps.clone();
    let mut d: i32 = 0;

    while !state.equals_normalized(&best_intersection) {
        let moves = state.canonical_moves_by_state();
        let mut found = false;
        for (next_state, move_name) in &moves {
            if let Some(ref linked) = linked_states[(d + 1) as usize] {
                if next_state.equals_normalized(linked) {
                    state = next_state.clone();
                    solution.append_move(move_name).ok();
                    d = *seen_scrambled.get(&state.get_normalized()).unwrap();
                    found = true;
                    break;
                }
            }
        }
        debug_assert!(found);
    }

    // bestIntersection -> solved
    let mut dist_from_solved = *seen_solved.get(&state.get_normalized()).unwrap();
    while dist_from_solved > 0 {
        let moves = state.canonical_moves_by_state();
        let mut found = false;
        for (next_state, move_name) in &moves {
            let next_norm = next_state.get_normalized();
            if let Some(&nd) = seen_solved.get(&next_norm) {
                if nd < dist_from_solved {
                    state = next_state.clone();
                    dist_from_solved = nd;
                    solution.append_move(move_name).ok();
                    found = true;
                    break;
                }
            }
        }
        debug_assert!(found);
    }

    Some(solution.to_string())
}

// ---------------------------------------------------------------------------
// SortedBuckets — priority queue grouped by cost
// ---------------------------------------------------------------------------

struct SortedBuckets<S> {
    buckets: BTreeMap<i32, VecDeque<S>>,
}

impl<S> SortedBuckets<S> {
    fn new() -> Self {
        Self {
            buckets: BTreeMap::new(),
        }
    }

    fn add(&mut self, element: S, value: i32) {
        self.buckets.entry(value).or_default().push_back(element);
    }

    fn smallest_value(&self) -> i32 {
        *self.buckets.keys().next().unwrap()
    }

    fn is_empty(&self) -> bool {
        self.buckets.is_empty()
    }

    fn pop(&mut self) -> S {
        let first_key = *self.buckets.keys().next().unwrap();
        let bucket = self.buckets.get_mut(&first_key).unwrap();
        let item = bucket.pop_back().unwrap();
        if bucket.is_empty() {
            self.buckets.remove(&first_key);
        }
        item
    }
}

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

/// Split a scramble algorithm string into individual moves.
pub fn split_algorithm(algorithm: &str) -> Vec<&str> {
    let trimmed = algorithm.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }
    trimmed.split_whitespace().collect()
}

/// Pick a random key from a HashMap.
pub fn choose_from_keys<V>(rng: &mut (impl Rng + ?Sized), map: &HashMap<String, V>) -> String {
    let keys: Vec<&String> = map.keys().collect();
    debug_assert!(!keys.is_empty(), "choose from empty map");
    let idx = rng.random_range(0..keys.len());
    keys[idx].clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_algorithm_basic() {
        assert_eq!(split_algorithm("R U R' F"), vec!["R", "U", "R'", "F"]);
        assert_eq!(split_algorithm(""), Vec::<&str>::new());
        assert_eq!(split_algorithm("  R  U  "), vec!["R", "U"]);
    }
}
