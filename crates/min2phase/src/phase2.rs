//! Phase-2 move and pruning tables on raw (non-symmetry-reduced) coordinates.
//!
//! Phase 2 solves a cube inside G1 = <U, D, R2, L2, F2, B2>. Its state is
//! described by three permutation coordinates:
//! - corner permutation (8! = 40320)
//! - U/D-layer edge permutation (8! = 40320)
//! - E-slice edge permutation (4! = 24)
//!
//! Java's min2phase stores these symmetry-reduced, which requires
//! conjugating every move by the current symmetry. Raw coordinates avoid that
//! bookkeeping at the cost of ~2 MB of tables, and the tables are derived
//! straight from cubie multiplication so they cannot drift from the moves.

use std::collections::VecDeque;
use std::sync::LazyLock;

use crate::coord_cube::{N_MOVES2, N_MPERM, N_PERM};
use crate::cubie_cube::{self, CubieCube, TABLES};
use crate::util;

pub struct Phase2Tables {
    pub c_perm_move: Vec<[u16; N_MOVES2]>,
    pub e_perm_move: Vec<[u16; N_MOVES2]>,
    pub m_perm_move: [[u8; N_MOVES2]; N_MPERM],
    /// Distance to solved for (corner perm, slice perm).
    pub c_prun: Vec<u8>,
    /// Distance to solved for (U/D edge perm, slice perm).
    pub e_prun: Vec<u8>,
}

pub static PHASE2_TABLES: LazyLock<Phase2Tables> = LazyLock::new(Phase2Tables::init);

impl Phase2Tables {
    fn init() -> Self {
        let ct = &*TABLES;
        let mut c_perm_move = vec![[0u16; N_MOVES2]; N_PERM];
        let mut e_perm_move = vec![[0u16; N_MOVES2]; N_PERM];
        let mut m_perm_move = [[0u8; N_MOVES2]; N_MPERM];

        let mut c = CubieCube::new();
        let mut d = CubieCube::new();
        for i in 0..N_PERM {
            c.set_c_perm(i as i32);
            c.set_e_perm(i as i32);
            for j in 0..N_MOVES2 {
                let mv = &ct.move_cube[util::UD2STD[j]];
                cubie_cube::corn_mult(&c, mv, &mut d);
                c_perm_move[i][j] = d.get_c_perm() as u16;
                cubie_cube::edge_mult(&c, mv, &mut d);
                e_perm_move[i][j] = d.get_e_perm() as u16;
            }
        }
        let mut c = CubieCube::new();
        for (i, row) in m_perm_move.iter_mut().enumerate() {
            c.set_m_perm(i as i32);
            for (j, entry) in row.iter_mut().enumerate() {
                cubie_cube::edge_mult(&c, &ct.move_cube[util::UD2STD[j]], &mut d);
                *entry = d.get_m_perm() as u8;
            }
        }

        let c_prun = bfs(&c_perm_move, &m_perm_move);
        let e_prun = bfs(&e_perm_move, &m_perm_move);
        Self { c_perm_move, e_perm_move, m_perm_move, c_prun, e_prun }
    }

    /// Lower bound on the number of phase-2 moves needed.
    pub fn prun(&self, c_perm: usize, e_perm: usize, m_perm: usize) -> i32 {
        i32::from(
            self.c_prun[c_perm * N_MPERM + m_perm].max(self.e_prun[e_perm * N_MPERM + m_perm]),
        )
    }
}

fn bfs(perm_move: &[[u16; N_MOVES2]], m_perm_move: &[[u8; N_MOVES2]; N_MPERM]) -> Vec<u8> {
    let mut dist = vec![u8::MAX; N_PERM * N_MPERM];
    let mut queue = VecDeque::new();
    dist[0] = 0;
    queue.push_back(0usize);
    while let Some(idx) = queue.pop_front() {
        let (p, m) = (idx / N_MPERM, idx % N_MPERM);
        let next_dist = dist[idx] + 1;
        for j in 0..N_MOVES2 {
            let next = perm_move[p][j] as usize * N_MPERM + m_perm_move[m][j] as usize;
            if dist[next] == u8::MAX {
                dist[next] = next_dist;
                queue.push_back(next);
            }
        }
    }
    dist
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_phase2_state_is_reachable() {
        let t = &*PHASE2_TABLES;
        // U flips the corner and U/D edge parities, R2 flips the edge and
        // slice parities, so every (perm, slice perm) pair is reachable.
        assert!(t.c_prun.iter().all(|&d| d != u8::MAX));
        assert!(t.e_prun.iter().all(|&d| d != u8::MAX));
    }
}
