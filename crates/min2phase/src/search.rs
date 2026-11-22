//! Two-phase solver for the 3x3x3.
//!
//! Port of `cs.min2phase.Search` and `cs.min2phase.SearchWCA` as tnoodle-lib v0.20.0
//! ships them. [`Search`] returns Java's exact text for the same input; [`SearchWca`]
//! adds the axis restrictions and the wall-clock limits tnoodle uses for scrambles.

use std::fmt::Write as _;
use std::time::{Duration, Instant};

use crate::coord_cube::{COORD_TABLES, CoordCube, N_COMB, N_MPERM, get_pruning};
use crate::cubie_cube::{self, CubieCube, TABLES, URF_MOVE};
use crate::util::{self, CKMV2BIT, MOVE2STR, STD2UD, UD2STD};

const MAX_PRE_MOVES: i32 = 20;
const MIN_P1LENGTH_PRE: i32 = 7;
const MAX_DEPTH2: i32 = 12;

/// `SearchWCA.move2str`: moves without the padding `Util.move2str` carries.
const WCA_MOVE2STR: [&str; 18] = [
    "U", "U2", "U'", "R", "R2", "R'", "F", "F2", "F'", "D", "D2", "D'", "L", "L2", "L'", "B", "B2",
    "B'",
];

/// The output flags of `Search.solution` (Java's `verbose` mask).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Verbose(u32);

impl Verbose {
    /// No flag: the plain solution.
    pub const NONE: Self = Self(0);
    /// `USE_SEPARATOR`: a `.` between the phase-1 and phase-2 moves.
    pub const USE_SEPARATOR: Self = Self(0x1);
    /// `INVERSE_SOLUTION`: the inverse of the solution, a scramble for the state.
    pub const INVERSE_SOLUTION: Self = Self(0x2);
    /// `APPEND_LENGTH`: a length tag such as `(21f)` after the moves.
    pub const APPEND_LENGTH: Self = Self(0x4);
    /// `OPTIMAL_SOLUTION`: search for an optimal solution.
    pub const OPTIMAL_SOLUTION: Self = Self(0x8);

    /// The flags as Java's `int` mask.
    #[must_use]
    pub const fn bits(self) -> u32 {
        self.0
    }

    /// Flags from Java's `int` mask; unknown bits are kept, as Java ignores them.
    #[must_use]
    pub const fn from_bits(bits: u32) -> Self {
        Self(bits)
    }

    /// Whether every flag of `other` is set.
    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

impl std::ops::BitOr for Verbose {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

/// The errors `Search.solution` reports as `"Error 1"` to `"Error 9"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchError {
    /// 1: there is not exactly one facelet of each colour.
    InvalidFacelets,
    /// 2: not all 12 edges exist exactly once.
    MissingEdge,
    /// 3: one edge has to be flipped.
    FlippedEdge,
    /// 4: not all 8 corners exist exactly once.
    MissingCorner,
    /// 5: one corner has to be twisted.
    TwistedCorner,
    /// 6: two corners or two edges have to be exchanged.
    Parity,
    /// 7: no solution exists within the maximum depth.
    NoSolution,
    /// 8: the probe or time limit ran out before a solution was found.
    ProbeLimit,
    /// 9: an axis restriction is not a face move (`SearchWCA` only).
    InvalidRestriction,
}

impl SearchError {
    /// Java's error number.
    #[must_use]
    pub const fn code(self) -> u8 {
        match self {
            Self::InvalidFacelets => 1,
            Self::MissingEdge => 2,
            Self::FlippedEdge => 3,
            Self::MissingCorner => 4,
            Self::TwistedCorner => 5,
            Self::Parity => 6,
            Self::NoSolution => 7,
            Self::ProbeLimit => 8,
            Self::InvalidRestriction => 9,
        }
    }

    fn from_verify(code: i32) -> Self {
        match code.abs() {
            2 => Self::MissingEdge,
            3 => Self::FlippedEdge,
            4 => Self::MissingCorner,
            5 => Self::TwistedCorner,
            6 => Self::Parity,
            _ => Self::InvalidFacelets,
        }
    }
}

impl std::fmt::Display for SearchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error {}", self.code())
    }
}

impl std::error::Error for SearchError {}

/// How long [`SearchWca`] keeps looking for shorter solutions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchBudget {
    /// tnoodle's limits: stop at the first phase-2 probe after `min` once a solution
    /// exists, and give up after `max` without one.
    Time { min: Duration, max: Duration },
    /// `Search`'s limits: stop after `min` phase-2 probes once a solution exists,
    /// and give up after `max` without one. Deterministic on every machine.
    Probes { min: u64, max: u64 },
}

impl Default for SearchBudget {
    /// tnoodle's `THREE_BY_THREE_TIMEMIN` (200 ms) and `THREE_BY_THREE_TIMEOUT` (60 s).
    fn default() -> Self {
        Self::Time {
            min: Duration::from_millis(200),
            max: Duration::from_secs(60),
        }
    }
}

/// The state `SearchWCA` adds to `Search`.
#[derive(Debug, Clone)]
struct WcaFilter {
    first_move_filter: [i32; 6],
    last_move_filter: [i32; 6],
    is_axis_restricted: bool,
    /// `Some` when the budget is wall-clock time, as in Java.
    start_time: Option<Instant>,
}

/// Port of `cs.min2phase.Search`: the two-phase solver with Java's exact output.
#[derive(Clone)]
pub struct Search {
    mv: [i32; 31],
    move_sol: [i32; 31],
    self_sym: u64,
    conj_mask: i32,
    urf_idx: usize,
    length1: i32,
    depth1: i32,
    max_dep2: i32,
    sol: i32,
    solution: Option<String>,
    probe: i64,
    probe_max: i64,
    probe_min: i64,
    verbose: u32,
    valid1: i32,
    allow_shorter: bool,
    cc: CubieCube,
    urf_cubie_cube: [CubieCube; 6],
    urf_coord_cube: [CoordCube; 6],
    phase1_cubie: [CubieCube; 21],
    pre_moves: [i32; MAX_PRE_MOVES as usize],
    pre_move_len: i32,
    max_pre_moves: i32,
    is_rec: bool,
    wca: Option<WcaFilter>,
}

impl Default for Search {
    fn default() -> Self {
        Self::new()
    }
}

fn low32(bits: u64) -> i32 {
    i32::from_ne_bytes(
        u32::try_from(bits & 0xffff_ffff)
            .expect("masked")
            .to_ne_bytes(),
    )
}

fn bit(mask: i32, index: i32) -> bool {
    (mask >> index) & 1 != 0
}

impl Search {
    /// A solver; the shared tables are built on first use.
    #[must_use]
    pub fn new() -> Self {
        Self {
            mv: [0; 31],
            move_sol: [0; 31],
            self_sym: 0,
            conj_mask: 0,
            urf_idx: 0,
            length1: 0,
            depth1: 0,
            max_dep2: 0,
            sol: 0,
            solution: None,
            probe: 0,
            probe_max: 0,
            probe_min: 0,
            verbose: 0,
            valid1: 0,
            allow_shorter: false,
            cc: CubieCube::new(),
            urf_cubie_cube: std::array::from_fn(|_| CubieCube::new()),
            urf_coord_cube: std::array::from_fn(|_| CoordCube::new()),
            phase1_cubie: std::array::from_fn(|_| CubieCube::new()),
            pre_moves: [0; MAX_PRE_MOVES as usize],
            pre_move_len: 0,
            max_pre_moves: 0,
            is_rec: false,
            wca: None,
        }
    }

    /// Port of `Search.solution`: a solution for `facelets` of at most `max_depth`
    /// moves, in Java's text format (moves padded to two characters, each followed
    /// by a space).
    ///
    /// # Errors
    ///
    /// Java's error 1-6 for invalid facelets, 7 when no solution exists within
    /// `max_depth`, and 8 when `probe_max` runs out first.
    pub fn solution(
        &mut self,
        facelets: &str,
        max_depth: i32,
        probe_max: i64,
        probe_min: i64,
        verbose: Verbose,
    ) -> Result<String, SearchError> {
        self.wca = None;
        self.solve(facelets, max_depth, probe_max, probe_min, verbose)
    }

    fn solve(
        &mut self,
        facelets: &str,
        max_depth: i32,
        probe_max: i64,
        probe_min: i64,
        verbose: Verbose,
    ) -> Result<String, SearchError> {
        let check = self.verify(facelets);
        if check != 0 {
            return Err(SearchError::from_verify(check));
        }
        self.sol = max_depth + 1;
        self.probe = 0;
        self.probe_max = probe_max;
        self.probe_min = probe_min.min(probe_max);
        self.verbose = verbose.bits();
        self.solution = None;
        self.is_rec = false;
        let _ = &*COORD_TABLES;
        self.init_search();
        if verbose.contains(Verbose::OPTIMAL_SOLUTION) {
            self.search_opt()
        } else {
            self.search()
        }
    }

    /// Port of `Search.next`: continues the last search for another solution.
    ///
    /// # Errors
    ///
    /// Java's error 7 or 8 when no further solution is found.
    pub fn next(
        &mut self,
        probe_max: i64,
        probe_min: i64,
        verbose: Verbose,
    ) -> Result<String, SearchError> {
        self.probe = 0;
        self.probe_max = probe_max;
        self.probe_min = probe_min.min(probe_max);
        self.solution = None;
        let optimal = Verbose::OPTIMAL_SOLUTION.bits();
        self.is_rec = (self.verbose & optimal) == (verbose.bits() & optimal);
        self.verbose = verbose.bits();
        if verbose.contains(Verbose::OPTIMAL_SOLUTION) {
            self.search_opt()
        } else {
            self.search()
        }
    }

    /// Port of `Search.numberOfProbes`.
    #[must_use]
    pub fn number_of_probes(&self) -> i64 {
        self.probe
    }

    /// Port of `Search.length`.
    #[must_use]
    pub fn length(&self) -> i32 {
        self.sol
    }

    /// Port of `Search.verify`: 0, or Java's negative error code. Loads the cube.
    pub(crate) fn verify(&mut self, facelets: &str) -> i32 {
        let chars: Vec<char> = facelets.chars().collect();
        if chars.len() < 54 {
            return -1;
        }
        let center = [
            chars[util::U5 as usize],
            chars[util::R5 as usize],
            chars[util::F5 as usize],
            chars[util::D5 as usize],
            chars[util::L5 as usize],
            chars[util::B5 as usize],
        ];
        let mut count = 0u32;
        let mut f = [0u8; 54];
        for (i, slot) in f.iter_mut().enumerate() {
            let Some(color) = center.iter().position(|&c| c == chars[i]) else {
                return -1;
            };
            *slot = u8::try_from(color).expect("six centers");
            count += 1 << (color << 2);
        }
        if count != 0x0099_9999 {
            return -1;
        }
        let mut cc = CubieCube::new();
        util::to_cubie_cube(&f, &mut cc.ca, &mut cc.ea);
        self.cc = cc;
        self.cc.verify()
    }

    fn init_search(&mut self) {
        self.conj_mask = 0;
        self.self_sym = cubie_cube::self_symmetry(&self.cc);
        self.conj_mask |= if (self.self_sym >> 16) & 0xffff != 0 {
            0x12
        } else {
            0
        };
        self.conj_mask |= if (self.self_sym >> 32) & 0xffff != 0 {
            0x24
        } else {
            0
        };
        self.conj_mask |= if (self.self_sym >> 48) & 0xffff != 0 {
            0x38
        } else {
            0
        };
        self.self_sym &= 0xffff_ffff_ffff;
        self.max_pre_moves = if self.conj_mask > 7 { 0 } else { MAX_PRE_MOVES };
        for i in 0..6 {
            self.urf_cubie_cube[i] = self.cc.clone();
            self.urf_coord_cube[i].set_with_prun(&self.urf_cubie_cube[i], 20);
            cubie_cube::urf_conjugate(&mut self.cc);
            if i % 3 == 2 {
                self.cc.inv_cubie_cube();
            }
        }
        if self.wca.as_ref().is_some_and(|w| w.is_axis_restricted) {
            self.self_sym = 1;
            self.conj_mask = 0;
            self.max_pre_moves = MAX_PRE_MOVES;
        }
    }

    fn phase1_pre_moves(&mut self, maxl: i32, lm: i32, cc: &CubieCube, ssym: i32) -> i32 {
        if let Some(w) = &self.wca
            && maxl == self.max_pre_moves - 1
            && bit(w.last_move_filter[self.urf_idx], lm)
        {
            return 1;
        }
        self.pre_move_len = self.max_pre_moves - maxl;
        let enter = if self.is_rec {
            self.depth1 == self.length1 - self.pre_move_len
        } else {
            self.pre_move_len == 0 || !bit(0x36FB7, lm)
        };
        if enter {
            self.depth1 = self.length1 - self.pre_move_len;
            self.phase1_cubie[0] = cc.clone();
            self.allow_shorter = self.depth1 == MIN_P1LENGTH_PRE && self.pre_move_len != 0;
            let mut node = CoordCube::new();
            if node.set_with_prun(cc, self.depth1) && self.phase1(&node, ssym, self.depth1, -1) == 0
            {
                return 0;
            }
        }
        if maxl == 0 || self.pre_move_len + MIN_P1LENGTH_PRE >= self.length1 {
            return 1;
        }
        let mut skip_moves =
            cubie_cube::get_skip_moves(u64::try_from(ssym).expect("non-negative symmetry mask"));
        if maxl == 1 || self.pre_move_len + 1 + MIN_P1LENGTH_PRE >= self.length1 {
            skip_moves |= 0x36FB7;
        }
        let lm = lm / 3 * 3;
        let mut m = 0;
        while m < 18 {
            if m == lm || m == lm - 9 || m == lm + 9 {
                m += 3;
                continue;
            }
            let pre_index = (self.max_pre_moves - maxl) as usize;
            if self.is_rec && m != self.pre_moves[pre_index] || bit(skip_moves, m) {
                m += 1;
                continue;
            }
            let mut next = CubieCube::new();
            cubie_cube::corn_mult(&TABLES.move_cube[m as usize], cc, &mut next);
            cubie_cube::edge_mult(&TABLES.move_cube[m as usize], cc, &mut next);
            self.pre_moves[pre_index] = m;
            let ret = self.phase1_pre_moves(
                maxl - 1,
                m,
                &next,
                ssym & low32(TABLES.move_cube_sym[m as usize]),
            );
            if ret == 0 {
                return 0;
            }
            m += 1;
        }
        1
    }

    fn search(&mut self) -> Result<String, SearchError> {
        self.length1 = if self.is_rec { self.length1 } else { 0 };
        while self.length1 < self.sol {
            self.max_dep2 = MAX_DEPTH2.min(self.sol - self.length1 - 1);
            self.urf_idx = if self.is_rec { self.urf_idx } else { 0 };
            while self.urf_idx < 6 {
                if self.conj_mask & (1 << self.urf_idx) == 0 {
                    let cc = self.urf_cubie_cube[self.urf_idx].clone();
                    let ssym = low32(self.self_sym & 0xffff);
                    if self.phase1_pre_moves(self.max_pre_moves, -30, &cc, ssym) == 0 {
                        return self.solution.clone().ok_or(SearchError::ProbeLimit);
                    }
                }
                self.urf_idx += 1;
            }
            self.length1 += 1;
        }
        self.solution.clone().ok_or(SearchError::NoSolution)
    }

    /// Port of `initPhase2Pre`. Returns 0 when found or out of probes, 1 to try the
    /// next power, 2 to try the next axis.
    #[allow(
        clippy::too_many_lines,
        reason = "kept in one piece to mirror the Java method"
    )]
    fn init_phase2_pre(&mut self) -> i32 {
        if let Some(start) = self.wca.as_ref().and_then(|w| w.start_time) {
            self.is_rec = false;
            let elapsed = i64::try_from(start.elapsed().as_millis()).unwrap_or(i64::MAX);
            if elapsed
                >= if self.solution.is_none() {
                    self.probe_max
                } else {
                    self.probe_min
                }
            {
                return 0;
            }
            self.probe = -1;
        }
        self.is_rec = false;
        if self.probe
            >= if self.solution.is_none() {
                self.probe_max
            } else {
                self.probe_min
            }
        {
            return 0;
        }
        self.probe += 1;

        let ct = &*TABLES;
        let t = &*COORD_TABLES;
        let depth1 = self.depth1 as usize;
        for i in self.valid1 as usize..depth1 {
            let mut next = CubieCube::new();
            let mv = &ct.move_cube[self.mv[i] as usize];
            cubie_cube::corn_mult(&self.phase1_cubie[i], mv, &mut next);
            cubie_cube::edge_mult(&self.phase1_cubie[i], mv, &mut next);
            self.phase1_cubie[i + 1] = next;
        }
        self.valid1 = self.depth1;

        let c = &self.phase1_cubie[depth1];
        let mut p2corn = c.get_c_perm_sym();
        let mut p2csym = p2corn & 0xf;
        p2corn >>= 4;
        let mut p2edge = c.get_e_perm_sym();
        let mut p2esym = p2edge & 0xf;
        p2edge >>= 4;
        let mut p2mid = c.get_m_perm();
        let mut edgei = cubie_cube::get_perm_sym_inv(p2edge as usize, p2esym as usize, false);
        let mut corni = cubie_cube::get_perm_sym_inv(p2corn as usize, p2csym as usize, true);

        let last_move = if depth1 == 0 { -1 } else { self.mv[depth1 - 1] };
        let pre_len = self.pre_move_len as usize;
        let last_pre = if pre_len == 0 {
            -1
        } else {
            self.pre_moves[pre_len - 1]
        };

        let mut ret = 0;
        let p2switch_max = (if pre_len == 0 { 1 } else { 2 }) * (if depth1 == 0 { 1 } else { 2 });
        let mut p2switch_mask = (1 << p2switch_max) - 1;
        for p2switch in 0..p2switch_max {
            // 0 normal; 1 lastmove; 2 lastmove + premove; 3 premove
            if bit(p2switch_mask, p2switch) {
                p2switch_mask &= !(1 << p2switch);
                ret = self.init_phase2(p2corn, p2csym, p2edge, p2esym, p2mid, edgei, corni);
                if ret == 0 || ret > 2 {
                    break;
                } else if ret == 2 {
                    p2switch_mask &= 0x4 << p2switch;
                }
            }
            if p2switch_mask == 0 {
                break;
            }
            if p2switch & 1 == 0 && depth1 > 0 {
                let m = STD2UD[(last_move / 3 * 3 + 1) as usize];
                self.mv[depth1 - 1] = UD2STD[m] as i32 * 2 - self.mv[depth1 - 1];
                p2mid = i32::from(t.m_perm_move[p2mid as usize][m]);
                p2corn =
                    i32::from(t.c_perm_move[p2corn as usize][ct.sym_move_ud[p2csym as usize][m]]);
                p2csym = ct.sym_mult[(p2corn & 0xf) as usize][p2csym as usize] as i32;
                p2corn >>= 4;
                p2edge =
                    i32::from(t.e_perm_move[p2edge as usize][ct.sym_move_ud[p2esym as usize][m]]);
                p2esym = ct.sym_mult[(p2edge & 0xf) as usize][p2esym as usize] as i32;
                p2edge >>= 4;
                corni = cubie_cube::get_perm_sym_inv(p2corn as usize, p2csym as usize, true);
                edgei = cubie_cube::get_perm_sym_inv(p2edge as usize, p2esym as usize, false);
            } else if pre_len > 0 {
                let m = STD2UD[(last_pre / 3 * 3 + 1) as usize];
                self.pre_moves[pre_len - 1] = UD2STD[m] as i32 * 2 - self.pre_moves[pre_len - 1];
                p2mid = i32::from(
                    ct.m_perm_inv
                        [usize::from(t.m_perm_move[usize::from(ct.m_perm_inv[p2mid as usize])][m])],
                );
                p2corn = i32::from(
                    t.c_perm_move[(corni >> 4) as usize][ct.sym_move_ud[(corni & 0xf) as usize][m]],
                );
                corni = p2corn & !0xf
                    | ct.sym_mult[(p2corn & 0xf) as usize][(corni & 0xf) as usize] as i32;
                p2corn = cubie_cube::get_perm_sym_inv(
                    (corni >> 4) as usize,
                    (corni & 0xf) as usize,
                    true,
                );
                p2csym = p2corn & 0xf;
                p2corn >>= 4;
                p2edge = i32::from(
                    t.e_perm_move[(edgei >> 4) as usize][ct.sym_move_ud[(edgei & 0xf) as usize][m]],
                );
                edgei = p2edge & !0xf
                    | ct.sym_mult[(p2edge & 0xf) as usize][(edgei & 0xf) as usize] as i32;
                p2edge = cubie_cube::get_perm_sym_inv(
                    (edgei >> 4) as usize,
                    (edgei & 0xf) as usize,
                    false,
                );
                p2esym = p2edge & 0xf;
                p2edge >>= 4;
            }
        }
        if depth1 > 0 {
            self.mv[depth1 - 1] = last_move;
        }
        if pre_len > 0 {
            self.pre_moves[pre_len - 1] = last_pre;
        }
        if ret == 0 { 0 } else { 2 }
    }

    /// The phase-2 pruning value of `EPermCCombPPrun` for an edge and corner coordinate pair.
    fn comb_p_prun(edge: i32, esym: i32, corn: i32, csym: i32) -> i32 {
        let ct = &*TABLES;
        let t = &*COORD_TABLES;
        let comb = usize::from(
            t.c_comb_p_conj[usize::from(ct.perm2_comb_p[corn as usize])]
                [ct.sym_mult_inv[esym as usize][csym as usize]],
        );
        get_pruning(&t.e_perm_c_comb_p_prun, edge as usize * N_COMB + comb) as i32
    }

    #[allow(clippy::too_many_arguments, reason = "mirrors the Java signature")]
    fn init_phase2(
        &mut self,
        p2corn: i32,
        p2csym: i32,
        p2edge: i32,
        p2esym: i32,
        p2mid: i32,
        edgei: i32,
        corni: i32,
    ) -> i32 {
        let t = &*COORD_TABLES;
        let mc = get_pruning(
            &t.mc_perm_prun,
            p2corn as usize * N_MPERM + usize::from(t.m_perm_conj[p2mid as usize][p2csym as usize]),
        );
        let prun = Self::comb_p_prun(edgei >> 4, edgei & 0xf, corni >> 4, corni & 0xf)
            .max(Self::comb_p_prun(p2edge, p2esym, p2corn, p2csym).max(mc as i32));
        if prun > self.max_dep2 {
            return prun - self.max_dep2;
        }
        let mut depth2 = self.max_dep2;
        while depth2 >= prun {
            let ret = self.phase2(
                p2edge,
                p2esym,
                p2corn,
                p2csym,
                p2mid,
                depth2,
                self.depth1,
                10,
            );
            if ret < 0 {
                break;
            }
            depth2 -= ret;
            self.sol = 0;
            for i in 0..(self.depth1 + depth2) as usize {
                self.append_sol_move(self.mv[i]);
            }
            for i in (0..self.pre_move_len as usize).rev() {
                self.append_sol_move(self.pre_moves[i]);
            }
            self.solution = Some(self.solution_to_string());
            depth2 -= 1;
        }
        if depth2 != self.max_dep2 {
            self.max_dep2 = MAX_DEPTH2.min(self.sol - self.length1 - 1);
            return i32::from(self.probe < self.probe_min);
        }
        1
    }

    /// Port of `phase1`. Returns 0 when found or out of probes, 1 to try the next
    /// power, 2 to try the next axis.
    fn phase1(&mut self, node: &CoordCube, ssym: i32, maxl: i32, lm: i32) -> i32 {
        if let Some(w) = &self.wca
            && maxl == self.depth1 - 1
            && bit(w.first_move_filter[self.urf_idx], lm)
        {
            return 1;
        }
        if node.prun == 0 && maxl < 5 {
            if self.allow_shorter || maxl == 0 {
                self.depth1 -= maxl;
                let ret = self.init_phase2_pre();
                self.depth1 += maxl;
                return ret;
            }
            return 1;
        }
        let skip_moves =
            cubie_cube::get_skip_moves(u64::try_from(ssym).expect("non-negative symmetry mask"));
        for axis in (0..18).step_by(3) {
            if axis == lm || axis == lm - 9 {
                continue;
            }
            for power in 0..3 {
                let m = axis + power;
                let index = (self.depth1 - maxl) as usize;
                if self.is_rec && m != self.mv[index] || skip_moves != 0 && bit(skip_moves, m) {
                    continue;
                }
                let mut next = CoordCube::new();
                let prun = next.do_move_prun(node, m as usize);
                if prun > maxl {
                    break;
                } else if prun == maxl {
                    continue;
                }
                let prun = next.do_move_prun_conj(node, m as usize);
                if prun > maxl {
                    break;
                } else if prun == maxl {
                    continue;
                }
                self.mv[index] = m;
                self.valid1 = self.valid1.min(self.depth1 - maxl);
                let ret = self.phase1(
                    &next,
                    ssym & low32(TABLES.move_cube_sym[m as usize]),
                    maxl - 1,
                    axis,
                );
                if ret == 0 {
                    return 0;
                } else if ret >= 2 {
                    break;
                }
            }
        }
        1
    }

    fn search_opt(&mut self) -> Result<String, SearchError> {
        let mut maxprun1 = 0;
        let mut maxprun2 = 0;
        for i in 0..6 {
            self.urf_coord_cube[i].calc_pruning();
            if i < 3 {
                maxprun1 = maxprun1.max(self.urf_coord_cube[i].prun);
            } else {
                maxprun2 = maxprun2.max(self.urf_coord_cube[i].prun);
            }
        }
        self.urf_idx = if maxprun2 > maxprun1 { 3 } else { 0 };
        self.phase1_cubie[0] = self.urf_cubie_cube[self.urf_idx].clone();
        self.length1 = if self.is_rec { self.length1 } else { 0 };
        while self.length1 < self.sol {
            let ud = self.urf_coord_cube[self.urf_idx].clone();
            let rl = self.urf_coord_cube[self.urf_idx + 1].clone();
            let fb = self.urf_coord_cube[self.urf_idx + 2].clone();
            if ud.prun <= self.length1
                && rl.prun <= self.length1
                && fb.prun <= self.length1
                && self.phase1_opt(&ud, &rl, &fb, self.self_sym, self.length1, -1) == 0
            {
                return self.solution.clone().ok_or(SearchError::ProbeLimit);
            }
            self.length1 += 1;
        }
        self.solution.clone().ok_or(SearchError::NoSolution)
    }

    fn phase1_opt(
        &mut self,
        ud: &CoordCube,
        rl: &CoordCube,
        fb: &CoordCube,
        ssym: u64,
        maxl: i32,
        lm: i32,
    ) -> i32 {
        if ud.prun == 0 && rl.prun == 0 && fb.prun == 0 && maxl < 5 {
            self.max_dep2 = maxl;
            self.depth1 = self.length1 - maxl;
            return i32::from(self.init_phase2_pre() != 0);
        }
        let skip_moves = cubie_cube::get_skip_moves(ssym);
        for axis in (0..18).step_by(3) {
            if axis == lm || axis == lm - 9 {
                continue;
            }
            for power in 0..3 {
                let mut m = axis + power;
                let index = (self.length1 - maxl) as usize;
                if self.is_rec && m != self.mv[index] || skip_moves != 0 && bit(skip_moves, m) {
                    continue;
                }
                let mut next_ud = CoordCube::new();
                let prun_ud = next_ud
                    .do_move_prun(ud, m as usize)
                    .max(next_ud.do_move_prun_conj(ud, m as usize));
                if prun_ud > maxl {
                    break;
                } else if prun_ud == maxl {
                    continue;
                }
                m = i32::from(URF_MOVE[2][m as usize]);
                let mut next_rl = CoordCube::new();
                let prun_rl = next_rl
                    .do_move_prun(rl, m as usize)
                    .max(next_rl.do_move_prun_conj(rl, m as usize));
                if prun_rl > maxl {
                    break;
                } else if prun_rl == maxl {
                    continue;
                }
                m = i32::from(URF_MOVE[2][m as usize]);
                let mut next_fb = CoordCube::new();
                let mut prun_fb = next_fb
                    .do_move_prun(fb, m as usize)
                    .max(next_fb.do_move_prun_conj(fb, m as usize));
                if prun_ud == prun_rl && prun_rl == prun_fb && prun_fb != 0 {
                    prun_fb += 1;
                }
                if prun_fb > maxl {
                    break;
                } else if prun_fb == maxl {
                    continue;
                }
                m = i32::from(URF_MOVE[2][m as usize]);
                self.mv[index] = m;
                self.valid1 = self.valid1.min(self.length1 - maxl);
                let ssym = ssym & TABLES.move_cube_sym[m as usize];
                let ret = self.phase1_opt(&next_ud, &next_rl, &next_fb, ssym, maxl - 1, axis);
                if ret == 0 {
                    return 0;
                }
            }
        }
        1
    }

    fn append_sol_move(&mut self, cur_move: i32) {
        let sol = self.sol as usize;
        if sol == 0 {
            self.move_sol[0] = cur_move;
            self.sol = 1;
            return;
        }
        let axis_cur = cur_move / 3;
        let axis_last = self.move_sol[sol - 1] / 3;
        if axis_cur == axis_last {
            let pow = (cur_move % 3 + self.move_sol[sol - 1] % 3 + 1) % 4;
            if pow == 3 {
                self.sol -= 1;
            } else {
                self.move_sol[sol - 1] = axis_cur * 3 + pow;
            }
            return;
        }
        if sol > 1 && axis_cur % 3 == axis_last % 3 && axis_cur == self.move_sol[sol - 2] / 3 {
            let pow = (cur_move % 3 + self.move_sol[sol - 2] % 3 + 1) % 4;
            if pow == 3 {
                self.move_sol[sol - 2] = self.move_sol[sol - 1];
                self.sol -= 1;
            } else {
                self.move_sol[sol - 2] = axis_cur * 3 + pow;
            }
            return;
        }
        self.move_sol[sol] = cur_move;
        self.sol += 1;
    }

    /// Port of `phase2` (and `SearchWCA.phase2`). Returns -1 when no solution is
    /// found, or how many moves shorter than `maxl` the solution is.
    #[allow(clippy::too_many_arguments, reason = "mirrors the Java signature")]
    fn phase2(
        &mut self,
        edge: i32,
        esym: i32,
        corn: i32,
        csym: i32,
        mid: i32,
        maxl: i32,
        depth: i32,
        lm: usize,
    ) -> i32 {
        let solved = edge == 0 && corn == 0 && mid == 0;
        if let Some(w) = &self.wca {
            if self.depth1 == 0
                && depth == 1
                && bit(w.first_move_filter[self.urf_idx], UD2STD[lm] as i32)
            {
                return -1;
            }
            if solved
                && (self.pre_move_len > 0
                    || !bit(w.last_move_filter[self.urf_idx], UD2STD[lm] as i32))
            {
                return maxl;
            }
        } else if solved {
            return maxl;
        }
        let ct = &*TABLES;
        let t = &*COORD_TABLES;
        let move_mask = CKMV2BIT[lm];
        let mut m = 0usize;
        while m < 10 {
            let skip = (0x42 >> m) & 3;
            if bit(move_mask, m as i32) {
                m += skip + 1;
                continue;
            }
            let midx = i32::from(t.m_perm_move[mid as usize][m]);
            let mut cornx =
                i32::from(t.c_perm_move[corn as usize][ct.sym_move_ud[csym as usize][m]]);
            let csymx = ct.sym_mult[(cornx & 0xf) as usize][csym as usize] as i32;
            cornx >>= 4;
            let mut edgex =
                i32::from(t.e_perm_move[edge as usize][ct.sym_move_ud[esym as usize][m]]);
            let esymx = ct.sym_mult[(edgex & 0xf) as usize][esym as usize] as i32;
            edgex >>= 4;
            let edgei = cubie_cube::get_perm_sym_inv(edgex as usize, esymx as usize, false);
            let corni = cubie_cube::get_perm_sym_inv(cornx as usize, csymx as usize, true);
            let mut prun = Self::comb_p_prun(edgei >> 4, edgei & 0xf, corni >> 4, corni & 0xf);
            if prun > maxl + 1 {
                return maxl - prun + 1;
            } else if prun >= maxl {
                m += (skip as i32 & (maxl - prun)) as usize + 1;
                continue;
            }
            let mc = get_pruning(
                &t.mc_perm_prun,
                cornx as usize * N_MPERM
                    + usize::from(t.m_perm_conj[midx as usize][csymx as usize]),
            );
            prun = (mc as i32).max(Self::comb_p_prun(edgex, esymx, cornx, csymx));
            if prun >= maxl {
                m += (skip as i32 & (maxl - prun)) as usize + 1;
                continue;
            }
            let ret = self.phase2(edgex, esymx, cornx, csymx, midx, maxl - 1, depth + 1, m);
            if ret >= 0 {
                self.mv[depth as usize] = UD2STD[m] as i32;
                return ret;
            }
            m += 1;
        }
        -1
    }

    fn solution_to_string(&self) -> String {
        let is_wca = self.wca.is_some();
        let (separator, names) = if is_wca {
            (".", WCA_MOVE2STR)
        } else {
            (".  ", MOVE2STR)
        };
        let urf = if self.verbose & Verbose::INVERSE_SOLUTION.bits() != 0 {
            (self.urf_idx + 3) % 6
        } else {
            self.urf_idx
        };
        let use_separator = self.verbose & Verbose::USE_SEPARATOR.bits() != 0;
        let name = |s: usize| names[usize::from(URF_MOVE[urf][self.move_sol[s] as usize])];
        let mut sb = String::new();
        if urf < 3 {
            for s in 0..self.sol as usize {
                if use_separator && s as i32 == self.depth1 {
                    sb.push_str(separator);
                }
                sb.push_str(name(s));
                sb.push(' ');
            }
        } else {
            for s in (0..self.sol as usize).rev() {
                sb.push_str(name(s));
                sb.push(' ');
                if use_separator && s as i32 == self.depth1 {
                    sb.push_str(separator);
                }
            }
        }
        if self.verbose & Verbose::APPEND_LENGTH.bits() != 0 {
            let _ = write!(sb, "({}f)", self.sol);
        }
        sb
    }
}

/// Port of `cs.min2phase.SearchWCA`: [`Search`] with restrictions on the axis of the
/// first and last move, and tnoodle's wall-clock limits.
#[derive(Clone, Default)]
pub struct SearchWca {
    search: Search,
}

impl SearchWca {
    /// A solver; the shared tables are built on first use.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Port of `SearchWCA.solution(facelets, maxDepth, probeMax, probeMin, verbose,
    /// firstAxisRestriction, lastAxisRestriction)`. The solution may not start on the
    /// axis of `first_axis` nor end on the axis of `last_axis` (e.g. `"R"`, `"U2"`).
    ///
    /// # Errors
    ///
    /// Java's error 9 for a restriction that is not a face move, 1-6 for invalid
    /// facelets, 7 when no solution exists within `max_depth`, and 8 when the budget's
    /// maximum runs out first.
    pub fn solution(
        &mut self,
        facelets: &str,
        max_depth: i32,
        budget: SearchBudget,
        verbose: Verbose,
        first_axis: Option<&str>,
        last_axis: Option<&str>,
    ) -> Result<String, SearchError> {
        let mut filter = WcaFilter {
            first_move_filter: [0; 6],
            last_move_filter: [0; 6],
            is_axis_restricted: false,
            start_time: None,
        };
        let str2axis = |s: &str| {
            WCA_MOVE2STR
                .iter()
                .position(|&m| m == s)
                .map(|i| i / 3 % 3)
                .ok_or(SearchError::InvalidRestriction)
        };
        if let Some(first) = first_axis {
            let axis = str2axis(first)?;
            for i in 0..3 {
                let shift = i32::from(URF_MOVE[(3 - i) % 3][axis * 3]) / 3 * 3;
                filter.first_move_filter[i] |= 0xe07 << shift;
                filter.last_move_filter[i + 3] |= 0xe07 << shift;
            }
            filter.is_axis_restricted = true;
        }
        if let Some(last) = last_axis {
            let axis = str2axis(last)?;
            for i in 0..3 {
                let shift = i32::from(URF_MOVE[(3 - i) % 3][axis * 3]) / 3 * 3;
                filter.last_move_filter[i] |= 0xe07 << shift;
                filter.first_move_filter[i + 3] |= 0xe07 << shift;
            }
            filter.is_axis_restricted = true;
        }
        let (probe_max, probe_min) = match budget {
            SearchBudget::Time { min, max } => {
                filter.start_time = Some(Instant::now());
                (millis(max), millis(min))
            }
            SearchBudget::Probes { min, max } => (count(max), count(min)),
        };
        self.search.wca = Some(filter);
        self.search
            .solve(facelets, max_depth, probe_max, probe_min, verbose)
    }
}

fn millis(d: Duration) -> i64 {
    i64::try_from(d.as_millis()).unwrap_or(i64::MAX)
}

fn count(n: u64) -> i64 {
    i64::try_from(n).unwrap_or(i64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn axis(mv: &str) -> usize {
        "URFDLB".find(&mv[..1]).unwrap() % 3
    }

    #[test]
    fn solved_cube_has_an_empty_solution() {
        let facelets = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";
        assert_eq!(
            Search::new().solution(facelets, 21, 100_000, 0, Verbose::NONE),
            Ok(String::new())
        );
    }

    #[test]
    fn inverse_solution_reproduces_state_within_21_moves() {
        let mut rng = tnoodle_random::JavaRandom::from_seed(b"inverse");
        let mut search = Search::new();
        for _ in 0..200 {
            let facelets = crate::tools::random_cube(&mut rng);
            let scramble = search
                .solution(&facelets, 21, 1_000_000, 0, Verbose::INVERSE_SOLUTION)
                .unwrap();
            let moves: Vec<&str> = scramble.split_whitespace().collect();
            assert!(moves.len() <= 21, "{} moves: {scramble}", moves.len());
            assert_eq!(
                crate::tools::from_scramble(&scramble).unwrap(),
                facelets,
                "scramble {scramble}"
            );
        }
    }

    #[test]
    fn axis_restrictions_are_respected() {
        let mut rng = tnoodle_random::JavaRandom::from_seed(b"axis");
        let mut search = SearchWca::new();
        let budget = SearchBudget::Probes {
            min: 0,
            max: 1_000_000,
        };
        for (first, last) in [("R", "F"), ("L", "L"), ("U", "B"), ("F", "D")] {
            for _ in 0..20 {
                let facelets = crate::tools::random_cube(&mut rng);
                let solution = search
                    .solution(
                        &facelets,
                        21,
                        budget,
                        Verbose::NONE,
                        Some(first),
                        Some(last),
                    )
                    .unwrap();
                let moves: Vec<&str> = solution.split_whitespace().collect();
                assert_ne!(
                    axis(moves[0]),
                    axis(first),
                    "{solution} starts on {first} axis"
                );
                assert_ne!(
                    axis(moves[moves.len() - 1]),
                    axis(last),
                    "{solution} ends on {last} axis"
                );
            }
        }
    }

    #[test]
    fn invalid_facelets_return_java_codes() {
        let solved = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";
        let mut search = Search::new();
        let mut solve = |f: &str| {
            search
                .solution(f, 21, 100_000, 0, Verbose::NONE)
                .map_err(SearchError::code)
        };
        assert_eq!(solve(&solved[..53]), Err(1));
        // A flipped edge: swap the two stickers of UF (U8 = 7, F2 = 19).
        let mut flipped: Vec<char> = solved.chars().collect();
        flipped.swap(7, 19);
        assert_eq!(solve(&flipped.iter().collect::<String>()), Err(3));
        // A twisted corner: cycle the stickers of URF (U9 = 8, R1 = 9, F3 = 20).
        let mut twisted: Vec<char> = solved.chars().collect();
        let (a, b, c) = (twisted[8], twisted[9], twisted[20]);
        (twisted[8], twisted[9], twisted[20]) = (b, c, a);
        assert_eq!(solve(&twisted.iter().collect::<String>()), Err(5));
        // Odd parity: swap the UF and UR edges, both stickers of each.
        let mut parity: Vec<char> = solved.chars().collect();
        parity.swap(7, 5);
        parity.swap(19, 10);
        assert_eq!(solve(&parity.iter().collect::<String>()), Err(6));
    }

    #[test]
    fn exhausted_searches_return_java_codes() {
        let facelets = crate::tools::from_scramble("R U F D L B R' U' F' D' L' B'").unwrap();
        let mut search = Search::new();
        assert_eq!(
            search
                .solution(&facelets, 3, 1_000_000, 0, Verbose::NONE)
                .map_err(SearchError::code),
            Err(7)
        );
        assert_eq!(
            search
                .solution(&facelets, 21, 0, 0, Verbose::NONE)
                .map_err(SearchError::code),
            Err(8)
        );
        let mut wca = SearchWca::new();
        let budget = SearchBudget::Probes { min: 0, max: 1000 };
        assert_eq!(
            wca.solution(&facelets, 21, budget, Verbose::NONE, Some("X"), None)
                .map_err(SearchError::code),
            Err(9)
        );
        assert_eq!(
            wca.solution(&facelets, 21, budget, Verbose::NONE, None, Some("Rw"))
                .map_err(SearchError::code),
            Err(9)
        );
    }
}
