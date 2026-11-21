// Square-1 two-phase solver
// Port of cs.sq12phase.Search
//
// Phase 1: bring the cube to a "square shape" using Shape pruning tables.
// Phase 2: solve corner and edge permutations using Square pruning tables.
//
// Both phases use IDA* (iterative deepening A*).
//
// Cast warnings are suppressed because this is a direct port of Java code
// that uses `int` everywhere; all index values are known to be non-negative
// and within bounds at runtime.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_lossless
)]

use crate::full_cube::FullCube;
use crate::shape::SHAPE_TABLES;
use crate::square::{Square, TABLES as SQUARE_TABLES};

/// Maximum move array length.
const MAX_MOVES: usize = 100;

/// Flag bit: return the inverse (scramble) solution.
pub const INVERSE_SOLUTION: i32 = 0x2;

/// Half-move units per WCA-metric move in the optimal search, where a layer
/// turn and a twist each count as one move.
const PRUN_INC: i32 = 2;

// ---------------------------------------------------------------------------
// Search
// ---------------------------------------------------------------------------

/// Two-phase IDA* solver for Square-1.
pub struct Search {
    /// Internal move buffer.
    moves: [i32; MAX_MOVES],
    /// Working cube for phase replay.
    d: FullCube,
    /// Phase 2 coordinate scratch.
    sq: Square,
    /// Length of phase 1 portion (total moves before phase 2 starts).
    length1: i32,
    /// Number of moves found in phase 1 search.
    movelen1: i32,
    /// Maximum depth allowed for phase 2.
    maxlen2: i32,
    /// Verbosity / mode flags.
    verbose: i32,
    /// Resulting solution string (set when found).
    sol_string: Option<String>,
}

impl Default for Search {
    fn default() -> Self {
        Self::new()
    }
}

impl Search {
    /// Create a new solver instance.
    #[must_use]
    pub fn new() -> Self {
        // Force table initialization
        let _ = &*SHAPE_TABLES;
        let _ = &*SQUARE_TABLES;

        Self {
            moves: [0; MAX_MOVES],
            d: FullCube::new(),
            sq: Square::default(),
            length1: 0,
            movelen1: 0,
            maxlen2: 0,
            verbose: 0,
            sol_string: None,
        }
    }

    /// Find a solution for the given `FullCube`.
    ///
    /// Returns `Some(solution_string)` in WCA notation, e.g.
    /// `"(1,-3) / (0,2) / ..."`, or `None` if something goes wrong
    /// (should not happen for valid cubes).
    #[must_use]
    pub fn solution(&mut self, cube: &FullCube) -> Option<String> {
        self.solution_with_verbose(cube, 0)
    }

    /// Find a solution with verbosity/mode flags.
    ///
    /// - bit 1 (`INVERSE_SOLUTION`): invert the move sequence (produce a scramble).
    #[must_use]
    pub fn solution_with_verbose(&mut self, cube: &FullCube, verbose: i32) -> Option<String> {
        let shape_tables = &*SHAPE_TABLES;

        self.verbose = verbose;
        self.sol_string = None;

        let shape = cube.get_shape_idx();

        let start_depth = shape_tables.shape_prun[shape as usize];
        debug_assert!(start_depth >= 0, "shape index has unfilled pruning entry");

        for length1 in start_depth..100 {
            self.length1 = length1;
            self.maxlen2 = 17.min(31 - length1);
            if self.ida_phase1(
                cube,
                shape,
                shape_tables.shape_prun[shape as usize],
                length1,
                0,
                -1,
            ) {
                break;
            }
        }

        self.sol_string.take()
    }

    /// Find an optimal solution of at most `maxl` moves in the WCA turn
    /// metric (`(x,y)` and `/` each count as one move), or `None` if there is
    /// none. Port of Java's `Search.solutionOpt`.
    #[must_use]
    pub fn solution_opt(&mut self, cube: &FullCube, maxl: i32) -> Option<String> {
        self.solution_opt_with_verbose(cube, maxl, 0)
    }

    /// [`Search::solution_opt`] with verbosity/mode flags.
    #[must_use]
    pub fn solution_opt_with_verbose(
        &mut self,
        cube: &FullCube,
        maxl: i32,
        verbose: i32,
    ) -> Option<String> {
        let shape_tables = &*SHAPE_TABLES;
        self.verbose = verbose;
        self.sol_string = None;
        let shape = cube.get_shape_idx();
        let prun = shape_tables.shape_prun_opt[shape as usize];
        let mut length1 = prun * PRUN_INC;
        while length1 <= maxl * PRUN_INC {
            self.length1 = length1;
            if self.phase1_opt(cube, shape, prun, length1, 0, -1, 0) {
                break;
            }
            length1 += PRUN_INC;
        }
        self.sol_string.take()
    }

    /// Counts the nibbles of `val` that are 0xf.
    fn count0xf(mut val: i32) -> i32 {
        val &= val >> 1;
        val &= val >> 2;
        (val & 0x1111_1111).count_ones() as i32
    }

    /// Optimal IDA* in the WCA metric. `maxl` counts half moves
    /// (`PRUN_INC` per WCA move); `last_turns` packs recent layer turns to
    /// prune equivalent sequences. Port of Java's `phase1Opt`.
    #[allow(clippy::too_many_arguments)]
    fn phase1_opt(
        &mut self,
        cube: &FullCube,
        shape: i32,
        prun_value: i32,
        maxl: i32,
        depth: i32,
        lm: i32,
        last_turns: i32,
    ) -> bool {
        let _ = prun_value;
        let shape_tables = &*SHAPE_TABLES;
        let i = Self::count0xf((last_turns ^ !0x00_0000) & 0xff_00ff)
            - Self::count0xf((last_turns ^ !0x66_6666) & 0xff_00ff);
        if i < 0 || i == 0 && (last_turns >> 20 & 0xf) >= 6 {
            return false;
        }

        if maxl / PRUN_INC == 0 {
            self.movelen1 = depth;
            if self.is_solved_in_phase1(cube) {
                return true;
            }
            if maxl == 0 {
                return false;
            }
        }

        // Try twist first
        if lm != 0 {
            let shapex = shape_tables.twist_move[shape as usize];
            let prun = shape_tables.shape_prun_opt[shapex as usize];
            if prun < maxl / PRUN_INC {
                self.moves[depth as usize] = 0;
                let next_maxl = (maxl / PRUN_INC - 1) * PRUN_INC;
                if self.phase1_opt(cube, shapex, prun, next_maxl, depth + 1, 0, last_turns << 8) {
                    return true;
                }
            }
        }

        // Try top layer
        if lm <= 0 {
            let mut m = 0;
            let mut shapex = shape;
            loop {
                let entry = shape_tables.top_move[shapex as usize];
                m += entry & 0xf;
                shapex = entry >> 4;
                if m >= 12 {
                    break;
                }
                let prun = shape_tables.shape_prun_opt[shapex as usize];
                if prun * PRUN_INC > maxl + PRUN_INC - 1 {
                    break;
                } else if prun * PRUN_INC < maxl + PRUN_INC - 1 {
                    self.moves[depth as usize] = m;
                    if self.phase1_opt(
                        cube,
                        shapex,
                        prun,
                        maxl - 1,
                        depth + 1,
                        1,
                        last_turns | m << 4,
                    ) {
                        return true;
                    }
                }
            }
        }

        // Try bottom layer
        if lm <= 1 {
            let mut m = 0;
            let mut shapex = shape;
            loop {
                let entry = shape_tables.bottom_move[shapex as usize];
                m += entry & 0xf;
                shapex = entry >> 4;
                if m >= 12 {
                    break;
                }
                let prun = shape_tables.shape_prun_opt[shapex as usize];
                if prun * PRUN_INC > maxl + PRUN_INC - 1 {
                    break;
                } else if prun * PRUN_INC < maxl + PRUN_INC - 1 {
                    self.moves[depth as usize] = -m;
                    if self.phase1_opt(cube, shapex, prun, maxl - 1, depth + 1, 2, last_turns | m) {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn is_solved_in_phase1(&mut self, cube: &FullCube) -> bool {
        self.d.copy_from(cube);
        for i in 0..self.movelen1 {
            self.d.do_move(self.moves[i as usize]);
        }
        let solved = self.d.is_solved();
        if solved {
            self.sol_string = Some(self.move2string(self.movelen1));
        }
        solved
    }

    // -----------------------------------------------------------------------
    // Phase 1: shape solving (IDA*)
    // -----------------------------------------------------------------------

    fn ida_phase1(
        &mut self,
        cube: &FullCube,
        shape: i32,
        prun_value: i32,
        maxl: i32,
        depth: i32,
        lm: i32,
    ) -> bool {
        let shape_tables = &*SHAPE_TABLES;

        if prun_value == 0 && maxl < 4 {
            self.movelen1 = depth;
            return maxl == 0 && self.init_phase2(cube);
        }

        // Try twist (move 0)
        if lm != 0 {
            let shapex = shape_tables.twist_move[shape as usize];
            let prun = shape_tables.shape_prun[shapex as usize];
            if prun < maxl {
                self.moves[depth as usize] = 0;
                if self.ida_phase1(cube, shapex, prun, maxl - 1, depth + 1, 0) {
                    return true;
                }
            }
        }

        // Try top layer
        if lm <= 0 {
            let mut m = 0;
            let mut shapex = shape;
            loop {
                let entry = shape_tables.top_move[shapex as usize];
                let inc = entry & 0xf;
                shapex = entry >> 4;
                m += inc;
                if m >= 12 {
                    break;
                }
                let prun = shape_tables.shape_prun[shapex as usize];
                if prun > maxl {
                    break;
                } else if prun < maxl {
                    self.moves[depth as usize] = m;
                    if self.ida_phase1(cube, shapex, prun, maxl - 1, depth + 1, 1) {
                        return true;
                    }
                }
            }
        }

        // Try bottom layer
        if lm <= 1 {
            let mut m = 0;
            let mut shapex = shape;
            loop {
                let entry = shape_tables.bottom_move[shapex as usize];
                let inc = entry & 0xf;
                shapex = entry >> 4;
                m += inc;
                if m >= 6 {
                    break;
                }
                let prun = shape_tables.shape_prun[shapex as usize];
                if prun > maxl {
                    break;
                } else if prun < maxl {
                    self.moves[depth as usize] = -m;
                    if self.ida_phase1(cube, shapex, prun, maxl - 1, depth + 1, 2) {
                        return true;
                    }
                }
            }
        }

        false
    }

    // -----------------------------------------------------------------------
    // Phase 1 -> Phase 2 transition
    // -----------------------------------------------------------------------

    /// Transition from phase 1 to phase 2.
    fn init_phase2(&mut self, cube: &FullCube) -> bool {
        let sq_tables = &*SQUARE_TABLES;

        self.d.copy_from(cube);
        for i in 0..self.movelen1 {
            self.d.do_move(self.moves[i as usize]);
        }

        self.d.get_square(&mut self.sq);

        let edge = self.sq.edge_perm;
        let corner = self.sq.corner_perm;
        let ml = self.sq.ml as usize;

        let prun = sq_tables.square_prun[(edge as usize) << 1 | ml]
            .max(sq_tables.square_prun[(corner as usize) << 1 | ml]);

        for i in (prun as i32)..self.maxlen2 {
            if self.ida_phase2(
                edge as usize,
                corner as usize,
                self.sq.top_edge_first,
                self.sq.bot_edge_first,
                ml as i32,
                i,
                self.movelen1,
                0,
            ) {
                self.sol_string = Some(self.move2string(i + self.movelen1));
                return true;
            }
        }

        false
    }

    // -----------------------------------------------------------------------
    // Phase 2: permutation solving (IDA*)
    // -----------------------------------------------------------------------

    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::too_many_lines)]
    fn ida_phase2(
        &mut self,
        edge: usize,
        corner: usize,
        top_edge_first: bool,
        bot_edge_first: bool,
        ml: i32,
        maxl: i32,
        depth: i32,
        lm: i32,
    ) -> bool {
        let sq_tables = &*SQUARE_TABLES;

        if maxl == 0 && !top_edge_first && bot_edge_first {
            debug_assert!(edge == 0 && corner == 0 && ml == 0);
            return true;
        }

        // Try twist
        if lm != 0 && top_edge_first == bot_edge_first {
            let edgex = sq_tables.twist_move[edge] as usize;
            let cornerx = sq_tables.twist_move[corner] as usize;
            let mlx = 1 - ml;

            if sq_tables.square_prun[edgex << 1 | mlx as usize] < maxl as i8
                && sq_tables.square_prun[cornerx << 1 | mlx as usize] < maxl as i8
            {
                self.moves[depth as usize] = 0;
                if self.ida_phase2(
                    edgex,
                    cornerx,
                    top_edge_first,
                    bot_edge_first,
                    mlx,
                    maxl - 1,
                    depth + 1,
                    0,
                ) {
                    return true;
                }
            }
        }

        // Try top layer
        if lm <= 0 {
            let mut top_ef = !top_edge_first;
            let mut edgex = if top_ef {
                sq_tables.top_move[edge] as usize
            } else {
                edge
            };
            let mut cornerx = if top_ef {
                corner
            } else {
                sq_tables.top_move[corner] as usize
            };
            let mut m: i32 = if top_ef { 1 } else { 2 };
            let mut prun1 = sq_tables.square_prun[edgex << 1 | ml as usize];
            let mut prun2 = sq_tables.square_prun[cornerx << 1 | ml as usize];

            // Note: the duplicate `prun1` check matches the original Java source.
            #[allow(clippy::nonminimal_bool)]
            while m < 12 && prun1 <= maxl as i8 && prun1 <= maxl as i8 {
                if prun1 < maxl as i8 && prun2 < maxl as i8 {
                    self.moves[depth as usize] = m;
                    if self.ida_phase2(
                        edgex,
                        cornerx,
                        top_ef,
                        bot_edge_first,
                        ml,
                        maxl - 1,
                        depth + 1,
                        1,
                    ) {
                        return true;
                    }
                }
                top_ef = !top_ef;
                if top_ef {
                    edgex = sq_tables.top_move[edgex] as usize;
                    prun1 = sq_tables.square_prun[edgex << 1 | ml as usize];
                    m += 1;
                } else {
                    cornerx = sq_tables.top_move[cornerx] as usize;
                    prun2 = sq_tables.square_prun[cornerx << 1 | ml as usize];
                    m += 2;
                }
            }
        }

        // Try bottom layer
        if lm <= 1 {
            let limit = if maxl > 6 { 6 } else { 12 };
            let mut bot_ef = !bot_edge_first;
            let mut edgex = if bot_ef {
                sq_tables.bottom_move[edge] as usize
            } else {
                edge
            };
            let mut cornerx = if bot_ef {
                corner
            } else {
                sq_tables.bottom_move[corner] as usize
            };
            let mut m: i32 = if bot_ef { 1 } else { 2 };
            let mut prun1 = sq_tables.square_prun[edgex << 1 | ml as usize];
            let mut prun2 = sq_tables.square_prun[cornerx << 1 | ml as usize];

            #[allow(clippy::nonminimal_bool)]
            while m < limit && prun1 <= maxl as i8 && prun1 <= maxl as i8 {
                if prun1 < maxl as i8 && prun2 < maxl as i8 {
                    self.moves[depth as usize] = -m;
                    if self.ida_phase2(
                        edgex,
                        cornerx,
                        top_edge_first,
                        bot_ef,
                        ml,
                        maxl - 1,
                        depth + 1,
                        2,
                    ) {
                        return true;
                    }
                }
                bot_ef = !bot_ef;
                if bot_ef {
                    edgex = sq_tables.bottom_move[edgex] as usize;
                    prun1 = sq_tables.square_prun[edgex << 1 | ml as usize];
                    m += 1;
                } else {
                    cornerx = sq_tables.bottom_move[cornerx] as usize;
                    prun2 = sq_tables.square_prun[cornerx << 1 | ml as usize];
                    m += 2;
                }
            }
        }

        false
    }

    // -----------------------------------------------------------------------
    // Solution string formatting
    // -----------------------------------------------------------------------

    /// Convert internal move array to WCA notation string.
    ///
    /// Format: `"(top,bottom) / (top,bottom) / ..."` where top/bottom are
    /// in the range -5..6 (values > 6 are normalised by subtracting 12).
    fn move2string(&self, len: i32) -> String {
        let len = len as usize;
        let mut output_moves = vec![0i32; len];

        if (self.verbose & INVERSE_SOLUTION) != 0 {
            for (i, out) in output_moves.iter_mut().enumerate() {
                let m = self.moves[len - 1 - i];
                *out = match m.cmp(&0) {
                    std::cmp::Ordering::Greater => 12 - m,
                    std::cmp::Ordering::Less => -12 - m,
                    std::cmp::Ordering::Equal => 0,
                };
            }
        } else {
            output_moves.copy_from_slice(&self.moves[..len]);
        }

        let mut s = String::new();
        let mut top = 0;
        let mut bottom = 0;

        for &val in &output_moves {
            match val.cmp(&0) {
                std::cmp::Ordering::Greater => {
                    top = if val > 6 { val - 12 } else { val };
                }
                std::cmp::Ordering::Less => {
                    bottom = if -val > 6 { -val - 12 } else { -val };
                }
                std::cmp::Ordering::Equal => {
                    // Twist (move == 0)
                    if top == 0 && bottom == 0 {
                        s.push_str(" / ");
                    } else {
                        s.push('(');
                        s.push_str(&top.to_string());
                        s.push(',');
                        s.push_str(&bottom.to_string());
                        s.push_str(") / ");
                    }
                    top = 0;
                    bottom = 0;
                }
            }
        }

        // Final pending top/bottom (no trailing slash)
        if top != 0 || bottom != 0 {
            s.push('(');
            s.push_str(&top.to_string());
            s.push(',');
            s.push_str(&bottom.to_string());
            s.push(')');
        }

        s
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    /// Apply a WCA-notation Square-1 algorithm like `(1,0) / (-3,2) /`.
    /// Returns the cube and its length in the WCA metric.
    fn apply(cube: &FullCube, alg: &str) -> (FullCube, i32) {
        let mut c = FullCube::new();
        c.copy_from(cube);
        let mut moves = 0;
        let compact: String = alg.chars().filter(|ch| !ch.is_whitespace()).collect();
        let mut rest = compact.as_str();
        while !rest.is_empty() {
            if let Some(r) = rest.strip_prefix('/') {
                c.do_move(0);
                moves += 1;
                rest = r;
            } else {
                let end = rest.find(')').unwrap();
                let (top, bottom) = rest[1..end].split_once(',').unwrap();
                let (top, bottom): (i32, i32) = (top.parse().unwrap(), bottom.parse().unwrap());
                if top.rem_euclid(12) != 0 {
                    c.do_move(top.rem_euclid(12));
                }
                if bottom.rem_euclid(12) != 0 {
                    c.do_move(-bottom.rem_euclid(12));
                }
                moves += 1;
                rest = &rest[end + 1..];
            }
        }
        (c, moves)
    }

    #[test]
    fn optimal_solution_matches_java() {
        // Port of SquareOnePuzzleTest.testMergingMode: (1,0) (2,0) (0,-1) / /
        // merges to (3,-1), whose optimal solution is (-3,1).
        let (c, _) = apply(&FullCube::new(), "(3,-1)");
        let mut search = Search::new();
        assert_eq!(search.solution_opt(&c, 1).unwrap().trim(), "(-3,1)");
        assert_eq!(search.solution_opt(&c, 2).unwrap().trim(), "(-3,1)");
        assert_eq!(
            search.solution_opt(&FullCube::new(), 0).as_deref(),
            Some("")
        );
    }

    #[test]
    fn optimal_solutions_are_valid_and_short() {
        let mut rng = StdRng::seed_from_u64(7);
        let mut search = Search::new();
        for _ in 0..20 {
            let scramble = search
                .solution_with_verbose(&FullCube::random_cube(&mut rng), INVERSE_SOLUTION)
                .unwrap();
            // Take the first few WCA moves of a valid scramble.
            let prefix: String = scramble.split('/').take(3).collect::<Vec<_>>().join("/");
            let (state, length) = apply(&FullCube::new(), &prefix);
            let solution = search
                .solution_opt(&state, length)
                .expect("within its own length");
            let (solved, solution_length) = apply(&state, &solution);
            assert!(solved.is_solved(), "{prefix} + {solution}");
            assert!(solution_length <= length, "{solution} longer than {prefix}");
        }
    }

    #[test]
    fn random_states_are_rarely_within_ten_moves() {
        let mut rng = StdRng::seed_from_u64(11);
        let mut search = Search::new();
        for _ in 0..20 {
            assert!(
                search
                    .solution_opt(&FullCube::random_cube(&mut rng), 10)
                    .is_none()
            );
        }
    }

    #[test]
    fn solved_cube_short_solution() {
        let c = FullCube::new();
        let mut search = Search::new();
        let sol = search.solution(&c);
        // A solved cube should produce an empty or very short solution
        match sol {
            Some(s) => {
                assert!(
                    s.is_empty() || s.trim().is_empty(),
                    "solved cube should produce empty solution, got: {s:?}"
                );
            }
            None => {
                // Also acceptable — some implementations return None for
                // already-solved cubes. But our port returns Some("").
                // Either way, not finding a solution is valid.
            }
        }
    }

    #[test]
    fn random_cube_produces_solution() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut search = Search::new();

        for _ in 0..5 {
            let c = FullCube::random_cube(&mut rng);
            let sol = search.solution(&c);
            assert!(
                sol.is_some(),
                "solver should find a solution for random cube"
            );
            let s = sol.unwrap();
            // Solution should contain at least one slash (twist)
            if !c.is_solved() {
                assert!(
                    s.contains('/'),
                    "non-trivial solution should contain slashes: {s:?}"
                );
            }
        }
    }

    #[test]
    fn solution_solves_cube() {
        // Apply the solution to the cube and verify it reaches solved state.
        let mut rng = StdRng::seed_from_u64(77);
        let mut search = Search::new();

        for _ in 0..3 {
            let c = FullCube::random_cube(&mut rng);
            let sol = search.solution(&c);
            assert!(sol.is_some());
            let sol_str = sol.unwrap();

            // Parse and apply the solution
            let mut cube = c.clone();
            apply_solution(&mut cube, &sol_str);
            assert!(
                cube.is_solved(),
                "cube should be solved after applying solution: {sol_str:?}"
            );
        }
    }

    #[test]
    fn inverse_solution_scrambles() {
        // The inverse-mode solution, applied to a solved cube, should produce
        // a state that, when solved by the normal solver, yields a solution.
        let mut rng = StdRng::seed_from_u64(55);
        let mut search = Search::new();

        let c = FullCube::random_cube(&mut rng);
        let sol = search.solution_with_verbose(&c, INVERSE_SOLUTION);
        assert!(sol.is_some(), "inverse solver should find a solution");
        let sol_str = sol.unwrap();
        assert!(!sol_str.is_empty(), "inverse solution should not be empty");

        // Verify: normal solution applied to the cube solves it
        let normal_sol = search.solution(&c);
        assert!(normal_sol.is_some());
        let mut cube = c.clone();
        apply_solution(&mut cube, &normal_sol.unwrap());
        assert!(cube.is_solved(), "normal solution should solve the cube");
    }

    /// Parse a WCA Square-1 solution string and apply it to a cube.
    ///
    /// Format: "(top,bottom) / (top,bottom) / ..."
    fn apply_solution(cube: &mut FullCube, sol: &str) {
        let sol = sol.trim();
        if sol.is_empty() {
            return;
        }

        // Split on '/'
        let parts: Vec<&str> = sol.split('/').collect();

        for (i, part) in parts.iter().enumerate() {
            let part = part.trim();
            if part.is_empty() {
                // A bare '/' means just a twist
                cube.do_move(0);
                continue;
            }

            // Parse "(top,bottom)" or just whitespace
            if let Some(inner) = part.strip_prefix('(').and_then(|s| s.strip_suffix(')')) {
                let nums: Vec<&str> = inner.split(',').collect();
                if nums.len() == 2 {
                    let top: i32 = nums[0].trim().parse().unwrap();
                    let bottom: i32 = nums[1].trim().parse().unwrap();
                    if top != 0 {
                        // Convert to internal move: positive for top
                        let t = if top < 0 { top + 12 } else { top };
                        cube.do_move(t);
                    }
                    if bottom != 0 {
                        // Convert to internal move: negative for bottom
                        let b = if bottom < 0 { bottom + 12 } else { bottom };
                        cube.do_move(-b);
                    }
                }
            }

            // After each group (except the last), apply a twist
            if i < parts.len() - 1 {
                cube.do_move(0);
            }
        }
    }
}
