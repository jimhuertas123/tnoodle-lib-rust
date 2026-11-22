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
use tnoodle_min2phase::search::{
    Search as TwoPhase, SearchBudget, SearchError, SearchWca, Verbose,
};
use tnoodle_min2phase::tools;
use tnoodle_random::JavaRandom;
use tnoodle_scrambles::puzzles::cube_puzzle::{CubePuzzle, CubeState};
use tnoodle_scrambles::puzzles::cube_svg;
use tnoodle_scrambles::{
    AlgorithmBuilder, MergingMode, Puzzle, PuzzleState, PuzzleType, generate_scramble,
};
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

/// Seeds for the `JavaRandom` comparison: empty, ASCII, long and non-ASCII.
fn random_seeds() -> Vec<Vec<u8>> {
    let long: Vec<u8> = (0..1000u32).map(|i| ((i * 31 + 7) % 256) as u8).collect();
    vec![
        Vec::new(),
        b"abc".to_vec(),
        long,
        "ñandú".as_bytes().to_vec(),
    ]
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// `JavaRandom` draws, 1,000 per seed for each bound and each `Random` method.
fn print_java_random_cases() {
    const DRAWS: usize = 1000;
    for seed in random_seeds() {
        for bound in [1, 2, 3, 12, 40320, 479_001_600, i32::MAX] {
            let mut r = JavaRandom::from_seed(&seed);
            let values: Vec<String> = (0..DRAWS)
                .map(|_| {
                    r.next_int_bounded(bound)
                        .expect("positive bound")
                        .to_string()
                })
                .collect();
            println!("jr-bounded\t{}\t{bound}\t{}", hex(&seed), values.join(","));
        }
        let methods: [(&str, fn(&mut JavaRandom) -> String); 5] = [
            ("int", |r| r.next_int().to_string()),
            ("long", |r| r.next_long().to_string()),
            ("boolean", |r| r.next_boolean().to_string()),
            ("double", |r| format!("{:016x}", r.next_double().to_bits())),
            ("bytes", |r| {
                let mut b = [0u8; 37];
                r.next_bytes(&mut b);
                hex(&b)
            }),
        ];
        for (name, draw) in methods {
            let mut r = JavaRandom::from_seed(&seed);
            let values: Vec<String> = (0..DRAWS).map(|_| draw(&mut r)).collect();
            println!("jr-other\t{}\t{name}\t{}", hex(&seed), values.join(","));
        }
    }
}

fn fnv(values: impl IntoIterator<Item = u32>) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for v in values {
        h = (h ^ u64::from(v)).wrapping_mul(0x0100_0000_01b3);
    }
    format!("{h:x}")
}

fn rows(t: &[Vec<u16>]) -> String {
    fnv(t.iter().flatten().map(|&v| u32::from(v)))
}

/// Hashes of every min2phase table, compared with Java's after `Search.init()`.
fn print_min2phase_table_cases() {
    use tnoodle_min2phase::coord_cube::COORD_TABLES as T;
    use tnoodle_min2phase::cubie_cube::TABLES as C;
    let flat = |t: &[u32]| fnv(t.iter().copied());
    let small = |t: &[u16]| fnv(t.iter().map(|&v| u32::from(v)));
    let cases = [
        ("UDSliceMove", rows(&T.ud_slice_move)),
        ("TwistMove", rows(&T.twist_move)),
        ("FlipMove", rows(&T.flip_move)),
        ("UDSliceConj", rows(&T.ud_slice_conj)),
        ("UDSliceTwistPrun", flat(&T.ud_slice_twist_prun)),
        ("UDSliceFlipPrun", flat(&T.ud_slice_flip_prun)),
        ("TwistFlipPrun", flat(&T.twist_flip_prun)),
        ("CPermMove", rows(&T.c_perm_move)),
        ("EPermMove", rows(&T.e_perm_move)),
        ("MPermMove", rows(&T.m_perm_move)),
        ("MPermConj", rows(&T.m_perm_conj)),
        ("CCombPMove", rows(&T.c_comb_p_move)),
        ("CCombPConj", rows(&T.c_comb_p_conj)),
        ("MCPermPrun", flat(&T.mc_perm_prun)),
        ("EPermCCombPPrun", flat(&T.e_perm_c_comb_p_prun)),
        ("FlipS2RF", small(&C.flip_s2rf)),
        ("FlipR2S", small(&C.flip_r2s)),
        ("SymStateTwist", small(&C.sym_state_twist)),
        ("SymStatePerm", small(&C.sym_state_perm)),
        (
            "Perm2CombP",
            fnv(C.perm2_comb_p.iter().map(|&v| u32::from(v))),
        ),
    ];
    for (name, hash) in cases {
        println!("m2p-table\t{name}\t{hash}");
    }
}

fn shown(result: Result<String, SearchError>) -> String {
    result.unwrap_or_else(|e| e.to_string())
}

/// A random sequence of `n` face moves (0-17) with no two on the same face in a row.
fn random_moves(rng: &mut JavaRandom, n: usize) -> Vec<usize> {
    let mut moves: Vec<usize> = Vec::new();
    while moves.len() < n {
        let m = rng.next_index(18);
        if moves.last().is_none_or(|&last| last / 3 != m / 3) {
            moves.push(m);
        }
    }
    moves
}

/// min2phase `Search`, `SearchWca` and `Tools` on the same inputs as Java.
fn print_min2phase_cases(scale: usize) {
    const MOVE_NAMES: [&str; 18] = [
        "U", "U2", "U'", "R", "R2", "R'", "F", "F2", "F'", "D", "D2", "D'", "L", "L2", "L'", "B",
        "B2", "B'",
    ];
    let mut rng = JavaRandom::from_seed(b"min2phase");
    let mut search = TwoPhase::new();
    let flags = [0, 2, 1, 4, 7];
    for _ in 0..1000 * scale {
        let facelets = tools::random_cube(&mut rng);
        for verbose in flags {
            let out =
                shown(search.solution(&facelets, 21, 100_000, 0, Verbose::from_bits(verbose)));
            println!("m2p-search\t{facelets}\t21\t100000\t0\t{verbose}\t{out}");
        }
        let out = shown(search.solution(&facelets, 21, 100_000, 2000, Verbose::NONE));
        println!("m2p-search\t{facelets}\t21\t100000\t2000\t0\t{out}");
    }
    for _ in 0..50 * scale {
        let facelets = tools::from_scramble_moves(&random_moves(&mut rng, 13));
        let out = shown(search.solution(&facelets, 13, 100_000, 0, Verbose::OPTIMAL_SOLUTION));
        println!("m2p-opt\t{facelets}\t13\t100000\t0\t8\t{out}");
    }
    let restrictions = [
        (Some("R"), Some("F")),
        (Some("U"), Some("B")),
        (Some("L"), Some("L")),
        (Some("F"), Some("D")),
        (None, Some("F")),
        (Some("R"), None),
    ];
    let mut wca = SearchWca::new();
    let budget = SearchBudget::Time {
        min: std::time::Duration::ZERO,
        max: std::time::Duration::from_secs(60),
    };
    for _ in 0..100 * scale {
        let facelets = tools::random_cube(&mut rng);
        for (first, last) in restrictions {
            let out = shown(wca.solution(
                &facelets,
                21,
                budget,
                Verbose::INVERSE_SOLUTION,
                first,
                last,
            ));
            println!(
                "m2p-wca\t{facelets}\t{}\t{}\t{out}",
                first.unwrap_or(NONE),
                last.unwrap_or(NONE)
            );
        }
    }
    for _ in 0..100 * scale {
        let facelets = tools::random_cube(&mut rng);
        let mut outs = vec![shown(search.solution(
            &facelets,
            21,
            100_000,
            0,
            Verbose::NONE,
        ))];
        outs.push(format!("{}/{}", search.number_of_probes(), search.length()));
        for _ in 0..3 {
            outs.push(shown(search.next(100_000, 0, Verbose::NONE)));
            outs.push(format!("{}/{}", search.number_of_probes(), search.length()));
        }
        println!("m2p-next\t{facelets}\t{}", outs.join("|"));
    }
    for i in 0..200 * scale {
        let mut chars: Vec<char> = tools::random_cube(&mut rng).chars().collect();
        // Swap two stickers, or drop one, to corrupt the cube in different ways.
        if i % 10 == 0 {
            chars.pop();
        } else {
            let a = rng.next_index(54);
            let b = rng.next_index(54);
            chars.swap(a, b);
        }
        let facelets: String = chars.into_iter().collect();
        let out = shown(search.solution(&facelets, 21, 100_000, 0, Verbose::NONE));
        println!("m2p-error\t{facelets}\t{out}");
    }
    type Generator = fn(&mut JavaRandom) -> String;
    let generators: [(&str, Generator); 10] = [
        ("randomCube", tools::random_cube),
        ("randomLastLayer", tools::random_last_layer),
        ("randomLastSlot", tools::random_last_slot),
        ("randomZBLastLayer", tools::random_zb_last_layer),
        (
            "randomCornerOfLastLayer",
            tools::random_corner_of_last_layer,
        ),
        ("randomEdgeOfLastLayer", tools::random_edge_of_last_layer),
        ("randomCrossSolved", tools::random_cross_solved),
        ("randomEdgeSolved", tools::random_edge_solved),
        ("randomCornerSolved", tools::random_corner_solved),
        ("superFlip", |_| tools::super_flip()),
    ];
    for (name, generate) in generators {
        let mut gen_rng = JavaRandom::from_seed(format!("tools-{name}").as_bytes());
        let states: Vec<String> = (0..500).map(|_| generate(&mut gen_rng)).collect();
        println!("m2p-tools\t{name}\t{}", states.join(","));
    }
    for _ in 0..200 * scale {
        let moves = random_moves(&mut rng, 25);
        let scramble: Vec<&str> = moves.iter().map(|&m| MOVE_NAMES[m]).collect();
        let scramble = scramble.join(" ");
        let facelets = tools::from_scramble(&scramble).expect("canonical moves");
        println!("m2p-from-scramble\t{scramble}\t{facelets}");
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let seed: u64 = args.get(1).map_or(1, |s| s.parse().expect("seed"));
    let scale: usize = args.get(2).map_or(1, |s| s.parse().expect("scale"));
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    std::panic::set_hook(Box::new(|_| {}));
    print_java_random_cases();
    print_min2phase_table_cases();

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
        println!(
            "222\t{}\t{}\t{solve}\t{exact}",
            s.permutation, s.orientation
        );
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
        let c = if i % 2 == 0 {
            FullCube::random_cube(&mut rng)
        } else {
            near_solved_sq1(&mut rng)
        };
        let solution = or_none(sq1.solution_with_verbose(&c, INVERSE_SOLUTION));
        let optimal = or_none(sq1.solution_opt_with_verbose(&c, 10, INVERSE_SOLUTION));
        println!(
            "sq1\t{}\t{}\t{}\t{}\t{}\t{solution}\t{optimal}",
            c.ul, c.ur, c.dl, c.dr, c.ml
        );
    }

    print_min2phase_cases(scale);

    // Full scrambles for every event, judged by Java. JUDGE_COUNT overrides
    // the per-event count.
    let judge_count: Option<usize> = std::env::var("JUDGE_COUNT")
        .ok()
        .map(|s| s.parse().expect("JUDGE_COUNT"));
    for &puzzle in PuzzleType::ALL {
        let count = judge_count.unwrap_or(
            match puzzle {
                PuzzleType::Four | PuzzleType::FourNi => 20,
                _ => 50,
            } * scale,
        );
        for _ in 0..count {
            let scramble = generate_scramble(puzzle, &mut rng).replace('\n', "\\n");
            println!("scramble\t{}\t{scramble}", puzzle.short_name());
        }
    }
}
