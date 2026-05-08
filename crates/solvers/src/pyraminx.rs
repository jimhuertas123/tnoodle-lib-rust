#![allow(clippy::all)]

//! Pyraminx solver.
//!
//! Ported from `PyraminxSolver.java` in tnoodle-lib.
//! State is represented as (edge_perm, edge_orient, corner_orient, tips).
//!
//! 8 moves for edges/corners: U, U', L, L', R, R', B, B'.
//! 4 tip moves appended to the solution.
//! IDA* search with pruning on edge perm and combined (corner_orient, edge_orient).

use rand::Rng;
use std::sync::LazyLock;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const N_EDGE_PERM: usize = 720; // 6!
const N_EDGE_ORIENT: usize = 32; // 2^5
const N_CORNER_ORIENT: usize = 81; // 3^4
const N_ORIENT: usize = N_EDGE_ORIENT * N_CORNER_ORIENT; // 2592
const N_TIPS: usize = 81; // 3^4
const N_MOVES: usize = 8;
const MAX_LENGTH: usize = 20;

const MOVE_TO_STRING: [&str; N_MOVES] = ["U", "U'", "L", "L'", "R", "R'", "B", "B'"];
const INVERSE_MOVE_TO_STRING: [&str; N_MOVES] = ["U'", "U", "L'", "L", "R'", "R", "B'", "B"];
const TIP_TO_STRING: [&str; 8] = ["u", "u'", "l", "l'", "r", "r'", "b", "b'"];
const INVERSE_TIP_TO_STRING: [&str; 8] = ["u'", "u", "l'", "l", "r'", "r", "b'", "b"];

const FACT: [usize; 7] = [1, 1, 2, 6, 24, 120, 720];

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

/// Compact Pyraminx state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct State {
    pub edge_perm: usize,
    pub edge_orient: usize,
    pub corner_orient: usize,
    pub tips: usize,
}

impl State {
    pub const SOLVED: State = State {
        edge_perm: 0,
        edge_orient: 0,
        corner_orient: 0,
        tips: 0,
    };

    /// Number of unsolved tip positions (0..4).
    pub fn unsolved_tips(&self) -> usize {
        let mut count = 0;
        let mut t = self.tips;
        while t != 0 {
            if t % 3 > 0 {
                count += 1;
            }
            t /= 3;
        }
        count
    }
}

// ---------------------------------------------------------------------------
// Edge helpers
// ---------------------------------------------------------------------------

fn pack_edge_perm(edges: &[i32; 6]) -> usize {
    let mut idx: i32 = 0;
    let mut val: i32 = 0x543210;
    for i in 0..5 {
        let v = (edges[i] & 0x7) << 2;
        idx = (6 - i as i32) * idx + ((val >> v) & 0x7);
        val -= 0x111110 << v;
    }
    idx as usize
}

fn unpack_edge_perm(perm: usize, edges: &mut [i32; 6]) {
    let mut perm = perm as i32;
    let mut val: i32 = 0x543210;
    for i in 0..5 {
        let p = FACT[5 - i] as i32;
        let v = perm / p;
        perm -= v * p;
        let v = v << 2;
        edges[i] = (val >> v) & 0x7;
        let m = (1 << v) - 1;
        val = (val & m) + ((val >> 4) & !m);
    }
    edges[5] = val;
}

fn pack_edge_orient(edges: &[i32; 6]) -> usize {
    let mut ori: i32 = 0;
    for i in 0..5 {
        ori = 2 * ori + (edges[i] >> 3);
    }
    ori as usize
}

fn unpack_edge_orient(ori: usize, edges: &mut [i32; 6]) {
    let mut ori = ori as i32;
    let mut sum_ori: i32 = 0;
    for i in (0..5).rev() {
        edges[i] = (ori & 1) << 3;
        sum_ori ^= ori & 1;
        ori >>= 1;
    }
    edges[5] = sum_ori << 3;
}

fn pack_corner_orient(corners: &[i32; 4]) -> usize {
    let mut ori: i32 = 0;
    for i in 0..4 {
        ori = 3 * ori + corners[i];
    }
    ori as usize
}

fn unpack_corner_orient(ori: usize, corners: &mut [i32; 4]) {
    let mut ori = ori as i32;
    for i in (0..4).rev() {
        corners[i] = ori % 3;
        ori /= 3;
    }
}

/// Cycle three edges with orientation flip.
fn cycle_and_orient(edges: &mut [i32; 6], a: usize, b: usize, c: usize, times: usize) {
    for _ in 0..times {
        let temp = edges[c];
        edges[c] = (edges[b] + 8) % 16;
        edges[b] = (edges[a] + 8) % 16;
        edges[a] = temp;
    }
}

fn move_edges(edges: &mut [i32; 6], mv: usize) {
    let face = mv / 2;
    let times = (mv % 2) + 1;
    match face {
        0 => cycle_and_orient(edges, 5, 3, 1, times),
        1 => cycle_and_orient(edges, 2, 1, 0, times),
        2 => cycle_and_orient(edges, 0, 3, 4, times),
        3 => cycle_and_orient(edges, 2, 4, 5, times),
        _ => unreachable!(),
    }
}

fn move_corners(corners: &mut [i32; 4], mv: usize) {
    let face = mv / 2;
    let times = (mv % 2) + 1;
    corners[face] = (corners[face] + times as i32) % 3;
}

// ---------------------------------------------------------------------------
// Tables
// ---------------------------------------------------------------------------

pub struct Tables {
    move_edge_perm: [[usize; N_MOVES]; N_EDGE_PERM],
    move_edge_orient: [[usize; N_MOVES]; N_EDGE_ORIENT],
    move_corner_orient: [[usize; N_MOVES]; N_CORNER_ORIENT],
    prun_perm: [i32; N_EDGE_PERM],
    prun_orient: [i32; N_ORIENT],
}

fn init_tables() -> Tables {
    let mut move_edge_perm = [[0usize; N_MOVES]; N_EDGE_PERM];
    let mut move_edge_orient = [[0usize; N_MOVES]; N_EDGE_ORIENT];
    let mut move_corner_orient = [[0usize; N_MOVES]; N_CORNER_ORIENT];
    let mut prun_perm = [-1i32; N_EDGE_PERM];
    let mut prun_orient = [-1i32; N_ORIENT];

    // Edge perm move table
    let mut edges1 = [0i32; 6];
    let mut edges2;
    for perm in 0..N_EDGE_PERM {
        unpack_edge_perm(perm, &mut edges1);
        for mv in 0..N_MOVES {
            edges2 = edges1;
            move_edges(&mut edges2, mv);
            move_edge_perm[perm][mv] = pack_edge_perm(&edges2);
        }
    }

    // Edge orient move table
    for orient in 0..N_EDGE_ORIENT {
        unpack_edge_orient(orient, &mut edges1);
        for mv in 0..N_MOVES {
            edges2 = edges1;
            move_edges(&mut edges2, mv);
            move_edge_orient[orient][mv] = pack_edge_orient(&edges2);
        }
    }

    // Corner orient move table
    let mut corners1 = [0i32; 4];
    let mut corners2;
    for orient in 0..N_CORNER_ORIENT {
        unpack_corner_orient(orient, &mut corners1);
        for mv in 0..N_MOVES {
            corners2 = corners1;
            move_corners(&mut corners2, mv);
            move_corner_orient[orient][mv] = pack_corner_orient(&corners2);
        }
    }

    // Perm pruning (BFS) — only half are reachable due to parity
    prun_perm[0] = 0;
    let mut done = 1usize;
    let target = N_EDGE_PERM / 2;
    let mut length = 0i32;
    while done < target {
        for perm in 0..N_EDGE_PERM {
            if prun_perm[perm] == length {
                for mv in 0..N_MOVES {
                    let np = move_edge_perm[perm][mv];
                    if prun_perm[np] == -1 {
                        prun_perm[np] = length + 1;
                        done += 1;
                    }
                }
            }
        }
        length += 1;
    }

    // Orient pruning (BFS on combined edge_orient * corner_orient)
    prun_orient[0] = 0;
    done = 1;
    length = 0;
    while done < N_ORIENT {
        for orient in 0..N_ORIENT {
            if prun_orient[orient] == length {
                for mv in 0..N_MOVES {
                    let new_edge_orient = move_edge_orient[orient % N_EDGE_ORIENT][mv];
                    let new_corner_orient = move_corner_orient[orient / N_EDGE_ORIENT][mv];
                    let new_orient = new_corner_orient * N_EDGE_ORIENT + new_edge_orient;
                    if prun_orient[new_orient] == -1 {
                        prun_orient[new_orient] = length + 1;
                        done += 1;
                    }
                }
            }
        }
        length += 1;
    }

    Tables {
        move_edge_perm,
        move_edge_orient,
        move_corner_orient,
        prun_perm,
        prun_orient,
    }
}

static TABLES: LazyLock<Tables> = LazyLock::new(init_tables);

// ---------------------------------------------------------------------------
// IDA* Search
// ---------------------------------------------------------------------------

fn search(
    tables: &Tables,
    edge_perm: usize,
    edge_orient: usize,
    corner_orient: usize,
    depth: usize,
    length: usize,
    last_move: usize,
    solution: &mut [usize; MAX_LENGTH],
    rng: &mut impl Rng,
) -> bool {
    if length == 0 {
        return edge_perm == 0 && edge_orient == 0 && corner_orient == 0;
    }

    if tables.prun_perm[edge_perm] > length as i32
        || tables.prun_orient[corner_orient * N_EDGE_ORIENT + edge_orient] > length as i32
    {
        return false;
    }

    let offset: usize = rng.random_range(0..N_MOVES);
    for m in 0..N_MOVES {
        let mv = (m + offset) % N_MOVES;
        if mv / 2 == last_move / 2 {
            continue;
        }
        let nep = tables.move_edge_perm[edge_perm][mv];
        let neo = tables.move_edge_orient[edge_orient][mv];
        let nco = tables.move_corner_orient[corner_orient][mv];
        if search(tables, nep, neo, nco, depth + 1, length - 1, mv, solution, rng) {
            solution[depth] = mv;
            return true;
        }
    }
    false
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Generate a random Pyraminx state. Edge perm is guaranteed to have correct parity.
pub fn random_state(rng: &mut impl Rng) -> State {
    let tables = &*TABLES;
    let mut edge_perm;
    loop {
        edge_perm = rng.random_range(0..N_EDGE_PERM);
        if tables.prun_perm[edge_perm] != -1 {
            break;
        }
    }
    State {
        edge_perm,
        edge_orient: rng.random_range(0..N_EDGE_ORIENT),
        corner_orient: rng.random_range(0..N_CORNER_ORIENT),
        tips: rng.random_range(0..N_TIPS),
    }
}

/// Solve the given state in at most `max_depth` moves (not counting tips).
/// Returns the solution with tip moves appended.
pub fn solve(state: State, max_depth: usize) -> Option<String> {
    let tables = &*TABLES;
    let mut rng = rand::rng();
    let mut solution = [0usize; MAX_LENGTH];
    let max_depth = max_depth.min(MAX_LENGTH);

    let mut found_length = None;
    for length in 0..=max_depth {
        if search(
            tables,
            state.edge_perm,
            state.edge_orient,
            state.corner_orient,
            0,
            length,
            42,
            &mut solution,
            &mut rng,
        ) {
            found_length = Some(length);
            break;
        }
    }

    let length = found_length?;
    let mut parts: Vec<&str> = Vec::new();
    for i in 0..length {
        parts.push(MOVE_TO_STRING[solution[i]]);
    }

    // Append tip moves (inverse, since this is a solution)
    let mut array_tips = [0i32; 4];
    unpack_corner_orient(state.tips, &mut array_tips);
    for tip in 0..4 {
        let dir = array_tips[tip] as usize;
        if dir > 0 {
            parts.push(INVERSE_TIP_TO_STRING[tip * 2 + dir - 1]);
        }
    }

    Some(parts.join(" "))
}

/// Generate a scramble (inverse) of exactly `length` moves (optionally counting tips).
pub fn generate_exactly(state: State, length: usize) -> Option<String> {
    let tables = &*TABLES;
    let mut rng = rand::rng();
    let mut solution = [0usize; MAX_LENGTH];

    // Subtract unsolved tips from the target length for the main search
    let tips_count = state.unsolved_tips();
    let main_length = if length >= tips_count {
        length - tips_count
    } else {
        return None;
    };
    let main_length = main_length.min(MAX_LENGTH);

    if !search(
        tables,
        state.edge_perm,
        state.edge_orient,
        state.corner_orient,
        0,
        main_length,
        42,
        &mut solution,
        &mut rng,
    ) {
        return None;
    }

    let mut parts: Vec<&str> = Vec::new();

    // Inverse solution (scramble)
    for i in (0..main_length).rev() {
        parts.push(INVERSE_MOVE_TO_STRING[solution[i]]);
    }

    // Tip moves (as scramble = inverse tip solution)
    let mut array_tips = [0i32; 4];
    unpack_corner_orient(state.tips, &mut array_tips);
    for tip in 0..4 {
        let dir = array_tips[tip] as usize;
        if dir > 0 {
            parts.push(TIP_TO_STRING[tip * 2 + dir - 1]);
        }
    }

    Some(parts.join(" "))
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
    fn random_state_is_valid() {
        let mut rng = rand::rng();
        let tables = &*TABLES;
        for _ in 0..100 {
            let s = random_state(&mut rng);
            assert!(s.edge_perm < N_EDGE_PERM);
            assert!(s.edge_orient < N_EDGE_ORIENT);
            assert!(s.corner_orient < N_CORNER_ORIENT);
            assert!(s.tips < N_TIPS);
            // Edge perm must have correct parity (reachable from solved)
            assert!(tables.prun_perm[s.edge_perm] != -1);
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
}
