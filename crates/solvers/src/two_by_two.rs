#![allow(clippy::all)]

//! 2x2x2 Rubik's Cube solver.
//!
//! Ported from `TwoByTwoSolver.java` in tnoodle-lib.
//! State is represented as (permutation index, orientation index) for the
//! 7 movable corner cubies (the 8th corner is fixed as a reference frame).
//!
//! Moves: U, U2, U', R, R2, R', F, F2, F' (9 total).
//! IDA* search with pruning tables on perm and orient coordinates.

use rand::Rng;
use std::sync::LazyLock;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const N_PERM: usize = 5040; // 7!
const N_ORIENT: usize = 729; // 3^6
const N_MOVES: usize = 9;
const MAX_LENGTH: usize = 20;

const MOVE_TO_STRING: [&str; N_MOVES] = ["U", "U2", "U'", "R", "R2", "R'", "F", "F2", "F'"];
const INVERSE_MOVE_TO_STRING: [&str; N_MOVES] =
    ["U'", "U2", "U", "R'", "R2", "R", "F'", "F2", "F"];

const FACT: [usize; 7] = [1, 1, 2, 6, 24, 120, 720]; // fact[x] = x!

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

/// A compact representation of a 2x2 cube position.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct State {
    pub permutation: usize,
    pub orientation: usize,
}

impl State {
    /// The solved (identity) state.
    pub const SOLVED: State = State {
        permutation: 0,
        orientation: 0,
    };
}

// ---------------------------------------------------------------------------
// Cubie helpers
// ---------------------------------------------------------------------------

/// Pack cubie permutation into an index 0..5039.
fn pack_perm(cubies: &[i32; 7]) -> usize {
    let mut idx: i32 = 0;
    let mut val: i32 = 0x6543210;
    for i in 0..6 {
        let v = (cubies[i] & 0x7) << 2;
        idx = (7 - i as i32) * idx + ((val >> v) & 0x7);
        val -= 0x1111110 << v;
    }
    idx as usize
}

/// Unpack permutation index into cubie array.
fn unpack_perm(perm: usize, cubies: &mut [i32; 7]) {
    let mut perm = perm as i32;
    let mut val: i32 = 0x6543210;
    for i in 0..6 {
        let p = FACT[6 - i] as i32;
        let v = perm / p;
        perm -= v * p;
        let v = v << 2;
        cubies[i] = (val >> v) & 0x7;
        let m = (1 << v) - 1;
        val = (val & m) + ((val >> 4) & !m);
    }
    cubies[6] = val;
}

/// Pack cubie orientation into an index 0..728.
fn pack_orient(cubies: &[i32; 7]) -> usize {
    let mut ori: i32 = 0;
    for i in 0..6 {
        ori = 3 * ori + (cubies[i] >> 3);
    }
    ori as usize
}

/// Unpack orientation index into cubie array.
fn unpack_orient(ori: usize, cubies: &mut [i32; 7]) {
    let mut ori = ori as i32;
    let mut sum_ori: i32 = 0;
    for i in (0..6).rev() {
        cubies[i] = (ori % 3) << 3;
        sum_ori += ori % 3;
        ori /= 3;
    }
    cubies[6] = ((42 - sum_ori) % 3) << 3;
}

/// Cycle four elements: a <- d <- c <- b <- a, repeated `times` times.
fn cycle(cubies: &mut [i32; 7], a: usize, b: usize, c: usize, d: usize, times: usize) {
    for _ in 0..times {
        let temp = cubies[d];
        cubies[d] = cubies[c];
        cubies[c] = cubies[b];
        cubies[b] = cubies[a];
        cubies[a] = temp;
    }
}

/// Cycle four elements with orientation twists.
fn cycle_and_orient(cubies: &mut [i32; 7], a: usize, b: usize, c: usize, d: usize, times: usize) {
    for _ in 0..times {
        let temp = cubies[d];
        cubies[d] = (cubies[c] + 8) % 24;
        cubies[c] = (cubies[b] + 16) % 24;
        cubies[b] = (cubies[a] + 8) % 24;
        cubies[a] = (temp + 16) % 24;
    }
}

/// Apply a move (0..8) to cubies.
fn move_cubies(cubies: &mut [i32; 7], mv: usize) {
    let face = mv / 3;
    let times = (mv % 3) + 1;
    match face {
        0 => cycle(cubies, 1, 3, 2, 0, times),
        1 => cycle_and_orient(cubies, 0, 2, 6, 4, times),
        2 => cycle_and_orient(cubies, 1, 0, 4, 5, times),
        _ => unreachable!(),
    }
}

// ---------------------------------------------------------------------------
// Tables
// ---------------------------------------------------------------------------

/// Precomputed move and pruning tables (heap-allocated to avoid stack overflow in debug).
pub struct Tables {
    move_perm: Box<[[usize; N_MOVES]; N_PERM]>,
    move_orient: Box<[[usize; N_MOVES]; N_ORIENT]>,
    prun_perm: Box<[i32; N_PERM]>,
    prun_orient: Box<[i32; N_ORIENT]>,
}

fn init_tables() -> Tables {
    let mut move_perm = vec![[0usize; N_MOVES]; N_PERM].into_boxed_slice();
    let mut move_orient = vec![[0usize; N_MOVES]; N_ORIENT].into_boxed_slice();
    let mut prun_perm = vec![-1i32; N_PERM].into_boxed_slice();
    let mut prun_orient = vec![-1i32; N_ORIENT].into_boxed_slice();

    // Move tables
    let mut cubies1 = [0i32; 7];
    let mut cubies2;
    for perm in 0..N_PERM {
        unpack_perm(perm, &mut cubies1);
        for mv in 0..N_MOVES {
            cubies2 = cubies1;
            move_cubies(&mut cubies2, mv);
            move_perm[perm][mv] = pack_perm(&cubies2);
        }
    }
    for orient in 0..N_ORIENT {
        unpack_orient(orient, &mut cubies1);
        for mv in 0..N_MOVES {
            cubies2 = cubies1;
            move_cubies(&mut cubies2, mv);
            move_orient[orient][mv] = pack_orient(&cubies2);
        }
    }

    // Pruning tables (BFS)
    prun_perm[0] = 0;
    let mut done = 1usize;
    let mut length = 0i32;
    while done < N_PERM {
        for perm in 0..N_PERM {
            if prun_perm[perm] == length {
                for mv in 0..N_MOVES {
                    let np = move_perm[perm][mv];
                    if prun_perm[np] == -1 {
                        prun_perm[np] = length + 1;
                        done += 1;
                    }
                }
            }
        }
        length += 1;
    }

    prun_orient[0] = 0;
    done = 1;
    length = 0;
    while done < N_ORIENT {
        for orient in 0..N_ORIENT {
            if prun_orient[orient] == length {
                for mv in 0..N_MOVES {
                    let no = move_orient[orient][mv];
                    if prun_orient[no] == -1 {
                        prun_orient[no] = length + 1;
                        done += 1;
                    }
                }
            }
        }
        length += 1;
    }

    // Convert boxed slices into boxed arrays
    let move_perm: Box<[[usize; N_MOVES]; N_PERM]> = move_perm.try_into().ok().unwrap();
    let move_orient: Box<[[usize; N_MOVES]; N_ORIENT]> = move_orient.try_into().ok().unwrap();
    let prun_perm: Box<[i32; N_PERM]> = prun_perm.try_into().ok().unwrap();
    let prun_orient: Box<[i32; N_ORIENT]> = prun_orient.try_into().ok().unwrap();

    Tables {
        move_perm,
        move_orient,
        prun_perm,
        prun_orient,
    }
}

static TABLES: LazyLock<Tables> = LazyLock::new(init_tables);

// ---------------------------------------------------------------------------
// Cost heuristic (for choosing best scramble among solutions of same length)
// ---------------------------------------------------------------------------

const COST_U: i32 = 8;
const COST_U_LOW: i32 = 20;
const COST_U2: i32 = 10;
const COST_U3: i32 = 7;
const COST_R: i32 = 6;
const COST_R2: i32 = 10;
const COST_R3: i32 = 6;
const COST_F: i32 = 10;
const COST_F2: i32 = 30;
const COST_F3: i32 = 19;
const COST_REGRIP: i32 = 20;

/// Evaluate cost of applying a scramble iteratively.
/// When a move has two grip options we use a BFS-style expansion with
/// a small work-list so we never blow the stack.
fn compute_cost(solution: &[usize], depth: usize, initial_cost: i32, initial_grip: i32) -> i32 {
    // Each entry is (next_index_to_process, accumulated_cost, grip).
    // `next_index_to_process` counts down from depth-1 to -1.
    let mut stack: Vec<(isize, i32, i32)> = vec![(depth as isize - 1, initial_cost, initial_grip)];
    let mut best = i32::MAX;

    while let Some((index, cost, grip)) = stack.pop() {
        if index < 0 {
            best = best.min(cost);
            continue;
        }
        if cost >= best {
            continue; // prune: already worse than best known
        }
        let idx = index as usize;
        let prev = index - 1;
        match solution[idx] {
            0 => stack.push((prev, cost + COST_U3, grip)),
            1 => stack.push((prev, cost + COST_U2, grip)),
            2 => {
                if grip == 0 {
                    stack.push((prev, cost + COST_U, 0));
                } else if grip == -1 {
                    stack.push((prev, cost + COST_REGRIP + COST_U, 0));
                    stack.push((prev, cost + COST_U_LOW, grip));
                } else {
                    stack.push((prev, cost + COST_REGRIP + COST_U, 0));
                }
            }
            3 => {
                if grip > -1 {
                    stack.push((prev, cost + COST_R3, grip - 1));
                } else {
                    stack.push((prev, cost + COST_REGRIP + COST_R3, -1));
                }
            }
            4 => {
                if grip != 0 {
                    stack.push((prev, cost + COST_R2, -grip));
                } else {
                    stack.push((prev, cost + COST_REGRIP + COST_R2, -1));
                    stack.push((prev, cost + COST_REGRIP + COST_R2, 1));
                }
            }
            5 => {
                if grip < 1 {
                    stack.push((prev, cost + COST_R, grip + 1));
                } else {
                    stack.push((prev, cost + COST_REGRIP + COST_R, 1));
                }
            }
            6 => {
                if grip != 0 {
                    stack.push((prev, cost + COST_F3, grip));
                } else {
                    stack.push((prev, cost + COST_REGRIP + COST_F3, -1));
                    stack.push((prev, cost + COST_REGRIP + COST_F3, 1));
                }
            }
            7 => {
                if grip == -1 {
                    stack.push((prev, cost + COST_F2, -1));
                } else {
                    stack.push((prev, cost + COST_REGRIP + COST_F2, -1));
                }
            }
            8 => {
                if grip == -1 {
                    stack.push((prev, cost + COST_F, -1));
                } else {
                    stack.push((prev, cost + COST_REGRIP + COST_F, -1));
                }
            }
            _ => unreachable!(),
        }
    }
    best
}

// ---------------------------------------------------------------------------
// IDA* Search
// ---------------------------------------------------------------------------

/// Full IDA* search that enumerates all solutions at a given depth and picks
/// the one with the lowest ergonomic cost. Used by `solve`.
fn search_best(
    tables: &Tables,
    perm: usize,
    orient: usize,
    depth: usize,
    length: usize,
    last_move: usize,
    solution: &mut [usize; MAX_LENGTH],
    best_solution: &mut [usize; MAX_LENGTH + 1],
) -> bool {
    if length == 0 {
        if perm == 0 && orient == 0 {
            let cost = compute_cost(solution, depth, 0, 0);
            if (cost as usize) < best_solution[depth] {
                best_solution[..depth].copy_from_slice(&solution[..depth]);
                best_solution[depth] = cost as usize;
            }
            return true;
        }
        return false;
    }

    if tables.prun_perm[perm] > length as i32 || tables.prun_orient[orient] > length as i32 {
        return false;
    }

    let mut found = false;
    for mv in 0..N_MOVES {
        if mv / 3 == last_move / 3 {
            continue;
        }
        let np = tables.move_perm[perm][mv];
        let no = tables.move_orient[orient][mv];
        solution[depth] = mv;
        found |= search_best(tables, np, no, depth + 1, length - 1, mv, solution, best_solution);
    }
    found
}

/// Simplified IDA* that returns the first solution found. Used by `generate_exactly`.
fn search_first(
    tables: &Tables,
    perm: usize,
    orient: usize,
    depth: usize,
    length: usize,
    last_move: usize,
    solution: &mut [usize; MAX_LENGTH],
) -> bool {
    if length == 0 {
        return perm == 0 && orient == 0;
    }

    if tables.prun_perm[perm] > length as i32 || tables.prun_orient[orient] > length as i32 {
        return false;
    }

    for mv in 0..N_MOVES {
        if mv / 3 == last_move / 3 {
            continue;
        }
        let np = tables.move_perm[perm][mv];
        let no = tables.move_orient[orient][mv];
        solution[depth] = mv;
        if search_first(tables, np, no, depth + 1, length - 1, mv, solution) {
            return true;
        }
    }
    false
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Generate a random 2x2 state.
pub fn random_state(rng: &mut impl Rng) -> State {
    State {
        permutation: rng.random_range(0..N_PERM),
        orientation: rng.random_range(0..N_ORIENT),
    }
}

/// Solve the given state in at most `max_depth` moves.
/// Returns the forward solution (not the inverse/generator).
/// An empty string is returned for the solved state.
/// `None` is returned when no solution of that length exists.
pub fn solve(state: State, max_depth: usize) -> Option<String> {
    let tables = &*TABLES;
    let mut solution = [0usize; MAX_LENGTH];
    let mut best_solution = [0usize; MAX_LENGTH + 1];
    let max_depth = max_depth.min(MAX_LENGTH);

    for length in 0..=max_depth {
        best_solution[length] = usize::MAX;
        if search_best(
            tables,
            state.permutation,
            state.orientation,
            0,
            length,
            42, // sentinel: no previous move
            &mut solution,
            &mut best_solution,
        ) {
            if length == 0 {
                return Some(String::new());
            }
            let parts: Vec<&str> = best_solution[..length]
                .iter()
                .map(|&m| MOVE_TO_STRING[m])
                .collect();
            return Some(parts.join(" "));
        }
    }
    None
}

/// Generate a scramble (inverse of solution) of exactly `length` moves.
/// Returns `None` if no solution of exactly that length exists.
pub fn generate_exactly(state: State, length: usize) -> Option<String> {
    let tables = &*TABLES;
    let mut solution = [0usize; MAX_LENGTH];
    let length = length.min(MAX_LENGTH);

    if search_first(
        tables,
        state.permutation,
        state.orientation,
        0,
        length,
        42,
        &mut solution,
    ) {
        if length == 0 {
            return Some(String::new());
        }
        let parts: Vec<&str> = solution[..length]
            .iter()
            .rev()
            .map(|&m| INVERSE_MOVE_TO_STRING[m])
            .collect();
        return Some(parts.join(" "));
    }
    None
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solved_state_returns_empty_solution() {
        let sol = solve(State::SOLVED, MAX_LENGTH);
        assert_eq!(sol, Some(String::new()));
    }

    #[test]
    fn random_state_is_in_range() {
        let mut rng = rand::rng();
        for _ in 0..100 {
            let s = random_state(&mut rng);
            assert!(s.permutation < N_PERM);
            assert!(s.orientation < N_ORIENT);
        }
    }

    #[test]
    fn random_state_is_solvable() {
        let mut rng = rand::rng();
        for _ in 0..20 {
            let s = random_state(&mut rng);
            let sol = solve(s, MAX_LENGTH);
            assert!(sol.is_some(), "state {:?} should be solvable", s);
        }
    }

    #[test]
    fn generate_exactly_produces_correct_length() {
        let mut rng = rand::rng();
        // Try a few random states with a target length of 9
        for _ in 0..10 {
            let s = random_state(&mut rng);
            if let Some(scramble) = generate_exactly(s, 9) {
                if !scramble.is_empty() {
                    let count = scramble.split_whitespace().count();
                    assert_eq!(count, 9, "scramble '{}' should have 9 moves", scramble);
                }
            }
        }
    }
}
