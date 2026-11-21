#![allow(clippy::all)]
//! Port of cs.threephase.Moves
//!
//! Move constants, string representations, standard-move mappings,
//! and move-pruning tables for the 4x4 three-phase solver.

use std::sync::LazyLock;

// ---------------------------------------------------------------------------
// Facelet-level move indices (U1..B9, 54 values)
// ---------------------------------------------------------------------------

pub const U1: u8 = 0;
pub const U2: u8 = 1;
pub const U3: u8 = 2;
pub const U4: u8 = 3;
pub const U5: u8 = 4;
pub const U6: u8 = 5;
pub const U7: u8 = 6;
pub const U8: u8 = 7;
pub const U9: u8 = 8;
pub const R1: u8 = 9;
pub const R2: u8 = 10;
pub const R3: u8 = 11;
pub const R4: u8 = 12;
pub const R5: u8 = 13;
pub const R6: u8 = 14;
pub const R7: u8 = 15;
pub const R8: u8 = 16;
pub const R9: u8 = 17;
pub const F1: u8 = 18;
pub const F2: u8 = 19;
pub const F3: u8 = 20;
pub const F4: u8 = 21;
pub const F5: u8 = 22;
pub const F6: u8 = 23;
pub const F7: u8 = 24;
pub const F8: u8 = 25;
pub const F9: u8 = 26;
pub const D1: u8 = 27;
pub const D2: u8 = 28;
pub const D3: u8 = 29;
pub const D4: u8 = 30;
pub const D5: u8 = 31;
pub const D6: u8 = 32;
pub const D7: u8 = 33;
pub const D8: u8 = 34;
pub const D9: u8 = 35;
pub const L1: u8 = 36;
pub const L2: u8 = 37;
pub const L3: u8 = 38;
pub const L4: u8 = 39;
pub const L5: u8 = 40;
pub const L6: u8 = 41;
pub const L7: u8 = 42;
pub const L8: u8 = 43;
pub const L9: u8 = 44;
pub const B1: u8 = 45;
pub const B2: u8 = 46;
pub const B3: u8 = 47;
pub const B4: u8 = 48;
pub const B5: u8 = 49;
pub const B6: u8 = 50;
pub const B7: u8 = 51;
pub const B8: u8 = 52;
pub const B9: u8 = 53;

// ---------------------------------------------------------------------------
// Sub-facelet indices (u0..bf, 96 values, packed as face*16 + index)
// ---------------------------------------------------------------------------

pub const FU0: u8 = 0x0;
pub const FU1: u8 = 0x1;
pub const FU2: u8 = 0x2;
pub const FU3: u8 = 0x3;
pub const FU4: u8 = 0x4;
pub const FU5: u8 = 0x5;
pub const FU6: u8 = 0x6;
pub const FU7: u8 = 0x7;
pub const FU8: u8 = 0x8;
pub const FU9: u8 = 0x9;
pub const FUA: u8 = 0xa;
pub const FUB: u8 = 0xb;
pub const FUC: u8 = 0xc;
pub const FUD: u8 = 0xd;
pub const FUE: u8 = 0xe;
pub const FUF: u8 = 0xf;
pub const FR0: u8 = 0x10;
pub const FR1: u8 = 0x11;
pub const FR2: u8 = 0x12;
pub const FR3: u8 = 0x13;
pub const FR4: u8 = 0x14;
pub const FR5: u8 = 0x15;
pub const FR6: u8 = 0x16;
pub const FR7: u8 = 0x17;
pub const FR8: u8 = 0x18;
pub const FR9: u8 = 0x19;
pub const FRA: u8 = 0x1a;
pub const FRB: u8 = 0x1b;
pub const FRC: u8 = 0x1c;
pub const FRD: u8 = 0x1d;
pub const FRE: u8 = 0x1e;
pub const FRF: u8 = 0x1f;
pub const FF0: u8 = 0x20;
pub const FF1: u8 = 0x21;
pub const FF2: u8 = 0x22;
pub const FF3: u8 = 0x23;
pub const FF4: u8 = 0x24;
pub const FF5: u8 = 0x25;
pub const FF6: u8 = 0x26;
pub const FF7: u8 = 0x27;
pub const FF8: u8 = 0x28;
pub const FF9: u8 = 0x29;
pub const FFA: u8 = 0x2a;
pub const FFB: u8 = 0x2b;
pub const FFC: u8 = 0x2c;
pub const FFD: u8 = 0x2d;
pub const FFE: u8 = 0x2e;
pub const FFF: u8 = 0x2f;
pub const FD0: u8 = 0x30;
pub const FD1: u8 = 0x31;
pub const FD2: u8 = 0x32;
pub const FD3: u8 = 0x33;
pub const FD4: u8 = 0x34;
pub const FD5: u8 = 0x35;
pub const FD6: u8 = 0x36;
pub const FD7: u8 = 0x37;
pub const FD8: u8 = 0x38;
pub const FD9: u8 = 0x39;
pub const FDA: u8 = 0x3a;
pub const FDB: u8 = 0x3b;
pub const FDC: u8 = 0x3c;
pub const FDD: u8 = 0x3d;
pub const FDE: u8 = 0x3e;
pub const FDF: u8 = 0x3f;
pub const FL0: u8 = 0x40;
pub const FL1: u8 = 0x41;
pub const FL2: u8 = 0x42;
pub const FL3: u8 = 0x43;
pub const FL4: u8 = 0x44;
pub const FL5: u8 = 0x45;
pub const FL6: u8 = 0x46;
pub const FL7: u8 = 0x47;
pub const FL8: u8 = 0x48;
pub const FL9: u8 = 0x49;
pub const FLA: u8 = 0x4a;
pub const FLB: u8 = 0x4b;
pub const FLC: u8 = 0x4c;
pub const FLD: u8 = 0x4d;
pub const FLE: u8 = 0x4e;
pub const FLF: u8 = 0x4f;
pub const FB0: u8 = 0x50;
pub const FB1: u8 = 0x51;
pub const FB2: u8 = 0x52;
pub const FB3: u8 = 0x53;
pub const FB4: u8 = 0x54;
pub const FB5: u8 = 0x55;
pub const FB6: u8 = 0x56;
pub const FB7: u8 = 0x57;
pub const FB8: u8 = 0x58;
pub const FB9: u8 = 0x59;
pub const FBA: u8 = 0x5a;
pub const FBB: u8 = 0x5b;
pub const FBC: u8 = 0x5c;
pub const FBD: u8 = 0x5d;
pub const FBE: u8 = 0x5e;
pub const FBF: u8 = 0x5f;

// ---------------------------------------------------------------------------
// Face axis indices
// ---------------------------------------------------------------------------

pub const FACE_U: i32 = 0;
pub const FACE_R: i32 = 1;
pub const FACE_F: i32 = 2;
pub const FACE_D: i32 = 3;
pub const FACE_L: i32 = 4;
pub const FACE_B: i32 = 5;

/// Face axis indices as `usize` (for use in array indexing).
pub const U_FACE: usize = 0;
pub const R_FACE: usize = 1;
pub const F_FACE: usize = 2;
pub const D_FACE: usize = 3;
pub const L_FACE: usize = 4;
pub const B_FACE: usize = 5;

// ---------------------------------------------------------------------------
// Outer-block turn (OBT) move indices  (Ux1..bx3, 36 moves + eom sentinel)
// ---------------------------------------------------------------------------

pub const UX1: i32 = 0;
pub const UX2: i32 = 1;
pub const UX3: i32 = 2;
pub const RX1: i32 = 3;
pub const RX2: i32 = 4;
pub const RX3: i32 = 5;
pub const FX1: i32 = 6;
pub const FX2: i32 = 7;
pub const FX3: i32 = 8;
pub const DX1: i32 = 9;
pub const DX2: i32 = 10;
pub const DX3: i32 = 11;
pub const LX1: i32 = 12;
pub const LX2: i32 = 13;
pub const LX3: i32 = 14;
pub const BX1: i32 = 15;
pub const BX2: i32 = 16;
pub const BX3: i32 = 17;

// Wide moves
pub const UWX1: i32 = 18;
pub const UWX2: i32 = 19;
pub const UWX3: i32 = 20;
pub const RWX1: i32 = 21;
pub const RWX2: i32 = 22;
pub const RWX3: i32 = 23;
pub const FWX1: i32 = 24;
pub const FWX2: i32 = 25;
pub const FWX3: i32 = 26;
pub const DWX1: i32 = 27;
pub const DWX2: i32 = 28;
pub const DWX3: i32 = 29;
pub const LWX1: i32 = 30;
pub const LWX2: i32 = 31;
pub const LWX3: i32 = 32;
pub const BWX1: i32 = 33;
pub const BWX2: i32 = 34;
pub const BWX3: i32 = 35;

/// End-of-moves sentinel.
pub const EOM: i32 = 36;

// ---------------------------------------------------------------------------
// Move-to-string tables
// ---------------------------------------------------------------------------

/// Human-readable string for each move index 0..35.
pub const MOVE2STR: [&str; 36] = [
    "U  ", "U2 ", "U' ", "R  ", "R2 ", "R' ", "F  ", "F2 ", "F' ", "D  ", "D2 ", "D' ", "L  ",
    "L2 ", "L' ", "B  ", "B2 ", "B' ", "Uw ", "Uw2", "Uw'", "Rw ", "Rw2", "Rw'", "Fw ", "Fw2",
    "Fw'", "Dw ", "Dw2", "Dw'", "Lw ", "Lw2", "Lw'", "Bw ", "Bw2", "Bw'",
];

/// Human-readable string for the *inverse* of each move index 0..35.
pub const MOVE_ISTR: [&str; 36] = [
    "U' ", "U2 ", "U  ", "R' ", "R2 ", "R  ", "F' ", "F2 ", "F  ", "D' ", "D2 ", "D  ", "L' ",
    "L2 ", "L  ", "B' ", "B2 ", "B  ", "Uw'", "Uw2", "Uw ", "Rw'", "Rw2", "Rw ", "Fw'", "Fw2",
    "Fw ", "Dw'", "Dw2", "Dw ", "Lw'", "Lw2", "Lw ", "Bw'", "Bw2", "Bw ",
];

// ---------------------------------------------------------------------------
// Phase-2 / Phase-3 move subsets and mappings
// ---------------------------------------------------------------------------

/// Phase-2 move set (29 entries, terminated by `EOM`).
pub const MOVE2STD: [i32; 29] = [
    UX1, UX2, UX3, RX1, RX2, RX3, FX1, FX2, FX3, DX1, DX2, DX3, LX1, LX2, LX3, BX1, BX2, BX3, UWX2,
    RWX1, RWX2, RWX3, FWX2, DWX2, LWX1, LWX2, LWX3, BWX2, EOM,
];

/// Phase-3 move set (21 entries, terminated by `EOM`).
pub const MOVE3STD: [i32; 21] = [
    UX1, UX2, UX3, RX2, FX1, FX2, FX3, DX1, DX2, DX3, LX2, BX1, BX2, BX3, UWX2, RWX2, FWX2, DWX2,
    LWX2, BWX2, EOM,
];

/// Maps a standard move index (0..36) back to its position in [`MOVE2STD`].
pub static STD2MOVE: LazyLock<[i32; 37]> = LazyLock::new(|| {
    let mut arr = [0i32; 37];
    for i in 0..29 {
        arr[MOVE2STD[i] as usize] = i as i32;
    }
    arr
});

/// Maps a standard move index (0..36) back to its position in [`MOVE3STD`].
pub static STD3MOVE: LazyLock<[i32; 37]> = LazyLock::new(|| {
    let mut arr = [0i32; 37];
    for i in 0..21 {
        arr[MOVE3STD[i] as usize] = i as i32;
    }
    arr
});

// ---------------------------------------------------------------------------
// Move-checking (pruning) tables
// ---------------------------------------------------------------------------

/// `ckmv[i][j]` is `true` when move `j` should be skipped after move `i`.
///
/// Two moves on the same face, or two moves on parallel faces where the
/// earlier one has a higher index, are redundant.
pub static CKMV: LazyLock<[[bool; 36]; 37]> = LazyLock::new(|| {
    let mut table = [[false; 36]; 37];
    for i in 0..36 {
        for j in 0..36 {
            table[i][j] = (i / 3 == j / 3) || ((i / 3 % 3 == j / 3 % 3) && (i > j));
        }
    }
    // row 36 (eom) is all false — already initialised
    table
});

/// Phase-2 move-checking table (29 x 28), indexed by [`MOVE2STD`] positions.
pub static CKMV2: LazyLock<[[bool; 28]; 29]> = LazyLock::new(|| {
    let ckmv = &*CKMV;
    let mut table = [[false; 28]; 29];
    for i in 0..29 {
        for j in 0..28 {
            table[i][j] = ckmv[MOVE2STD[i] as usize][MOVE2STD[j] as usize];
        }
    }
    table
});

/// Phase-3 move-checking table (21 x 20), indexed by [`MOVE3STD`] positions.
pub static CKMV3: LazyLock<[[bool; 20]; 21]> = LazyLock::new(|| {
    let ckmv = &*CKMV;
    let mut table = [[false; 20]; 21];
    for i in 0..21 {
        for j in 0..20 {
            table[i][j] = ckmv[MOVE3STD[i] as usize][MOVE3STD[j] as usize];
        }
    }
    table
});

/// For each move `i` (0..35), the last move index that should still be
/// skipped (i.e. all `j` in `i..=skipAxis[i]` have `ckmv[i][j] == true`).
/// Defaults to 36 if nothing is skipped.
pub static SKIP_AXIS: LazyLock<[i32; 36]> = LazyLock::new(|| {
    let ckmv = &*CKMV;
    let mut arr = [0i32; 36];
    for i in 0..36 {
        arr[i] = 36;
        for j in i..36 {
            if !ckmv[i][j] {
                arr[i] = j as i32 - 1;
                break;
            }
        }
    }
    arr
});

/// Phase-2 skip-axis table (28 entries).
pub static SKIP_AXIS2: LazyLock<[i32; 28]> = LazyLock::new(|| {
    let ckmv2 = &*CKMV2;
    let mut arr = [0i32; 28];
    for i in 0..28 {
        arr[i] = 28;
        for j in i..28 {
            if !ckmv2[i][j] {
                arr[i] = j as i32 - 1;
                break;
            }
        }
    }
    arr
});

/// Phase-3 skip-axis table (20 entries).
pub static SKIP_AXIS3: LazyLock<[i32; 20]> = LazyLock::new(|| {
    let ckmv3 = &*CKMV3;
    let mut arr = [0i32; 20];
    for i in 0..20 {
        arr[i] = 20;
        for j in i..20 {
            if !ckmv3[i][j] {
                arr[i] = j as i32 - 1;
                break;
            }
        }
    }
    arr
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_constants_range() {
        assert_eq!(UX1, 0);
        assert_eq!(BWX3, 35);
        assert_eq!(EOM, 36);
    }

    #[test]
    fn test_move2str_length() {
        assert_eq!(MOVE2STR.len(), 36);
        assert_eq!(MOVE_ISTR.len(), 36);
    }

    #[test]
    fn test_move2str_values() {
        assert_eq!(MOVE2STR[UX1 as usize], "U  ");
        assert_eq!(MOVE2STR[UX2 as usize], "U2 ");
        assert_eq!(MOVE2STR[UX3 as usize], "U' ");
        assert_eq!(MOVE2STR[RX1 as usize], "R  ");
        assert_eq!(MOVE2STR[BWX3 as usize], "Bw'");
    }

    #[test]
    fn test_move_istr_inverse() {
        // Inverse of U is U', inverse of U2 is U2
        assert_eq!(MOVE_ISTR[UX1 as usize], "U' ");
        assert_eq!(MOVE_ISTR[UX2 as usize], "U2 ");
        assert_eq!(MOVE_ISTR[UX3 as usize], "U  ");
    }

    #[test]
    fn test_move2std_terminated() {
        assert_eq!(MOVE2STD[28], EOM);
        assert_eq!(MOVE2STD.len(), 29);
    }

    #[test]
    fn test_move3std_terminated() {
        assert_eq!(MOVE3STD[20], EOM);
        assert_eq!(MOVE3STD.len(), 21);
    }

    #[test]
    fn test_std2move_roundtrip() {
        let s2m = &*STD2MOVE;
        // For every entry in move2std (excluding eom), round-tripping should work.
        for i in 0..28 {
            let std_move = MOVE2STD[i];
            if std_move != EOM {
                assert_eq!(MOVE2STD[s2m[std_move as usize] as usize], std_move);
            }
        }
    }

    #[test]
    fn test_std3move_roundtrip() {
        let s3m = &*STD3MOVE;
        for i in 0..20 {
            let std_move = MOVE3STD[i];
            if std_move != EOM {
                assert_eq!(MOVE3STD[s3m[std_move as usize] as usize], std_move);
            }
        }
    }

    #[test]
    fn test_ckmv_same_face() {
        let ckmv = &*CKMV;
        // U moves: indices 0,1,2 share face 0 => should all conflict
        assert!(ckmv[UX1 as usize][UX2 as usize]);
        assert!(ckmv[UX2 as usize][UX1 as usize]);
        assert!(ckmv[UX1 as usize][UX3 as usize]);
    }

    #[test]
    fn test_ckmv_parallel_faces() {
        let ckmv = &*CKMV;
        // U (face 0) and D (face 3) are parallel (0%3 == 3%3 == 0)
        // U after D is ok (U < D), but D after U should be skipped (D > U and parallel).
        assert!(!ckmv[UX1 as usize][DX1 as usize]); // U then D => ok
        assert!(ckmv[DX1 as usize][UX1 as usize]); // D then U => skip
    }

    #[test]
    fn test_ckmv_eom_row() {
        let ckmv = &*CKMV;
        // The eom row (36) should be all false
        for j in 0..36 {
            assert!(!ckmv[36][j]);
        }
    }

    #[test]
    fn test_skip_axis_basic() {
        let sa = &*SKIP_AXIS;
        // For U moves (indices 0,1,2), the skip should cover at least those three
        // U is face 0, D is face 3 — both have axis 0%3==0, so D moves (9,10,11) also skipped.
        // After U1 (index 0), skipped: 0,1,2 (same face), then 3..8 not same axis,
        // so skip_axis[0] should be 2.
        assert_eq!(sa[0], 2);
    }

    #[test]
    fn test_ckmv2_dimensions() {
        let c = &*CKMV2;
        assert_eq!(c.len(), 29);
        assert_eq!(c[0].len(), 28);
    }

    #[test]
    fn test_ckmv3_dimensions() {
        let c = &*CKMV3;
        assert_eq!(c.len(), 21);
        assert_eq!(c[0].len(), 20);
    }

    #[test]
    fn test_facelet_constants() {
        assert_eq!(U1, 0);
        assert_eq!(B9, 53);
    }

    #[test]
    fn test_sub_facelet_constants() {
        assert_eq!(FU0, 0x0);
        assert_eq!(FBF, 0x5f);
        assert_eq!(FR0, 0x10);
        assert_eq!(FD0, 0x30);
    }
}
