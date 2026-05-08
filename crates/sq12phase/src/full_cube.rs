// Square-1 full cube state
// Port of cs.sq12phase.FullCube
//
// This module intentionally uses i32 arithmetic and casts to match the
// Java original as closely as possible.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]

use rand::Rng;

use crate::shape::{self, SHAPE_TABLES};
use crate::square::{self, Square};

// ---------------------------------------------------------------------------
// Solved state constants
// ---------------------------------------------------------------------------

/// Solved state nibble values.
///
/// The 24 piece positions are packed as nibbles into four 24-bit integers:
///   ul (positions 0-5), ur (positions 6-11),
///   dl (positions 12-17), dr (positions 18-23).
///
/// Odd nibble = corner, even nibble = edge.  Each corner occupies two
/// adjacent positions with the same nibble value.
const SOLVED_UL: i32 = 0x01_1233;
const SOLVED_UR: i32 = 0x45_5677;
const SOLVED_DL: i32 = 0x99_8BBA;
const SOLVED_DR: i32 = 0xDD_CFFE;

// ---------------------------------------------------------------------------
// FullCube
// ---------------------------------------------------------------------------

/// Full Square-1 cube state.
///
/// State is stored as four 24-bit packed-nibble integers (`ul`, `ur`, `dl`,
/// `dr`) encoding the 24 piece slots, plus `ml` for the middle layer (0 or 1).
#[derive(Clone, Debug)]
pub struct FullCube {
    pub ul: i32,
    pub ur: i32,
    pub dl: i32,
    pub dr: i32,
    pub ml: i32,
}

impl Default for FullCube {
    fn default() -> Self {
        Self::new()
    }
}

impl FullCube {
    /// Create a solved cube.
    #[must_use]
    pub fn new() -> Self {
        Self {
            ul: SOLVED_UL,
            ur: SOLVED_UR,
            dl: SOLVED_DL,
            dr: SOLVED_DR,
            ml: 0,
        }
    }

    /// Check whether the cube is in the solved state.
    #[must_use]
    pub fn is_solved(&self) -> bool {
        self.ul == SOLVED_UL
            && self.ur == SOLVED_UR
            && self.dl == SOLVED_DL
            && self.dr == SOLVED_DR
            && self.ml == 0
    }

    /// Copy state from another cube.
    pub fn copy_from(&mut self, other: &Self) {
        self.ul = other.ul;
        self.ur = other.ur;
        self.dl = other.dl;
        self.dr = other.dr;
        self.ml = other.ml;
    }

    // -----------------------------------------------------------------------
    // Moves
    // -----------------------------------------------------------------------

    /// Apply a move to the cube.
    ///
    /// - `mv == 0`: twist (swap right-top half with left-bottom half, flip middle layer)
    /// - `mv > 0` (1..=11): rotate top layer clockwise by `mv` positions
    /// - `mv < 0` (-11..=-1): rotate bottom layer clockwise by `-mv` positions
    ///
    /// Internally the move value is multiplied by 4 (shift left 2) because
    /// each position occupies 4 bits (one nibble).
    pub fn do_move(&mut self, mv: i32) {
        let mv = mv << 2;

        if mv > 24 {
            // Top layer: rotate right (move > 6 in original units)
            let shift = 48 - mv;
            let temp = self.ul;
            self.ul = (self.ul >> shift | self.ur << (24 - shift)) & 0xFF_FFFF;
            self.ur = (self.ur >> shift | temp << (24 - shift)) & 0xFF_FFFF;
        } else if mv > 0 {
            // Top layer: rotate left (move 1..=6 in original units)
            let temp = self.ul;
            self.ul = (self.ul << mv | self.ur >> (24 - mv)) & 0xFF_FFFF;
            self.ur = (self.ur << mv | temp >> (24 - mv)) & 0xFF_FFFF;
        } else if mv == 0 {
            // Twist: swap ur and dl, flip middle layer
            std::mem::swap(&mut self.ur, &mut self.dl);
            self.ml = 1 - self.ml;
        } else if mv >= -24 {
            // Bottom layer: rotate left (|move| 1..=6 in original units)
            let shift = -mv;
            let temp = self.dl;
            self.dl = (self.dl << shift | self.dr >> (24 - shift)) & 0xFF_FFFF;
            self.dr = (self.dr << shift | temp >> (24 - shift)) & 0xFF_FFFF;
        } else {
            // Bottom layer: rotate right (|move| > 6)
            let shift = 48 + mv;
            let temp = self.dl;
            self.dl = (self.dl >> shift | self.dr << (24 - shift)) & 0xFF_FFFF;
            self.dr = (self.dr >> shift | temp << (24 - shift)) & 0xFF_FFFF;
        }
    }

    // -----------------------------------------------------------------------
    // Piece access
    // -----------------------------------------------------------------------

    /// Read the nibble at position `idx` (0..23).
    #[must_use]
    fn piece_at(&self, idx: usize) -> u8 {
        let ret = if idx < 6 {
            self.ul >> ((5 - idx) << 2)
        } else if idx < 12 {
            self.ur >> ((11 - idx) << 2)
        } else if idx < 18 {
            self.dl >> ((17 - idx) << 2)
        } else {
            self.dr >> ((23 - idx) << 2)
        };
        (ret & 0x0f) as u8
    }

    /// Write a nibble value at position `idx` (0..23), or set `ml` at idx 24.
    pub fn set_piece(&mut self, idx: usize, value: i32) {
        if idx < 6 {
            let shift = (5 - idx) << 2;
            self.ul = (self.ul & !(0xf << shift)) | (value << shift);
        } else if idx < 12 {
            let shift = (11 - idx) << 2;
            self.ur = (self.ur & !(0xf << shift)) | (value << shift);
        } else if idx < 18 {
            let shift = (17 - idx) << 2;
            self.dl = (self.dl & !(0xf << shift)) | (value << shift);
        } else if idx < 24 {
            let shift = (23 - idx) << 2;
            self.dr = (self.dr & !(0xf << shift)) | (value << shift);
        } else {
            self.ml = value;
        }
    }

    // -----------------------------------------------------------------------
    // Phase 1 coordinate: shape index
    // -----------------------------------------------------------------------

    /// Compute the shape index for phase 1.
    ///
    /// Extracts which positions hold corners (bit=1) vs edges (bit=0) from
    /// each quarter, packs them with parity, and looks up in the Shape table.
    #[must_use]
    pub fn get_shape_idx(&self) -> i32 {
        let urx = compress_shape_bits(self.ur);
        let ulx = compress_shape_bits(self.ul);
        let drx = compress_shape_bits(self.dr);
        let dlx = compress_shape_bits(self.dl);

        shape::get_shape2_idx(
            self.get_parity() << 24 | ulx << 18 | urx << 12 | dlx << 6 | drx,
        )
    }

    /// Compute the parity of the piece permutation.
    fn get_parity(&self) -> i32 {
        // Collect the 16 distinct piece identities (collapsing adjacent
        // identical nibbles that form a corner pair).
        let mut arr = [0u8; 16];
        arr[0] = self.piece_at(0);
        let mut cnt = 0usize;
        for i in 1..24 {
            let p = self.piece_at(i);
            if p != arr[cnt] {
                cnt += 1;
                arr[cnt] = p;
            }
        }
        // Count inversions mod 2
        let mut p = 0;
        for a in 0..16 {
            for b in (a + 1)..16 {
                if arr[a] > arr[b] {
                    p ^= 1;
                }
            }
        }
        p
    }

    // -----------------------------------------------------------------------
    // Phase 2 coordinate: Square extraction
    // -----------------------------------------------------------------------

    /// Fill a `Square` with the phase-2 coordinates of this cube.
    ///
    /// Extracts corner permutation, edge permutation, edge-first flags,
    /// and middle-layer state.
    #[allow(clippy::needless_range_loop)]
    pub fn get_square(&self, sq: &mut Square) {
        let mut prm = [0u8; 8];

        // Corner permutation: corners sit at positions 1, 4, 7, 10, 13, 16, 19, 22
        for a in 0..8 {
            prm[a] = self.piece_at(a * 3 + 1) >> 1;
        }
        sq.corner_perm = square::get8perm(&prm);

        // Edge permutation
        sq.top_edge_first = self.piece_at(0) == self.piece_at(1);
        let mut a = if sq.top_edge_first { 2 } else { 0 };
        for b in 0..4 {
            prm[b] = self.piece_at(a) >> 1;
            a += 3;
        }

        sq.bot_edge_first = self.piece_at(12) == self.piece_at(13);
        a = if sq.bot_edge_first { 14 } else { 12 };
        for b in 4..8 {
            prm[b] = self.piece_at(a) >> 1;
            a += 3;
        }
        sq.edge_perm = square::get8perm(&prm);
        sq.ml = self.ml;
    }

    // -----------------------------------------------------------------------
    // Random cube generation
    // -----------------------------------------------------------------------

    /// Generate a uniformly random Square-1 state.
    ///
    /// Uses the shape table to pick a random valid shape, then fills in
    /// corners and edges by random permutation.
    #[must_use]
    pub fn random_cube(rng: &mut impl Rng) -> Self {
        // Force table initialization
        let tables = &*SHAPE_TABLES;

        let shape = tables.shape_idx[rng.random_range(0..shape::N_SHAPE_IDX)];
        let mut f = Self::new();

        // Corners: 8 corners, nibble values are (index << 1 | 1)
        // Packed into a 32-bit register: 0x01234567 << 1 | 0x11111111
        let mut corner: u32 = 0x0123_4567_u32 << 1 | 0x1111_1111;
        // Edges: 8 edges, nibble values are (index << 1)
        let mut edge: u32 = 0x0123_4567_u32 << 1;
        let mut n_corner: usize = 8;
        let mut n_edge: usize = 8;

        let mut i = 0;
        while i < 24 {
            if ((shape >> i) & 1) == 0 {
                // Edge position
                let rnd = (rng.random_range(0..n_edge) as u32) << 2;
                let piece = (edge >> rnd) & 0xf;
                f.set_piece(23 - i, piece as i32);
                let m = (1u32 << rnd).wrapping_sub(1);
                edge = (edge & m) + ((edge >> 4) & !m);
                n_edge -= 1;
            } else {
                // Corner position (occupies two slots)
                let rnd = (rng.random_range(0..n_corner) as u32) << 2;
                let piece = (corner >> rnd) & 0xf;
                f.set_piece(23 - i, piece as i32);
                f.set_piece(22 - i, piece as i32);
                let m = (1u32 << rnd).wrapping_sub(1);
                corner = (corner & m) + ((corner >> 4) & !m);
                n_corner -= 1;
                i += 1; // extra increment for the second corner slot
            }
            i += 1;
        }

        f.ml = rng.random_range(0..2_i32);
        f
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Extract the corner/edge shape bits from a 24-bit quarter.
///
/// Each nibble's LSB indicates corner (1) vs edge (0).  This function
/// compresses those 6 bits into a 6-bit value.
fn compress_shape_bits(quarter: i32) -> i32 {
    let mut x = quarter & 0x11_1111;
    x |= x >> 3;
    x |= x >> 6;
    (x & 0xf) | ((x >> 12) & 0x30)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn solved_cube_is_solved() {
        let c = FullCube::new();
        assert!(c.is_solved());
    }

    #[test]
    fn twist_twice_identity() {
        let mut c = FullCube::new();
        c.do_move(0);
        c.do_move(0);
        assert!(c.is_solved(), "two twists should return to solved");
    }

    #[test]
    fn top_12_identity() {
        // Rotating the top layer by 12 total should be the identity
        // (6+6, since each "move" rotates by the specified amount).
        let mut c = FullCube::new();
        c.do_move(6);
        c.do_move(6);
        assert!(c.is_solved(), "top 6+6 should be identity");
    }

    #[test]
    fn bottom_12_identity() {
        let mut c = FullCube::new();
        c.do_move(-6);
        c.do_move(-6);
        assert!(c.is_solved(), "bottom 6+6 should be identity");
    }

    #[test]
    fn top_move_not_solved() {
        let mut c = FullCube::new();
        c.do_move(1);
        assert!(!c.is_solved(), "single top move should not be solved");
    }

    #[test]
    fn bottom_move_not_solved() {
        let mut c = FullCube::new();
        c.do_move(-1);
        assert!(!c.is_solved(), "single bottom move should not be solved");
    }

    #[test]
    fn twist_not_solved() {
        let mut c = FullCube::new();
        c.do_move(0);
        assert!(!c.is_solved(), "single twist should not be solved");
    }

    #[test]
    fn piece_at_solved() {
        let c = FullCube::new();
        // Verify a few piece values from the solved state
        assert_eq!(c.piece_at(0), 0x0); // first nibble of ul=0x011233
        assert_eq!(c.piece_at(1), 0x1);
        assert_eq!(c.piece_at(2), 0x1);
        assert_eq!(c.piece_at(5), 0x3);
        assert_eq!(c.piece_at(6), 0x4); // first nibble of ur=0x455677
    }

    #[test]
    fn solved_shape_idx() {
        // Force table init
        let _ = &*SHAPE_TABLES;
        let c = FullCube::new();
        let idx = c.get_shape_idx();
        // Solved shape should have pruning distance 0
        let tables = &*SHAPE_TABLES;
        assert_eq!(
            tables.shape_prun[idx as usize], 0,
            "solved cube should have shape pruning value 0"
        );
    }

    #[test]
    fn random_cube_valid_shape() {
        let mut rng = StdRng::seed_from_u64(42);
        let tables = &*SHAPE_TABLES;

        for _ in 0..20 {
            let c = FullCube::random_cube(&mut rng);
            let idx = c.get_shape_idx();
            assert!(
                (idx as usize) < shape::N_SHAPE,
                "random cube shape index out of range: {idx}"
            );
            assert!(
                tables.shape_prun[idx as usize] >= 0,
                "random cube has unfilled shape pruning entry"
            );
        }
    }

    #[test]
    fn random_cube_pieces_count() {
        // A valid Square-1 state should have exactly 8 corner nibbles
        // (odd values) and 8 edge nibbles (even values), with each
        // identity appearing exactly the right number of times.
        let mut rng = StdRng::seed_from_u64(123);

        for _ in 0..10 {
            let c = FullCube::random_cube(&mut rng);
            let mut corners = 0;
            let mut edges = 0;
            for i in 0..24 {
                let p = c.piece_at(i);
                if p & 1 == 1 {
                    corners += 1;
                } else {
                    edges += 1;
                }
            }
            // 8 corners * 2 slots each = 16 corner slots, 8 edge slots
            assert_eq!(corners, 16, "expected 16 corner slots");
            assert_eq!(edges, 8, "expected 8 edge slots");
        }
    }

    #[test]
    fn get_square_solved() {
        let c = FullCube::new();
        let mut sq = Square::default();
        c.get_square(&mut sq);
        assert_eq!(sq.corner_perm, 0, "solved corner perm should be 0");
        assert_eq!(sq.edge_perm, 0, "solved edge perm should be 0");
        assert_eq!(sq.ml, 0, "solved ml should be 0");
    }

    #[test]
    fn copy_from_works() {
        let mut rng = StdRng::seed_from_u64(99);
        let c = FullCube::random_cube(&mut rng);
        let mut d = FullCube::new();
        d.copy_from(&c);
        assert_eq!(d.ul, c.ul);
        assert_eq!(d.ur, c.ur);
        assert_eq!(d.dl, c.dl);
        assert_eq!(d.dr, c.dr);
        assert_eq!(d.ml, c.ml);
    }
}
