#![allow(clippy::all)]
//! Port of cs.threephase.FullCube
//!
//! Combines EdgeCube, CenterCube, CornerCube into a single 4x4 cube state.
//! Supports facelet construction, lazy move application, and facelet output.

use rand::Rng;

use crate::center_cube::CenterCube;
use crate::center1;
use crate::corner_cube::CornerCube;
use crate::edge_cube::EdgeCube;
use crate::moves;
use crate::moves::*;

// ─── facelet constants ───────────────────────────────────────────────

/// Center facelet indices in the 4x4 sub-facelet numbering.
///
/// `CENTER_FACELET[i]` gives the sub-facelet index for center position `i`.
/// Dividing by 16 gives the face index (0=U, 1=R, 2=F, 3=D, 4=L, 5=B).
pub const CENTER_FACELET: [u8; 24] = [
    FU5, FU6, FUA, FU9, // U centers
    FD5, FD6, FDA, FD9, // D centers
    FF5, FF6, FFA, FF9, // F centers
    FB5, FB6, FBA, FB9, // B centers
    FR5, FR6, FRA, FR9, // R centers
    FL5, FL6, FLA, FL9, // L centers
];

/// Edge facelet indices: each edge has 2 sub-facelets.
const EDGE_FACELET: [[u8; 2]; 24] = [
    [FUD, FF1],
    [FU4, FL1],
    [FU2, FB1],
    [FUB, FR1],
    [FDD, FBE],
    [FD4, FLE],
    [FD2, FFE],
    [FDB, FRE],
    [FLB, FF8],
    [FL4, FB7],
    [FRB, FB8],
    [FR4, FF7],
    [FF2, FUE],
    [FL2, FU8],
    [FB2, FU1],
    [FR2, FU7],
    [FBD, FDE],
    [FLD, FD8],
    [FFD, FD1],
    [FRD, FD7],
    [FF4, FL7],
    [FBB, FL8],
    [FB4, FR7],
    [FFB, FR8],
];

/// Corner facelet indices: each corner has 3 sub-facelets.
const CORNER_FACELET: [[u8; 3]; 8] = [
    [FUF, FR0, FF3], // UFR
    [FUC, FF0, FL3], // UFL
    [FU0, FL0, FB3], // ULB
    [FU3, FB0, FR3], // UBR
    [FD3, FFF, FRC], // DFR (note: FFF = 0x2f)
    [FD0, FLF, FFC], // DLF
    [FDC, FBF, FLC], // DBL
    [FDF, FRF, FBC], // DRB
];

// ─── move→rotation table (from FullCube.java) ────────────────────────

const MOVE2ROT: [usize; 9] = [35, 1, 34, 2, 4, 6, 22, 5, 19];

// ─── FullCube ────────────────────────────────────────────────────────

pub struct FullCube {
    edge: EdgeCube,
    center: CenterCube,
    corner: CornerCube,

    pub value: i32,
    pub add1: bool,
    pub length1: i32,
    pub length2: i32,
    pub length3: i32,

    pub move_buffer: [u8; 60],
    pub move_length: usize,
    edge_avail: usize,
    center_avail: usize,
    corner_avail: usize,

    pub sym: usize,
}

impl FullCube {
    pub fn new() -> Self {
        FullCube {
            edge: EdgeCube::new(),
            center: CenterCube::new(),
            corner: CornerCube::new(),
            value: 0,
            add1: false,
            length1: 0,
            length2: 0,
            length3: 0,
            move_buffer: [0u8; 60],
            move_length: 0,
            edge_avail: 0,
            center_avail: 0,
            corner_avail: 0,
            sym: 0,
        }
    }

    /// Construct from a 96-byte facelet array (values 0..5 = URFDLB).
    pub fn from_facelets(f: &[u8; 96]) -> Self {
        let mut fc = FullCube::new();

        // Centers
        for i in 0..24 {
            fc.center.ct[i] = f[CENTER_FACELET[i] as usize];
        }

        // Edges
        for i in 0..24 {
            for j in 0u8..24 {
                if f[EDGE_FACELET[i][0] as usize] == EDGE_FACELET[j as usize][0] / 16
                    && f[EDGE_FACELET[i][1] as usize] == EDGE_FACELET[j as usize][1] / 16
                {
                    fc.edge.ep[i] = j;
                }
            }
        }

        // Corners
        for i in 0u8..8 {
            let mut ori: u8 = 0;
            for o in 0..3u8 {
                let fval = f[CORNER_FACELET[i as usize][o as usize] as usize];
                // u0/16 = 0x00/16 = 0 (U face) or d0/16 = 0x30/16 = 3 (D face)
                if fval == FU0 / 16 || fval == FD0 / 16 {
                    ori = o;
                    break;
                }
            }
            let col1 = f[CORNER_FACELET[i as usize][((ori + 1) % 3) as usize] as usize];
            let col2 = f[CORNER_FACELET[i as usize][((ori + 2) % 3) as usize] as usize];

            for j in 0u8..8 {
                if col1 == CORNER_FACELET[j as usize][1] / 16
                    && col2 == CORNER_FACELET[j as usize][2] / 16
                {
                    fc.corner.cp[i as usize] = j;
                    fc.corner.co[i as usize] = ori % 3;
                    break;
                }
            }
        }

        fc
    }

    /// Construct from a random state.
    pub fn random<R: Rng>(rng: &mut R) -> Self {
        FullCube {
            edge: EdgeCube::new_random(rng),
            center: CenterCube::new_random(rng),
            corner: CornerCube::new_random(rng),
            value: 0,
            add1: false,
            length1: 0,
            length2: 0,
            length3: 0,
            move_buffer: [0u8; 60],
            move_length: 0,
            edge_avail: 0,
            center_avail: 0,
            corner_avail: 0,
            sym: 0,
        }
    }

    /// Construct from a move sequence.
    pub fn from_moves(moveseq: &[usize]) -> Self {
        let mut fc = FullCube::new();
        for &m in moveseq {
            fc.do_move(m);
        }
        fc
    }

    pub fn copy_from(&mut self, c: &FullCube) {
        self.edge.copy_from(&c.edge);
        self.center.copy_from(&c.center);
        self.corner.copy_from(&c.corner);
        self.value = c.value;
        self.add1 = c.add1;
        self.length1 = c.length1;
        self.length2 = c.length2;
        self.length3 = c.length3;
        self.sym = c.sym;
        self.move_buffer = c.move_buffer;
        self.move_length = c.move_length;
        self.edge_avail = c.edge_avail;
        self.center_avail = c.center_avail;
        self.corner_avail = c.corner_avail;
    }

    pub fn clone_cube(&self) -> Self {
        let mut c = FullCube::new();
        c.copy_from(self);
        c
    }

    pub fn check_edge(&mut self) -> bool {
        self.get_edge().check_edge()
    }

    /// Write a solution string from the recorded moves.
    pub fn get_move_string(&mut self, inverse: bool, rotation: bool) -> String {
        let tables = &*center1::TABLES;
        let symmove = &tables.symmove;
        let symmult = &tables.symmult;
        let syminv = &tables.syminv;

        let skip = if self.add1 { 2 } else { 0 };
        let mut fixed_moves = Vec::with_capacity(self.move_length.saturating_sub(skip));
        for i in 0..self.length1 as usize {
            fixed_moves.push(self.move_buffer[i] as usize);
        }
        let mut sym = self.sym;
        for i in (self.length1 as usize + skip)..self.move_length {
            let m = self.move_buffer[i] as usize;
            let sm = symmove[sym][m] as usize;
            if sm >= moves::DWX1 as usize {
                fixed_moves.push(sm - 9);
                let rot = MOVE2ROT[sm - moves::DWX1 as usize];
                sym = symmult[sym][rot] as usize;
            } else {
                fixed_moves.push(sm);
            }
        }
        let finish_sym = symmult[syminv[sym] as usize]
            [center1::Center1::get_solved_sym(&self.get_center()) as usize]
            as usize;

        let mut sb = String::new();
        let mut sym_cur = finish_sym;

        if inverse {
            for i in (0..fixed_moves.len()).rev() {
                let mut mv = fixed_moves[i];
                mv = mv / 3 * 3 + (2 - mv % 3);
                let sm = symmove[sym_cur][mv] as usize;
                if sm >= moves::DWX1 as usize {
                    sb.push_str(moves::MOVE2STR[sm - 9]);
                    sb.push(' ');
                    let rot = MOVE2ROT[sm - moves::DWX1 as usize];
                    sym_cur = symmult[sym_cur][rot] as usize;
                } else {
                    sb.push_str(moves::MOVE2STR[sm]);
                    sb.push(' ');
                }
            }
            if rotation {
                sb.push_str(center1::ROT2STR[syminv[sym_cur] as usize]);
                sb.push(' ');
            }
        } else {
            for &fm in &fixed_moves {
                sb.push_str(moves::MOVE2STR[fm]);
                sb.push(' ');
            }
            if rotation {
                sb.push_str(center1::ROT2STR[finish_sym]);
            }
        }
        sb
    }

    /// Convert solved-down-to-3x3 cube to a 54-char facelet string for min2phase.
    pub fn to_333_facelet(&mut self) -> String {
        let mut ret = [0u8; 54];
        self.get_edge().fill_333_facelet(&mut ret);
        self.get_center().fill_333_facelet(&mut ret);
        self.get_corner().fill_333_facelet(&mut ret);
        // The fill methods write ASCII face chars (b'U', b'R', b'F', b'D', b'L', b'B').
        ret.iter().map(|&b| b as char).collect()
    }

    /// Write state to a 96-byte facelet array.
    pub fn to_facelet(&mut self, f: &mut [u8; 96]) {
        // Flush lazy moves
        self.get_edge();
        self.get_center();
        self.get_corner();

        // Centers
        for i in 0..24 {
            f[CENTER_FACELET[i] as usize] = self.center.ct[i];
        }
        // Edges
        for i in 0..24 {
            let ep = self.edge.ep[i] as usize;
            f[EDGE_FACELET[i][0] as usize] = EDGE_FACELET[ep][0] / 16;
            f[EDGE_FACELET[i][1] as usize] = EDGE_FACELET[ep][1] / 16;
        }
        // Corners
        for c in 0u8..8 {
            let j = self.corner.cp[c as usize] as usize;
            let ori = self.corner.co[c as usize] as usize;
            for n in 0..3usize {
                f[CORNER_FACELET[c as usize][(n + ori) % 3] as usize] = CORNER_FACELET[j][n] / 16;
            }
        }
    }

    /// Record a move in the buffer (lazy application).
    pub fn push_move(&mut self, m: usize) {
        self.move_buffer[self.move_length] = m as u8;
        self.move_length += 1;
    }

    /// Apply a move immediately to all sub-cubes.
    pub fn do_move(&mut self, m: usize) {
        self.get_edge().apply_move(m);
        self.get_center().apply_move(m);
        self.get_corner().apply_move(m % 18);
    }

    // ── lazy-eval getters ──

    pub fn get_edge(&mut self) -> &mut EdgeCube {
        while self.edge_avail < self.move_length {
            let m = self.move_buffer[self.edge_avail] as usize;
            self.edge.apply_move(m);
            self.edge_avail += 1;
        }
        &mut self.edge
    }

    pub fn get_center(&mut self) -> &mut CenterCube {
        while self.center_avail < self.move_length {
            let m = self.move_buffer[self.center_avail] as usize;
            self.center.apply_move(m);
            self.center_avail += 1;
        }
        &mut self.center
    }

    pub fn get_corner(&mut self) -> &mut CornerCube {
        while self.corner_avail < self.move_length {
            let m = (self.move_buffer[self.corner_avail] as usize) % 18;
            self.corner.apply_move(m);
            self.corner_avail += 1;
        }
        &mut self.corner
    }
}

// ─── Ordering for priority queue ─────────────────────────────────────

impl PartialEq for FullCube {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for FullCube {}

impl PartialOrd for FullCube {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FullCube {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}

// ─── tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// The solved 4x4 facelet string: 16 U's, 16 R's, 16 F's, 16 D's, 16 L's, 16 B's.
    fn solved_facelets() -> [u8; 96] {
        let mut f = [0u8; 96];
        for i in 0..96 {
            f[i] = (i / 16) as u8; // 0=U,1=R,2=F,3=D,4=L,5=B
        }
        f
    }

    #[test]
    fn test_solved_cube_roundtrip() {
        let f = solved_facelets();
        let mut fc = FullCube::from_facelets(&f);
        let mut out = [0u8; 96];
        fc.to_facelet(&mut out);
        assert_eq!(f, out);
    }

    #[test]
    fn test_random_cube_creates_valid_state() {
        let mut rng = rand::rng();
        let mut fc = FullCube::random(&mut rng);
        // The edge permutation should contain all 24 distinct values.
        let edge = fc.get_edge();
        let mut seen = [false; 24];
        for &e in &edge.ep {
            seen[e as usize] = true;
        }
        assert!(seen.iter().all(|&b| b));
    }

    #[test]
    fn every_move_has_order_four() {
        let solved = solved_facelets();
        for m in 0..36 {
            let mut fc = FullCube::from_moves(&[m, m, m, m]);
            let mut out = [0u8; 96];
            fc.to_facelet(&mut out);
            assert_eq!(out, solved, "move {}", MOVE2STR[m]);
            let mut once = FullCube::from_moves(&[m]);
            once.to_facelet(&mut out);
            assert_ne!(out, solved, "move {} does nothing", MOVE2STR[m]);
        }
    }

    #[test]
    fn scrambled_facelet_roundtrip() {
        let mut rng = rand::rng();
        for _ in 0..100 {
            let moves: Vec<usize> = (0..40).map(|_| rng.random_range(0..36)).collect();
            let mut fc = FullCube::from_moves(&moves);
            let mut f = [0u8; 96];
            fc.to_facelet(&mut f);
            let mut back = FullCube::from_facelets(&f);
            let mut g = [0u8; 96];
            back.to_facelet(&mut g);
            assert_eq!(f, g);
        }
    }

    #[test]
    fn test_new_is_solved() {
        let mut fc = FullCube::new();
        // Edge should be identity
        let edge = fc.get_edge();
        for i in 0..24 {
            assert_eq!(edge.ep[i], i as u8);
        }
    }
}
