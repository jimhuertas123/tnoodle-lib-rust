//! Random states and scramble helpers.
//!
//! Port of `cs.min2phase.Tools`. Every generator draws from a [`JavaRandom`] in the
//! order Java does, so the same seed gives the same facelets as tnoodle-lib.

use tnoodle_random::JavaRandom;

use crate::cubie_cube::{self, CubieCube};
use crate::search::Search;
use crate::util;

/// A scramble token that is not a face move (`U`, `R2`, `F'`, ...).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidMove(pub String);

impl std::fmt::Display for InvalidMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid move: {}", self.0)
    }
}

impl std::error::Error for InvalidMove {}

/// What `randomState` knows about one kind of piece: Java's `STATE_RANDOM` (`null`),
/// `STATE_SOLVED` (an empty array), or each position's piece with `-1` for unknown.
#[derive(Debug, Clone)]
enum Pieces {
    Random,
    Solved,
    Given(Vec<i8>),
}

/// Port of `Tools.randomCube(Random)`: a uniformly random solvable state.
pub fn random_cube(rng: &mut JavaRandom) -> String {
    random_state(
        Pieces::Random,
        Pieces::Random,
        Pieces::Random,
        Pieces::Random,
        rng,
    )
}

/// Port of `Tools.randomLastLayer`.
pub fn random_last_layer(rng: &mut JavaRandom) -> String {
    random_state(
        Pieces::Given(vec![-1, -1, -1, -1, 4, 5, 6, 7]),
        Pieces::Given(vec![-1, -1, -1, -1, 0, 0, 0, 0]),
        Pieces::Given(vec![-1, -1, -1, -1, 4, 5, 6, 7, 8, 9, 10, 11]),
        Pieces::Given(vec![-1, -1, -1, -1, 0, 0, 0, 0, 0, 0, 0, 0]),
        rng,
    )
}

/// Port of `Tools.randomLastSlot`.
pub fn random_last_slot(rng: &mut JavaRandom) -> String {
    random_state(
        Pieces::Given(vec![-1, -1, -1, -1, -1, 5, 6, 7]),
        Pieces::Given(vec![-1, -1, -1, -1, -1, 0, 0, 0]),
        Pieces::Given(vec![-1, -1, -1, -1, 4, 5, 6, 7, -1, 9, 10, 11]),
        Pieces::Given(vec![-1, -1, -1, -1, 0, 0, 0, 0, -1, 0, 0, 0]),
        rng,
    )
}

/// Port of `Tools.randomZBLastLayer`.
pub fn random_zb_last_layer(rng: &mut JavaRandom) -> String {
    random_state(
        Pieces::Given(vec![-1, -1, -1, -1, 4, 5, 6, 7]),
        Pieces::Given(vec![-1, -1, -1, -1, 0, 0, 0, 0]),
        Pieces::Given(vec![-1, -1, -1, -1, 4, 5, 6, 7, 8, 9, 10, 11]),
        Pieces::Solved,
        rng,
    )
}

/// Port of `Tools.randomCornerOfLastLayer`.
pub fn random_corner_of_last_layer(rng: &mut JavaRandom) -> String {
    random_state(
        Pieces::Given(vec![-1, -1, -1, -1, 4, 5, 6, 7]),
        Pieces::Given(vec![-1, -1, -1, -1, 0, 0, 0, 0]),
        Pieces::Solved,
        Pieces::Solved,
        rng,
    )
}

/// Port of `Tools.randomEdgeOfLastLayer`.
pub fn random_edge_of_last_layer(rng: &mut JavaRandom) -> String {
    random_state(
        Pieces::Solved,
        Pieces::Solved,
        Pieces::Given(vec![-1, -1, -1, -1, 4, 5, 6, 7, 8, 9, 10, 11]),
        Pieces::Given(vec![-1, -1, -1, -1, 0, 0, 0, 0, 0, 0, 0, 0]),
        rng,
    )
}

/// Port of `Tools.randomCrossSolved`.
pub fn random_cross_solved(rng: &mut JavaRandom) -> String {
    random_state(
        Pieces::Random,
        Pieces::Random,
        Pieces::Given(vec![-1, -1, -1, -1, 4, 5, 6, 7, -1, -1, -1, -1]),
        Pieces::Given(vec![-1, -1, -1, -1, 0, 0, 0, 0, -1, -1, -1, -1]),
        rng,
    )
}

/// Port of `Tools.randomEdgeSolved`.
pub fn random_edge_solved(rng: &mut JavaRandom) -> String {
    random_state(
        Pieces::Random,
        Pieces::Random,
        Pieces::Solved,
        Pieces::Solved,
        rng,
    )
}

/// Port of `Tools.randomCornerSolved`.
pub fn random_corner_solved(rng: &mut JavaRandom) -> String {
    random_state(
        Pieces::Solved,
        Pieces::Solved,
        Pieces::Random,
        Pieces::Random,
        rng,
    )
}

/// Port of `Tools.superFlip`: every edge flipped in place.
#[must_use]
pub fn super_flip() -> String {
    let cc = CubieCube::from_coords(0, 0, 0, 2047);
    util::to_face_cube(&cc.ca, &cc.ea)
}

/// Port of `Tools.fromScramble(int[])`: the facelets after the moves (0-17, `U` to `B'`).
#[must_use]
pub fn from_scramble_moves(moves: &[usize]) -> String {
    let tables = &*cubie_cube::TABLES;
    let mut cc = CubieCube::new();
    for &m in moves {
        let mut next = CubieCube::new();
        cubie_cube::corn_mult(&cc, &tables.move_cube[m], &mut next);
        cubie_cube::edge_mult(&cc, &tables.move_cube[m], &mut next);
        cc = next;
    }
    util::to_face_cube(&cc.ca, &cc.ea)
}

/// Port of `Tools.fromScramble(String)`: the facelets after a scramble such as
/// `"R U2 F'"`.
///
/// # Errors
///
/// [`InvalidMove`] for a token that is not a face move, where Java would guess a
/// move (`R2'` becomes `F`).
pub fn from_scramble(scramble: &str) -> Result<String, InvalidMove> {
    let moves = scramble
        .split_whitespace()
        .map(|token| parse_move(token).ok_or_else(|| InvalidMove(token.to_string())))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(from_scramble_moves(&moves))
}

fn parse_move(token: &str) -> Option<usize> {
    let mut chars = token.chars();
    let axis = "URFDLB".find(chars.next()?)? * 3;
    let power = match (chars.next(), chars.next()) {
        (None, _) => 0,
        (Some('2'), None) => 1,
        (Some('\''), None) => 2,
        _ => return None,
    };
    Some(axis + power)
}

/// Port of `Tools.verify`: 0 for a valid cube, or Java's negative error code
/// (-1 facelets, -2 edges, -3 flip, -4 corners, -5 twist, -6 parity).
#[must_use]
pub fn verify(facelets: &str) -> i32 {
    Search::new().verify(facelets)
}

/// Port of `Tools.getNPerm(byte[], int)`: the Lehmer index of a permutation.
fn get_n_perm(arr: &[i8], n: usize) -> i32 {
    let mut idx = 0i32;
    for i in 0..n {
        idx *= (n - i) as i32;
        for j in i + 1..n {
            if arr[j] < arr[i] {
                idx += 1;
            }
        }
    }
    idx
}

fn count_unknown(pieces: &Pieces) -> usize {
    match pieces {
        Pieces::Solved | Pieces::Random => 0,
        Pieces::Given(arr) => arr.iter().filter(|&&p| p == -1).count(),
    }
}

/// Port of `Tools.resolveOri`: draws the unknown orientations and fixes the last
/// one so the total is a multiple of `base`; returns the orientation index.
fn resolve_ori(arr: &mut [i8], base: i32, rng: &mut JavaRandom) -> i32 {
    let mut sum = 0i32;
    let mut last_unknown = None;
    for (i, p) in arr.iter_mut().enumerate() {
        if *p == -1 {
            *p = i8::try_from(rng.next_index(base as usize)).expect("below base");
            last_unknown = Some(i);
        }
        sum += i32::from(*p);
    }
    if let Some(last) = last_unknown
        && sum % base != 0
    {
        arr[last] = i8::try_from((30 + i32::from(arr[last]) - sum) % base).expect("below base");
    }
    let mut idx = 0;
    for &p in &arr[..arr.len() - 1] {
        idx = idx * base + i32::from(p);
    }
    idx
}

/// Port of `Tools.resolvePerm`: places the missing pieces at random and swaps two
/// of them when the parity must be `parity`; returns the parity of the result.
fn resolve_perm(pieces: &mut Pieces, cnt_u: usize, parity: i32, rng: &mut JavaRandom) -> i32 {
    let arr = match pieces {
        Pieces::Solved => return 0,
        Pieces::Random => {
            return if parity == -1 {
                rng.next_index(2) as i32
            } else {
                parity
            };
        }
        Pieces::Given(arr) => arr,
    };
    let mut val: [i8; 12] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
    for &p in arr.iter() {
        if p != -1 {
            val[p as usize] = -1;
        }
    }
    let mut idx = 0usize;
    for i in 0..arr.len() {
        if val[i] != -1 {
            let j = rng.next_index(idx + 1);
            let temp = val[i];
            val[idx] = val[j];
            idx += 1;
            val[j] = temp;
        }
    }
    let mut cnt_u = cnt_u;
    let mut last = None;
    idx = 0;
    while idx < arr.len() && cnt_u > 0 {
        if arr[idx] == -1 {
            if cnt_u == 2 {
                last = Some(idx);
            }
            cnt_u -= 1;
            arr[idx] = val[cnt_u];
        }
        idx += 1;
    }
    let p = util::get_n_parity(get_n_perm(arr, arr.len()), arr.len() as i32);
    if let Some(last) = last
        && p == 1 - parity
    {
        arr.swap(idx - 1, last);
    }
    p
}

/// Port of `Tools.randomState(cp, co, ep, eo, rng)`.
fn random_state(
    mut cp: Pieces,
    co: Pieces,
    mut ep: Pieces,
    eo: Pieces,
    rng: &mut JavaRandom,
) -> String {
    let cnt_ue = if matches!(ep, Pieces::Random) {
        12
    } else {
        count_unknown(&ep)
    };
    let cnt_uc = if matches!(cp, Pieces::Random) {
        8
    } else {
        count_unknown(&cp)
    };
    let parity;
    let cp_val;
    let ep_val;
    if cnt_ue < 2 {
        if matches!(ep, Pieces::Solved) {
            ep_val = 0;
            parity = 0;
        } else {
            parity = resolve_perm(&mut ep, cnt_ue, -1, rng);
            ep_val = given_perm(&ep, 12);
        }
        cp_val = match cp {
            Pieces::Solved => 0,
            Pieces::Random => loop {
                let v = rng.next_index(40320) as i32;
                if util::get_n_parity(v, 8) == parity {
                    break v;
                }
            },
            Pieces::Given(_) => {
                resolve_perm(&mut cp, cnt_uc, parity, rng);
                given_perm(&cp, 8)
            }
        };
    } else {
        match cp {
            Pieces::Solved => {
                cp_val = 0;
                parity = 0;
            }
            Pieces::Random => {
                cp_val = rng.next_index(40320) as i32;
                parity = util::get_n_parity(cp_val, 8);
            }
            Pieces::Given(_) => {
                parity = resolve_perm(&mut cp, cnt_uc, -1, rng);
                cp_val = given_perm(&cp, 8);
            }
        }
        ep_val = match ep {
            Pieces::Random => loop {
                let v = rng.next_index(479_001_600) as i32;
                if util::get_n_parity(v, 12) == parity {
                    break v;
                }
            },
            Pieces::Solved | Pieces::Given(_) => {
                resolve_perm(&mut ep, cnt_ue, parity, rng);
                given_perm(&ep, 12)
            }
        };
    }
    let twist = orientation(co, 3, 2187, rng);
    let flip = orientation(eo, 2, 2048, rng);
    let cc = CubieCube::from_coords(cp_val, twist, ep_val, flip);
    util::to_face_cube(&cc.ca, &cc.ea)
}

fn given_perm(pieces: &Pieces, n: usize) -> i32 {
    match pieces {
        Pieces::Given(arr) => get_n_perm(arr, n),
        Pieces::Solved | Pieces::Random => 0,
    }
}

fn orientation(pieces: Pieces, base: i32, n: usize, rng: &mut JavaRandom) -> i32 {
    match pieces {
        Pieces::Random => rng.next_index(n) as i32,
        Pieces::Solved => 0,
        Pieces::Given(mut arr) => resolve_ori(&mut arr, base, rng),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_tokens_are_errors() {
        for token in ["R2'", "R3", "Rw", "RU", "x"] {
            assert_eq!(
                from_scramble(&format!("U {token} F")),
                Err(InvalidMove(token.to_string())),
                "{token}"
            );
        }
        assert!(from_scramble("R U2 F' D L B").is_ok());
    }

    #[test]
    fn scramble_then_inverse_is_solved() {
        let solved = from_scramble("").unwrap();
        assert_eq!(from_scramble("R U R' U'  U R U' R'").unwrap(), solved);
    }

    #[test]
    fn random_cube_is_valid() {
        let mut rng = JavaRandom::from_seed(b"valid");
        for _ in 0..100 {
            assert_eq!(verify(&random_cube(&mut rng)), 0);
        }
    }
}
