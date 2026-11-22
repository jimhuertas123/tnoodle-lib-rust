/// Constants, combinatorial utilities, facelet definitions.
/// Port of cs.min2phase.Util

// ---------------------------------------------------------------------------
// Move indices (18 moves: 6 faces x 3 powers)
// ---------------------------------------------------------------------------
pub const UX1: u8 = 0;
pub const UX2: u8 = 1;
pub const UX3: u8 = 2;
pub const RX1: u8 = 3;
pub const RX2: u8 = 4;
pub const RX3: u8 = 5;
pub const FX1: u8 = 6;
pub const FX2: u8 = 7;
pub const FX3: u8 = 8;
pub const DX1: u8 = 9;
pub const DX2: u8 = 10;
pub const DX3: u8 = 11;
pub const LX1: u8 = 12;
pub const LX2: u8 = 13;
pub const LX3: u8 = 14;
pub const BX1: u8 = 15;
pub const BX2: u8 = 16;
pub const BX3: u8 = 17;

// ---------------------------------------------------------------------------
// Face color indices
// ---------------------------------------------------------------------------
pub const U: u8 = 0;
pub const R: u8 = 1;
pub const F: u8 = 2;
pub const D: u8 = 3;
pub const L: u8 = 4;
pub const B: u8 = 5;

// ---------------------------------------------------------------------------
// Facelet indices (0..53)
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
// Corner and edge facelets
// ---------------------------------------------------------------------------

/// `CORNER_FACELET[i]` = the 3 facelets of corner `i` (U/D facelet first).
pub static CORNER_FACELET: [[u8; 3]; 8] = [
    [U9, R1, F3],
    [U7, F1, L3],
    [U1, L1, B3],
    [U3, B1, R3],
    [D3, F9, R7],
    [D1, L9, F7],
    [D7, B9, L7],
    [D9, R9, B7],
];

/// `EDGE_FACELET[i]` = the 2 facelets of edge `i`.
pub static EDGE_FACELET: [[u8; 2]; 12] = [
    [U6, R2],
    [U8, F2],
    [U4, L2],
    [U2, B2],
    [D6, R8],
    [D2, F8],
    [D4, L8],
    [D8, B8],
    [F6, R4],
    [F4, L6],
    [B6, L4],
    [B4, R6],
];

// ---------------------------------------------------------------------------
// Move string representations
// ---------------------------------------------------------------------------
pub static MOVE2STR: [&str; 18] = [
    "U ", "U2", "U'", "R ", "R2", "R'", "F ", "F2", "F'", "D ", "D2", "D'", "L ", "L2", "L'", "B ",
    "B2", "B'",
];

/// Phase-2 (UD) move order -> standard move index.
pub static UD2STD: [usize; 18] = [
    UX1 as usize,
    UX2 as usize,
    UX3 as usize,
    RX2 as usize,
    FX2 as usize,
    DX1 as usize,
    DX2 as usize,
    DX3 as usize,
    LX2 as usize,
    BX2 as usize,
    RX1 as usize,
    RX3 as usize,
    FX1 as usize,
    FX3 as usize,
    LX1 as usize,
    LX3 as usize,
    BX1 as usize,
    BX3 as usize,
];

// ---------------------------------------------------------------------------
// Binomial coefficients C(n,k)
// ---------------------------------------------------------------------------

/// `CNK[n][k]` = C(n, k) for n,k in 0..12.
pub static CNK: [[i32; 13]; 13] = {
    let mut table = [[0i32; 13]; 13];
    let mut i = 0;
    while i < 13 {
        table[i][0] = 1;
        table[i][i] = 1;
        let mut j = 1;
        while j < i {
            table[i][j] = table[i - 1][j - 1] + table[i - 1][j];
            j += 1;
        }
        i += 1;
    }
    table
};

// ---------------------------------------------------------------------------
// std2ud mapping (computed from UD2STD)
// ---------------------------------------------------------------------------

pub static STD2UD: [usize; 18] = {
    let mut table = [0usize; 18];
    let mut i = 0;
    while i < 18 {
        table[UD2STD[i]] = i;
        i += 1;
    }
    table
};

// ---------------------------------------------------------------------------
// ckmv2bit (move pruning bitmasks)
// ---------------------------------------------------------------------------

pub static CKMV2BIT: [i32; 11] = {
    let mut table = [0i32; 11];
    let mut i = 0;
    while i < 10 {
        let ix = UD2STD[i] / 3;
        table[i] = 0;
        let mut j = 0;
        while j < 10 {
            let jx = UD2STD[j] / 3;
            if (ix == jx) || ((ix % 3 == jx % 3) && (ix >= jx)) {
                table[i] |= 1 << j;
            }
            j += 1;
        }
        i += 1;
    }
    table[10] = 0;
    table
};

// ---------------------------------------------------------------------------
// Combinatorial utilities
// ---------------------------------------------------------------------------

/// Get the parity of a permutation from its factorial number system index.
pub fn get_n_parity(mut idx: i32, n: i32) -> i32 {
    let mut p = 0;
    for i in (0..=(n - 2)).rev() {
        p ^= idx % (n - i);
        idx /= n - i;
    }
    p & 1
}

/// Pack a permutation value, handling edge vs corner encoding.
pub fn set_val(val0: i32, val: i32, is_edge: bool) -> u8 {
    if is_edge {
        (val << 1 | val0 & 1) as u8
    } else {
        (val | val0 & 0xf8) as u8
    }
}

/// Unpack a permutation value.
pub fn get_val(val0: i32, is_edge: bool) -> i32 {
    if is_edge { val0 >> 1 } else { val0 & 7 }
}

/// Convert a permutation array to a factorial number system index.
pub fn get_n_perm(arr: &[u8], n: usize, is_edge: bool) -> i32 {
    let mut idx: i32 = 0;
    let mut val: u64 = 0xFEDC_BA98_7654_3210;
    for i in 0..(n - 1) {
        let v = (get_val(arr[i] as i32, is_edge) as u64) << 2;
        idx = (n - i) as i32 * idx + ((val >> v) & 0xf) as i32;
        val -= 0x1111_1111_1111_1110u64 << v;
    }
    idx
}

/// Set a permutation array from a factorial number system index.
pub fn set_n_perm(arr: &mut [u8], mut idx: i32, n: usize, is_edge: bool) {
    let mut val: u64 = 0xFEDC_BA98_7654_3210;
    let mut extract: u64 = 0;
    for p in 2..=n {
        extract = extract << 4 | (idx % p as i32) as u64;
        idx /= p as i32;
    }
    for i in 0..(n - 1) {
        let v = ((extract & 0xf) as u32) << 2;
        extract >>= 4;
        arr[i] = set_val(arr[i] as i32, ((val >> v) & 0xf) as i32, is_edge);
        let m = (1u64 << v).wrapping_sub(1);
        val = (val & m) | ((val >> 4) & !m);
    }
    arr[n - 1] = set_val(arr[n - 1] as i32, (val & 0xf) as i32, is_edge);
}

/// Get a combination coordinate: which 4 of N positions have pieces
/// matching `mask` (0x8 for UD-slice edges, 0x0 for corners).
pub fn get_comb(arr: &[u8], mask: i32, is_edge: bool) -> i32 {
    let end = arr.len() - 1;
    let mut idx_c = 0i32;
    let mut r = 4usize;
    for i in (0..=end).rev() {
        let perm = get_val(arr[i] as i32, is_edge);
        if (perm & 0xc) == mask {
            idx_c += CNK[i][r];
            r = r.wrapping_sub(1);
        }
    }
    idx_c
}

/// Set a combination coordinate.
pub fn set_comb(arr: &mut [u8], mut idx_c: i32, mask: i32, is_edge: bool) {
    let end = arr.len() - 1;
    let mut r: i32 = 4;
    let mut fill = end as i32;
    for i in (0..=end).rev() {
        if idx_c >= CNK[i][r as usize] {
            idx_c -= CNK[i][r as usize];
            r -= 1;
            arr[i] = set_val(arr[i] as i32, r | mask, is_edge);
        } else {
            if (fill & 0xc) == mask {
                fill -= 4;
            }
            arr[i] = set_val(arr[i] as i32, fill, is_edge);
            fill -= 1;
        }
    }
}

/// Convert a 54-byte facelet array into CubieCube arrays (ca, ea).
pub fn to_cubie_cube(facelets: &[u8; 54], ca: &mut [u8; 8], ea: &mut [u8; 12]) {
    for i in 0..8 {
        ca[i] = 0;
    }
    for i in 0..12 {
        ea[i] = 0;
    }

    // Corners
    for i in 0u8..8 {
        let mut ori = 0u8;
        for o in 0..3u8 {
            if facelets[CORNER_FACELET[i as usize][o as usize] as usize] == U
                || facelets[CORNER_FACELET[i as usize][o as usize] as usize] == D
            {
                ori = o;
                break;
            }
        }
        let col1 = facelets[CORNER_FACELET[i as usize][((ori + 1) % 3) as usize] as usize];
        let col2 = facelets[CORNER_FACELET[i as usize][((ori + 2) % 3) as usize] as usize];

        for j in 0u8..8 {
            if col1 == CORNER_FACELET[j as usize][1] / 9
                && col2 == CORNER_FACELET[j as usize][2] / 9
            {
                ca[i as usize] = ((ori % 3) << 3) | j;
                break;
            }
        }
    }

    // Edges
    for i in 0u8..12 {
        for j in 0u8..12 {
            if facelets[EDGE_FACELET[i as usize][0] as usize] == EDGE_FACELET[j as usize][0] / 9
                && facelets[EDGE_FACELET[i as usize][1] as usize] == EDGE_FACELET[j as usize][1] / 9
            {
                ea[i as usize] = j << 1;
                break;
            }
            if facelets[EDGE_FACELET[i as usize][0] as usize] == EDGE_FACELET[j as usize][1] / 9
                && facelets[EDGE_FACELET[i as usize][1] as usize] == EDGE_FACELET[j as usize][0] / 9
            {
                ea[i as usize] = j << 1 | 1;
                break;
            }
        }
    }
}

/// Convert CubieCube arrays (ca, ea) back to a 54-char facelet string.
pub fn to_face_cube(ca: &[u8; 8], ea: &[u8; 12]) -> String {
    let ts: [char; 6] = ['U', 'R', 'F', 'D', 'L', 'B'];
    let mut f = [0u8; 54];
    for i in 0..54 {
        f[i] = (i / 9) as u8;
    }

    for c in 0u8..8 {
        let j = (ca[c as usize] & 7) as usize;
        let ori = (ca[c as usize] >> 3) as usize;
        for n in 0..3usize {
            f[CORNER_FACELET[c as usize][(n + ori) % 3] as usize] = CORNER_FACELET[j][n] / 9;
        }
    }

    for e in 0u8..12 {
        let j = (ea[e as usize] >> 1) as usize;
        let ori = (ea[e as usize] & 1) as usize;
        for n in 0..2usize {
            f[EDGE_FACELET[e as usize][(n + ori) % 2] as usize] = EDGE_FACELET[j][n] / 9;
        }
    }

    f.iter().map(|&c| ts[c as usize]).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binomial_coefficients() {
        assert_eq!(CNK[0][0], 1);
        assert_eq!(CNK[4][2], 6);
        assert_eq!(CNK[12][4], 495);
    }

    #[test]
    fn parity() {
        assert_eq!(get_n_parity(0, 8), 0); // identity permutation
    }

    #[test]
    fn perm_roundtrip() {
        let mut arr = [0u8, 2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22];
        let original = arr;
        let idx = get_n_perm(&arr, 12, true);
        set_n_perm(&mut arr, idx, 12, true);
        assert_eq!(arr, original);
    }

    #[test]
    fn std2ud_mapping() {
        // std2ud should be the inverse of ud2std
        for i in 0..18 {
            assert_eq!(STD2UD[UD2STD[i]], i);
        }
    }

    #[test]
    fn face_cube_solved() {
        let ca = [0, 1, 2, 3, 4, 5, 6, 7];
        let ea = [0, 2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22];
        let face = to_face_cube(&ca, &ea);
        assert_eq!(
            face,
            "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB"
        );
    }
}
