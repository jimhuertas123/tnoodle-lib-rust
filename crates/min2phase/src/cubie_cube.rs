use crate::coord_cube;
use crate::util;
/// CubieCube representation: corners as `ca[8]`, edges as `ea[12]`.
/// Each corner byte: bits 0-2 = permutation, bits 3-4 = orientation (0-2).
/// Each edge byte: bit 0 = flip, bits 1+ = permutation.
///
/// Port of cs.min2phase.CubieCube
use std::sync::LazyLock;

// ---------------------------------------------------------------------------
// CubieCube
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct CubieCube {
    pub ca: [u8; 8],
    pub ea: [u8; 12],
}

impl CubieCube {
    pub fn new() -> Self {
        Self {
            ca: [0, 1, 2, 3, 4, 5, 6, 7],
            ea: [0, 2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22],
        }
    }

    pub fn from_coords(cperm: i32, twist: i32, eperm: i32, flip: i32) -> Self {
        let mut cc = Self::new();
        cc.set_c_perm(cperm);
        cc.set_twist(twist);
        util::set_n_perm(&mut cc.ea, eperm, 12, true);
        cc.set_flip(flip);
        cc
    }

    pub fn copy_from(&mut self, other: &CubieCube) {
        self.ca.copy_from_slice(&other.ca);
        self.ea.copy_from_slice(&other.ea);
    }

    pub fn inv_cubie_cube(&mut self) {
        let mut temp_ea = [0u8; 12];
        let mut temp_ca = [0u8; 8];

        for edge in 0u8..12 {
            temp_ea[(self.ea[edge as usize] >> 1) as usize] =
                edge << 1 | (self.ea[edge as usize] & 1);
        }
        for corn in 0u8..8 {
            let ci = self.ca[corn as usize];
            temp_ca[(ci & 7) as usize] = corn | (0x20u8 >> (ci >> 3)) & 0x18;
        }

        self.ea.copy_from_slice(&temp_ea);
        self.ca.copy_from_slice(&temp_ca);
    }

    // ---- Phase 1 coordinates ----

    pub fn get_flip(&self) -> i32 {
        let mut idx = 0i32;
        for i in 0..11 {
            idx = idx << 1 | (self.ea[i] & 1) as i32;
        }
        idx
    }

    pub fn set_flip(&mut self, mut idx: i32) {
        let mut parity = 0;
        for i in (0..=10).rev() {
            let val = idx & 1;
            parity ^= val;
            self.ea[i] = (self.ea[i] & 0xfe) | val as u8;
            idx >>= 1;
        }
        self.ea[11] = (self.ea[11] & 0xfe) | parity as u8;
    }

    pub fn get_flip_sym(&self) -> i32 {
        TABLES.flip_r2s[self.get_flip() as usize] as i32
    }

    pub fn get_twist(&self) -> i32 {
        let mut idx = 0i32;
        for i in 0..7 {
            idx += (idx << 1) + (self.ca[i] >> 3) as i32;
        }
        idx
    }

    pub fn set_twist(&mut self, mut idx: i32) {
        let mut twst = 15i32;
        for i in (0..=6).rev() {
            let val = idx % 3;
            twst -= val;
            self.ca[i] = (self.ca[i] & 0x7) | (val as u8) << 3;
            idx /= 3;
        }
        self.ca[7] = (self.ca[7] & 0x7) | ((twst % 3) as u8) << 3;
    }

    pub fn get_twist_sym(&self) -> i32 {
        TABLES.twist_r2s[self.get_twist() as usize] as i32
    }

    pub fn get_ud_slice(&self) -> i32 {
        494 - util::get_comb(&self.ea, 8, true)
    }

    pub fn set_ud_slice(&mut self, idx: i32) {
        util::set_comb(&mut self.ea, 494 - idx, 8, true);
    }

    // ---- Phase 2 coordinates ----

    pub fn get_c_perm(&self) -> i32 {
        util::get_n_perm(&self.ca, 8, false)
    }

    pub fn set_c_perm(&mut self, idx: i32) {
        util::set_n_perm(&mut self.ca, idx, 8, false);
    }

    pub fn get_c_perm_sym(&self) -> i32 {
        e_sym2c_sym(TABLES.e_perm_r2s[self.get_c_perm() as usize] as i32)
    }

    pub fn get_e_perm(&self) -> i32 {
        util::get_n_perm(&self.ea, 8, true)
    }

    pub fn set_e_perm(&mut self, idx: i32) {
        util::set_n_perm(&mut self.ea, idx, 8, true);
    }

    pub fn get_e_perm_sym(&self) -> i32 {
        TABLES.e_perm_r2s[self.get_e_perm() as usize] as i32
    }

    pub fn get_m_perm(&self) -> i32 {
        util::get_n_perm(&self.ea, 12, true) % 24
    }

    pub fn set_m_perm(&mut self, idx: i32) {
        util::set_n_perm(&mut self.ea, idx, 12, true);
    }

    pub fn get_c_comb(&self) -> i32 {
        util::get_comb(&self.ca, 0, false)
    }

    pub fn set_c_comb(&mut self, idx: i32) {
        util::set_comb(&mut self.ca, idx, 0, false);
    }

    // ---- Parity ----

    pub fn get_edge_parity_bit(&self) -> i32 {
        util::get_n_parity(util::get_n_perm(&self.ea, 12, true), 12)
    }

    pub fn get_corner_parity_bit(&self) -> i32 {
        util::get_n_parity(self.get_c_perm(), 8)
    }

    /// Verify solvability. 0 = ok, negative = error code.
    pub fn verify(&self) -> i32 {
        let mut sum = 0i32;
        let mut edge_mask = 0u32;
        for e in 0..12 {
            edge_mask |= 1 << (self.ea[e] >> 1);
            sum ^= (self.ea[e] & 1) as i32;
        }
        if edge_mask != 0xfff {
            return -2;
        }
        if sum != 0 {
            return -3;
        }

        let mut corn_mask = 0u32;
        sum = 0;
        for c in 0..8 {
            corn_mask |= 1 << (self.ca[c] & 7);
            sum += (self.ca[c] >> 3) as i32;
        }
        if corn_mask != 0xff {
            return -4;
        }
        if sum % 3 != 0 {
            return -5;
        }
        if (self.get_edge_parity_bit() ^ self.get_corner_parity_bit()) != 0 {
            return -6;
        }
        0
    }
}

// ---------------------------------------------------------------------------
// Static cube multiplication operations
// ---------------------------------------------------------------------------

/// prod = a * b, corners only.
pub fn corn_mult(a: &CubieCube, b: &CubieCube, prod: &mut CubieCube) {
    for corn in 0..8 {
        let b_corn = (b.ca[corn] & 7) as usize;
        let ori_a = (a.ca[b_corn] >> 3) as i32;
        let ori_b = (b.ca[corn] >> 3) as i32;
        prod.ca[corn] = (a.ca[b_corn] & 7) | (((ori_a + ori_b) % 3) as u8) << 3;
    }
}

/// prod = a * b, corners only, with mirrored cases.
pub fn corn_mult_full(a: &CubieCube, b: &CubieCube, prod: &mut CubieCube) {
    for corn in 0..8 {
        let b_corn = (b.ca[corn] & 7) as usize;
        let ori_a = (a.ca[b_corn] >> 3) as i32;
        let ori_b = (b.ca[corn] >> 3) as i32;
        let mut ori = ori_a + if ori_a < 3 { ori_b } else { 6 - ori_b };
        ori = ori % 3 + if (ori_a < 3) == (ori_b < 3) { 0 } else { 3 };
        prod.ca[corn] = (a.ca[b_corn] & 7) | (ori as u8) << 3;
    }
}

/// prod = a * b, edges only.
pub fn edge_mult(a: &CubieCube, b: &CubieCube, prod: &mut CubieCube) {
    for ed in 0..12 {
        prod.ea[ed] = a.ea[(b.ea[ed] >> 1) as usize] ^ (b.ea[ed] & 1);
    }
}

/// b = S_idx^-1 * a * S_idx, corners only. Uses global TABLES.
pub fn corn_conjugate(a: &CubieCube, idx: usize, b: &mut CubieCube) {
    let tables = &*TABLES;
    corn_conjugate_with(a, idx, b, &tables.cube_sym, &tables.sym_mult_inv);
}

/// b = S_idx^-1 * a * S_idx, corners only. Takes explicit table references.
pub fn corn_conjugate_with(
    a: &CubieCube,
    idx: usize,
    b: &mut CubieCube,
    cube_sym: &[CubieCube; 16],
    sym_mult_inv: &[[usize; 16]; 16],
) {
    let sinv = &cube_sym[sym_mult_inv[0][idx]];
    let s = &cube_sym[idx];
    for corn in 0..8 {
        let s_corn = (s.ca[corn] & 7) as usize;
        let a_s_corn = (a.ca[s_corn] & 7) as usize;
        let ori_a = (sinv.ca[a_s_corn] >> 3) as i32;
        let ori_b = (a.ca[s_corn] >> 3) as i32;
        let ori = if ori_a < 3 { ori_b } else { (3 - ori_b) % 3 };
        b.ca[corn] = (sinv.ca[a_s_corn] & 7) | (ori as u8) << 3;
    }
}

/// b = S_idx^-1 * a * S_idx, edges only. Uses global TABLES.
pub fn edge_conjugate(a: &CubieCube, idx: usize, b: &mut CubieCube) {
    let tables = &*TABLES;
    edge_conjugate_with(a, idx, b, &tables.cube_sym, &tables.sym_mult_inv);
}

/// b = S_idx^-1 * a * S_idx, edges only. Takes explicit table references.
pub fn edge_conjugate_with(
    a: &CubieCube,
    idx: usize,
    b: &mut CubieCube,
    cube_sym: &[CubieCube; 16],
    sym_mult_inv: &[[usize; 16]; 16],
) {
    let sinv = &cube_sym[sym_mult_inv[0][idx]];
    let s = &cube_sym[idx];
    for ed in 0..12 {
        let s_ed = (s.ea[ed] >> 1) as usize;
        let a_s_ed = (a.ea[s_ed] >> 1) as usize;
        b.ea[ed] = sinv.ea[a_s_ed] ^ (a.ea[s_ed] & 1) ^ (s.ea[ed] & 1);
    }
}

// ---------------------------------------------------------------------------
// Symmetry coordinate mapping
// ---------------------------------------------------------------------------

/// Edge perm sym -> corner perm sym coordinate conversion.
pub const SYM_E2C_MAGIC: u32 = 0x00DDDD00;

pub fn e_sym2c_sym(idx: i32) -> i32 {
    idx ^ ((SYM_E2C_MAGIC >> ((idx & 0xf) << 1)) & 3) as i32
}

pub fn get_perm_sym_inv(idx: usize, sym: usize, is_corner: bool) -> i32 {
    let tables = &*TABLES;
    let mut idxi = tables.perm_inv_edge_sym[idx] as i32;
    if is_corner {
        idxi = e_sym2c_sym(idxi);
    }
    (idxi & 0xfff0) | tables.sym_mult[(idxi & 0xf) as usize][sym] as i32
}

// ---------------------------------------------------------------------------
// Global tables (lazy initialized)
// ---------------------------------------------------------------------------

pub static TABLES: LazyLock<CubieCubeTables> = LazyLock::new(CubieCubeTables::init);

pub struct CubieCubeTables {
    /// 16 symmetry cubes
    pub cube_sym: [CubieCube; 16],
    /// 18 move cubes
    pub move_cube: [CubieCube; 18],
    pub move_cube_sym: [u64; 18],
    pub first_move_sym: [i32; 48],

    pub sym_mult: [[usize; 16]; 16],
    pub sym_mult_inv: [[usize; 16]; 16],
    pub sym_move: [[usize; 18]; 16],
    pub sym8_move: [usize; 144], // 8 * 18
    pub sym_move_ud: [[usize; 18]; 16],

    // Sym <-> Raw mappings
    pub flip_s2r: Vec<u16>,          // [N_FLIP_SYM]
    pub twist_s2r: Vec<u16>,         // [N_TWIST_SYM]
    pub e_perm_s2r: Vec<u16>,        // [N_PERM_SYM]
    pub perm2_comb_p: Vec<u8>,       // [N_PERM_SYM]
    pub perm_inv_edge_sym: Vec<u16>, // [N_PERM_SYM]
    pub m_perm_inv: Vec<u8>,         // [N_MPERM]

    pub flip_r2s: Vec<u16>,   // [N_FLIP]
    pub twist_r2s: Vec<u16>,  // [N_TWIST]
    pub e_perm_r2s: Vec<u16>, // [N_PERM]

    pub flip_s2rf: Vec<u16>, // [N_FLIP_SYM * 8]

    pub sym_state_twist: Vec<u16>,
    pub sym_state_flip: Vec<u16>,
    pub sym_state_perm: Vec<u16>,
}

/// URF conjugation cubes (used in Search).
static URF1: LazyLock<CubieCube> =
    LazyLock::new(|| CubieCube::from_coords(2531, 1373, 67026819, 1367));
static URF2: LazyLock<CubieCube> =
    LazyLock::new(|| CubieCube::from_coords(2089, 1906, 322752913, 2040));

pub static URF_MOVE: [[u8; 18]; 6] = [
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17],
    [6, 7, 8, 0, 1, 2, 3, 4, 5, 15, 16, 17, 9, 10, 11, 12, 13, 14],
    [3, 4, 5, 6, 7, 8, 0, 1, 2, 12, 13, 14, 15, 16, 17, 9, 10, 11],
    [2, 1, 0, 5, 4, 3, 8, 7, 6, 11, 10, 9, 14, 13, 12, 17, 16, 15],
    [8, 7, 6, 2, 1, 0, 5, 4, 3, 17, 16, 15, 11, 10, 9, 14, 13, 12],
    [5, 4, 3, 8, 7, 6, 2, 1, 0, 14, 13, 12, 17, 16, 15, 11, 10, 9],
];

impl CubieCubeTables {
    fn init() -> Self {
        let mut tables = Self {
            cube_sym: std::array::from_fn(|_| CubieCube::new()),
            move_cube: std::array::from_fn(|_| CubieCube::new()),
            move_cube_sym: [0; 18],
            first_move_sym: [0; 48],
            sym_mult: [[0; 16]; 16],
            sym_mult_inv: [[0; 16]; 16],
            sym_move: [[0; 18]; 16],
            sym8_move: [0; 144],
            sym_move_ud: [[0; 18]; 16],
            flip_s2r: vec![0; coord_cube::N_FLIP_SYM],
            twist_s2r: vec![0; coord_cube::N_TWIST_SYM],
            e_perm_s2r: vec![0; coord_cube::N_PERM_SYM],
            perm2_comb_p: vec![0; coord_cube::N_PERM_SYM],
            perm_inv_edge_sym: vec![0; coord_cube::N_PERM_SYM],
            m_perm_inv: vec![0; coord_cube::N_MPERM],
            flip_r2s: vec![0; coord_cube::N_FLIP],
            twist_r2s: vec![0; coord_cube::N_TWIST],
            e_perm_r2s: vec![0; coord_cube::N_PERM],
            flip_s2rf: vec![0; coord_cube::N_FLIP_SYM * 8],
            sym_state_twist: vec![0; coord_cube::N_TWIST_SYM],
            sym_state_flip: vec![0; coord_cube::N_FLIP_SYM],
            sym_state_perm: vec![0; coord_cube::N_PERM_SYM],
        };

        tables.init_move();
        tables.init_sym();

        // Sym2Raw mappings (must happen before CoordTables uses them)
        tables.init_perm_sym2raw();
        tables.init_flip_sym2raw();
        tables.init_twist_sym2raw();

        tables
    }

    fn init_move(&mut self) {
        self.move_cube[0] = CubieCube::from_coords(15120, 0, 119750400, 0);
        self.move_cube[3] = CubieCube::from_coords(21021, 1494, 323403417, 0);
        self.move_cube[6] = CubieCube::from_coords(8064, 1236, 29441808, 550);
        self.move_cube[9] = CubieCube::from_coords(9, 0, 5880, 0);
        self.move_cube[12] = CubieCube::from_coords(1230, 412, 2949660, 0);
        self.move_cube[15] = CubieCube::from_coords(224, 137, 328552, 137);

        for a in (0..18).step_by(3) {
            for p in 0..2 {
                let mut prod = CubieCube::new();
                edge_mult(&self.move_cube[a + p], &self.move_cube[a], &mut prod);
                corn_mult(&self.move_cube[a + p], &self.move_cube[a], &mut prod);
                self.move_cube[a + p + 1] = prod;
            }
        }
    }

    fn init_sym(&mut self) {
        let mut c = CubieCube::new();
        let mut d = CubieCube::new();

        let f2 = CubieCube::from_coords(28783, 0, 259268407, 0);
        let u4 = CubieCube::from_coords(15138, 0, 119765538, 7);
        let mut lr2 = CubieCube::from_coords(5167, 0, 83473207, 0);
        for i in 0..8 {
            lr2.ca[i] |= 3 << 3;
        }

        for i in 0..16 {
            self.cube_sym[i] = c.clone();
            let mut temp = CubieCube::new();
            corn_mult_full(&c, &u4, &mut temp);
            edge_mult(&c, &u4, &mut d);
            d.ca.copy_from_slice(&temp.ca);
            std::mem::swap(&mut c, &mut d);

            if i % 4 == 3 {
                let mut temp2 = CubieCube::new();
                corn_mult_full(&c, &lr2, &mut temp2);
                edge_mult(&c, &lr2, &mut d);
                d.ca.copy_from_slice(&temp2.ca);
                std::mem::swap(&mut c, &mut d);
            }
            if i % 8 == 7 {
                let mut temp2 = CubieCube::new();
                corn_mult_full(&c, &f2, &mut temp2);
                edge_mult(&c, &f2, &mut d);
                d.ca.copy_from_slice(&temp2.ca);
                std::mem::swap(&mut c, &mut d);
            }
        }

        // SymMult, SymMultInv
        for i in 0..16 {
            for j in 0..16 {
                let mut prod = CubieCube::new();
                corn_mult_full(&self.cube_sym[i], &self.cube_sym[j], &mut prod);
                for k in 0..16 {
                    if self.cube_sym[k].ca == prod.ca {
                        self.sym_mult[i][j] = k;
                        self.sym_mult_inv[k][j] = i;
                        break;
                    }
                }
            }
        }

        // SymMove, SymMoveUD, Sym8Move
        for j in 0..18 {
            for s in 0..16 {
                let mut conj = CubieCube::new();
                corn_conjugate_with(
                    &self.move_cube[j],
                    self.sym_mult_inv[0][s],
                    &mut conj,
                    &self.cube_sym,
                    &self.sym_mult_inv,
                );
                for m in 0..18 {
                    if self.move_cube[m].ca == conj.ca {
                        self.sym_move[s][j] = m;
                        self.sym_move_ud[s][util::STD2UD[j]] = util::STD2UD[m];
                        break;
                    }
                }
                if s % 2 == 0 {
                    self.sym8_move[j << 3 | s >> 1] = self.sym_move[s][j];
                }
            }
        }

        // moveCubeSym and firstMoveSym
        for i in 0..18 {
            self.move_cube_sym[i] = self.self_symmetry(&self.move_cube[i]);
            let mut j = i;
            for s in 0..48 {
                if self.sym_move[s % 16][j] < i {
                    self.first_move_sym[s] |= 1 << i;
                }
                if s % 16 == 15 {
                    j = URF_MOVE[2][j] as usize;
                }
            }
        }
    }

    fn self_symmetry(&self, cube: &CubieCube) -> u64 {
        let mut c = cube.clone();
        let mut d = CubieCube::new();
        let mut sym = 0u64;

        for i in 0..96 {
            let s = self.sym_mult_inv[0][i % 16];
            corn_conjugate_with(&c, s, &mut d, &self.cube_sym, &self.sym_mult_inv);
            if d.ca == cube.ca {
                edge_conjugate_with(&c, s, &mut d, &self.cube_sym, &self.sym_mult_inv);
                if d.ea == cube.ea {
                    sym |= 1u64 << i.min(48);
                }
            }
            if i % 16 == 15 {
                // URF conjugate
                let mut temp = CubieCube::new();
                corn_mult(&*URF2, &c, &mut temp);
                let mut temp2 = CubieCube::new();
                corn_mult(&temp, &*URF1, &mut temp2);
                edge_mult(&*URF2, &c, &mut temp);
                edge_mult(&temp, &*URF1, &mut d);
                d.ca.copy_from_slice(&temp2.ca);
                c = d.clone();
            }
            if i % 48 == 47 {
                c.inv_cubie_cube();
            }
        }
        sym
    }

    // ---- Sym2Raw / Raw2Sym initialization ----

    pub fn init_sym2raw(
        &mut self,
        n_raw: usize,
        sym2raw: &mut [u16],
        raw2sym: &mut [u16],
        sym_state: &mut [u16],
        coord: usize,
    ) -> usize {
        let mut c = CubieCube::new();
        let mut d = CubieCube::new();
        let mut count = 0usize;
        let sym_inc = if coord >= 2 { 1 } else { 2 };
        let is_edge = coord != 1;

        for i in 0..n_raw {
            if raw2sym[i] != 0 {
                continue;
            }
            match coord {
                0 => c.set_flip(i as i32),
                1 => c.set_twist(i as i32),
                2 => c.set_e_perm(i as i32),
                _ => unreachable!(),
            }

            let mut s = 0;
            while s < 16 {
                if is_edge {
                    edge_conjugate_with(&c, s, &mut d, &self.cube_sym, &self.sym_mult_inv);
                } else {
                    corn_conjugate_with(&c, s, &mut d, &self.cube_sym, &self.sym_mult_inv);
                }
                let idx = match coord {
                    0 => d.get_flip(),
                    1 => d.get_twist(),
                    2 => d.get_e_perm(),
                    _ => unreachable!(),
                } as usize;

                if coord == 0 {
                    self.flip_s2rf[count << 3 | s >> 1] = idx as u16;
                }

                if idx == i {
                    sym_state[count] |= 1 << (s / sym_inc);
                }

                let sym_idx = ((count << 4 | s) / sym_inc) as u16;
                raw2sym[idx] = sym_idx;

                s += sym_inc;
            }
            sym2raw[count] = i as u16;
            count += 1;
        }
        count
    }

    pub fn init_flip_sym2raw(&mut self) {
        let mut flip_s2r = std::mem::take(&mut self.flip_s2r);
        let mut flip_r2s = std::mem::take(&mut self.flip_r2s);
        let mut sym_state_flip = std::mem::take(&mut self.sym_state_flip);
        self.init_sym2raw(
            coord_cube::N_FLIP,
            &mut flip_s2r,
            &mut flip_r2s,
            &mut sym_state_flip,
            0,
        );
        self.flip_s2r = flip_s2r;
        self.flip_r2s = flip_r2s;
        self.sym_state_flip = sym_state_flip;
    }

    pub fn init_twist_sym2raw(&mut self) {
        let mut twist_s2r = std::mem::take(&mut self.twist_s2r);
        let mut twist_r2s = std::mem::take(&mut self.twist_r2s);
        let mut sym_state_twist = std::mem::take(&mut self.sym_state_twist);
        self.init_sym2raw(
            coord_cube::N_TWIST,
            &mut twist_s2r,
            &mut twist_r2s,
            &mut sym_state_twist,
            1,
        );
        self.twist_s2r = twist_s2r;
        self.twist_r2s = twist_r2s;
        self.sym_state_twist = sym_state_twist;
    }

    pub fn init_perm_sym2raw(&mut self) {
        let mut e_perm_s2r = std::mem::take(&mut self.e_perm_s2r);
        let mut e_perm_r2s = std::mem::take(&mut self.e_perm_r2s);
        let mut sym_state_perm = std::mem::take(&mut self.sym_state_perm);
        self.init_sym2raw(
            coord_cube::N_PERM,
            &mut e_perm_s2r,
            &mut e_perm_r2s,
            &mut sym_state_perm,
            2,
        );
        self.e_perm_s2r = e_perm_s2r;
        self.e_perm_r2s = e_perm_r2s;
        self.sym_state_perm = sym_state_perm;

        // Perm2CombP and PermInvEdgeSym
        let mut cc = CubieCube::new();
        for i in 0..coord_cube::N_PERM_SYM {
            cc.set_e_perm(self.e_perm_s2r[i] as i32);
            self.perm2_comb_p[i] = util::get_comb(&cc.ea, 0, true) as u8;
            cc.inv_cubie_cube();
            // Can't call cc.get_e_perm_sym() here because that accesses TABLES (deadlock).
            // Inline the lookup using self.e_perm_r2s instead.
            self.perm_inv_edge_sym[i] = self.e_perm_r2s[cc.get_e_perm() as usize];
        }

        // MPermInv
        for i in 0..coord_cube::N_MPERM {
            cc.set_m_perm(i as i32);
            cc.inv_cubie_cube();
            self.m_perm_inv[i] = cc.get_m_perm() as u8;
        }
    }
}

pub fn get_skip_moves(ssym: u64) -> i32 {
    let tables = &*TABLES;
    let mut ret = 0i32;
    let mut s = ssym >> 1;
    let mut i = 1;
    while s != 0 {
        if s & 1 == 1 {
            ret |= tables.first_move_sym[i];
        }
        s >>= 1;
        i += 1;
    }
    ret
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cubie_cube_identity() {
        let cc = CubieCube::new();
        assert_eq!(cc.get_flip(), 0);
        assert_eq!(cc.get_twist(), 0);
        assert_eq!(cc.get_ud_slice(), 0);
        assert_eq!(cc.get_c_perm(), 0);
        assert_eq!(cc.get_e_perm(), 0);
        assert_eq!(cc.get_m_perm(), 0);
        assert_eq!(cc.verify(), 0);
    }

    #[test]
    fn cubie_cube_corner_roundtrip() {
        // Test corner coords roundtrip (twist + cperm)
        let cc = CubieCube::from_coords(1234, 567, 0, 0);
        let cperm = cc.get_c_perm();
        let twist = cc.get_twist();
        assert_eq!(cperm, 1234);
        assert_eq!(twist, 567);

        let mut cc2 = CubieCube::new();
        cc2.set_c_perm(cperm);
        cc2.set_twist(twist);
        assert_eq!(cc.ca, cc2.ca);
    }

    #[test]
    fn cubie_cube_edge_flip_roundtrip() {
        let mut cc = CubieCube::new();
        cc.set_flip(891);
        let flip = cc.get_flip();
        assert_eq!(flip, 891);
    }

    #[test]
    fn move_cube_solvable() {
        let tables = &*TABLES;
        for i in 0..18 {
            assert_eq!(
                tables.move_cube[i].verify(),
                0,
                "move {i} fails verification"
            );
        }
    }

    #[test]
    fn symmetry_mult_identity() {
        let tables = &*TABLES;
        for i in 0..16 {
            assert_eq!(tables.sym_mult[i][0], i);
            assert_eq!(tables.sym_mult[0][i], i);
        }
    }

    #[test]
    fn inverse_restores_identity() {
        let mut cc = CubieCube::from_coords(1234, 567, 12345678, 891);
        let original = cc.clone();
        cc.inv_cubie_cube();
        // a * a^-1 should be identity
        let mut prod = CubieCube::new();
        corn_mult(&original, &cc, &mut prod);
        edge_mult(&original, &cc, &mut prod);
        assert_eq!(prod.get_c_perm(), 0);
        assert_eq!(prod.get_e_perm(), 0);
        assert_eq!(prod.get_flip(), 0);
        assert_eq!(prod.get_twist(), 0);
    }
}
