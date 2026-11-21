//! Differential test against Java tnoodle-lib.
//!
//! Prints one tab-separated case per line: the input, then this crate's
//! answer. `tools/java-crosscheck/CrossCheck.java` recomputes every answer
//! with the original Java code and reports any difference. Run both through
//! `tools/java-crosscheck/run.sh`.
//!
//! ```sh
//! cargo run --release -p tnoodle-scrambles --example crosscheck -- <seed> <scale>
//! ```

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use tnoodle_min2phase::{search::{Search as TwoPhase, SearchResult}, tools};
use tnoodle_scrambles::puzzles::cube_puzzle::{CubePuzzle, CubeState};
use tnoodle_scrambles::puzzles::cube_svg;
use tnoodle_scrambles::{AlgorithmBuilder, MergingMode, Puzzle, PuzzleState, PuzzleType, generate_scramble};
use tnoodle_solvers::{pyraminx, skewb, two_by_two};
use tnoodle_sq12phase::full_cube::FullCube;
use tnoodle_sq12phase::search::{INVERSE_SOLUTION, Search as Sq1Search};

const NONE: &str = "-";

fn or_none(s: Option<String>) -> String {
    s.unwrap_or_else(|| NONE.to_string())
}

fn axis(mv: &str) -> Option<usize> {
    mv.chars().find_map(|c| "URFDLB".find(c)).map(|f| f % 3)
}

/// A random move sequence that is deliberately full of redundancy: moves
/// often repeat an axis (`R L R`). With `rotations`, whole-cube rotations
/// appear too; scramble generation never feeds those to the canonicalizer.
fn messy_algorithm(rng: &mut impl Rng, size: usize, length: usize, rotations: bool) -> String {
    let mut moves: Vec<String> = CubePuzzle::new(size)
        .solved_state()
        .scramble_successors()
        .into_keys()
        .collect();
    moves.sort();
    let rotation_moves = ["x", "x'", "x2", "y", "y'", "y2", "z", "z'", "z2"];
    let mut alg: Vec<String> = Vec::new();
    for _ in 0..length {
        let previous_axis = alg.last().and_then(|m| axis(m));
        let mv = if rotations && rng.random_bool(0.05) {
            rotation_moves[rng.random_range(0..rotation_moves.len())].to_string()
        } else if let (Some(a), true) = (previous_axis, rng.random_bool(0.5)) {
            let same_axis: Vec<&String> = moves.iter().filter(|m| axis(m) == Some(a)).collect();
            same_axis[rng.random_range(0..same_axis.len())].clone()
        } else {
            moves[rng.random_range(0..moves.len())].clone()
        };
        alg.push(mv);
    }
    alg.join(" ")
}

fn canonicalize(size: usize, alg: &str) -> String {
    let result = std::panic::catch_unwind(|| {
        let mut ab = AlgorithmBuilder::new(MergingMode::CanonicalizeMoves, CubeState::solved(size));
        ab.append_algorithm(alg).map(|()| ab.to_string())
    });
    match result {
        Ok(Ok(canonical)) => canonical,
        Ok(Err(_)) => "ERR".to_string(),
        Err(_) => "PANIC".to_string(),
    }
}

/// A twist is legal when neither layer has a corner straddling the cut.
fn twistable(c: &FullCube) -> bool {
    let clear = |a: i32, b: i32| (a & 0xf) != ((b >> 20) & 0xf) && (b & 0xf) != ((a >> 20) & 0xf);
    clear(c.ul, c.ur) && clear(c.dl, c.dr)
}

/// A Square-1 state a few twists from solved, so the optimal search finds it.
fn near_solved_sq1(rng: &mut impl Rng) -> FullCube {
    let mut c = FullCube::new();
    for _ in 0..rng.random_range(1..=4) {
        loop {
            let mut next = c.clone();
            let top = rng.random_range(0..12);
            let bottom = rng.random_range(0..12);
            if top != 0 {
                next.do_move(top);
            }
            if bottom != 0 {
                next.do_move(-bottom);
            }
            if twistable(&next) {
                next.do_move(0);
                c = next;
                break;
            }
        }
    }
    c
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let seed: u64 = args.get(1).map_or(1, |s| s.parse().expect("seed"));
    let scale: usize = args.get(2).map_or(1, |s| s.parse().expect("scale"));
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    std::panic::set_hook(Box::new(|_| {}));

    // Canonicalization (AlgorithmBuilder) and the app's SVG renderer.
    for size in 2..=7 {
        // The solved render lets the checker map this crate's colours to Java's.
        let solved = cube_svg::render_cube_svg(size, "").replace('\n', "\\n");
        println!("svg\t{size}\t\t{solved}");
        for _ in 0..40 * scale {
            let alg = messy_algorithm(&mut rng, size, 30, false);
            println!("canon\t{size}\t{alg}\t{}", canonicalize(size, &alg));
            let svg = cube_svg::render_cube_svg(size, &alg).replace('\n', "\\n");
            println!("svg\t{size}\t{alg}\t{svg}");
        }
        for _ in 0..10 * scale {
            let alg = messy_algorithm(&mut rng, size, 30, true);
            println!("canonrot\t{size}\t{alg}\t{}", canonicalize(size, &alg));
            let svg = cube_svg::render_cube_svg(size, &alg).replace('\n', "\\n");
            println!("svgrot\t{size}\t{alg}\t{svg}");
        }
    }

    // Small-puzzle solvers: same coordinates in, compare answers.
    for _ in 0..100 * scale {
        let s = two_by_two::random_state(&mut rng);
        let solve = or_none(two_by_two::solve(s, 11));
        let exact = or_none(two_by_two::generate_exactly(s, 11));
        println!("222\t{}\t{}\t{solve}\t{exact}", s.permutation, s.orientation);
    }
    for _ in 0..100 * scale {
        let s = skewb::random_state(&mut rng);
        let solve = or_none(skewb::solve(s, 11));
        let exact = or_none(skewb::generate_exactly(s, 11));
        println!("skewb\t{}\t{}\t{solve}\t{exact}", s.perm, s.twist);
    }
    for _ in 0..100 * scale {
        let s = pyraminx::random_state(&mut rng);
        let solve = or_none(pyraminx::solve(s, 20));
        let exact = or_none(pyraminx::generate_exactly(s, 11 + s.unsolved_tips()));
        println!(
            "pyram\t{}\t{}\t{}\t{}\t{solve}\t{exact}",
            s.edge_perm, s.edge_orient, s.corner_orient, s.tips
        );
    }

    // Square-1: the two-phase search on random states, the optimal search
    // on states close to solved.
    let mut sq1 = Sq1Search::new();
    for i in 0..60 * scale {
        let c = if i % 2 == 0 { FullCube::random_cube(&mut rng) } else { near_solved_sq1(&mut rng) };
        let solution = or_none(sq1.solution_with_verbose(&c, INVERSE_SOLUTION));
        let optimal = or_none(sq1.solution_opt_with_verbose(&c, 10, INVERSE_SOLUTION));
        println!("sq1\t{}\t{}\t{}\t{}\t{}\t{solution}\t{optimal}", c.ul, c.ur, c.dl, c.dr, c.ml);
    }

    // min2phase: the same facelets go to both solvers.
    let mut two_phase = TwoPhase::new();
    for _ in 0..100 * scale {
        let facelets = tools::random_state(&mut rng);
        let scramble = match two_phase.solution(&facelets, 21, 1_000_000, 0, INVERSE_SOLUTION as u32) {
            SearchResult::Solution(s) => s,
            other => format!("ERR {other:?}"),
        };
        println!("333solve\t{facelets}\t{}", scramble.trim());
    }

    // Full scrambles for every event, judged by Java.
    for &puzzle in PuzzleType::ALL {
        let count = match puzzle {
            PuzzleType::Four | PuzzleType::FourNi => 20,
            _ => 50,
        } * scale;
        for _ in 0..count {
            let scramble = generate_scramble(puzzle, &mut rng).replace('\n', "\\n");
            println!("scramble\t{}\t{scramble}", puzzle.short_name());
        }
    }
}
