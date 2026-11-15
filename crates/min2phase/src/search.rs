/// Two-phase IDA* solver for 3x3x3.
/// Port of cs.min2phase.Search and cs.min2phase.SearchWCA
///
/// The solver works in two phases:
/// 1. Reduce cube to G1 subgroup (edges oriented, corners oriented, E-slice edges in E-slice)
/// 2. Solve within G1 (only U, D, R2, L2, F2, B2 moves)

use crate::coord_cube::{CoordCube, COORD_TABLES};
use crate::cubie_cube::{self, CubieCube, TABLES, URF_MOVE};
use crate::phase2::PHASE2_TABLES;
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
    c_perm: [usize; 32],
    e_perm: [usize; 32],
    m_perm: [usize; 32],

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

    // Axis restrictions (0 = U/D, 1 = R/L, 2 = F/B) on the solution's
    // first/last move. Port of SearchWCA's firstMoveFilter/lastMoveFilter.
    first_axis: Option<usize>,
    last_axis: Option<usize>,
    /// Phase-2 move index (UD order) that the phase-2 search must not
    /// follow redundantly: the face of the last phase-1 move, or 10 for none.
    boundary_lm: usize,
}

impl Search {
    pub fn new() -> Self {
        Self {
            move1: [0; 31],
            node0: std::array::from_fn(|_| CoordCube::new()),
            move2: [0; 31],
            c_perm: [0; 32],
            e_perm: [0; 32],
            m_perm: [0; 32],
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
            first_axis: None,
            last_axis: None,
            boundary_lm: 10,
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
        min_probe: i64,
        verbose: u32,
    ) -> SearchResult {
        self.solution_with_restrictions(facelets, max_depth, max_probe, min_probe, verbose, None, None)
    }

    /// Like [`Search::solution`], but the solution (before any inversion
    /// requested by `verbose`) may not start with a move on `first_axis`
    /// nor end with a move on `last_axis`. Axes are named by any face on
    /// them ("U"/"D", "R"/"L", "F"/"B").
    ///
    /// Port of `SearchWCA.solution(..., firstAxisRestriction, lastAxisRestriction)`.
    #[allow(clippy::too_many_arguments)]
    pub fn solution_with_restrictions(
        &mut self,
        facelets: &str,
        max_depth: i32,
        max_probe: i64,
        _min_probe: i64,
        verbose: u32,
        first_axis: Option<&str>,
        last_axis: Option<&str>,
    ) -> SearchResult {
        let parse_axis = |face: Option<&str>| -> Result<Option<usize>, i32> {
            match face {
                None => Ok(None),
                Some(f) => util::MOVE2STR
                    .iter()
                    .position(|m| m.trim() == f)
                    .map(|i| Some(i / 3 % 3))
                    .ok_or(-9),
            }
        };
        self.first_axis = match parse_axis(first_axis) {
            Ok(a) => a,
            Err(e) => return SearchResult::Error(e),
        };
        self.last_axis = match parse_axis(last_axis) {
            Ok(a) => a,
            Err(e) => return SearchResult::Error(e),
        };
        // Force table initialization
        let _ = &*COORD_TABLES;

        let err = crate::tools::verify(facelets);
        if err != 0 {
            return SearchResult::Error(err);
        }

        // Parse facelets to cubie cube (colors named by their centers)
        let Ok(f) = crate::tools::parse_facelets(facelets) else {
            return SearchResult::Error(-1);
        };
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

        // Try solving from the URF-conjugated orientations. Inverse-cube
        // searches (urfidx >= 3) are not supported because build_solution
        // does not invert their result. Axis restrictions are expressed in
        // the original frame, so restricted searches only use urfidx 0.
        let orig_cc = self.cc.clone();
        let urf_count = if self.first_axis.is_some() || self.last_axis.is_some() { 1 } else { 3 };
        for urfidx in 0..urf_count {
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
            if depth == 0 && self.first_axis == Some(m / 3 % 3) {
                continue;
            }
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
        let ct = &*TABLES;
        let p2 = &*PHASE2_TABLES;
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
        debug_assert!(cc2.get_twist() == 0 && cc2.get_flip() == 0 && cc2.get_ud_slice() == 0);

        self.c_perm[0] = cc2.get_c_perm() as usize;
        self.e_perm[0] = cc2.get_e_perm() as usize;
        self.m_perm[0] = cc2.get_m_perm() as usize;

        // Phase 2 must not start on the face of the last phase-1 move (or
        // redundantly on its opposite face), same as Java's ckmv2bit[lm].
        self.boundary_lm = if self.length1 > 0 {
            let last = self.move1[(self.length1 - 1) as usize] as usize;
            util::STD2UD[last / 3 * 3 + 1]
        } else {
            10
        };

        let prun2 = p2.prun(self.c_perm[0], self.e_perm[0], self.m_perm[0]);
        let max_depth2 = self.sol_len - self.length1 - 1;
        if prun2 > max_depth2 {
            return false;
        }

        // Phase 2 IDA*
        for depth2 in prun2..=max_depth2 {
            if self.phase2(0, depth2) {
                let sol = self.build_solution(depth2);
                self.sol_len = self.length1 + depth2;
                self.solution = Some(sol);
                return true;
            }
        }
        false
    }

    /// Phase 2 IDA* search.
    fn phase2(&mut self, depth: i32, max_depth: i32) -> bool {
        let p2 = &*PHASE2_TABLES;
        let d = depth as usize;
        if depth == max_depth {
            return self.c_perm[d] == 0
                && self.e_perm[d] == 0
                && self.m_perm[d] == 0
                && !self.last_move_restricted(depth);
        }

        for m in 0..10usize {
            // Skip redundant moves
            let last_move = if depth > 0 {
                self.move2[d - 1] as usize
            } else {
                self.boundary_lm
            };
            if util::CKMV2BIT[last_move] >> m & 1 != 0 {
                continue;
            }
            if depth == 0 && self.length1 == 0 && self.first_axis == Some(util::UD2STD[m] / 3 % 3) {
                continue;
            }

            let c_perm = p2.c_perm_move[self.c_perm[d]][m] as usize;
            let e_perm = p2.e_perm_move[self.e_perm[d]][m] as usize;
            let m_perm = p2.m_perm_move[self.m_perm[d]][m] as usize;
            if p2.prun(c_perm, e_perm, m_perm) > max_depth - depth - 1 {
                continue;
            }

            self.move2[d] = m as i32;
            self.c_perm[d + 1] = c_perm;
            self.e_perm[d + 1] = e_perm;
            self.m_perm[d + 1] = m_perm;

            if self.phase2(depth + 1, max_depth) {
                return true;
            }
        }
        false
    }

    /// Whether the solution's last move (phase 2 of length `depth2`,
    /// falling back to phase 1) is on the restricted last axis.
    fn last_move_restricted(&self, depth2: i32) -> bool {
        let Some(axis) = self.last_axis else {
            return false;
        };
        let last = if depth2 > 0 {
            util::UD2STD[self.move2[(depth2 - 1) as usize] as usize]
        } else if self.length1 > 0 {
            self.move1[(self.length1 - 1) as usize] as usize
        } else {
            return false;
        };
        last / 3 % 3 == axis
    }

    /// Build solution string from move arrays (`depth2` phase-2 moves).
    fn build_solution(&self, depth2: i32) -> String {
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

    fn axis(mv: &str) -> usize {
        "URFDLB".find(&mv[..1]).unwrap() % 3
    }

    fn solve_ok(search: &mut Search, facelets: &str, verbose: u32, first: Option<&str>, last: Option<&str>) -> String {
        match search.solution_with_restrictions(facelets, 21, 1_000_000, 0, verbose, first, last) {
            SearchResult::Solution(s) => s,
            SearchResult::Error(e) => panic!("error {e} for {facelets}"),
        }
    }

    #[test]
    fn inverse_solution_reproduces_state_within_21_moves() {
        let mut rng = rand::rng();
        let mut search = Search::new();
        for _ in 0..200 {
            let facelets = crate::tools::random_state(&mut rng);
            let scramble = solve_ok(&mut search, &facelets, 2, None, None);
            let moves: Vec<&str> = scramble.split_whitespace().collect();
            assert!(moves.len() <= 21, "{} moves: {scramble}", moves.len());
            assert_eq!(crate::tools::from_scramble(&scramble), facelets, "scramble {scramble}");
            for w in moves.windows(2) {
                assert_ne!(&w[0][..1], &w[1][..1], "same face twice: {scramble}");
            }
            for w in moves.windows(3) {
                let same_axis = axis(w[0]) == axis(w[1]) && axis(w[1]) == axis(w[2]);
                assert!(!same_axis, "redundant same-axis triple: {scramble}");
            }
        }
    }

    #[test]
    fn axis_restrictions_are_respected() {
        let mut rng = rand::rng();
        let mut search = Search::new();
        for (first, last) in [("R", "F"), ("L", "L"), ("U", "B"), ("F", "D")] {
            for _ in 0..20 {
                let facelets = crate::tools::random_state(&mut rng);
                let solution = solve_ok(&mut search, &facelets, 0, Some(first), Some(last));
                let moves: Vec<&str> = solution.split_whitespace().collect();
                assert_ne!(axis(moves[0]), axis(first), "{solution} starts on {first} axis");
                assert_ne!(axis(moves[moves.len() - 1]), axis(last), "{solution} ends on {last} axis");
                // Applying the inverse of the solution must give back the state.
                let scramble = solve_ok(&mut search, &facelets, 2, Some(first), Some(last));
                assert_eq!(crate::tools::from_scramble(&scramble), facelets);
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
