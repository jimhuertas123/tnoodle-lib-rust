//! Coordinate-level cube representation and pruning tables.
//!
//! Port of `cs.min2phase.CoordCube` as tnoodle-lib configures it:
//! `USE_TWIST_FLIP_PRUN`, `USE_COMBP_PRUN` and `USE_CONJ_PRUN` are all on, and the
//! tables are always built to their full depth (Java's `init(true)`).

use crate::cubie_cube::{self, CubieCube, CubieCubeTables};
use crate::util;
use std::sync::LazyLock;

pub const N_MOVES: usize = 18;
pub const N_MOVES2: usize = 10;

pub const N_SLICE: usize = 495;
pub const N_TWIST: usize = 2187;
pub const N_TWIST_SYM: usize = 324;
pub const N_FLIP: usize = 2048;
pub const N_FLIP_SYM: usize = 336;
pub const N_PERM: usize = 40320;
pub const N_PERM_SYM: usize = 2768;
pub const N_MPERM: usize = 24;
/// Corner combinations times the corner parity (`USE_COMBP_PRUN`).
pub const N_COMB: usize = 140;
/// The phase-2 moves that flip the corner parity (`P2_PARITY_MOVE`).
const P2_PARITY_MOVE: usize = 0xA5;

/// Port of `CoordCube.setPruning`: entries are 4 bits, 8 to a `u32`.
pub fn set_pruning(table: &mut [u32], index: usize, value: u32) {
    table[index >> 3] ^= value << ((index & 7) << 2);
}

/// Port of `CoordCube.getPruning`.
#[must_use]
pub fn get_pruning(table: &[u32], index: usize) -> u32 {
    (table[index >> 3] >> ((index & 7) << 2)) & 0xf
}

pub static COORD_TABLES: LazyLock<CoordTables> = LazyLock::new(CoordTables::init);

/// The static tables of `cs.min2phase.CoordCube`.
pub struct CoordTables {
    pub ud_slice_move: Vec<Vec<u16>>, // [N_SLICE][N_MOVES]
    pub twist_move: Vec<Vec<u16>>,    // [N_TWIST_SYM][N_MOVES]
    pub flip_move: Vec<Vec<u16>>,     // [N_FLIP_SYM][N_MOVES]
    pub ud_slice_conj: Vec<Vec<u16>>, // [N_SLICE][8]
    pub ud_slice_twist_prun: Vec<u32>,
    pub ud_slice_flip_prun: Vec<u32>,
    pub twist_flip_prun: Vec<u32>,
    pub c_perm_move: Vec<Vec<u16>>,   // [N_PERM_SYM][N_MOVES2]
    pub e_perm_move: Vec<Vec<u16>>,   // [N_PERM_SYM][N_MOVES2]
    pub m_perm_move: Vec<Vec<u16>>,   // [N_MPERM][N_MOVES2]
    pub m_perm_conj: Vec<Vec<u16>>,   // [N_MPERM][16]
    pub c_comb_p_move: Vec<Vec<u16>>, // [N_COMB][N_MOVES2]
    pub c_comb_p_conj: Vec<Vec<u16>>, // [N_COMB][16]
    pub mc_perm_prun: Vec<u32>,
    pub e_perm_c_comb_p_prun: Vec<u32>,
}

impl CoordTables {
    /// Port of `CoordCube.init(true)`.
    fn init() -> Self {
        let ct = &*cubie_cube::TABLES;
        let mut t = Self {
            ud_slice_move: vec![vec![0; N_MOVES]; N_SLICE],
            twist_move: vec![vec![0; N_MOVES]; N_TWIST_SYM],
            flip_move: vec![vec![0; N_MOVES]; N_FLIP_SYM],
            ud_slice_conj: vec![vec![0; 8]; N_SLICE],
            ud_slice_twist_prun: vec![0; N_SLICE * N_TWIST_SYM / 8 + 1],
            ud_slice_flip_prun: vec![0; N_SLICE * N_FLIP_SYM / 8 + 1],
            twist_flip_prun: vec![0; N_FLIP * N_TWIST_SYM / 8 + 1],
            c_perm_move: vec![vec![0; N_MOVES2]; N_PERM_SYM],
            e_perm_move: vec![vec![0; N_MOVES2]; N_PERM_SYM],
            m_perm_move: vec![vec![0; N_MOVES2]; N_MPERM],
            m_perm_conj: vec![vec![0; 16]; N_MPERM],
            c_comb_p_move: vec![vec![0; N_MOVES2]; N_COMB],
            c_comb_p_conj: vec![vec![0; 16]; N_COMB],
            mc_perm_prun: vec![0; N_MPERM * N_PERM_SYM / 8 + 1],
            e_perm_c_comb_p_prun: vec![0; N_COMB * N_PERM_SYM / 8 + 1],
        };
        t.init_c_perm_move(ct);
        t.init_e_perm_move(ct);
        t.init_m_perm_move_conj(ct);
        t.init_comb_p_move_conj(ct);
        t.init_flip_move(ct);
        t.init_twist_move(ct);
        t.init_ud_slice_move_conj(ct);

        let mut prun = std::mem::take(&mut t.mc_perm_prun);
        init_raw_sym_prun(
            &mut prun,
            Raw::Table(&t.m_perm_move, &t.m_perm_conj),
            &t.c_perm_move,
            &ct.sym_state_perm,
            0x8ea34,
        );
        t.mc_perm_prun = prun;
        let mut prun = std::mem::take(&mut t.e_perm_c_comb_p_prun);
        init_raw_sym_prun(
            &mut prun,
            Raw::Table(&t.c_comb_p_move, &t.c_comb_p_conj),
            &t.e_perm_move,
            &ct.sym_state_perm,
            0x7d824,
        );
        t.e_perm_c_comb_p_prun = prun;
        let mut prun = std::mem::take(&mut t.ud_slice_twist_prun);
        init_raw_sym_prun(
            &mut prun,
            Raw::Table(&t.ud_slice_move, &t.ud_slice_conj),
            &t.twist_move,
            &ct.sym_state_twist,
            0x69603,
        );
        t.ud_slice_twist_prun = prun;
        let mut prun = std::mem::take(&mut t.ud_slice_flip_prun);
        init_raw_sym_prun(
            &mut prun,
            Raw::Table(&t.ud_slice_move, &t.ud_slice_conj),
            &t.flip_move,
            &ct.sym_state_flip,
            0x69603,
        );
        t.ud_slice_flip_prun = prun;
        let mut prun = std::mem::take(&mut t.twist_flip_prun);
        init_raw_sym_prun(
            &mut prun,
            Raw::Flip(&t.flip_move),
            &t.twist_move,
            &ct.sym_state_twist,
            0x19603,
        );
        t.twist_flip_prun = prun;
        t
    }

    fn init_ud_slice_move_conj(&mut self, ct: &CubieCubeTables) {
        let mut c = CubieCube::new();
        let mut d = CubieCube::new();
        for i in 0..N_SLICE {
            c.set_ud_slice(i as i32);
            for j in (0..N_MOVES).step_by(3) {
                cubie_cube::edge_mult(&c, &ct.move_cube[j], &mut d);
                self.ud_slice_move[i][j] = d.get_ud_slice() as u16;
            }
            for j in (0..16).step_by(2) {
                cubie_cube::edge_conjugate(&c, ct.sym_mult_inv[0][j], &mut d);
                self.ud_slice_conj[i][j >> 1] = d.get_ud_slice() as u16;
            }
        }
        for i in 0..N_SLICE {
            for j in (0..N_MOVES).step_by(3) {
                let mut udslice = self.ud_slice_move[i][j] as usize;
                for k in 1..3 {
                    udslice = self.ud_slice_move[udslice][j] as usize;
                    self.ud_slice_move[i][j + k] = udslice as u16;
                }
            }
        }
    }

    fn init_flip_move(&mut self, ct: &CubieCubeTables) {
        let mut c = CubieCube::new();
        let mut d = CubieCube::new();
        for i in 0..N_FLIP_SYM {
            c.set_flip(i32::from(ct.flip_s2r[i]));
            for j in 0..N_MOVES {
                cubie_cube::edge_mult(&c, &ct.move_cube[j], &mut d);
                self.flip_move[i][j] = d.get_flip_sym() as u16;
            }
        }
    }

    fn init_twist_move(&mut self, ct: &CubieCubeTables) {
        let mut c = CubieCube::new();
        let mut d = CubieCube::new();
        for i in 0..N_TWIST_SYM {
            c.set_twist(i32::from(ct.twist_s2r[i]));
            for j in 0..N_MOVES {
                cubie_cube::corn_mult(&c, &ct.move_cube[j], &mut d);
                self.twist_move[i][j] = d.get_twist_sym() as u16;
            }
        }
    }

    fn init_c_perm_move(&mut self, ct: &CubieCubeTables) {
        let mut c = CubieCube::new();
        let mut d = CubieCube::new();
        for i in 0..N_PERM_SYM {
            c.set_c_perm(i32::from(ct.e_perm_s2r[i]));
            for j in 0..N_MOVES2 {
                cubie_cube::corn_mult(&c, &ct.move_cube[util::UD2STD[j]], &mut d);
                self.c_perm_move[i][j] = d.get_c_perm_sym() as u16;
            }
        }
    }

    fn init_e_perm_move(&mut self, ct: &CubieCubeTables) {
        let mut c = CubieCube::new();
        let mut d = CubieCube::new();
        for i in 0..N_PERM_SYM {
            c.set_e_perm(i32::from(ct.e_perm_s2r[i]));
            for j in 0..N_MOVES2 {
                cubie_cube::edge_mult(&c, &ct.move_cube[util::UD2STD[j]], &mut d);
                self.e_perm_move[i][j] = d.get_e_perm_sym() as u16;
            }
        }
    }

    fn init_m_perm_move_conj(&mut self, ct: &CubieCubeTables) {
        let mut c = CubieCube::new();
        let mut d = CubieCube::new();
        for i in 0..N_MPERM {
            c.set_m_perm(i as i32);
            for j in 0..N_MOVES2 {
                cubie_cube::edge_mult(&c, &ct.move_cube[util::UD2STD[j]], &mut d);
                self.m_perm_move[i][j] = d.get_m_perm() as u16;
            }
            for j in 0..16 {
                cubie_cube::edge_conjugate(&c, ct.sym_mult_inv[0][j], &mut d);
                self.m_perm_conj[i][j] = d.get_m_perm() as u16;
            }
        }
    }

    /// Port of `initCombPMoveConj`: the upper 70 entries carry odd corner parity.
    fn init_comb_p_move_conj(&mut self, ct: &CubieCubeTables) {
        let mut c = CubieCube::new();
        let mut d = CubieCube::new();
        for i in 0..N_COMB {
            c.set_c_comb((i % 70) as i32);
            for j in 0..N_MOVES2 {
                cubie_cube::corn_mult(&c, &ct.move_cube[util::UD2STD[j]], &mut d);
                let parity = ((P2_PARITY_MOVE >> j) & 1) ^ (i / 70);
                self.c_comb_p_move[i][j] = (d.get_c_comb() as usize + 70 * parity) as u16;
            }
            for j in 0..16 {
                cubie_cube::corn_conjugate(&c, ct.sym_mult_inv[0][j], &mut d);
                self.c_comb_p_conj[i][j] = (d.get_c_comb() as usize + 70 * (i / 70)) as u16;
            }
        }
    }
}

/// The raw coordinate of a pruning table: a move and conjugation table pair, or,
/// for `TwistFlipPrun`, the flip coordinate expanded from its symmetry table.
#[derive(Clone, Copy)]
enum Raw<'a> {
    Table(&'a [Vec<u16>], &'a [Vec<u16>]),
    Flip(&'a [Vec<u16>]),
}

fn has_zero(val: u32) -> bool {
    (val.wrapping_sub(0x1111_1111) & !val & 0x8888_8888) != 0
}

/// Port of `CoordCube.initRawSymPrun` with `fullInit = true`. `prun_flag` packs
/// the symmetry shift, the edge-to-corner symmetry fix-up, the phase, the depth
/// at which the search turns backwards, and the maximum depth, as in Java.
#[allow(
    clippy::too_many_lines,
    reason = "kept in one piece to mirror the Java method"
)]
fn init_raw_sym_prun(
    prun_table: &mut [u32],
    raw: Raw<'_>,
    sym_move: &[Vec<u16>],
    sym_state: &[u16],
    prun_flag: u32,
) {
    let ct = &*cubie_cube::TABLES;
    let sym_shift = (prun_flag & 0xf) as usize;
    let sym_e2c_magic = if (prun_flag >> 4) & 1 == 1 {
        cubie_cube::SYM_E2C_MAGIC
    } else {
        0
    };
    let is_phase2 = (prun_flag >> 5) & 1 == 1;
    let inv_depth = (prun_flag >> 8) & 0xf;
    let max_depth = (prun_flag >> 12) & 0xf;
    let sym_mask = (1usize << sym_shift) - 1;
    let n_raw = match raw {
        Raw::Table(raw_move, _) => raw_move.len(),
        Raw::Flip(_) => N_FLIP,
    };
    let n_size = n_raw * sym_move.len();
    let n_moves = if is_phase2 { 10 } else { 18 };
    let next_axis_magic: u32 = if n_moves == 10 { 0x42 } else { 0x92492 };

    for entry in prun_table.iter_mut().take(n_size / 8 + 1) {
        *entry = 0x1111_1111;
    }
    set_pruning(prun_table, 0, 1);
    let mut depth = 0u32;

    while depth < max_depth {
        let mask = (depth + 1).wrapping_mul(0x1111_1111) ^ 0xffff_ffff;
        for entry in prun_table.iter_mut() {
            let mut val = *entry ^ mask;
            val &= val >> 1;
            *entry = entry.wrapping_add(val & (val >> 2) & 0x1111_1111);
        }
        let inv = depth > inv_depth;
        let select = if inv { depth + 2 } else { depth };
        let sel_arr_mask = select.wrapping_mul(0x1111_1111);
        let check = if inv { depth } else { depth + 2 };
        depth += 1;
        let xor_val = depth ^ (depth + 1);

        let mut val = 0u32;
        let mut i = 0usize;
        while i < n_size {
            if i % 8 == 0 {
                val = prun_table[i >> 3];
                if !has_zero(val ^ sel_arr_mask) {
                    i += 8;
                    continue;
                }
            }
            if val & 0xf == select {
                let raw_i = i % n_raw;
                let sym = i / n_raw;
                let (flip, fsym) = match raw {
                    Raw::Flip(_) => {
                        let f = usize::from(ct.flip_r2s[raw_i]);
                        (f >> 3, f & 7)
                    }
                    Raw::Table(..) => (0, 0),
                };
                let mut m = 0usize;
                while m < n_moves {
                    let symx_full = usize::from(sym_move[sym][m]);
                    let rawx = match raw {
                        Raw::Flip(flip_move) => usize::from(
                            ct.flip_s2rf[usize::from(flip_move[flip][ct.sym8_move[m << 3 | fsym]])
                                ^ fsym
                                ^ (symx_full & sym_mask)],
                        ),
                        Raw::Table(raw_move, raw_conj) => usize::from(
                            raw_conj[usize::from(raw_move[raw_i][m])][symx_full & sym_mask],
                        ),
                    };
                    let symx = symx_full >> sym_shift;
                    let idx = symx * n_raw + rawx;
                    let prun = get_pruning(prun_table, idx);
                    if prun != check {
                        if prun + 1 < depth {
                            m += ((next_axis_magic >> m) & 3) as usize;
                        }
                        m += 1;
                        continue;
                    }
                    if inv {
                        set_pruning(prun_table, i, xor_val);
                        break;
                    }
                    set_pruning(prun_table, idx, xor_val);
                    let mut sym_state_bits = sym_state[symx];
                    let mut j = 1usize;
                    loop {
                        sym_state_bits >>= 1;
                        if sym_state_bits == 0 {
                            break;
                        }
                        if sym_state_bits & 1 == 1 {
                            let idxx = symx * n_raw
                                + match raw {
                                    Raw::Flip(_) => usize::from(
                                        ct.flip_s2rf[usize::from(ct.flip_r2s[rawx]) ^ j],
                                    ),
                                    Raw::Table(_, raw_conj) => usize::from(
                                        raw_conj[rawx]
                                            [j ^ ((sym_e2c_magic >> (j << 1)) & 3) as usize],
                                    ),
                                };
                            if get_pruning(prun_table, idxx) == check {
                                set_pruning(prun_table, idxx, xor_val);
                            }
                        }
                        j += 1;
                    }
                    m += 1;
                }
            }
            val >>= 4;
            i += 1;
        }
    }
}

/// Port of `cs.min2phase.CoordCube`'s instance part: the phase-1 coordinates of
/// one search node and their pruning value.
#[derive(Clone, Debug, Default)]
pub struct CoordCube {
    pub twist: i32,
    pub tsym: i32,
    pub flip: i32,
    pub fsym: i32,
    pub slice: i32,
    pub prun: i32,
    pub twistc: i32,
    pub flipc: i32,
}

fn prun_at(table: &[u32], index: i32) -> i32 {
    get_pruning(table, index as usize) as i32
}

impl CoordCube {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Port of `CoordCube.set(CoordCube)`.
    pub fn set_from(&mut self, other: &CoordCube) {
        self.clone_from(other);
    }

    /// Port of `CoordCube.calcPruning`.
    pub fn calc_pruning(&mut self) {
        let t = &*COORD_TABLES;
        let ct = &*cubie_cube::TABLES;
        let slice = self.slice as usize;
        self.prun = prun_at(
            &t.ud_slice_twist_prun,
            self.twist * N_SLICE as i32 + i32::from(t.ud_slice_conj[slice][self.tsym as usize]),
        )
        .max(prun_at(
            &t.ud_slice_flip_prun,
            self.flip * N_SLICE as i32 + i32::from(t.ud_slice_conj[slice][self.fsym as usize]),
        ))
        .max(
            prun_at(
                &t.twist_flip_prun,
                (self.twistc >> 3) << 11
                    | i32::from(ct.flip_s2rf[(self.flipc ^ (self.twistc & 7)) as usize]),
            )
            .max(prun_at(
                &t.twist_flip_prun,
                self.twist << 11
                    | i32::from(ct.flip_s2rf[(self.flip << 3 | (self.fsym ^ self.tsym)) as usize]),
            )),
        );
    }

    /// Port of `CoordCube.setWithPrun`: sets the coordinates of `cc` and returns
    /// whether its pruning value is at most `depth`.
    pub fn set_with_prun(&mut self, cc: &CubieCube, depth: i32) -> bool {
        let t = &*COORD_TABLES;
        let ct = &*cubie_cube::TABLES;
        self.twist = cc.get_twist_sym();
        self.flip = cc.get_flip_sym();
        self.tsym = self.twist & 7;
        self.twist >>= 3;
        self.prun = prun_at(
            &t.twist_flip_prun,
            self.twist << 11 | i32::from(ct.flip_s2rf[(self.flip ^ self.tsym) as usize]),
        );
        if self.prun > depth {
            return false;
        }
        self.fsym = self.flip & 7;
        self.flip >>= 3;
        self.slice = cc.get_ud_slice();
        let slice = self.slice as usize;
        self.prun = self.prun.max(
            prun_at(
                &t.ud_slice_twist_prun,
                self.twist * N_SLICE as i32 + i32::from(t.ud_slice_conj[slice][self.tsym as usize]),
            )
            .max(prun_at(
                &t.ud_slice_flip_prun,
                self.flip * N_SLICE as i32 + i32::from(t.ud_slice_conj[slice][self.fsym as usize]),
            )),
        );
        if self.prun > depth {
            return false;
        }
        let mut pc = CubieCube::new();
        cubie_cube::corn_conjugate(cc, 1, &mut pc);
        cubie_cube::edge_conjugate(cc, 1, &mut pc);
        self.twistc = pc.get_twist_sym();
        self.flipc = pc.get_flip_sym();
        self.prun = self.prun.max(prun_at(
            &t.twist_flip_prun,
            (self.twistc >> 3) << 11
                | i32::from(ct.flip_s2rf[(self.flipc ^ (self.twistc & 7)) as usize]),
        ));
        self.prun <= depth
    }

    /// Port of `CoordCube.doMovePrun`: applies phase-1 move `m` to `cc` into `self`.
    pub fn do_move_prun(&mut self, cc: &CoordCube, m: usize) -> i32 {
        let t = &*COORD_TABLES;
        let ct = &*cubie_cube::TABLES;
        self.slice = i32::from(t.ud_slice_move[cc.slice as usize][m]);
        self.flip =
            i32::from(t.flip_move[cc.flip as usize][ct.sym8_move[m << 3 | cc.fsym as usize]]);
        self.fsym = (self.flip & 7) ^ cc.fsym;
        self.flip >>= 3;
        self.twist =
            i32::from(t.twist_move[cc.twist as usize][ct.sym8_move[m << 3 | cc.tsym as usize]]);
        self.tsym = (self.twist & 7) ^ cc.tsym;
        self.twist >>= 3;
        let slice = self.slice as usize;
        self.prun = prun_at(
            &t.ud_slice_twist_prun,
            self.twist * N_SLICE as i32 + i32::from(t.ud_slice_conj[slice][self.tsym as usize]),
        )
        .max(prun_at(
            &t.ud_slice_flip_prun,
            self.flip * N_SLICE as i32 + i32::from(t.ud_slice_conj[slice][self.fsym as usize]),
        ))
        .max(prun_at(
            &t.twist_flip_prun,
            self.twist << 11
                | i32::from(ct.flip_s2rf[(self.flip << 3 | (self.fsym ^ self.tsym)) as usize]),
        ));
        self.prun
    }

    /// Port of `CoordCube.doMovePrunConj`: the twist-flip pruning value of the
    /// conjugated cube after move `m`.
    pub fn do_move_prun_conj(&mut self, cc: &CoordCube, m: usize) -> i32 {
        let t = &*COORD_TABLES;
        let ct = &*cubie_cube::TABLES;
        let m = ct.sym_move[3][m];
        self.flipc = i32::from(
            t.flip_move[(cc.flipc >> 3) as usize][ct.sym8_move[m << 3 | (cc.flipc & 7) as usize]],
        ) ^ (cc.flipc & 7);
        self.twistc = i32::from(
            t.twist_move[(cc.twistc >> 3) as usize]
                [ct.sym8_move[m << 3 | (cc.twistc & 7) as usize]],
        ) ^ (cc.twistc & 7);
        prun_at(
            &t.twist_flip_prun,
            (self.twistc >> 3) << 11
                | i32::from(ct.flip_s2rf[(self.flipc ^ (self.twistc & 7)) as usize]),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pruning_pack_unpack() {
        let mut table = vec![0u32; 4];
        set_pruning(&mut table, 0, 5);
        set_pruning(&mut table, 1, 3);
        set_pruning(&mut table, 7, 15);
        assert_eq!(get_pruning(&table, 0), 5);
        assert_eq!(get_pruning(&table, 1), 3);
        assert_eq!(get_pruning(&table, 7), 15);
        assert_eq!(get_pruning(&table, 2), 0);
    }

    #[test]
    fn solved_cube_has_zero_pruning() {
        let mut coord = CoordCube::new();
        assert!(coord.set_with_prun(&CubieCube::new(), 20));
        assert_eq!(
            (coord.twist, coord.flip, coord.slice, coord.prun),
            (0, 0, 0, 0)
        );
    }
}
