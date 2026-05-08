#![allow(clippy::all)]

//! Port of cs.threephase.Edge3
//!
//! Symmetry-reduced edge permutation coordinate solver (phase 3).
//! N_SYM=1538 symmetry classes, N_RAW=20160 raw coordinates.
//! Pruning table: ~31M entries stored as 2 bits per entry in Vec<u32>.

use std::sync::LazyLock;

use crate::edge_cube::EdgeCube;

// ─── constants ───────────────────────────────────────────────────────

pub const N_SYM: usize = 1538;
pub const N_RAW: usize = 20160;
pub const N_EPRUN: usize = N_SYM * N_RAW;
pub const MAX_DEPTH: usize = 10;

pub const PRUN_VALUES: [u32; 14] = [
    1, 4, 16, 55, 324, 1922, 12275, 77640, 485359, 2778197, 11742425, 27492416, 31002941,
    31006080,
];

pub const FULL_EDGE_MAP: [usize; 12] = [0, 2, 4, 6, 1, 3, 7, 5, 8, 9, 10, 11];

const SYMINV: [usize; 8] = [0, 1, 6, 3, 4, 5, 2, 7];

const FACT_X: [i32; 13] = [
    1,
    1,
    2 / 2,
    6 / 2,
    24 / 2,
    120 / 2,
    720 / 2,
    5040 / 2,
    40320 / 2,
    362880 / 2,
    3628800 / 2,
    39916800 / 2,
    479001600 / 2,
];

// ─── lazy tables ─────────────────────────────────────────────────────

pub struct Edge3Tables {
    pub eprun: Vec<u32>,
    pub sym2raw: Vec<i32>,
    pub symstate: Vec<u16>,
    pub raw2sym: Vec<i32>,
    pub mvrot: Vec<[i32; 12]>,
    pub mvroto: Vec<[i32; 12]>,
    pub done: u32,
}

/// Global tables – initialised on first access.
pub static TABLES: LazyLock<Edge3Tables> = LazyLock::new(|| {
    let mut t = Edge3Tables {
        eprun: vec![0xFFFF_FFFFu32; N_EPRUN / 16],
        sym2raw: vec![0i32; N_SYM],
        symstate: vec![0u16; N_SYM],
        raw2sym: vec![0i32; 11880],
        mvrot: vec![[0i32; 12]; 20 * 8],
        mvroto: vec![[0i32; 12]; 20 * 8],
        done: 0,
    };
    init_mvrot(&mut t);
    init_raw2sym(&mut t);
    create_prun(&mut t);
    t
});

/// Returns initialisation progress as fraction in [0.0, 1.0].
pub fn init_status() -> f64 {
    // Before TABLES is accessed, done=0 → returns 0.
    // After full init, done = N_EPRUN → returns ≈1.0.
    // During init this is called from the same thread so not very useful,
    // but matches the Java API.
    TABLES.done as f64 / PRUN_VALUES[MAX_DEPTH - 1] as f64
}

// ─── table builders ──────────────────────────────────────────────────

fn init_mvrot(t: &mut Edge3Tables) {
    let mut e = Edge3::new();
    for m in 0..20 {
        for r in 0..8 {
            e.set_idx(0);
            e.do_move(m);
            e.rotate(r);
            for i in 0..12 {
                t.mvrot[(m << 3) | r][i] = e.edge[i];
            }
            e.std();
            for i in 0..12 {
                t.mvroto[(m << 3) | r][i] = e.temp[i];
            }
        }
    }
}

fn init_raw2sym(t: &mut Edge3Tables) {
    let mut e = Edge3::new();
    let mut occ = vec![0u8; 11880 / 8 + 1];
    let mut count: usize = 0;
    for i in 0..11880 {
        if (occ[i >> 3] & (1 << (i & 7))) == 0 {
            e.set_idx((i as i32) * FACT_X[8]);
            for j in 0..8usize {
                let idx = e.get(4) as usize;
                if idx == i {
                    t.symstate[count] |= 1 << j;
                }
                occ[idx >> 3] |= 1 << (idx & 7);
                t.raw2sym[idx] = ((count as i32) << 3) | (SYMINV[j] as i32);
                e.rot(0);
                if j % 2 == 1 {
                    e.rot(1);
                    e.rot(2);
                }
            }
            t.sym2raw[count] = i as i32;
            count += 1;
        }
    }
    debug_assert_eq!(count, N_SYM);
}

fn create_prun(t: &mut Edge3Tables) {
    let mut e = Edge3::new();
    let mut f = Edge3::new();
    let mut g = Edge3::new();

    // eprun is already filled with 0xFFFFFFFF (all 0x3 nibbles)
    let mut depth: i32 = 0;
    t.done = 1;
    set_pruning(&mut t.eprun, 0, 0);

    while t.done != N_EPRUN as u32 {
        let inv = depth > 9;
        let depm3 = (depth % 3) as u32;
        let dep1m3 = ((depth + 1) % 3) as u32;
        let find = if inv { 0x3 } else { depm3 };
        let chk = if inv { depm3 } else { 0x3 };

        if depth >= (MAX_DEPTH as i32) - 1 {
            break;
        }

        let mut i_ = 0usize;
        while i_ < N_EPRUN {
            let val_full = t.eprun[i_ >> 4];
            if !inv && val_full == 0xFFFF_FFFF {
                i_ += 16;
                continue;
            }
            let mut val = val_full;
            let end = i_ + 16;
            let mut i = i_;
            while i < end {
                if (val & 0x3) != find {
                    i += 1;
                    val >>= 2;
                    continue;
                }
                let symcord1 = i / N_RAW;
                let cord1 = t.sym2raw[symcord1];
                let cord2 = (i % N_RAW) as i32;
                e.set_idx(cord1 * (N_RAW as i32) + cord2);

                for m in 0..17 {
                    let cord1x = getmvrot_with(&e.edge, (m << 3) as usize, 4, &t.mvrot, &t.mvroto);
                    let symcord1x_full = t.raw2sym[cord1x as usize];
                    let symx = (symcord1x_full & 0x7) as usize;
                    let symcord1x = (symcord1x_full >> 3) as usize;
                    let cord2x =
                        getmvrot_with(&e.edge, (m << 3) | symx, 10, &t.mvrot, &t.mvroto) % (N_RAW as i32);
                    let idx = symcord1x * N_RAW + cord2x as usize;
                    if get_pruning(&t.eprun, idx) != chk {
                        continue;
                    }
                    set_pruning(&mut t.eprun, if inv { i } else { idx }, dep1m3);
                    t.done += 1;

                    if inv {
                        break;
                    }

                    let sym_state = t.symstate[symcord1x];
                    if sym_state == 1 {
                        continue;
                    }
                    f.set_from(&e);
                    f.do_move(m);
                    f.rotate(symx);
                    let mut ss = sym_state >> 1;
                    for j in 1..8usize {
                        if ss == 0 {
                            break;
                        }
                        if (ss & 1) == 1 {
                            g.set_from(&f);
                            g.rotate(j);
                            let idxx = symcord1x * N_RAW + (g.get(10) % (N_RAW as i32)) as usize;
                            if get_pruning(&t.eprun, idxx) == chk {
                                set_pruning(&mut t.eprun, idxx, dep1m3);
                                t.done += 1;
                            }
                        }
                        ss >>= 1;
                    }
                }

                i += 1;
                val >>= 2;
            }
            i_ += 16;
        }
        depth += 1;
    }
}

// ─── pruning helpers ─────────────────────────────────────────────────

pub fn set_pruning(table: &mut [u32], index: usize, value: u32) {
    table[index >> 4] ^= (0x3 ^ value) << ((index & 0xf) << 1);
}

pub fn get_pruning(table: &[u32], index: usize) -> u32 {
    (table[index >> 4] >> ((index & 0xf) << 1)) & 0x3
}

pub fn getprun(edge: usize, prun: i32) -> i32 {
    let t = &*TABLES;
    let depm3 = get_pruning(&t.eprun, edge);
    if depm3 == 0x3 {
        return MAX_DEPTH as i32;
    }
    ((depm3 as i32) - prun + 16) % 3 + prun - 1
}

/// Full distance lookup (used when no previous prun bound is known).
pub fn getprun_full(edge: usize) -> i32 {
    let t = &*TABLES;
    let mut e = Edge3::new();
    let mut depth: i32 = 0;
    let mut depm3 = get_pruning(&t.eprun, edge);
    if depm3 == 0x3 {
        return MAX_DEPTH as i32;
    }
    let mut cur = edge;
    while cur != 0 {
        if depm3 == 0 {
            depm3 = 2;
        } else {
            depm3 -= 1;
        }

        let symcord1 = cur / N_RAW;
        let cord1 = t.sym2raw[symcord1];
        let cord2 = (cur % N_RAW) as i32;
        e.set_idx(cord1 * (N_RAW as i32) + cord2);

        for m in 0..17usize {
            let cord1x = getmvrot_with(&e.edge, m << 3, 4, &t.mvrot, &t.mvroto);
            let symcord1x_full = t.raw2sym[cord1x as usize];
            let symx = (symcord1x_full & 0x7) as usize;
            let symcord1x = (symcord1x_full >> 3) as usize;
            let cord2x =
                getmvrot_with(&e.edge, (m << 3) | symx, 10, &t.mvrot, &t.mvroto) % (N_RAW as i32);
            let idx = symcord1x * N_RAW + cord2x as usize;
            if get_pruning(&t.eprun, idx) == depm3 {
                depth += 1;
                cur = idx;
                break;
            }
        }
    }
    depth
}

// ─── static getmvrot (table-based) ──────────────────────────────────

pub fn getmvrot(ep: &[i32; 12], mr_idx: usize, end: usize) -> i32 {
    let t = &*TABLES;
    getmvrot_with(ep, mr_idx, end, &t.mvrot, &t.mvroto)
}

fn getmvrot_with(
    ep: &[i32; 12],
    mr_idx: usize,
    end: usize,
    mvrot: &[[i32; 12]],
    mvroto: &[[i32; 12]],
) -> i32 {
    let movo = &mvroto[mr_idx];
    let mov = &mvrot[mr_idx];
    let mut idx: i64 = 0;

    let mut val: i64 = 0xba9876543210i64;
    for i in 0..end {
        let v = (movo[ep[mov[i] as usize] as usize] as i64) << 2;
        idx *= (12 - i) as i64;
        idx += (val >> v) & 0xf;
        val = val.wrapping_sub(0x111111111110i64.wrapping_shl(v as u32));
    }
    idx as i32
}

// ─── Edge3 struct ────────────────────────────────────────────────────

#[derive(Clone)]
pub struct Edge3 {
    pub edge: [i32; 12],
    pub edgeo: [i32; 12],
    pub temp: [i32; 12],
    pub is_std: bool,
}

impl Edge3 {
    pub fn new() -> Self {
        let mut e = Edge3 {
            edge: [0; 12],
            edgeo: [0; 12],
            temp: [0; 12],
            is_std: true,
        };
        for i in 0..12 {
            e.edge[i] = i as i32;
            e.edgeo[i] = i as i32;
        }
        e
    }

    /// Compute the symmetry-reduced coordinate for this edge state.
    pub fn getsym(&mut self) -> usize {
        let t = &*TABLES;
        let cord1x = self.get(4) as usize;
        let symcord1x_full = t.raw2sym[cord1x];
        let symx = (symcord1x_full & 0x7) as usize;
        let symcord1x = (symcord1x_full >> 3) as usize;
        self.rotate(symx);
        let cord2x = (self.get(10) % (N_RAW as i32)) as usize;
        symcord1x * N_RAW + cord2x
    }

    /// Set from an EdgeCube (the 4x4 edge state).
    /// Returns parity (0 or 1).
    pub fn set_edge_cube(&mut self, c: &EdgeCube) -> i32 {
        for i in 0..12 {
            self.temp[i] = i as i32;
            self.edge[i] = (c.ep[FULL_EDGE_MAP[i] + 12] % 12) as i32;
        }
        let mut parity: i32 = 1; // because of FullEdgeMap
        for i in 0..12 {
            while self.edge[i] != i as i32 {
                let t = self.edge[i] as usize;
                self.edge[i] = self.edge[t];
                self.edge[t] = t as i32;
                let s = self.temp[i];
                self.temp[i] = self.temp[t];
                self.temp[t] = s;
                parity ^= 1;
            }
        }
        for i in 0..12 {
            self.edge[i] = self.temp[(c.ep[FULL_EDGE_MAP[i]] % 12) as usize];
        }
        parity
    }

    pub fn set_from(&mut self, other: &Edge3) {
        self.edge.copy_from_slice(&other.edge);
        self.edgeo.copy_from_slice(&other.edgeo);
        self.is_std = other.is_std;
    }

    pub fn std(&mut self) {
        for i in 0..12 {
            self.temp[self.edgeo[i] as usize] = i as i32;
        }
        for i in 0..12 {
            self.edge[i] = self.temp[self.edge[i] as usize];
            self.edgeo[i] = i as i32;
        }
        self.is_std = true;
    }

    pub fn get(&mut self, end: usize) -> i32 {
        if !self.is_std {
            self.std();
        }
        let mut idx: i64 = 0;
        let mut val: i64 = 0xba9876543210i64;
        for i in 0..end {
            let v = (self.edge[i] as i64) << 2;
            idx *= (12 - i) as i64;
            idx += (val >> v) & 0xf;
            val = val.wrapping_sub(0x111111111110i64.wrapping_shl(v as u32));
        }
        idx as i32
    }

    pub fn set_idx(&mut self, idx: i32) {
        let mut idx = idx;
        let mut val: i64 = 0xba9876543210i64;
        let mut parity: i32 = 0;
        for i in 0..11 {
            let p = FACT_X[11 - i];
            let v = idx / p;
            idx %= p;
            parity ^= v;
            let v = (v as i64) << 2;
            self.edge[i] = ((val >> v) & 0xf) as i32;
            let m = (1i64 << v) - 1;
            val = (val & m) + ((val >> 4) & !m);
        }
        if (parity & 1) == 0 {
            self.edge[11] = val as i32;
        } else {
            self.edge[11] = self.edge[10];
            self.edge[10] = val as i32;
        }
        for i in 0..12 {
            self.edgeo[i] = i as i32;
        }
        self.is_std = true;
    }

    pub fn do_move(&mut self, i: usize) {
        self.is_std = false;
        match i {
            0 => {
                // U
                circle(&mut self.edge, 0, 4, 1, 5);
                circle(&mut self.edgeo, 0, 4, 1, 5);
            }
            1 => {
                // U2
                swap4(&mut self.edge, 0, 4, 1, 5);
                swap4(&mut self.edgeo, 0, 4, 1, 5);
            }
            2 => {
                // U'
                circle(&mut self.edge, 0, 5, 1, 4);
                circle(&mut self.edgeo, 0, 5, 1, 4);
            }
            3 => {
                // R2
                swap4(&mut self.edge, 5, 10, 6, 11);
                swap4(&mut self.edgeo, 5, 10, 6, 11);
            }
            4 => {
                // F
                circle(&mut self.edge, 0, 11, 3, 8);
                circle(&mut self.edgeo, 0, 11, 3, 8);
            }
            5 => {
                // F2
                swap4(&mut self.edge, 0, 11, 3, 8);
                swap4(&mut self.edgeo, 0, 11, 3, 8);
            }
            6 => {
                // F'
                circle(&mut self.edge, 0, 8, 3, 11);
                circle(&mut self.edgeo, 0, 8, 3, 11);
            }
            7 => {
                // D
                circle(&mut self.edge, 2, 7, 3, 6);
                circle(&mut self.edgeo, 2, 7, 3, 6);
            }
            8 => {
                // D2
                swap4(&mut self.edge, 2, 7, 3, 6);
                swap4(&mut self.edgeo, 2, 7, 3, 6);
            }
            9 => {
                // D'
                circle(&mut self.edge, 2, 6, 3, 7);
                circle(&mut self.edgeo, 2, 6, 3, 7);
            }
            10 => {
                // L2
                swap4(&mut self.edge, 4, 8, 7, 9);
                swap4(&mut self.edgeo, 4, 8, 7, 9);
            }
            11 => {
                // B
                circle(&mut self.edge, 1, 9, 2, 10);
                circle(&mut self.edgeo, 1, 9, 2, 10);
            }
            12 => {
                // B2
                swap4(&mut self.edge, 1, 9, 2, 10);
                swap4(&mut self.edgeo, 1, 9, 2, 10);
            }
            13 => {
                // B'
                circle(&mut self.edge, 1, 10, 2, 9);
                circle(&mut self.edgeo, 1, 10, 2, 9);
            }
            14 => {
                // u2
                swap4(&mut self.edge, 0, 4, 1, 5);
                swap4(&mut self.edgeo, 0, 4, 1, 5);
                swap2(&mut self.edge, 9, 11);
                swap2(&mut self.edgeo, 8, 10);
            }
            15 => {
                // r2
                swap4(&mut self.edge, 5, 10, 6, 11);
                swap4(&mut self.edgeo, 5, 10, 6, 11);
                swap2(&mut self.edge, 1, 3);
                swap2(&mut self.edgeo, 0, 2);
            }
            16 => {
                // f2
                swap4(&mut self.edge, 0, 11, 3, 8);
                swap4(&mut self.edgeo, 0, 11, 3, 8);
                swap2(&mut self.edge, 5, 7);
                swap2(&mut self.edgeo, 4, 6);
            }
            17 => {
                // d2
                swap4(&mut self.edge, 2, 7, 3, 6);
                swap4(&mut self.edgeo, 2, 7, 3, 6);
                swap2(&mut self.edge, 8, 10);
                swap2(&mut self.edgeo, 9, 11);
            }
            18 => {
                // l2
                swap4(&mut self.edge, 4, 8, 7, 9);
                swap4(&mut self.edgeo, 4, 8, 7, 9);
                swap2(&mut self.edge, 0, 2);
                swap2(&mut self.edgeo, 1, 3);
            }
            19 => {
                // b2
                swap4(&mut self.edge, 1, 9, 2, 10);
                swap4(&mut self.edgeo, 1, 9, 2, 10);
                swap2(&mut self.edge, 4, 6);
                swap2(&mut self.edgeo, 5, 7);
            }
            _ => {}
        }
    }

    pub fn rot(&mut self, r: usize) {
        self.is_std = false;
        match r {
            0 => {
                self.do_move(14);
                self.do_move(17);
            }
            1 => {
                self.circlex(11, 5, 10, 6);
                self.circlex2(5, 10, 6, 11);
                self.circlex2(1, 2, 3, 0);
                self.circlex(4, 9, 7, 8);
                self.circlex2(8, 4, 9, 7);
                self.circlex2(0, 1, 2, 3);
            }
            2 => {
                self.swapx(4, 5);
                self.swapx2(5, 4);
                self.swapx(11, 8);
                self.swapx2(8, 11);
                self.swapx(7, 6);
                self.swapx2(6, 7);
                self.swapx(9, 10);
                self.swapx2(10, 9);
                self.swapx(1, 1);
                self.swapx2(0, 0);
                self.swapx(3, 3);
                self.swapx2(2, 2);
            }
            _ => {}
        }
    }

    pub fn rotate(&mut self, r: usize) {
        let mut r = r;
        while r >= 2 {
            r -= 2;
            self.rot(1);
            self.rot(2);
        }
        if r != 0 {
            self.rot(0);
        }
    }

    // ── internal helpers ──

    fn circlex(&mut self, a: usize, b: usize, c: usize, d: usize) {
        // temp = edgeo[d]; edgeo[d] = edge[c]; edge[c] = edgeo[b]; edgeo[b] = edge[a]; edge[a] = temp;
        let temp = self.edgeo[d];
        self.edgeo[d] = self.edge[c];
        self.edge[c] = self.edgeo[b];
        self.edgeo[b] = self.edge[a];
        self.edge[a] = temp;
    }

    /// The Java code calls circlex twice with different argument orders;
    /// the second call is the same operation, just swapping which array
    /// starts the chain. They are identical so we alias.
    fn circlex2(&mut self, a: usize, b: usize, c: usize, d: usize) {
        self.circlex(a, b, c, d);
    }

    fn swapx(&mut self, x: usize, y: usize) {
        let temp = self.edge[x];
        self.edge[x] = self.edgeo[y];
        self.edgeo[y] = temp;
    }

    fn swapx2(&mut self, x: usize, y: usize) {
        self.swapx(x, y);
    }
}

// ── array helpers ────────────────────────────────────────────────────

fn circle(arr: &mut [i32; 12], a: usize, b: usize, c: usize, d: usize) {
    let temp = arr[d];
    arr[d] = arr[c];
    arr[c] = arr[b];
    arr[b] = arr[a];
    arr[a] = temp;
}

fn swap4(arr: &mut [i32; 12], a: usize, b: usize, c: usize, d: usize) {
    let temp = arr[a];
    arr[a] = arr[c];
    arr[c] = temp;
    let temp = arr[b];
    arr[b] = arr[d];
    arr[d] = temp;
}

fn swap2(arr: &mut [i32; 12], x: usize, y: usize) {
    arr.swap(x, y);
}

// ─── tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solved_edge3_coordinate() {
        let mut e = Edge3::new();
        // Identity permutation should give index 0.
        assert_eq!(e.get(10), 0);
        assert_eq!(e.getsym(), 0);
    }

    #[test]
    fn test_set_get_roundtrip() {
        let mut e = Edge3::new();
        for idx in [0, 1, 100, 5000, 11879] {
            e.set_idx(idx * FACT_X[8]);
            let got = e.get(4);
            assert_eq!(got, idx, "roundtrip failed for idx={idx}");
        }
    }

    #[test]
    fn test_pruning_pack_unpack() {
        let mut table = vec![0xFFFF_FFFFu32; 4];
        // Write some values
        set_pruning(&mut table, 0, 0);
        set_pruning(&mut table, 1, 1);
        set_pruning(&mut table, 2, 2);
        set_pruning(&mut table, 3, 0);
        assert_eq!(get_pruning(&table, 0), 0);
        assert_eq!(get_pruning(&table, 1), 1);
        assert_eq!(get_pruning(&table, 2), 2);
        assert_eq!(get_pruning(&table, 3), 0);
    }
}
