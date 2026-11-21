#![allow(clippy::all)]

//! Skewb solver.
//!
//! Ported from `SkewbSolver.java` in tnoodle-lib.
//! State = (perm: 0..4319, orient/twist: 0..2186).
//!
//! 4 base moves (R, L, D, B in Jaap notation), each with CW and CCW variants = 8 half-moves.
//! IDA* search with pruning on perm and twist coordinates.
//! Output in FCN (Fixed Corner Notation): L, R, B, U.

use rand::Rng;
use std::sync::LazyLock;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const N_PERM: usize = 4320;
const N_TWIST: usize = 2187; // 3^7
const N_MOVES: usize = 4;
const MAX_LENGTH: usize = 12;

const FACT: [usize; 7] = [1, 1, 1, 3, 12, 60, 360]; // fact[x] = x!/2

const CORNER_PERM_MV: [[usize; 4]; 12] = [
    [6, 5, 10, 1],
    [9, 7, 4, 2],
    [3, 11, 8, 0],
    [10, 1, 6, 5],
    [0, 8, 11, 3],
    [7, 9, 2, 4],
    [4, 2, 9, 7],
    [11, 3, 0, 8],
    [1, 10, 5, 6],
    [8, 0, 3, 11],
    [2, 4, 7, 9],
    [5, 6, 1, 10],
];

const ORI: [usize; 12] = [0, 1, 2, 0, 2, 1, 1, 2, 0, 2, 1, 0];

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

/// Compact Skewb position.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct State {
    pub perm: usize,
    pub twist: usize,
}

impl State {
    pub const SOLVED: State = State { perm: 0, twist: 0 };

    /// Whether the perm and twist are consistent (solvable).
    pub fn is_solvable(&self) -> bool {
        ORI[self.perm % 12]
            == (self.twist + self.twist / 3 + self.twist / 9 + self.twist / 27) % 3
    }
}

// ---------------------------------------------------------------------------
// Move computation helpers
// ---------------------------------------------------------------------------

fn get_perm_mv(idx: usize, mv: usize) -> usize {
    let mut center_index = (idx / 12) as i32;
    let corner_index = idx % 12;
    let mut val: i64 = 0x543210;
    let mut parity: i32 = 0;
    let mut center_perm = [0i32; 6];

    for i in 0..5 {
        let p = FACT[5 - i] as i32;
        let v = center_index / p;
        center_index -= v * p;
        parity ^= v;
        let v = (v << 2) as i64;
        center_perm[i] = ((val >> v) & 0xf) as i32;
        let m = (1i64 << v) - 1;
        val = (val & m) + ((val >> 4) & !m);
    }
    if parity & 1 == 0 {
        center_perm[5] = val as i32;
    } else {
        center_perm[5] = center_perm[4];
        center_perm[4] = val as i32;
    }

    // Apply the move as a 3-cycle on centers
    match mv {
        0 => {
            let t = center_perm[0];
            center_perm[0] = center_perm[1];
            center_perm[1] = center_perm[3];
            center_perm[3] = t;
        }
        1 => {
            let t = center_perm[0];
            center_perm[0] = center_perm[4];
            center_perm[4] = center_perm[2];
            center_perm[2] = t;
        }
        2 => {
            let t = center_perm[1];
            center_perm[1] = center_perm[2];
            center_perm[2] = center_perm[5];
            center_perm[5] = t;
        }
        3 => {
            let t = center_perm[3];
            center_perm[3] = center_perm[5];
            center_perm[5] = center_perm[4];
            center_perm[4] = t;
        }
        _ => unreachable!(),
    }

    // Pack center perm back
    val = 0x543210;
    center_index = 0;
    for i in 0..4 {
        let v = (center_perm[i] as i64) << 2;
        center_index *= (6 - i) as i32;
        center_index += ((val >> v) & 0xf) as i32;
        val -= 0x111110i64 << v;
    }

    (center_index as usize) * 12 + CORNER_PERM_MV[corner_index][mv]
}

fn get_twist_mv(idx: usize, mv: usize) -> usize {
    let mut idx = idx;
    let mut fixed_twist = [0usize; 4];
    let mut twist = [0usize; 4];

    for i in 0..4 {
        fixed_twist[i] = idx % 3;
        idx /= 3;
    }
    for i in 0..3 {
        twist[i] = idx % 3;
        idx /= 3;
    }
    twist[3] = (6 - twist[0] - twist[1] - twist[2]) % 3;

    fixed_twist[mv] = (fixed_twist[mv] + 1) % 3;

    match mv {
        0 => {
            let t = twist[0];
            twist[0] = (twist[2] + 2) % 3;
            twist[2] = (twist[1] + 2) % 3;
            twist[1] = (t + 2) % 3;
        }
        1 => {
            let t = twist[0];
            twist[0] = (twist[1] + 2) % 3;
            twist[1] = (twist[3] + 2) % 3;
            twist[3] = (t + 2) % 3;
        }
        2 => {
            let t = twist[0];
            twist[0] = (twist[3] + 2) % 3;
            twist[3] = (twist[2] + 2) % 3;
            twist[2] = (t + 2) % 3;
        }
        3 => {
            let t = twist[1];
            twist[1] = (twist[2] + 2) % 3;
            twist[2] = (twist[3] + 2) % 3;
            twist[3] = (t + 2) % 3;
        }
        _ => unreachable!(),
    }

    // Pack back
    let mut result = 0usize;
    for i in (0..3).rev() {
        result = result * 3 + twist[i] % 3;
    }
    for i in (0..4).rev() {
        result = result * 3 + fixed_twist[i];
    }
    result
}

// ---------------------------------------------------------------------------
// Tables
// ---------------------------------------------------------------------------

pub struct Tables {
    perm_mv: [[usize; N_MOVES]; N_PERM],
    twist_mv: [[usize; N_MOVES]; N_TWIST],
    perm_prun: [i8; N_PERM],
    twist_prun: [i8; N_TWIST],
}

fn init_tables() -> Tables {
    let mut perm_mv = [[0usize; N_MOVES]; N_PERM];
    let mut twist_mv = [[0usize; N_MOVES]; N_TWIST];
    let mut perm_prun = [-1i8; N_PERM];
    let mut twist_prun = [-1i8; N_TWIST];

    for i in 0..N_PERM {
        for j in 0..N_MOVES {
            perm_mv[i][j] = get_perm_mv(i, j);
        }
    }
    for i in 0..N_TWIST {
        for j in 0..N_MOVES {
            twist_mv[i][j] = get_twist_mv(i, j);
        }
    }

    // Perm pruning — BFS, each move applied twice (CW+CCW)
    perm_prun[0] = 0;
    for l in 0..6i8 {
        for p in 0..N_PERM {
            if perm_prun[p] == l {
                for m in 0..N_MOVES {
                    let mut q = p;
                    for _ in 0..2 {
                        q = perm_mv[q][m];
                        if perm_prun[q] == -1 {
                            perm_prun[q] = l + 1;
                        }
                    }
                }
            }
        }
    }

    // Twist pruning — BFS
    twist_prun[0] = 0;
    for l in 0..6i8 {
        for p in 0..N_TWIST {
            if twist_prun[p] == l {
                for m in 0..N_MOVES {
                    let mut q = p;
                    for _ in 0..2 {
                        q = twist_mv[q][m];
                        if twist_prun[q] == -1 {
                            twist_prun[q] = l + 1;
                        }
                    }
                }
            }
        }
    }

    Tables {
        perm_mv,
        twist_mv,
        perm_prun,
        twist_prun,
    }
}

static TABLES: LazyLock<Tables> = LazyLock::new(init_tables);

// ---------------------------------------------------------------------------
// IDA* Search
// ---------------------------------------------------------------------------

/// Returns depth reached, or -1 on failure.
fn ida_search(
    tables: &Tables,
    depth: usize,
    perm: usize,
    twist: usize,
    max_left: usize,
    last_move: isize,
    sol: &mut [usize; MAX_LENGTH],
    rng: &mut impl Rng,
) -> isize {
    if max_left == 0 {
        return if perm == 0 && twist == 0 {
            depth as isize
        } else {
            -1
        };
    }

    if tables.perm_prun[perm] as usize > max_left || tables.twist_prun[twist] as usize > max_left {
        return -1;
    }

    let offset: usize = rng.random_range(0..N_MOVES);
    for m in 0..N_MOVES {
        let mv = (m + offset) % N_MOVES;
        if mv as isize == last_move {
            continue;
        }
        let mut p = perm;
        let mut s = twist;
        for a in 0..2 {
            p = tables.perm_mv[p][mv];
            s = tables.twist_mv[s][mv];
            let result =
                ida_search(tables, depth + 1, p, s, max_left - 1, mv as isize, sol, rng);
            if result != -1 {
                sol[depth] = mv * 2 + a;
                return result;
            }
        }
    }
    -1
}

// ---------------------------------------------------------------------------
// FCN conversion (Jaap notation -> Fixed Corner Notation)
// ---------------------------------------------------------------------------

fn get_solution_string(sol: &[usize], solution_length: usize) -> String {
    // RLDB (Jaap) rotated by z2 -> L R B U (FCN)
    let mut move2str = ["L", "R", "B", "U"];
    let mut parts = Vec::new();

    for i in 0..solution_length {
        let axis = sol[i] >> 1;
        let pow = sol[i] & 1;

        // When axis == 2 ("B" in our mapping, which was originally "D"),
        // we need a coordinate transformation
        if axis == 2 {
            for _ in 0..=pow {
                let temp = move2str[0];
                move2str[0] = move2str[1];
                move2str[1] = move2str[3];
                move2str[3] = temp;
            }
        }

        let suffix = if pow == 1 { "'" } else { "" };
        parts.push(format!("{}{}", move2str[axis], suffix));
    }

    parts.join(" ")
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Generate a random solvable Skewb state.
pub fn random_state(rng: &mut impl Rng) -> State {
    let perm = rng.random_range(0..N_PERM);
    loop {
        let twist = rng.random_range(0..N_TWIST);
        let s = State { perm, twist };
        if s.is_solvable() {
            return s;
        }
    }
}

/// Solve the given state in at most `max_depth` moves.
/// Returns `Some("")` for the solved state, `None` if no solution within depth.
pub fn solve(state: State, max_depth: usize) -> Option<String> {
    let tables = &*TABLES;
    let mut rng = rand::rng();
    let mut sol = [0usize; MAX_LENGTH];
    let max_depth = max_depth.min(MAX_LENGTH);

    for length in 0..=max_depth {
        let result = ida_search(tables, 0, state.perm, state.twist, length, -1, &mut sol, &mut rng);
        if result != -1 {
            let solution_length = result as usize;
            if solution_length == 0 {
                return Some(String::new());
            }
            return Some(get_solution_string(&sol, solution_length));
        }
    }
    None
}

/// Generate a scramble (inverse of a solution) of exactly `length` moves.
/// Returns `None` if no solution of exactly that length exists.
pub fn generate_exactly(state: State, length: usize) -> Option<String> {
    let tables = &*TABLES;
    let mut rng = rand::rng();
    let mut sol = [0usize; MAX_LENGTH];
    let length = length.min(MAX_LENGTH);

    let result = ida_search(tables, 0, state.perm, state.twist, length, -1, &mut sol, &mut rng);
    if result == -1 {
        return None;
    }
    let solution = get_solution_string(&sol, result as usize);
    // Skewb turns are only ever 120 or -120 degrees, so inverting a move
    // just toggles its prime.
    let scramble: Vec<String> = solution
        .split_whitespace()
        .rev()
        .map(|m| m.strip_suffix('\'').map_or_else(|| format!("{m}'"), str::to_string))
        .collect();
    Some(scramble.join(" "))
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
        for _ in 0..100 {
            let s = random_state(&mut rng);
            assert!(s.perm < N_PERM);
            assert!(s.twist < N_TWIST);
            assert!(s.is_solvable(), "state {:?} must be solvable", s);
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
