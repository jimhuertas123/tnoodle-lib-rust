// Square-1 phase 1: shape solving
// Port of cs.sq12phase.Shape

use std::sync::LazyLock;

/// Face turn metric constant.
pub const FACE_TURN_METRIC: i32 = 0;

/// WCA turn metric constant.
pub const WCA_TURN_METRIC: i32 = 1;

/// 1 = corner, 0 = edge.
pub const HALFLAYER: [i32; 13] = [
    0x00, 0x03, 0x06, 0x0c, 0x0f, 0x18, 0x1b, 0x1e, 0x30, 0x33, 0x36, 0x3c, 0x3f,
];

/// Number of distinct shapes (ignoring parity).
pub const N_SHAPE_IDX: usize = 3678;

/// Total shape states (shapes x parity).
pub const N_SHAPE: usize = N_SHAPE_IDX * 2;

// ---------------------------------------------------------------------------
// Shape tables (lazy initialized)
// ---------------------------------------------------------------------------

/// Global lazily-initialized shape tables.
pub static SHAPE_TABLES: LazyLock<ShapeTables> = LazyLock::new(ShapeTables::init);

/// All precomputed shape tables for the Square-1 solver.
pub struct ShapeTables {
    /// Sorted array of 3678 valid shape values (top12 | bottom12).
    pub shape_idx: Vec<i32>,
    /// Phase-1 pruning table (face-turn metric). Indexed by shape*2+parity.
    pub shape_prun: Vec<i32>,
    /// Optimised pruning table (WCA turn metric). Indexed by shape*2+parity.
    pub shape_prun_opt: Vec<i32>,
    /// Top-layer move transition table. Indexed by shape*2+parity.
    /// Value: (new_idx << 4) | move_increment.
    pub top_move: Vec<i32>,
    /// Bottom-layer move transition table. Same format as `top_move`.
    pub bottom_move: Vec<i32>,
    /// Twist move transition table. Indexed by shape*2+parity.
    /// Value: new shape*2+parity index.
    pub twist_move: Vec<i32>,
}

impl ShapeTables {
    fn init() -> Self {
        // --- Enumerate all valid shapes ---
        let mut shape_idx = Vec::with_capacity(N_SHAPE_IDX);
        for i in 0..(13 * 13 * 13 * 13) {
            let dr = HALFLAYER[i % 13];
            let dl = HALFLAYER[i / 13 % 13];
            let ur = HALFLAYER[i / 13 / 13 % 13];
            let ul = HALFLAYER[i / 13 / 13 / 13];
            let value = ul << 18 | ur << 12 | dl << 6 | dr;
            if value.count_ones() == 16 {
                shape_idx.push(value);
            }
        }
        debug_assert_eq!(shape_idx.len(), N_SHAPE_IDX);

        // --- Build move tables ---
        let mut top_move = vec![0i32; N_SHAPE];
        let mut bottom_move = vec![0i32; N_SHAPE];
        let mut twist_move = vec![0i32; N_SHAPE];

        for i in 0..N_SHAPE {
            let mut s = ShapeState::from_idx(i, &shape_idx);
            let tm = s.top_move();
            top_move[i] = tm | (s.get_idx(&shape_idx) << 4);

            let mut s = ShapeState::from_idx(i, &shape_idx);
            let bm = s.bottom_move();
            bottom_move[i] = bm | (s.get_idx(&shape_idx) << 4);

            let mut s = ShapeState::from_idx(i, &shape_idx);
            s.twist_move();
            twist_move[i] = s.get_idx(&shape_idx);
        }

        // --- Build pruning tables ---
        let mut shape_prun = vec![-1i32; N_SHAPE];
        let mut shape_prun_opt = vec![-1i32; N_SHAPE];

        // Seed ShapePrun with the 4 "square" shapes.
        //   0x0db66db => parity 0, top=0xdb6 (110110110110), bottom=0x6db (011011011011)
        //   0x1db6db6 => parity 1, top=0xdb6, bottom=0xdb6
        //   0x16db6db => parity 1, top=0x6db, bottom=0x6db
        //   0x06dbdb6 => parity 0, top=0x6db, bottom=0xdb6
        let seeds: [i32; 4] = [0x0db6_6db, 0x1db6_db6, 0x16db_6db, 0x06db_db6];
        for &seed in &seeds {
            let idx = get_shape2idx_raw(&shape_idx, seed) as usize;
            shape_prun[idx] = 0;
        }

        // Seed ShapePrunOpt with the solved cube's shape index.
        let solved_idx = solved_cube_shape_idx(&shape_idx) as usize;
        shape_prun_opt[solved_idx] = 0;

        init_pruning(
            &mut shape_prun,
            4,
            FACE_TURN_METRIC,
            &top_move,
            &bottom_move,
            &twist_move,
        );
        init_pruning(
            &mut shape_prun_opt,
            1,
            WCA_TURN_METRIC,
            &top_move,
            &bottom_move,
            &twist_move,
        );

        Self {
            shape_idx,
            shape_prun,
            shape_prun_opt,
            top_move,
            bottom_move,
            twist_move,
        }
    }
}

// ---------------------------------------------------------------------------
// Public lookup
// ---------------------------------------------------------------------------

/// Look up the combined shape+parity index.
///
/// `shp`: bits 0..23 = shape (ul6 | ur6 | dl6 | dr6), bit 24 = parity.
pub fn get_shape2idx(shp: i32) -> i32 {
    let tables = &*SHAPE_TABLES;
    get_shape2idx_raw(&tables.shape_idx, shp)
}

/// Alias for [`get_shape2idx`] matching the snake_case convention used
/// elsewhere in the crate.
pub fn get_shape2_idx(shp: i32) -> i32 {
    get_shape2idx(shp)
}

/// Same as [`get_shape2idx`] but operates on a provided `shape_idx` slice
/// (used during table construction before the global is available).
fn get_shape2idx_raw(shape_idx: &[i32], shp: i32) -> i32 {
    let val = shp & 0x00ff_ffff;
    let parity = (shp >> 24) & 1;
    let pos = shape_idx
        .binary_search(&val)
        .expect("shape not found in ShapeIdx") as i32;
    (pos << 1) | parity
}

// ---------------------------------------------------------------------------
// Pruning-table BFS (ported from Shape.initPruning)
// ---------------------------------------------------------------------------

fn init_pruning(
    prun: &mut [i32],
    mut done: i32,
    metric: i32,
    top_move: &[i32],
    bottom_move: &[i32],
    twist_move: &[i32],
) {
    let mut done0 = 0i32;
    let mut depth = -1i32;

    while done != done0 {
        done0 = done;
        depth += 1;

        for i in 0..N_SHAPE {
            if prun[i] != depth {
                continue;
            }

            // Try twist
            {
                let idx = twist_move[i] as usize;
                if prun[idx] == -1 {
                    done += 1;
                    prun[idx] = depth + 1;
                }
            }

            if metric == FACE_TURN_METRIC {
                // Try top moves (replicate Java: for(m=0,inc=0,idx=i; m!=12; m+=inc){...})
                {
                    let mut m = 0i32;
                    let mut idx = i;
                    loop {
                        let entry = top_move[idx];
                        let inc = entry & 0xf;
                        idx = (entry >> 4) as usize;
                        if prun[idx] == -1 {
                            done += 1;
                            prun[idx] = depth + 1;
                        }
                        m += inc;
                        if m == 12 {
                            break;
                        }
                    }
                }
                // Try bottom moves
                {
                    let mut m = 0i32;
                    let mut idx = i;
                    loop {
                        let entry = bottom_move[idx];
                        let inc = entry & 0xf;
                        idx = (entry >> 4) as usize;
                        if prun[idx] == -1 {
                            done += 1;
                            prun[idx] = depth + 1;
                        }
                        m += inc;
                        if m == 12 {
                            break;
                        }
                    }
                }
            } else if metric == WCA_TURN_METRIC {
                // Try combined top/bottom moves
                let mut m = 0i32;
                let mut idx = i;
                loop {
                    let entry = top_move[idx];
                    let inc = entry & 0xf;
                    idx = (entry >> 4) as usize;

                    // For each top position, iterate all bottom moves
                    let mut m2 = 0i32;
                    let mut idx2 = idx;
                    loop {
                        let entry2 = bottom_move[idx2];
                        let inc2 = entry2 & 0xf;
                        idx2 = (entry2 >> 4) as usize;
                        if prun[idx2] == -1 {
                            done += 1;
                            prun[idx2] = depth + 1;
                        }
                        m2 += inc2;
                        if m2 == 12 {
                            break;
                        }
                    }

                    m += inc;
                    if m == 12 {
                        break;
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Solved cube shape index (avoids circular dependency with FullCube)
// ---------------------------------------------------------------------------

/// Compute the shape index for the solved (default) FullCube.
///
/// The solved cube has piece nibbles:
///   ul=0x011233, ur=0x455677, dl=0x998bba, dr=0xddcffe
///
/// Bit 0 of each nibble is 1 for corners, 0 for edges.
/// This replicates `FullCube.getShapeIdx()` for the identity state.
fn solved_cube_shape_idx(shape_idx: &[i32]) -> i32 {
    fn extract_shape(val: i32) -> i32 {
        let x = val & 0x11_1111;
        let x = x | (x >> 3);
        let x = x | (x >> 6);
        (x & 0xf) | ((x >> 12) & 0x30)
    }

    let ul = 0x01_1233_i32;
    let ur = 0x45_5677_i32;
    let dl = 0x99_8bba_u32 as i32;
    let dr = 0xdd_cffe_u32 as i32;

    let ulx = extract_shape(ul);
    let urx = extract_shape(ur);
    let dlx = extract_shape(dl);
    let drx = extract_shape(dr);

    // Solved cube parity is always 0.
    let shp = (ulx << 18) | (urx << 12) | (dlx << 6) | drx;

    get_shape2idx_raw(shape_idx, shp)
}

// ---------------------------------------------------------------------------
// Internal shape state used during table generation
// ---------------------------------------------------------------------------

struct ShapeState {
    top: i32,
    bottom: i32,
    parity: i32,
}

impl ShapeState {
    fn from_idx(idx: usize, shape_idx: &[i32]) -> Self {
        let parity = (idx & 1) as i32;
        let entry = shape_idx[idx >> 1];
        let top = entry >> 12;
        let bottom = entry & 0xfff;
        Self {
            top,
            bottom,
            parity,
        }
    }

    fn get_idx(&self, shape_idx: &[i32]) -> i32 {
        let val = self.top << 12 | self.bottom;
        let pos = shape_idx.binary_search(&val).expect("shape not found") as i32;
        (pos << 1) | self.parity
    }

    fn top_move(&mut self) -> i32 {
        let mut mv = 0i32;
        let mut move_parity = 0i32;
        loop {
            if (self.top & 0x800) == 0 {
                mv += 1;
                self.top <<= 1;
            } else {
                mv += 2;
                self.top = (self.top << 2) ^ 0x3003;
            }
            move_parity = 1 - move_parity;
            if (self.top & 0x3f).count_ones() & 1 == 0 {
                break;
            }
        }
        if self.top.count_ones() & 2 == 0 {
            self.parity ^= move_parity;
        }
        mv
    }

    fn bottom_move(&mut self) -> i32 {
        let mut mv = 0i32;
        let mut move_parity = 0i32;
        loop {
            if (self.bottom & 0x800) == 0 {
                mv += 1;
                self.bottom <<= 1;
            } else {
                mv += 2;
                self.bottom = (self.bottom << 2) ^ 0x3003;
            }
            move_parity = 1 - move_parity;
            if (self.bottom & 0x3f).count_ones() & 1 == 0 {
                break;
            }
        }
        if self.bottom.count_ones() & 2 == 0 {
            self.parity ^= move_parity;
        }
        mv
    }

    fn twist_move(&mut self) {
        let temp = self.top & 0x3f;
        let p1 = temp.count_ones() as i32;
        let p3 = (self.bottom & 0xfc0).count_ones() as i32;
        self.parity ^= 1 & ((p1 & p3) >> 1);
        self.top = (self.top & 0xfc0) | ((self.bottom >> 6) & 0x3f);
        self.bottom = (self.bottom & 0x3f) | (temp << 6);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shape_idx_count() {
        let tables = &*SHAPE_TABLES;
        assert_eq!(tables.shape_idx.len(), N_SHAPE_IDX);
        // Last entry should be the largest shape value.
        assert!(tables.shape_idx[N_SHAPE_IDX - 1] > 0);
    }

    #[test]
    fn test_shape_idx_sorted() {
        let tables = &*SHAPE_TABLES;
        for i in 1..tables.shape_idx.len() {
            assert!(
                tables.shape_idx[i] > tables.shape_idx[i - 1],
                "ShapeIdx not sorted at index {}",
                i
            );
        }
    }

    #[test]
    fn test_shape_idx_bitcount() {
        let tables = &*SHAPE_TABLES;
        for (i, &val) in tables.shape_idx.iter().enumerate() {
            assert_eq!(
                val.count_ones(),
                16,
                "ShapeIdx[{}] = 0x{:06x} has {} bits set, expected 16",
                i,
                val,
                val.count_ones()
            );
        }
    }

    #[test]
    fn test_get_shape2idx_seed_shapes() {
        let tables = &*SHAPE_TABLES;
        let seeds: [i32; 4] = [0x0db6_6db, 0x1db6_db6, 0x16db_6db, 0x06db_db6];
        for &shp in &seeds {
            let idx = get_shape2idx(shp);
            assert!(
                idx >= 0,
                "Seed shape 0x{:07x} mapped to negative index {}",
                shp,
                idx
            );
            assert!(
                (idx as usize) < N_SHAPE,
                "Seed shape 0x{:07x} index {} out of range",
                shp,
                idx
            );
            assert_eq!(
                tables.shape_prun[idx as usize], 0,
                "Seed shape 0x{:07x} should have pruning value 0",
                shp
            );
        }
    }

    #[test]
    fn test_shape_prun_fully_filled() {
        let tables = &*SHAPE_TABLES;
        for (i, &val) in tables.shape_prun.iter().enumerate() {
            assert!(val >= 0, "ShapePrun[{}] = {} (unfilled)", i, val);
        }
    }

    #[test]
    fn test_shape_prun_opt_fully_filled() {
        let tables = &*SHAPE_TABLES;
        for (i, &val) in tables.shape_prun_opt.iter().enumerate() {
            assert!(val >= 0, "ShapePrunOpt[{}] = {} (unfilled)", i, val);
        }
    }

    #[test]
    fn test_move_tables_valid_indices() {
        let tables = &*SHAPE_TABLES;
        for i in 0..N_SHAPE {
            let twist_idx = tables.twist_move[i];
            assert!(
                twist_idx >= 0 && (twist_idx as usize) < N_SHAPE,
                "TwistMove[{}] = {} out of range",
                i,
                twist_idx
            );
            let top_idx = tables.top_move[i] >> 4;
            assert!(
                top_idx >= 0 && (top_idx as usize) < N_SHAPE,
                "TopMove[{}] >> 4 = {} out of range",
                i,
                top_idx
            );
            let bottom_idx = tables.bottom_move[i] >> 4;
            assert!(
                bottom_idx >= 0 && (bottom_idx as usize) < N_SHAPE,
                "BottomMove[{}] >> 4 = {} out of range",
                i,
                bottom_idx
            );
        }
    }

    #[test]
    fn test_twist_move_is_involution() {
        let tables = &*SHAPE_TABLES;
        for i in 0..N_SHAPE {
            let j = tables.twist_move[i] as usize;
            let k = tables.twist_move[j] as usize;
            assert_eq!(
                k, i,
                "Twist is not an involution: twist(twist({})) = {} != {}",
                i, k, i
            );
        }
    }

    #[test]
    fn test_top_move_cycles_back() {
        let tables = &*SHAPE_TABLES;
        for i in 0..N_SHAPE {
            let mut m = 0;
            let mut idx = i;
            loop {
                let entry = tables.top_move[idx];
                let inc = (entry & 0xf) as usize;
                idx = (entry >> 4) as usize;
                m += inc;
                if m == 12 {
                    break;
                }
                assert!(m < 12, "TopMove cycle exceeded 12 at start index {}", i);
            }
            assert_eq!(
                idx, i,
                "TopMove cycle from {} did not return to start (got {})",
                i, idx
            );
        }
    }

    #[test]
    fn test_bottom_move_cycles_back() {
        let tables = &*SHAPE_TABLES;
        for i in 0..N_SHAPE {
            let mut m = 0;
            let mut idx = i;
            loop {
                let entry = tables.bottom_move[idx];
                let inc = (entry & 0xf) as usize;
                idx = (entry >> 4) as usize;
                m += inc;
                if m == 12 {
                    break;
                }
                assert!(m < 12, "BottomMove cycle exceeded 12 at start index {}", i);
            }
            assert_eq!(
                idx, i,
                "BottomMove cycle from {} did not return to start (got {})",
                i, idx
            );
        }
    }

    #[test]
    fn test_solved_cube_in_prun_opt() {
        let tables = &*SHAPE_TABLES;
        let idx = solved_cube_shape_idx(&tables.shape_idx);
        assert!(idx >= 0);
        assert!((idx as usize) < N_SHAPE);
        assert_eq!(tables.shape_prun_opt[idx as usize], 0);
    }

    #[test]
    fn test_halflayer_values() {
        for &h in &HALFLAYER {
            assert!(
                h >= 0 && h <= 0x3f,
                "halflayer value 0x{:02x} out of range",
                h
            );
        }
        assert_eq!(HALFLAYER.len(), 13);
    }

    #[test]
    fn test_prun_depth_monotonic() {
        // Verify that pruning depths are reasonable (max depth should be small).
        let tables = &*SHAPE_TABLES;
        let max_prun = *tables.shape_prun.iter().max().unwrap();
        let max_prun_opt = *tables.shape_prun_opt.iter().max().unwrap();
        // Shape pruning depths should be modest (typically around 15-20 for face turn,
        // and similar for WCA metric).
        assert!(
            max_prun <= 30,
            "ShapePrun max depth {} seems too high",
            max_prun
        );
        assert!(
            max_prun_opt <= 30,
            "ShapePrunOpt max depth {} seems too high",
            max_prun_opt
        );
    }
}
