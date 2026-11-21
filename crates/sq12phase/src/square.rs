// Square-1 phase 2: permutation solving
// Port of cs.sq12phase.Square
//
// This module intentionally uses i32 arithmetic and casts to match the
// Java original as closely as possible.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_lossless
)]

use std::sync::LazyLock;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// 8! = 40320 -- number of permutations of 8 elements.
const N_PERM: usize = 40320;

/// Pruning table size: two entries per permutation (one per middle-layer state).
const N_PRUN: usize = N_PERM * 2;

/// Factorials 0! through 7!, used for Lehmer-code encoding.
const FACT: [i32; 8] = [1, 1, 2, 6, 24, 120, 720, 5040];

/// Binomial coefficients C(n, k) for n in 0..12, k in 0..12.
/// Built at compile time via const evaluation.
const CNK: [[i32; 12]; 12] = {
    let mut cnk = [[0i32; 12]; 12];
    let mut i = 0;
    while i < 12 {
        cnk[i][0] = 1;
        cnk[i][i] = 1;
        let mut j = 1;
        while j < i {
            cnk[i][j] = cnk[i - 1][j - 1] + cnk[i - 1][j];
            j += 1;
        }
        i += 1;
    }
    cnk
};

// ---------------------------------------------------------------------------
// Permutation utilities
// ---------------------------------------------------------------------------

/// Decode a Lehmer-code index (0..40319) into an 8-element permutation.
///
/// Uses the classic "factorial number system" trick with a packed BCD
/// register (`val = 0x7654_3210`) so that element removal is a shift.
pub fn set8perm(arr: &mut [u8; 8], idx: u16) {
    let mut idx = idx as i32;
    let mut val: i32 = 0x7654_3210;
    for i in 0..7 {
        let p = FACT[7 - i];
        let v = idx / p;
        idx -= v * p;
        let v = v << 2;
        arr[i] = ((val >> v) & 0x7) as u8;
        let m = (1 << v) - 1;
        val = (val & m) + ((val >> 4) & !m);
    }
    arr[7] = val as u8;
}

/// Encode an 8-element permutation into its Lehmer-code index (0..40319).
#[must_use]
pub fn get8perm(arr: &[u8; 8]) -> u16 {
    let mut idx: i32 = 0;
    let mut val: i32 = 0x7654_3210;
    for i in 0..7 {
        let v = (arr[i] as i32) << 2;
        idx = (8 - i as i32) * idx + ((val >> v) & 0x7);
        val -= 0x1111_1110_i32 << v;
    }
    idx as u16
}

/// Compute a combination index for choosing 4 elements from 8.
///
/// Given an 8-element array `arr`, this treats each position as either
/// "selected" (value >= 4) or "not selected" and returns the combinatorial
/// index of that 4-from-8 selection, ranging from 0 to C(8,4)-1 = 69.
#[must_use]
pub fn get8comb(arr: &[u8; 8]) -> u16 {
    let mut idx: i32 = 0;
    let mut r: usize = 4;
    for i in 0..8 {
        if arr[i] >= 4 {
            idx += CNK[7 - i][r];
            r -= 1;
        }
    }
    idx as u16
}

// ---------------------------------------------------------------------------
// Square struct (per-solve state)
// ---------------------------------------------------------------------------

/// Phase-2 state for the Square-1 solver.
///
/// Holds the edge and corner permutation coordinates, plus the
/// middle-layer state and edge-first flags for top/bottom layers.
#[derive(Debug, Clone, Default)]
pub struct Square {
    pub edge_perm: u16,
    pub corner_perm: u16,
    pub top_edge_first: bool,
    pub bot_edge_first: bool,
    pub ml: i32,
}

// ---------------------------------------------------------------------------
// Precomputed tables (lazy-initialized once)
// ---------------------------------------------------------------------------

/// All precomputed tables for the phase-2 permutation solver.
pub struct SquareTables {
    /// Pruning table: `N_PERM * 2` entries. Index = `perm << 1 | ml`.
    /// Values are BFS depth (0..16). `-1` (as `i8`) means unfilled during
    /// construction; all entries are filled by the time init completes.
    pub square_prun: Vec<i8>,

    /// Twist move table: for each permutation index, the result of
    /// swapping the middle two pairs (pos\[2\]<->pos\[4\], pos\[3\]<->pos\[5\]).
    pub twist_move: Vec<u16>,

    /// Top-layer move table: cyclic rotation of positions 0..3.
    pub top_move: Vec<u16>,

    /// Bottom-layer move table: cyclic rotation of positions 4..7.
    pub bottom_move: Vec<u16>,
}

pub static TABLES: LazyLock<SquareTables> = LazyLock::new(SquareTables::init);

impl SquareTables {
    fn init() -> Self {
        let mut twist_move = vec![0u16; N_PERM];
        let mut top_move = vec![0u16; N_PERM];
        let mut bottom_move = vec![0u16; N_PERM];
        let mut pos = [0u8; 8];

        // -- Build move tables --------------------------------------------------
        for i in 0..N_PERM {
            let idx = i as u16;

            // Twist: swap pos[2]<->pos[4], pos[3]<->pos[5]
            set8perm(&mut pos, idx);
            pos.swap(2, 4);
            pos.swap(3, 5);
            twist_move[i] = get8perm(&pos);

            // Top: rotate pos[0..4] left by one
            set8perm(&mut pos, idx);
            let tmp = pos[0];
            pos[0] = pos[1];
            pos[1] = pos[2];
            pos[2] = pos[3];
            pos[3] = tmp;
            top_move[i] = get8perm(&pos);

            // Bottom: rotate pos[4..8] left by one
            set8perm(&mut pos, idx);
            let tmp = pos[4];
            pos[4] = pos[5];
            pos[5] = pos[6];
            pos[6] = pos[7];
            pos[7] = tmp;
            bottom_move[i] = get8perm(&pos);
        }

        // -- Build pruning table via BFS ----------------------------------------
        //
        // The state space is (perm, ml) where perm is 0..40319 and ml is 0 or 1.
        // These are packed into an index: idx = perm << 1 | ml.
        //
        // Once depth >= 11 the search is inverted: instead of expanding from known
        // states, we scan for unknown states and check if any neighbour is known.
        // This is the standard "meet in the middle" optimisation for BFS pruning
        // table construction (fewer states at the frontier).
        //
        // The Java code uses a labeled `continue OUT` to skip the rest of the
        // outer loop body when `inv` is true and a match is found. In Rust this
        // translates to a flag + continue.

        let mut square_prun = vec![-1i8; N_PRUN];
        square_prun[0] = 0;

        let mut depth: i32 = 0;
        let mut done: usize = 1;

        while done < N_PRUN {
            let inv = depth >= 11;
            let find: i8 = if inv { -1 } else { depth as i8 };
            let check: i8 = if inv { depth as i8 } else { -1 };
            depth += 1;

            for i in 0..N_PRUN {
                if square_prun[i] != find {
                    continue;
                }

                let mut perm: usize = i >> 1;
                let ml: usize = i & 1;

                // --- Try twist ---
                let twist_idx = (twist_move[perm] as usize) << 1 | (1 - ml);
                if square_prun[twist_idx] == check {
                    done += 1;
                    square_prun[if inv { i } else { twist_idx }] = depth as i8;
                    if inv {
                        continue;
                    }
                }

                // --- Try top (4 rotations) ---
                let mut cont_outer = false;
                for _m in 0..4 {
                    perm = top_move[perm] as usize;
                    let idx = perm << 1 | ml;
                    if square_prun[idx] == check {
                        done += 1;
                        square_prun[if inv { i } else { idx }] = depth as i8;
                        if inv {
                            cont_outer = true;
                            break;
                        }
                    }
                }
                if cont_outer {
                    continue;
                }

                // --- Try bottom (4 rotations) ---
                // After 4 top rotations perm is back to the original value
                // (top rotation has order 4), matching the Java code where
                // the bottom loop continues from the mutated perm variable.
                for _m in 0..4 {
                    perm = bottom_move[perm] as usize;
                    let idx = perm << 1 | ml;
                    if square_prun[idx] == check {
                        done += 1;
                        square_prun[if inv { i } else { idx }] = depth as i8;
                        if inv {
                            break;
                        }
                    }
                }
            }
        }

        Self {
            square_prun,
            twist_move,
            top_move,
            bottom_move,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perm_roundtrip_identity() {
        let mut arr = [0u8; 8];
        for idx in 0u16..100 {
            set8perm(&mut arr, idx);
            let result = get8perm(&arr);
            assert_eq!(idx, result, "roundtrip failed for idx={idx}");
        }
    }

    #[test]
    fn perm_roundtrip_all() {
        let mut arr = [0u8; 8];
        for idx in 0..N_PERM as u16 {
            set8perm(&mut arr, idx);
            let result = get8perm(&arr);
            assert_eq!(idx, result, "roundtrip failed for idx={idx}");
        }
    }

    #[test]
    fn perm_identity_is_zero() {
        let mut arr = [0u8; 8];
        set8perm(&mut arr, 0);
        assert_eq!(arr, [0, 1, 2, 3, 4, 5, 6, 7]);
    }

    #[test]
    fn perm_max_index() {
        let mut arr = [0u8; 8];
        set8perm(&mut arr, (N_PERM - 1) as u16);
        // The last Lehmer-code permutation is the reverse: [7,6,5,4,3,2,1,0]
        assert_eq!(arr, [7, 6, 5, 4, 3, 2, 1, 0]);
    }

    #[test]
    fn get8comb_identity() {
        // Identity permutation [0,1,2,3,4,5,6,7]:
        // values >= 4 at positions 4,5,6,7.
        // idx = Cnk[3][4] + Cnk[2][3] + Cnk[1][2] + Cnk[0][1] = 0
        let arr = [0u8, 1, 2, 3, 4, 5, 6, 7];
        let c = get8comb(&arr);
        assert_eq!(c, 0);
    }

    #[test]
    fn get8comb_swapped() {
        // All high values in positions 0..3:
        // values >= 4 at positions 0,1,2,3.
        // idx = Cnk[7][4] + Cnk[6][3] + Cnk[5][2] + Cnk[4][1] = 35+20+10+4 = 69
        let arr = [4u8, 5, 6, 7, 0, 1, 2, 3];
        let c = get8comb(&arr);
        assert_eq!(c, 69);
    }

    #[test]
    fn get8comb_range() {
        // get8comb should return values in 0..70 for all permutations
        let mut arr = [0u8; 8];
        for idx in 0..N_PERM as u16 {
            set8perm(&mut arr, idx);
            let c = get8comb(&arr);
            assert!(c < 70, "get8comb out of range at perm {idx}: got {c}");
        }
    }

    #[test]
    fn cnk_spot_check() {
        assert_eq!(CNK[0][0], 1);
        assert_eq!(CNK[4][2], 6);
        assert_eq!(CNK[8][4], 70);
        assert_eq!(CNK[11][11], 1);
    }

    #[test]
    fn tables_init_smoke() {
        // Force table initialization and sanity-check sizes.
        let t = &*TABLES;
        assert_eq!(t.twist_move.len(), N_PERM);
        assert_eq!(t.top_move.len(), N_PERM);
        assert_eq!(t.bottom_move.len(), N_PERM);
        assert_eq!(t.square_prun.len(), N_PRUN);
    }

    #[test]
    fn pruning_solved_is_zero() {
        let t = &*TABLES;
        assert_eq!(t.square_prun[0], 0);
    }

    #[test]
    fn pruning_no_unfilled() {
        let t = &*TABLES;
        for (i, &v) in t.square_prun.iter().enumerate() {
            assert!(
                v >= 0,
                "pruning table entry {i} was never filled (still -1)"
            );
        }
    }

    #[test]
    fn pruning_max_depth_is_16() {
        let t = &*TABLES;
        let max = *t.square_prun.iter().max().unwrap();
        assert_eq!(max, 16, "expected max pruning depth of 16, got {max}");
    }

    #[test]
    fn twist_move_involution() {
        // Applying twist twice should return to the original permutation.
        let t = &*TABLES;
        for i in 0..N_PERM as u16 {
            let twisted = t.twist_move[i as usize];
            let double_twisted = t.twist_move[twisted as usize];
            assert_eq!(i, double_twisted, "twist is not an involution at perm={i}");
        }
    }

    #[test]
    fn top_move_order_4() {
        // Applying top move 4 times should return to the original.
        let t = &*TABLES;
        for i in 0..N_PERM as u16 {
            let mut p = i;
            for _ in 0..4 {
                p = t.top_move[p as usize];
            }
            assert_eq!(i, p, "top move does not have order 4 at perm={i}");
        }
    }

    #[test]
    fn bottom_move_order_4() {
        let t = &*TABLES;
        for i in 0..N_PERM as u16 {
            let mut p = i;
            for _ in 0..4 {
                p = t.bottom_move[p as usize];
            }
            assert_eq!(i, p, "bottom move does not have order 4 at perm={i}");
        }
    }
}
