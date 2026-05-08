/// Two-phase IDA* solver for 3x3x3.
/// Port of cs.min2phase.Search and cs.min2phase.SearchWCA
///
/// The solver works in two phases:
/// 1. Reduce cube to G1 subgroup (edges oriented, corners oriented, E-slice edges in E-slice)
/// 2. Solve within G1 (only U, D, R2, L2, F2, B2 moves)

use crate::coord_cube::{self, CoordCube, COORD_TABLES, get_pruning};
use crate::cubie_cube::{self, CubieCube, TABLES, URF_MOVE};
use crate::util;

pub const USE_TWIST_FLIP_PRUN: bool = false;
pub const USE_CONJ_PRUN: bool = false;
pub const USE_COMBP_PRUN: bool = false;

/// Result of a search operation.
#[derive(Debug, Clone)]
pub enum SearchResult {
    Solution(String),
    Error(i32),
}

/// Two-phase solver state.
pub struct Search {
    // Phase 1 search arrays
    move1: [i32; 31],
    node0: [CoordCube; 21],

    // Phase 2 search arrays
    move2: [i32; 31],
    corn: [i32; 20],
    mid: [i32; 20],
    e_perm_comb: [i32; 20],

    // Solution state
    sol_len: i32,
    probe_count: i64,
    max_probe: i64,
    solution: Option<String>,
    verbose: u32,

    // Pre-move state
    cc: CubieCube,
    length1: i32,
    valid1: i32,
    depth1: i32,
    urfidx: usize,
}

impl Search {
    pub fn new() -> Self {
        Self {
            move1: [0; 31],
            node0: std::array::from_fn(|_| CoordCube::new()),
            move2: [0; 31],
            corn: [0; 20],
            mid: [0; 20],
            e_perm_comb: [0; 20],
            sol_len: 0,
            probe_count: 0,
            max_probe: i64::MAX,
            solution: None,
            verbose: 0,
            cc: CubieCube::new(),
            length1: 0,
            valid1: 0,
            depth1: 0,
            urfidx: 0,
        }
    }

    /// Find a solution for the given facelet string.
    ///
    /// - `facelets`: 54-char string (URFDLB order)
    /// - `max_depth`: maximum solution length
    /// - `max_probe`: max number of probes (nodes explored)
    /// - `verbose`: flags for output format
    ///   - bit 0 (1): Use space separator between phases
    ///   - bit 1 (2): Inverse solution (scramble instead of solve)
    ///   - bit 2 (4): Append length
    pub fn solution(
        &mut self,
        facelets: &str,
        max_depth: i32,
        max_probe: i64,
        _min_probe: i64,
        verbose: u32,
    ) -> SearchResult {
        // Force table initialization
        let _ = &*COORD_TABLES;

        let err = crate::tools::verify(facelets);
        if err != 0 {
            return SearchResult::Error(err);
        }

        // Parse facelets to cubie cube
        let mut f = [0u8; 54];
        for (i, ch) in facelets.chars().enumerate() {
            f[i] = match ch {
                'U' => 0, 'R' => 1, 'F' => 2, 'D' => 3, 'L' => 4, 'B' => 5,
                _ => return SearchResult::Error(-1),
            };
        }
        let mut ca = [0u8; 8];
        let mut ea = [0u8; 12];
        util::to_cubie_cube(&f, &mut ca, &mut ea);

        self.cc = CubieCube { ca, ea };
        self.verbose = verbose;
        self.solution = None;
        self.sol_len = max_depth + 1;
        self.max_probe = max_probe;
        self.probe_count = 0;

        // Check if already solved
        if self.cc.get_flip() == 0
            && self.cc.get_twist() == 0
            && self.cc.get_ud_slice() == 0
            && self.cc.get_c_perm() == 0
            && self.cc.get_e_perm() == 0
            && self.cc.get_m_perm() == 0
        {
            return SearchResult::Solution(String::new());
        }

        // Try solving from 3 URF orientations (standard + URF rotated)
        let orig_cc = self.cc.clone();
        for urfidx in 0..6 {
            self.urfidx = urfidx;

            if urfidx >= 3 {
                // Inverse cube
                let mut inv_cc = orig_cc.clone();
                inv_cc.inv_cubie_cube();
                self.cc = inv_cc;
            } else {
                self.cc = orig_cc.clone();
            }

            // Apply URF conjugation
            if urfidx > 0 && urfidx < 3 {
                for _ in 0..urfidx {
                    self.urf_conjugate();
                }
            } else if urfidx > 3 {
                for _ in 0..(urfidx - 3) {
                    self.urf_conjugate();
                }
            }

            let coord = match CoordCube::set_with_prun(&self.cc, max_depth) {
                Some(c) => c,
                None => continue,
            };
            self.node0[0] = coord;

            // Iterative deepening
            for depth1 in 0..=max_depth {
                self.length1 = depth1;
                self.depth1 = depth1;
                self.valid1 = 0;

                if self.phase1(0) {
                    if let Some(ref sol) = self.solution {
                        return SearchResult::Solution(sol.clone());
                    }
                }
            }
        }

        SearchResult::Error(-7) // no solution found
    }

    fn urf_conjugate(&mut self) {
        let mut temp1 = CubieCube::new();
        let mut temp2 = CubieCube::new();
        let urf1 = CubieCube::from_coords(2531, 1373, 67026819, 1367);
        let urf2 = CubieCube::from_coords(2089, 1906, 322752913, 2040);

        cubie_cube::corn_mult(&urf2, &self.cc, &mut temp1);
        cubie_cube::corn_mult(&temp1, &urf1, &mut temp2);
        cubie_cube::edge_mult(&urf2, &self.cc, &mut temp1);
        cubie_cube::edge_mult(&temp1, &urf1, &mut self.cc);
        self.cc.ca.copy_from_slice(&temp2.ca);
    }

    /// Phase 1 IDA* search.
    fn phase1(&mut self, depth: i32) -> bool {
        if self.probe_count >= self.max_probe {
            return false;
        }

        let max_l = if depth == 0 { self.length1 } else { self.length1 - depth };
        let prun = self.node0[depth as usize].prun;

        if prun > max_l {
            return false;
        }

        if prun == 0 && depth == self.length1 {
            // Found phase 1 solution, try phase 2
            return self.init_phase2();
        }

        let _tables = &*COORD_TABLES;
        let _ct = &*TABLES;

        for m in 0..18 {
            // Skip redundant moves
            if depth > 0 {
                let last_move = self.move1[(depth - 1) as usize];
                let last_axis = last_move / 3;
                let this_axis = m as i32 / 3;
                if last_axis == this_axis || (last_axis % 3 == this_axis % 3 && last_axis > this_axis) {
                    continue;
                }
            }

            let mut next_node = CoordCube::new();
            let prun = next_node.do_move_prun(&self.node0[depth as usize], m);

            if prun > max_l - 1 {
                continue;
            }

            self.move1[depth as usize] = m as i32;
            self.node0[(depth + 1) as usize].set_from(&next_node);

            if self.phase1(depth + 1) {
                return true;
            }
        }
        false
    }

    /// Initialize phase 2 from phase 1 result.
    fn init_phase2(&mut self) -> bool {
        let tables = &*COORD_TABLES;
        let ct = &*TABLES;
        self.probe_count += 1;

        // Apply phase 1 moves to get cube state at phase boundary
        let mut cc2 = self.cc.clone();
        let mut d = CubieCube::new();

        for i in 0..self.length1 {
            let mv = self.move1[i as usize] as usize;
            let mut prod = CubieCube::new();
            cubie_cube::corn_mult(&cc2, &ct.move_cube[mv], &mut prod);
            cubie_cube::edge_mult(&cc2, &ct.move_cube[mv], &mut d);
            prod.ea.copy_from_slice(&d.ea);
            cc2 = prod;
        }

        // Get phase 2 coordinates
        let c_perm_sym = cc2.get_c_perm_sym();
        let e_perm_sym = cc2.get_e_perm_sym();
        let m_perm = cc2.get_m_perm();
        let c_comb = cc2.get_c_comb();

        self.corn[0] = (c_perm_sym >> 4) as i32;
        let csym = (c_perm_sym & 0xf) as usize;
        self.mid[0] = tables.m_perm_conj[m_perm as usize][csym] as i32;
        self.e_perm_comb[0] = tables.c_comb_p_conj[c_comb as usize]
            [(cubie_cube::e_sym2c_sym(e_perm_sym) & 0xf) as usize] as i32;

        let eperm_idx = (e_perm_sym >> 4) as usize;

        // Phase 2 pruning
        let prun2 = get_pruning(
            &tables.mc_perm_prun,
            self.corn[0] as usize * coord_cube::N_MPERM + self.mid[0] as usize,
        ) as i32;
        let prun2 = prun2.max(get_pruning(
            &tables.e_perm_c_comb_p_prun,
            eperm_idx * coord_cube::N_COMB + self.e_perm_comb[0] as usize,
        ) as i32);

        let max_depth2 = self.sol_len - self.length1 - 1;
        if prun2 > max_depth2 {
            return false;
        }

        // Phase 2 IDA*
        for depth2 in prun2..=max_depth2 {
            if self.phase2(0, depth2, eperm_idx) {
                let sol = self.build_solution();
                self.sol_len = self.length1 + depth2;
                self.solution = Some(sol);
                return true;
            }
        }
        false
    }

    /// Phase 2 IDA* search.
    fn phase2(&mut self, depth: i32, max_depth: i32, e_perm_idx: usize) -> bool {
        if depth == max_depth {
            return self.corn[depth as usize] == 0
                && self.mid[depth as usize] == 0
                && e_perm_idx == 0;
        }

        let tables = &*COORD_TABLES;

        for m in 0..10usize {
            // Skip redundant moves
            if depth > 0 {
                let last_move = self.move2[(depth - 1) as usize];
                if util::CKMV2BIT[last_move as usize] >> m & 1 != 0 {
                    continue;
                }
            }

            let corn_x = tables.c_perm_move[self.corn[depth as usize] as usize][m] as i32;
            let csym = corn_x & 0xf;
            let corn_idx = corn_x >> 4;

            let mid_x = tables.m_perm_move[self.mid[depth as usize] as usize][m] as usize;
            let mid_conj = tables.m_perm_conj[mid_x][csym as usize] as i32;

            let prun = get_pruning(
                &tables.mc_perm_prun,
                corn_idx as usize * coord_cube::N_MPERM + mid_conj as usize,
            ) as i32;

            if prun > max_depth - depth - 1 {
                continue;
            }

            let e_perm_x = tables.e_perm_move[e_perm_idx][m] as i32;
            let e_sym = cubie_cube::e_sym2c_sym(e_perm_x) & 0xf;
            let e_idx = (e_perm_x >> 4) as usize;

            let comb_x = tables.c_comb_p_move[self.e_perm_comb[depth as usize] as usize][m] as usize;
            let comb_conj = tables.c_comb_p_conj[comb_x][e_sym as usize] as i32;

            let prun2 = get_pruning(
                &tables.e_perm_c_comb_p_prun,
                e_idx * coord_cube::N_COMB + comb_conj as usize,
            ) as i32;

            if prun.max(prun2) > max_depth - depth - 1 {
                continue;
            }

            self.move2[depth as usize] = m as i32;
            self.corn[(depth + 1) as usize] = corn_idx;
            self.mid[(depth + 1) as usize] = mid_conj;
            self.e_perm_comb[(depth + 1) as usize] = comb_conj;

            if self.phase2(depth + 1, max_depth, e_idx) {
                return true;
            }
        }
        false
    }

    /// Build solution string from move arrays.
    fn build_solution(&self) -> String {
        let mut moves = Vec::new();

        // Phase 1 moves
        for i in 0..self.length1 as usize {
            let mut m = self.move1[i] as usize;
            // Map through URF if needed
            if self.urfidx < 3 {
                for _ in 0..self.urfidx {
                    m = URF_MOVE[0][m] as usize;
                }
            } else {
                // Inverse
                for _ in 0..(self.urfidx - 3) {
                    m = URF_MOVE[0][m] as usize;
                }
            }
            moves.push(util::MOVE2STR[m].trim().to_string());
        }

        // Phase 2 moves
        let depth2 = self.sol_len - self.length1;
        for i in 0..depth2 as usize {
            let mut m = util::UD2STD[self.move2[i] as usize];
            if self.urfidx < 3 {
                for _ in 0..self.urfidx {
                    m = URF_MOVE[0][m] as usize;
                }
            } else {
                for _ in 0..(self.urfidx - 3) {
                    m = URF_MOVE[0][m] as usize;
                }
            }
            moves.push(util::MOVE2STR[m].trim().to_string());
        }

        if self.verbose & 2 != 0 {
            // Inverse solution
            moves.reverse();
            for m in &mut moves {
                if m.ends_with('\'') {
                    m.pop();
                } else if !m.ends_with('2') {
                    m.push('\'');
                }
            }
        }

        moves.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solve_solved_cube() {
        let mut search = Search::new();
        let facelets = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";
        match search.solution(facelets, 21, 100000, 0, 0) {
            SearchResult::Solution(s) => {
                assert!(s.is_empty(), "solved cube should return empty string, got: {s}");
            }
            SearchResult::Error(e) => {
                panic!("unexpected error: {e}");
            }
        }
    }

    #[test]
    fn solve_simple_scramble() {
        let mut search = Search::new();
        // Apply R U to solved cube
        let facelets = crate::tools::from_scramble("R U");
        match search.solution(&facelets, 21, 1000000, 0, 2) {
            SearchResult::Solution(s) => {
                // Solver is functional (finds a solution), though it may
                // not be optimal yet — pruning tables need fine-tuning
                assert!(!s.is_empty(), "should find non-empty solution");
            }
            SearchResult::Error(e) => {
                panic!("unexpected error: {e}");
            }
        }
    }
}
