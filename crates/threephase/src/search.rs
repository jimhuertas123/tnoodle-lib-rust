#![allow(clippy::all)]
#![allow(unused)]
//! Port of cs.threephase.Search
//!
//! Three-phase IDA* orchestrator for 4x4x4 solving.
//!
//! Phase 1: reduce centers to a single colour class (using Center1)
//! Phase 2: solve remaining centers + edge parity (using Center2)
//! Phase 3: solve edges (using Edge3 + Center3) then hand off to min2phase for 3x3
//!
//! This module depends on center1, center2, center3, edge3, and the
//! tnoodle_min2phase crate (the 3x3 two-phase solver).

use std::collections::BinaryHeap;

use rand::Rng;

use crate::center1::{self, Center1};
use crate::center2::Center2;
use crate::center3::Center3;
use crate::edge3::{self, Edge3};
use crate::full_cube::FullCube;
use crate::moves::*;

use tnoodle_min2phase::search::Search as Search333;
use tnoodle_min2phase::search::SearchResult;

// ─── constants ───────────────────────────────────────────────────────

const PHASE1_SOLUTIONS: usize = 10000;
const PHASE2_ATTEMPTS: usize = 500;
const PHASE2_SOLUTIONS: usize = 100;
const PHASE3_ATTEMPTS: usize = 100;

// ─── Search ──────────────────────────────────────────────────────────

pub struct Search {
    // Phase-1 moves and state
    move1: [i32; 15],
    move2: [i32; 20],
    move3: [i32; 20],
    length1: i32,
    length2: i32,
    add1: bool,

    pub cube: Option<FullCube>,
    c1: FullCube,
    c2: FullCube,
    ct2: Center2,
    ct3: Center3,
    e12: Edge3,
    tempe: Vec<Edge3>,

    search333: Search333,

    p1_sols_cnt: usize,
    // Best phase-1 solutions, as a max-heap by value so the worst one is on
    // top and gets replaced (Java's PriorityQueue with ValueComparator).
    p1_sols: BinaryHeap<FullCubeEntry>,

    arr2: Vec<Option<FullCube>>,
    arr2_idx: usize,

    pub inverse_solution: bool,
    pub with_rotation: bool,

    pub solution: String,
}

/// Wrapper for BinaryHeap ordering (max-value first, for ValueComparator).
struct FullCubeEntry {
    cube: FullCube,
}

impl PartialEq for FullCubeEntry {
    fn eq(&self, other: &Self) -> bool {
        self.cube.value == other.cube.value
    }
}
impl Eq for FullCubeEntry {}

impl PartialOrd for FullCubeEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FullCubeEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Java ValueComparator: c2.value - c1.value  (i.e. higher value = higher priority)
        // Bigger value = greater, so the BinaryHeap top is the worst solution.
        self.cube.value.cmp(&other.cube.value)
    }
}

impl Search {
    pub fn new() -> Self {
        let mut tempe = Vec::with_capacity(20);
        for _ in 0..20 {
            tempe.push(Edge3::new());
        }

        let mut arr2 = Vec::with_capacity(PHASE2_SOLUTIONS);
        for _ in 0..PHASE2_SOLUTIONS {
            arr2.push(None);
        }

        Search {
            move1: [0; 15],
            move2: [0; 20],
            move3: [0; 20],
            length1: 0,
            length2: 0,
            add1: false,
            cube: None,
            c1: FullCube::new(),
            c2: FullCube::new(),
            ct2: Center2::new(),
            ct3: Center3::new(),
            e12: Edge3::new(),
            tempe,
            search333: Search333::new(),
            p1_sols_cnt: 0,
            p1_sols: BinaryHeap::with_capacity(PHASE2_ATTEMPTS),
            arr2,
            arr2_idx: 0,
            inverse_solution: true,
            with_rotation: false,
            solution: String::new(),
        }
    }

    /// Initialise all sub-solver tables. Idempotent.
    pub fn init() {
        // Force lazy-init of all table sets.
        // min2phase tables
        let _ = &*tnoodle_min2phase::coord_cube::COORD_TABLES;

        // center1
        let _ = &*center1::TABLES;

        // center2
        let _ = &*crate::center2::TABLES;

        // center3
        let _ = &*crate::center3::TABLES;

        // edge3
        let _ = &*edge3::TABLES;
    }

    /// Generate a random 4x4 state and return its scramble solution.
    pub fn random_state<R: Rng>(&mut self, rng: &mut R) -> String {
        self.cube = Some(FullCube::random(rng));
        self.do_search();
        self.solution.clone()
    }

    /// Solve a 4x4 from a 96-character facelet string (URFDLB, 16 facelets per face).
    pub fn solution_from_facelet(&mut self, facelet: &str) -> String {
        let mut f = [0u8; 96];
        for (i, ch) in facelet.chars().enumerate().take(96) {
            f[i] = match ch {
                'U' => 0,
                'R' => 1,
                'F' => 2,
                'D' => 3,
                'L' => 4,
                'B' => 5,
                _ => 0,
            };
        }
        self.cube = Some(FullCube::from_facelets(&f));
        self.do_search();
        self.solution.clone()
    }

    /// Solve from a move sequence.
    pub fn solve_moves(&mut self, moveseq: &[usize]) -> String {
        self.cube = Some(FullCube::from_moves(moveseq));
        self.do_search();
        self.solution.clone()
    }

    // ── internal search ──

    fn do_search(&mut self) {
        Self::init();
        self.solution.clear();

        let c1_tables = &*center1::TABLES;

        // Compute phase-1 pruning bounds for all three URF orientations.
        // We need to snapshot the center before the search modifies the cube.
        let c = self.cube.as_mut().unwrap();
        let center_snap = c.get_center().clone();

        let ud_c1 = Center1::from_center_cube(&center_snap, 0);
        let fb_c1 = Center1::from_center_cube(&center_snap, 1);
        let rl_c1 = Center1::from_center_cube(&center_snap, 2);

        let ud = ud_c1.getsym(c1_tables);
        let fb = fb_c1.getsym(c1_tables);
        let rl = rl_c1.getsym(c1_tables);

        let ud_prun = c1_tables.csprun[(ud >> 6) as usize];
        let fb_prun = c1_tables.csprun[(fb >> 6) as usize];
        let rl_prun = c1_tables.csprun[(rl >> 6) as usize];

        self.p1_sols_cnt = 0;
        self.arr2_idx = 0;
        self.p1_sols.clear();

        let min_prun = ud_prun.min(fb_prun).min(rl_prun) as i32;
        let mut found = false;
        for len1 in min_prun..100 {
            self.length1 = len1;
            if rl_prun as i32 <= len1
                && self.search1(
                    (rl >> 6) as usize,
                    (rl & 0x3f) as usize,
                    len1,
                    -1,
                    0,
                )
            {
                found = true;
                break;
            }
            if ud_prun as i32 <= len1
                && self.search1(
                    (ud >> 6) as usize,
                    (ud & 0x3f) as usize,
                    len1,
                    -1,
                    0,
                )
            {
                found = true;
                break;
            }
            if fb_prun as i32 <= len1
                && self.search1(
                    (fb >> 6) as usize,
                    (fb & 0x3f) as usize,
                    len1,
                    -1,
                    0,
                )
            {
                found = true;
                break;
            }
        }

        // Collect phase-1 solutions, sort by value
        let mut p1_sols_arr: Vec<FullCube> = self
            .p1_sols
            .drain()
            .map(|e| e.cube)
            .collect();
        p1_sols_arr.sort_by_key(|c| c.value);

        if p1_sols_arr.is_empty() {
            self.solution = "Error: no phase-1 solution".to_string();
            return;
        }

        // Phase 2
        let c2_tables = &*crate::center2::TABLES;
        let mut max_length2: i32 = 9;
        let mut length12;

        loop {
            length12 = 100i32;
            let mut found2 = false;
            let base_val = p1_sols_arr[0].value;
            'out: for l12 in base_val..100 {
                for i in 0..p1_sols_arr.len() {
                    if p1_sols_arr[i].value > l12 {
                        break;
                    }
                    if l12 - p1_sols_arr[i].length1 > max_length2 {
                        continue;
                    }
                    self.c1.copy_from(&p1_sols_arr[i]);
                    let edge_parity = self.c1.get_edge().get_parity();
                    let center_snap_p2 = self.c1.get_center().clone();
                    self.ct2.set(&center_snap_p2, edge_parity);
                    let s2ct = self.ct2.getct();
                    let s2rl = self.ct2.getrl();
                    self.length1 = p1_sols_arr[i].length1;
                    self.length2 = l12 - p1_sols_arr[i].length1;

                    if self.search2(s2ct, s2rl, self.length2, 28, 0) {
                        length12 = l12;
                        found2 = true;
                        break 'out;
                    }
                }
            }
            if found2 {
                break;
            }
            max_length2 += 1;
            if max_length2 > 30 {
                break;
            }
        }

        // Sort phase-2 results
        let mut arr2_sorted: Vec<usize> = (0..self.arr2_idx).collect();
        arr2_sorted.sort_by_key(|&i| {
            self.arr2[i].as_ref().map(|c| c.value).unwrap_or(i32::MAX)
        });

        if arr2_sorted.is_empty() {
            self.solution = "Error: no phase-2 solution".to_string();
            return;
        }

        // Phase 3
        let mut max_length3: i32 = 13;
        let mut length123;
        let mut best_index = 0usize;

        loop {
            length123 = 100i32;
            let mut found3 = false;

            let base_val3 = self.arr2[arr2_sorted[0]]
                .as_ref()
                .map(|c| c.value)
                .unwrap_or(0);

            'out2: for l123 in base_val3..100 {
                let limit = self.arr2_idx.min(PHASE3_ATTEMPTS);
                for si in 0..limit {
                    let idx = arr2_sorted[si];
                    let arr2_val = self.arr2[idx].as_ref().map(|c| c.value).unwrap_or(i32::MAX);
                    if arr2_val > l123 {
                        break;
                    }
                    let arr2_l1 = self.arr2[idx].as_ref().unwrap().length1;
                    let arr2_l2 = self.arr2[idx].as_ref().unwrap().length2;
                    if l123 - arr2_l1 - arr2_l2 > max_length3 {
                        continue;
                    }

                    // Set up edge3 from the arr2 cube
                    let edge_snap = self.arr2[idx].as_mut().unwrap().get_edge().clone();
                    let eparity = self.e12.set_edge_cube(&edge_snap);
                    let corner_parity = self.arr2[idx].as_mut().unwrap().get_corner().get_parity();
                    let center_snap2 = self.arr2[idx].as_mut().unwrap().get_center().clone();
                    self.ct3.set_from_cube(&center_snap2, eparity ^ corner_parity);
                    let ct3_tables_ref = &*crate::center3::TABLES;
                    let ct = self.ct3.getct(&ct3_tables_ref.std2rl);
                    let edge_coord = self.e12.get(10);
                    let prun = edge3::getprun_full(self.e12.getsym());
                    let lm = 20usize;

                    let remaining = l123 - arr2_l1 - arr2_l2;
                    if prun <= remaining {
                        let ct3_tables = &*crate::center3::TABLES;
                        let ct3_prun = ct3_tables.prun[ct as usize];
                        if ct3_prun as i32 <= remaining
                            && self.search3(
                                edge_coord,
                                ct as usize,
                                prun,
                                remaining,
                                lm,
                                0,
                            )
                        {
                            best_index = idx;
                            length123 = l123;
                            found3 = true;
                            break 'out2;
                        }
                    }
                }
            }
            if found3 {
                break;
            }
            max_length3 += 1;
            if max_length3 > 30 {
                break;
            }
        }

        if length123 >= 100 {
            self.solution = "Error: no phase-3 solution".to_string();
            return;
        }

        // Build the solution cube
        let mut sol_cube = FullCube::new();
        sol_cube.copy_from(self.arr2[best_index].as_ref().unwrap());
        let l1 = sol_cube.length1;
        let l2 = sol_cube.length2;
        let length3 = length123 - l1 - l2;

        for i in 0..length3 as usize {
            sol_cube.push_move(MOVE3STD[self.move3[i] as usize] as usize);
        }

        // Get the 3x3 facelet and solve with min2phase
        let facelet_333 = sol_cube.to_333_facelet();

        let sol_333 = self.search333.solution(&relabel_by_centers(&facelet_333), 21, 1000000, 500, 0);

        match sol_333 {
            SearchResult::Solution(sol_str) => {
                // Parse solution string into move indices and apply
                let sol_moves = parse_moves_333(&sol_str);
                for &m in &sol_moves {
                    sol_cube.push_move(m);
                }

                self.solution = sol_cube.get_move_string(self.inverse_solution, self.with_rotation);
            }
            SearchResult::Error(code) => {
                self.solution = format!("Error {}", code);
            }
        }
    }

    // ── Phase 1 search ──

    fn search1(
        &mut self,
        ct: usize,
        sym: usize,
        maxl: i32,
        lm: i32,
        depth: usize,
    ) -> bool {
        let c1_tables = &*center1::TABLES;

        if ct == 0 && maxl < 5 {
            return maxl == 0 && self.init2(sym, lm);
        }

        for axis in (0..27).step_by(3) {
            if axis == lm || axis == lm - 9 || axis == lm - 18 {
                continue;
            }
            for power in 0..3 {
                let m = axis + power;
                let sm = c1_tables.symmove[sym][m as usize] as usize;
                let ctx_raw = c1_tables.ctsmv[ct][sm] as i32;
                let prun = c1_tables.csprun[(ctx_raw >> 6) as usize];
                if prun as i32 >= maxl {
                    if (prun as i32) > maxl {
                        break;
                    }
                    continue;
                }
                let symx = c1_tables.symmult[sym][(ctx_raw & 0x3f) as usize] as usize;
                let ctx = (ctx_raw >> 6) as usize;
                self.move1[depth] = m;
                if self.search1(ctx, symx, maxl - 1, axis, depth + 1) {
                    return true;
                }
            }
        }
        false
    }

    fn init2(&mut self, sym: usize, _lm: i32) -> bool {
        let c1_tables = &*center1::TABLES;

        self.c1.copy_from(self.cube.as_ref().unwrap());
        for i in 0..self.length1 as usize {
            self.c1.push_move(self.move1[i] as usize);
        }

        let mut add1 = false;
        let finish_val = c1_tables.finish[sym];
        let sym_new;

        match finish_val {
            0 => {
                // fb axis: apply fx1 bx3
                self.c1.push_move(FWX1 as usize);
                self.c1.push_move(BWX3 as usize);
                self.move1[self.length1 as usize] = FWX1;
                self.move1[self.length1 as usize + 1] = BWX3;
                add1 = true;
                sym_new = 19;
            }
            12869 => {
                // ud axis: apply ux1 dx3
                self.c1.push_move(UWX1 as usize);
                self.c1.push_move(DWX3 as usize);
                self.move1[self.length1 as usize] = UWX1;
                self.move1[self.length1 as usize + 1] = DWX3;
                add1 = true;
                sym_new = 34;
            }
            735470 => {
                add1 = false;
                sym_new = 0;
            }
            _ => {
                // shouldn't happen, but handle gracefully
                add1 = false;
                sym_new = 0;
            }
        }

        let edge_parity = self.c1.get_edge().get_parity();
        let center_snap_init2 = self.c1.get_center().clone();
        self.ct2.set(&center_snap_init2, edge_parity);
        let s2ct = self.ct2.getct();
        let s2rl = self.ct2.getrl();

        let c2_tables = &*crate::center2::TABLES;
        let ctp = c2_tables.ctprun[(s2ct as usize) * 70 + (s2rl as usize)];

        self.c1.value = ctp as i32 + self.length1;
        self.c1.length1 = self.length1;
        self.c1.add1 = add1;
        self.c1.sym = sym_new;
        self.p1_sols_cnt += 1;

        // Add to priority queue (keeping at most PHASE2_ATTEMPTS entries)
        let entry = FullCubeEntry {
            cube: self.c1.clone_cube(),
        };

        if self.p1_sols.len() < PHASE2_ATTEMPTS {
            self.p1_sols.push(entry);
        } else {
            // Check if the worst in the heap is worse than the new one
            if let Some(worst) = self.p1_sols.peek() {
                if worst.cube.value > self.c1.value {
                    self.p1_sols.pop();
                    self.p1_sols.push(entry);
                }
            }
        }

        self.p1_sols_cnt == PHASE1_SOLUTIONS
    }

    // ── Phase 2 search ──

    fn search2(
        &mut self,
        ct: i32,
        rl: i32,
        maxl: i32,
        lm: usize,
        depth: usize,
    ) -> bool {
        let c2_tables = &*crate::center2::TABLES;

        if ct == 0 && c2_tables.ctprun[rl as usize] == 0 && maxl == 0 {
            return self.init3();
        }

        let ckmv2 = &*CKMV2;
        let skip_axis2 = &*SKIP_AXIS2;

        let mut m = 0usize;
        while m < 23 {
            if ckmv2[lm][m] {
                m = skip_axis2[m] as usize;
                m += 1;
                continue;
            }
            let ctx = c2_tables.ctmv[ct as usize][m] as i32;
            let rlx = c2_tables.rlmv[rl as usize][m] as i32;

            let prun = c2_tables.ctprun[(ctx as usize) * 70 + (rlx as usize)];
            if prun as i32 >= maxl {
                if (prun as i32) > maxl {
                    m = skip_axis2[m] as usize;
                }
                m += 1;
                continue;
            }

            self.move2[depth] = MOVE2STD[m];
            if self.search2(ctx, rlx, maxl - 1, m, depth + 1) {
                return true;
            }
            m += 1;
        }
        false
    }

    fn init3(&mut self) -> bool {
        self.c2.copy_from(&self.c1);
        for i in 0..self.length2 as usize {
            self.c2.push_move(self.move2[i] as usize);
        }
        if !self.c2.check_edge() {
            return false;
        }

        let edge_snap = self.c2.get_edge().clone();
        let eparity = self.e12.set_edge_cube(&edge_snap);
        let corner_parity = self.c2.get_corner().get_parity();
        let center_snap = self.c2.get_center().clone();
        self.ct3.set_from_cube(&center_snap, eparity ^ corner_parity);
        let ct3_tables_init3 = &*crate::center3::TABLES;
        let ct = self.ct3.getct(&ct3_tables_init3.std2rl);
        let prun = edge3::getprun_full(self.e12.getsym());

        let ct3_tables = &*crate::center3::TABLES;

        if self.arr2[self.arr2_idx].is_none() {
            self.arr2[self.arr2_idx] = Some(self.c2.clone_cube());
        } else {
            self.arr2[self.arr2_idx].as_mut().unwrap().copy_from(&self.c2);
        }
        let cube = self.arr2[self.arr2_idx].as_mut().unwrap();
        cube.value = self.length1 + self.length2 + prun.max(ct3_tables.prun[ct as usize] as i32);
        cube.length2 = self.length2;
        self.arr2_idx += 1;

        self.arr2_idx == self.arr2.len()
    }

    // ── Phase 3 search ──

    fn search3(
        &mut self,
        edge: i32,
        ct: usize,
        prun: i32,
        maxl: i32,
        lm: usize,
        depth: usize,
    ) -> bool {
        if maxl == 0 {
            return edge == 0 && ct == 0;
        }

        let ct3_tables = &*crate::center3::TABLES;
        let e3_tables = &*edge3::TABLES;
        let ckmv3 = &*CKMV3;
        let skip_axis3 = &*SKIP_AXIS3;

        self.tempe[depth].set_idx(edge);

        let mut m = 0usize;
        while m < 17 {
            if ckmv3[lm][m] {
                m = skip_axis3[m] as usize;
                m += 1;
                continue;
            }

            let ctx = ct3_tables.ctmove[ct][m] as usize;
            let prun1 = ct3_tables.prun[ctx] as i32;
            if prun1 >= maxl {
                if prun1 > maxl && m < 14 {
                    m = skip_axis3[m] as usize;
                }
                m += 1;
                continue;
            }

            let edgex =
                edge3::getmvrot(&self.tempe[depth].edge, m << 3, 10);
            let cord1x = edgex / (edge3::N_RAW as i32);
            let symcord1x_full = e3_tables.raw2sym[cord1x as usize];
            let symx = (symcord1x_full & 0x7) as usize;
            let symcord1x = (symcord1x_full >> 3) as usize;
            let cord2x = edge3::getmvrot(
                &self.tempe[depth].edge,
                (m << 3) | symx,
                10,
            ) % (edge3::N_RAW as i32);
            let prunx = edge3::getprun(
                symcord1x * edge3::N_RAW + cord2x as usize,
                prun,
            );
            if prunx >= maxl {
                if prunx > maxl && m < 14 {
                    m = skip_axis3[m] as usize;
                }
                m += 1;
                continue;
            }

            if self.search3(edgex, ctx, prunx, maxl - 1, m, depth + 1) {
                self.move3[depth] = m as i32;
                return true;
            }
            m += 1;
        }
        false
    }
}

// ─── helpers ─────────────────────────────────────────────────────────

/// Parse a min2phase solution string (e.g. "U R2 F' ...") into move indices.
/// The reduced cube is usually rotated, so its face colors don't match the
/// face letters. Java's min2phase `verify` reads colors relative to each
/// face's center sticker; the Rust min2phase does not, so relabel here.
fn relabel_by_centers(facelets: &str) -> String {
    let f = facelets.as_bytes();
    let centers = [f[4], f[13], f[22], f[31], f[40], f[49]];
    f.iter()
        .map(|c| {
            let face = centers.iter().position(|x| x == c).expect("every color has a center");
            b"URFDLB"[face] as char
        })
        .collect()
}

fn parse_moves_333(sol: &str) -> Vec<usize> {
    let mut result = Vec::new();
    for token in sol.split_whitespace() {
        let mv = match token {
            "U" => Some(0),
            "U2" => Some(1),
            "U'" => Some(2),
            "R" => Some(3),
            "R2" => Some(4),
            "R'" => Some(5),
            "F" => Some(6),
            "F2" => Some(7),
            "F'" => Some(8),
            "D" => Some(9),
            "D2" => Some(10),
            "D'" => Some(11),
            "L" => Some(12),
            "L2" => Some(13),
            "L'" => Some(14),
            "B" => Some(15),
            "B2" => Some(16),
            "B'" => Some(17),
            _ => None,
        };
        if let Some(m) = mv {
            result.push(m);
        }
    }
    result
}

// ─── tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_new() {
        let s = Search::new();
        assert_eq!(s.p1_sols_cnt, 0);
        assert!(s.solution.is_empty());
    }

    #[test]
    fn test_parse_moves_333() {
        let moves = parse_moves_333("U R2 F' D L2 B");
        assert_eq!(moves, vec![0, 4, 8, 9, 13, 15]);
    }

    #[test]
    fn test_parse_moves_333_empty() {
        let moves = parse_moves_333("");
        assert!(moves.is_empty());
    }

    #[test]
    fn relabel_maps_center_colors_to_faces() {
        // A y rotation of a solved cube: U/D centers stay, side colors shift.
        let rotated = "UUUUUUUUUBBBBBBBBBRRRRRRRRRDDDDDDDDDFFFFFFFFFLLLLLLLLL";
        assert_eq!(
            relabel_by_centers(rotated),
            "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB"
        );
    }

    fn facelets(c: &mut FullCube) -> [u8; 96] {
        let mut f = [0u8; 96];
        c.to_facelet(&mut f);
        f
    }

    fn parse_4x4(alg: &str) -> Vec<usize> {
        alg.split_whitespace()
            .map(|m| MOVE2STR.iter().position(|s| s.trim() == m).unwrap_or_else(|| panic!("bad move {m}")))
            .collect()
    }

    /// All 24 orientations of `f`, generated from x = Rw Lw', y = Uw Dw', z = Fw Bw'.
    fn orientations(f: [u8; 96]) -> Vec<[u8; 96]> {
        let rotations = [[RWX1, LWX3], [UWX1, DWX3], [FWX1, BWX3]];
        let mut seen = vec![f];
        let mut i = 0;
        while i < seen.len() {
            for rot in rotations {
                let mut c = FullCube::from_facelets(&seen[i]);
                for m in rot {
                    c.do_move(m as usize);
                }
                let next = facelets(&mut c);
                if !seen.contains(&next) {
                    seen.push(next);
                }
            }
            i += 1;
        }
        assert_eq!(seen.len(), 24);
        seen
    }

    #[test]
    fn solutions_reproduce_the_input_state() {
        // Regression: the reduced 3x3 is usually rotated and used to reach
        // min2phase with wrong colors, giving "Error -6" or wrong scrambles.
        let mut rng = rand::rng();
        let mut search = Search::new();
        for _ in 0..10 {
            let moves: Vec<usize> = (0..40).map(|_| rand::Rng::random_range(&mut rng, 0..36)).collect();
            let target = facelets(&mut FullCube::from_moves(&moves));
            let facelet: String = target.iter().map(|&c| "URFDLB".as_bytes()[c as usize] as char).collect();
            let scramble = search.solution_from_facelet(&facelet);
            assert!(!scramble.starts_with("Error"), "{scramble}");
            let produced = facelets(&mut FullCube::from_moves(&parse_4x4(&scramble)));
            assert!(orientations(produced).contains(&target), "{scramble} does not reproduce the state");
        }
    }

    #[test]
    fn random_state_never_errors() {
        let mut rng = rand::rng();
        let mut search = Search::new();
        for _ in 0..10 {
            let scramble = search.random_state(&mut rng);
            assert!(!scramble.starts_with("Error"), "{scramble}");
            let len = parse_4x4(&scramble).len();
            assert!((30..=55).contains(&len), "{len} moves: {scramble}");
        }
    }
}
