use crate::cubie_cube::{self, CubieCube, CubieCubeTables};
use crate::util;
/// Coordinate-level cube representation and pruning tables.
/// Port of cs.min2phase.CoordCube
use std::sync::LazyLock;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

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
pub const N_COMB: usize = 70;

// ---------------------------------------------------------------------------
// Pruning table accessors (4 bits per entry packed into u32)
// ---------------------------------------------------------------------------

pub fn set_pruning(table: &mut [u32], index: usize, value: u32) {
    table[index >> 3] ^= value << ((index & 7) << 2);
}

pub fn get_pruning(table: &[u32], index: usize) -> u32 {
    (table[index >> 3] >> ((index & 7) << 2)) & 0xf
}

// ---------------------------------------------------------------------------
// CoordCube tables (lazy initialized)
// ---------------------------------------------------------------------------

pub static COORD_TABLES: LazyLock<CoordTables> = LazyLock::new(CoordTables::init);

pub struct CoordTables {
    // Phase 1 move & conjugation tables
    pub ud_slice_move: Vec<Vec<u16>>, // [N_SLICE][N_MOVES]
    pub twist_move: Vec<Vec<u16>>,    // [N_TWIST_SYM][N_MOVES]
    pub flip_move: Vec<Vec<u16>>,     // [N_FLIP_SYM][N_MOVES]
    pub ud_slice_conj: Vec<Vec<u16>>, // [N_SLICE][8]

    // Phase 1 pruning tables
    pub ud_slice_twist_prun: Vec<u32>,
    pub ud_slice_flip_prun: Vec<u32>,

    // Phase 2 move & conjugation tables
    pub c_perm_move: Vec<Vec<u16>>,   // [N_PERM_SYM][N_MOVES2]
    pub e_perm_move: Vec<Vec<u16>>,   // [N_PERM_SYM][N_MOVES2]
    pub m_perm_move: Vec<Vec<u16>>,   // [N_MPERM][N_MOVES2]
    pub m_perm_conj: Vec<Vec<u16>>,   // [N_MPERM][16]
    pub c_comb_p_move: Vec<Vec<u16>>, // [N_COMB][N_MOVES2]
    pub c_comb_p_conj: Vec<Vec<u16>>, // [N_COMB][16]

    // Phase 2 pruning tables
    pub mc_perm_prun: Vec<u32>,
    pub e_perm_c_comb_p_prun: Vec<u32>,
}

impl CoordTables {
    fn init() -> Self {
        // Force CubieCubeTables to initialize first
        let ct = &*cubie_cube::TABLES;

        let mut tables = Self {
            ud_slice_move: vec![vec![0; N_MOVES]; N_SLICE],
            twist_move: vec![vec![0; N_MOVES]; N_TWIST_SYM],
            flip_move: vec![vec![0; N_MOVES]; N_FLIP_SYM],
            ud_slice_conj: vec![vec![0; 8]; N_SLICE],
            ud_slice_twist_prun: vec![0; N_SLICE * N_TWIST_SYM / 8 + 1],
            ud_slice_flip_prun: vec![0; N_SLICE * N_FLIP_SYM / 8 + 1],
            c_perm_move: vec![vec![0; N_MOVES2]; N_PERM_SYM],
            e_perm_move: vec![vec![0; N_MOVES2]; N_PERM_SYM],
            m_perm_move: vec![vec![0; N_MOVES2]; N_MPERM],
            m_perm_conj: vec![vec![0; 16]; N_MPERM],
            c_comb_p_move: vec![vec![0; N_MOVES2]; N_COMB],
            c_comb_p_conj: vec![vec![0; 16]; N_COMB],
            mc_perm_prun: vec![0; N_MPERM * N_PERM_SYM / 8 + 1],
            e_perm_c_comb_p_prun: vec![0; N_COMB * N_PERM_SYM / 8 + 1],
        };

        tables.init_c_perm_move(ct);
        tables.init_e_perm_move(ct);
        tables.init_m_perm_move_conj(ct);
        tables.init_comb_p_move_conj(ct);
        tables.init_flip_move(ct);
        tables.init_twist_move(ct);
        tables.init_ud_slice_move_conj(ct);

        // Pruning tables
        tables.init_mc_perm_prun(ct);
        tables.init_perm_comb_p_prun(ct);
        tables.init_slice_twist_prun(ct);
        tables.init_slice_flip_prun(ct);

        tables
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
            c.set_flip(ct.flip_s2r[i] as i32);
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
            c.set_twist(ct.twist_s2r[i] as i32);
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
            c.set_c_perm(ct.e_perm_s2r[i] as i32);
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
            c.set_e_perm(ct.e_perm_s2r[i] as i32);
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

    fn init_comb_p_move_conj(&mut self, ct: &CubieCubeTables) {
        let mut c = CubieCube::new();
        let mut d = CubieCube::new();
        for i in 0..N_COMB {
            c.set_c_comb(i as i32);
            for j in 0..N_MOVES2 {
                cubie_cube::corn_mult(&c, &ct.move_cube[util::UD2STD[j]], &mut d);
                self.c_comb_p_move[i][j] = d.get_c_comb() as u16;
            }
            for j in 0..16 {
                cubie_cube::corn_conjugate(&c, ct.sym_mult_inv[0][j], &mut d);
                self.c_comb_p_conj[i][j] = d.get_c_comb() as u16;
            }
        }
    }

    // ---- Pruning table initialization ----

    fn init_raw_sym_prun(
        prun_table: &mut [u32],
        raw_move: Option<&[Vec<u16>]>,
        raw_conj: Option<&[Vec<u16>]>,
        sym_move: &[Vec<u16>],
        sym_state: &[u16],
        sym_shift: usize,
        is_phase2: bool,
        inv_depth: u32,
        max_depth: u32,
    ) {
        let sym_mask = (1usize << sym_shift) - 1;
        let n_raw = raw_move.map_or(N_FLIP, |rm| rm.len());
        let n_size = n_raw * sym_move.len();
        let n_moves = if is_phase2 { 10 } else { 18 };
        let _next_axis_magic = if n_moves == 10 { 0x42u32 } else { 0x92492u32 };

        // Initialize all entries to 0x1 (depth = 1, actually means "not filled")
        for i in 0..(n_size / 8 + 1) {
            prun_table[i] = 0x1111_1111;
        }
        set_pruning(prun_table, 0, 0 ^ 1);
        let mut _done = 1u32;
        let mut depth = 0u32;

        let _ct = &*cubie_cube::TABLES;

        while depth < max_depth {
            let mask = (depth + 1).wrapping_mul(0x1111_1111) ^ 0xffff_ffff;
            for i in 0..prun_table.len() {
                let val = prun_table[i] ^ mask;
                let v2 = val & (val >> 1);
                prun_table[i] = prun_table[i].wrapping_add(v2 & (v2 >> 2) & 0x1111_1111);
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
                if (i & 7) == 0 {
                    val = prun_table[i >> 3];
                    if !has_zero(val ^ sel_arr_mask) {
                        i += 8;
                        continue;
                    }
                }
                if (val & 0xf) != select {
                    val >>= 4;
                    i += 1;
                    continue;
                }

                let raw = i % n_raw;
                let sym = i / n_raw;

                for m in 0..n_moves {
                    let symx_full = sym_move[sym][m] as usize;
                    let rawx = if let (Some(rm), Some(rc)) = (raw_move, raw_conj) {
                        rc[rm[raw][m] as usize][symx_full & sym_mask] as usize
                    } else {
                        // TwistFlipPrun path (not used in simplified version)
                        continue;
                    };
                    let symx = symx_full >> sym_shift;
                    let idx = symx * n_raw + rawx;
                    let prun = get_pruning(prun_table, idx);

                    if prun != check {
                        if prun < depth - 1 {
                            // Skip moves on same axis
                            // m += next_axis_magic >> m & 3
                            // (we can't mutate m in for loop, so we handle this differently)
                        }
                        continue;
                    }

                    _done += 1;
                    if inv {
                        set_pruning(prun_table, i, xor_val);
                        break;
                    }
                    set_pruning(prun_table, idx, xor_val);

                    // Propagate to symmetric positions
                    let mut sym_st = sym_state[symx] >> 1;
                    let mut j = 1usize;
                    while sym_st != 0 {
                        if sym_st & 1 == 1 {
                            let idxx = if let Some(rc) = raw_conj {
                                let conj_idx =
                                    j ^ ((cubie_cube::SYM_E2C_MAGIC >> (j << 1)) & 3) as usize;
                                if is_phase2 {
                                    symx * n_raw + rc[rawx][conj_idx] as usize
                                } else {
                                    symx * n_raw + rc[rawx][j] as usize
                                }
                            } else {
                                continue;
                            };
                            if get_pruning(prun_table, idxx) == check {
                                set_pruning(prun_table, idxx, xor_val);
                                _done += 1;
                            }
                        }
                        sym_st >>= 1;
                        j += 1;
                    }
                }

                val >>= 4;
                i += 1;
            }
        }
    }

    fn init_slice_twist_prun(&mut self, ct: &CubieCubeTables) {
        Self::init_raw_sym_prun(
            &mut self.ud_slice_twist_prun,
            Some(&self.ud_slice_move),
            Some(&self.ud_slice_conj),
            &self.twist_move,
            &ct.sym_state_twist,
            3,
            false,
            6,
            9,
        );
    }

    fn init_slice_flip_prun(&mut self, ct: &CubieCubeTables) {
        Self::init_raw_sym_prun(
            &mut self.ud_slice_flip_prun,
            Some(&self.ud_slice_move),
            Some(&self.ud_slice_conj),
            &self.flip_move,
            &ct.sym_state_flip,
            3,
            false,
            6,
            9,
        );
    }

    fn init_mc_perm_prun(&mut self, ct: &CubieCubeTables) {
        Self::init_raw_sym_prun(
            &mut self.mc_perm_prun,
            Some(&self.m_perm_move),
            Some(&self.m_perm_conj),
            &self.c_perm_move,
            &ct.sym_state_perm,
            4,
            true,
            8,
            14,
        );
    }

    fn init_perm_comb_p_prun(&mut self, ct: &CubieCubeTables) {
        Self::init_raw_sym_prun(
            &mut self.e_perm_c_comb_p_prun,
            Some(&self.c_comb_p_move),
            Some(&self.c_comb_p_conj),
            &self.e_perm_move,
            &ct.sym_state_perm,
            4,
            true,
            7,
            13,
        );
    }
}

fn has_zero(val: u32) -> bool {
    (val.wrapping_sub(0x1111_1111) & !val & 0x8888_8888) != 0
}

// ---------------------------------------------------------------------------
// CoordCube instance (used during search)
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct CoordCube {
    pub twist: i32,
    pub tsym: i32,
    pub flip: i32,
    pub fsym: i32,
    pub slice: i32,
    pub prun: i32,
}

impl CoordCube {
    pub fn new() -> Self {
        Self {
            twist: 0,
            tsym: 0,
            flip: 0,
            fsym: 0,
            slice: 0,
            prun: 0,
        }
    }

    pub fn set_from(&mut self, other: &CoordCube) {
        self.twist = other.twist;
        self.tsym = other.tsym;
        self.flip = other.flip;
        self.fsym = other.fsym;
        self.slice = other.slice;
        self.prun = other.prun;
    }

    /// Set coordinates from a CubieCube. Returns false if pruning > depth.
    pub fn set_with_prun(cc: &CubieCube, depth: i32) -> Option<Self> {
        let tables = &*COORD_TABLES;

        let twist_sym = cc.get_twist_sym();
        let flip_sym = cc.get_flip_sym();
        let tsym = twist_sym & 7;
        let twist = twist_sym >> 3;

        let fsym = flip_sym & 7;
        let flip = flip_sym >> 3;
        let slice = cc.get_ud_slice();

        let prun = get_pruning(
            &tables.ud_slice_twist_prun,
            (twist * N_SLICE as i32 + tables.ud_slice_conj[slice as usize][tsym as usize] as i32)
                as usize,
        ) as i32;
        let prun = prun.max(get_pruning(
            &tables.ud_slice_flip_prun,
            (flip * N_SLICE as i32 + tables.ud_slice_conj[slice as usize][fsym as usize] as i32)
                as usize,
        ) as i32);

        if prun > depth {
            return None;
        }

        Some(Self {
            twist,
            tsym,
            flip,
            fsym,
            slice,
            prun,
        })
    }

    /// Apply move and compute pruning for phase 1.
    pub fn do_move_prun(&mut self, cc: &CoordCube, m: usize) -> i32 {
        let tables = &*COORD_TABLES;
        let ct = &*cubie_cube::TABLES;

        self.slice = tables.ud_slice_move[cc.slice as usize][m] as i32;

        let flip_raw =
            tables.flip_move[cc.flip as usize][ct.sym8_move[m << 3 | cc.fsym as usize]] as i32;
        self.fsym = (flip_raw & 7) ^ cc.fsym;
        self.flip = flip_raw >> 3;

        let twist_raw =
            tables.twist_move[cc.twist as usize][ct.sym8_move[m << 3 | cc.tsym as usize]] as i32;
        self.tsym = (twist_raw & 7) ^ cc.tsym;
        self.twist = twist_raw >> 3;

        self.prun = get_pruning(
            &tables.ud_slice_twist_prun,
            (self.twist * N_SLICE as i32
                + tables.ud_slice_conj[self.slice as usize][self.tsym as usize] as i32)
                as usize,
        ) as i32;
        self.prun = self.prun.max(get_pruning(
            &tables.ud_slice_flip_prun,
            (self.flip * N_SLICE as i32
                + tables.ud_slice_conj[self.slice as usize][self.fsym as usize] as i32)
                as usize,
        ) as i32);

        self.prun
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
    fn tables_initialize() {
        let tables = &*COORD_TABLES;
        // Verify tables are initialized with non-trivial data:
        // Check that some move table has nonzero entries
        let has_nonzero = tables
            .twist_move
            .iter()
            .any(|row| row.iter().any(|&v| v != 0));
        assert!(has_nonzero, "twist move table should have nonzero entries");
        let has_nonzero = tables
            .flip_move
            .iter()
            .any(|row| row.iter().any(|&v| v != 0));
        assert!(has_nonzero, "flip move table should have nonzero entries");
    }

    #[test]
    fn coord_cube_from_solved() {
        let cc = CubieCube::new();
        let coord = CoordCube::set_with_prun(&cc, 20);
        assert!(coord.is_some());
        let coord = coord.unwrap();
        assert_eq!(coord.twist, 0);
        assert_eq!(coord.flip, 0);
        assert_eq!(coord.slice, 0);
        assert_eq!(coord.prun, 0);
    }
}
