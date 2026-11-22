//! Statistical check that 3x3x3 scrambles produce uniformly random states.
//!
//! Port of Java's `scrambleanalysis` module (`CubeTest.testScrambles`). It is
//! slow (thousands of scrambles), so it only runs on demand:
//!
//! ```sh
//! cargo test --release -p tnoodle-scrambles --test randomness -- --ignored --nocapture
//! ```

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_lossless,
    clippy::many_single_char_names,
    clippy::type_complexity,
    clippy::format_collect,
    clippy::too_many_lines,
    clippy::items_after_statements,
    reason = "statistics over a few thousand counts, far below the precision limits; the single-letter names follow the formulas they implement"
)]

use tnoodle_min2phase::{tools, util};
use tnoodle_scrambles::{
    JavaRandom, PuzzleType, ScrambleOptions, SearchBudget, generate_wca_scramble,
};

/// Java's `Distribution.minimumSampleSize()` is 6144; App.java uses 6500.
const SCRAMBLE_COUNT: usize = 6500;
/// Family-wise significance level. Java tests each of its 23 hypotheses at
/// 0.01, which fails a perfect generator about 20% of the time; we split the
/// level across the tests (Bonferroni) so a failure means something.
const ALPHA: f64 = 0.01;
const TEST_COUNT: f64 = 1.0 + 12.0 + 1.0 + 8.0 + 1.0;

/// Cubie view of a scramble: `ca[i] = (twist << 3) | corner`,
/// `ea[i] = (edge << 1) | flip`, for the piece sitting at position `i`.
fn cubies(scramble: &str) -> ([u8; 8], [u8; 12]) {
    let facelets = tools::from_scramble(scramble).expect("generated scrambles are valid moves");
    let mut f = [0u8; 54];
    for (i, c) in facelets.chars().enumerate() {
        f[i] = "URFDLB".find(c).unwrap() as u8;
    }
    let (mut ca, mut ea) = ([0u8; 8], [0u8; 12]);
    util::to_cubie_cube(&f, &mut ca, &mut ea);
    (ca, ea)
}

fn odd_permutation(perm: &[u8]) -> bool {
    let mut inversions = 0;
    for i in 0..perm.len() {
        for j in i + 1..perm.len() {
            if perm[i] > perm[j] {
                inversions += 1;
            }
        }
    }
    inversions % 2 == 1
}

fn binomial(n: u64, k: u64) -> f64 {
    if k > n {
        return 0.0;
    }
    (0..k).fold(1.0, |acc, i| acc * (n - i) as f64 / (i + 1) as f64)
}

/// Probability of 0, 2, 4, ..., 12 flipped edges (Java `expectedEdgesOrientationProbability`).
fn expected_edge_orientation() -> Vec<f64> {
    let counts: Vec<f64> = (0..7).map(|i| binomial(12, 2 * i)).collect();
    let total: f64 = counts.iter().sum();
    counts.iter().map(|c| c / total).collect()
}

/// Probability of corner twist sums 0, 3, ..., 15 (Java `expectedCornersOrientationProbability`).
fn expected_corner_orientation() -> Vec<f64> {
    let counts: Vec<f64> = (0..6)
        .map(|i| {
            let sum = 3 * i;
            let mut partial = 0.0;
            for j in 0..8u64 {
                for k in 0..8u64 {
                    if j + k <= 8 && 2 * j + k == sum {
                        partial += binomial(8, j) * binomial(8 - j, k);
                    }
                }
            }
            partial
        })
        .collect();
    let total: f64 = counts.iter().sum();
    counts.iter().map(|c| c / total).collect()
}

// --- Statistics -------------------------------------------------------------

fn ln_gamma(x: f64) -> f64 {
    // Lanczos approximation (g = 7, n = 9).
    const COEF: [f64; 9] = [
        0.999_999_999_999_809_9,
        676.520_368_121_885_1,
        -1_259.139_216_722_402_8,
        771.323_428_777_653_1,
        -176.615_029_162_140_6,
        12.507_343_278_686_905,
        -0.138_571_095_265_720_12,
        9.984_369_578_019_572e-6,
        1.505_632_735_149_311_6e-7,
    ];
    let x = x - 1.0;
    let t = x + 7.5;
    let series: f64 = COEF[0] + (1..9).map(|i| COEF[i] / (x + i as f64)).sum::<f64>();
    0.5 * (2.0 * std::f64::consts::PI).ln() + (x + 0.5) * t.ln() - t + series.ln()
}

/// Regularized upper incomplete gamma function Q(a, x).
fn gamma_q(a: f64, x: f64) -> f64 {
    if x <= 0.0 {
        return 1.0;
    }
    let ln_prefix = a * x.ln() - x - ln_gamma(a);
    if x < a + 1.0 {
        // Series for P(a, x).
        let (mut term, mut sum, mut n) = (1.0 / a, 1.0 / a, a);
        while term.abs() > sum.abs() * 1e-15 {
            n += 1.0;
            term *= x / n;
            sum += term;
        }
        1.0 - sum * ln_prefix.exp()
    } else {
        // Continued fraction for Q(a, x) (modified Lentz).
        let tiny = 1e-300;
        let mut b = x + 1.0 - a;
        let mut c = 1.0 / tiny;
        let mut d = 1.0 / b;
        let mut h = d;
        for i in 1..1000 {
            let an = -(i as f64) * (i as f64 - a);
            b += 2.0;
            d = an * d + b;
            if d.abs() < tiny {
                d = tiny;
            }
            c = b + an / c;
            if c.abs() < tiny {
                c = tiny;
            }
            d = 1.0 / d;
            let delta = d * c;
            h *= delta;
            if (delta - 1.0).abs() < 1e-15 {
                break;
            }
        }
        ln_prefix.exp() * h
    }
}

/// p-value of Pearson's chi-square goodness-of-fit test.
fn chi_square_p(expected_probabilities: &[f64], observed: &[u64]) -> f64 {
    let n: u64 = observed.iter().sum();
    let statistic: f64 = expected_probabilities
        .iter()
        .zip(observed)
        .map(|(&p, &o)| {
            let e = p * n as f64;
            (o as f64 - e).powi(2) / e
        })
        .sum();
    let degrees_of_freedom = (observed.len() - 1) as f64;
    gamma_q(degrees_of_freedom / 2.0, statistic / 2.0)
}

/// Two-sided p-value that `successes` out of `n` came from probability 1/2
/// (normal approximation, accurate for thousands of trials).
fn fair_coin_p(n: usize, successes: usize) -> f64 {
    let z = (successes as f64 - n as f64 / 2.0) / (n as f64 / 4.0).sqrt();
    // erfc(|z| / sqrt 2) = Q(1/2, z^2 / 2)
    gamma_q(0.5, z * z / 2.0)
}

#[test]
fn statistics_helpers_match_known_values() {
    // Chi-square with 1 df at 3.841 has p = 0.05; with 10 df at 18.307, p = 0.05.
    assert!((gamma_q(0.5, 3.841 / 2.0) - 0.05).abs() < 1e-3);
    assert!((gamma_q(5.0, 18.307 / 2.0) - 0.05).abs() < 1e-3);
    assert!((fair_coin_p(100, 50) - 1.0).abs() < 1e-9);
    let sum: f64 = expected_corner_orientation().iter().sum();
    assert!((sum - 1.0).abs() < 1e-12);
    // Every corner twist vector with sum divisible by 3 is counted: 3^7.
    let counts: f64 = (0..6)
        .map(|i| expected_corner_orientation()[i] * 2187.0)
        .sum();
    assert!((counts - 2187.0).abs() < 1e-6);
}

#[test]
#[ignore = "slow: generates thousands of scrambles; run with --ignored"]
fn three_by_three_states_are_uniformly_random() {
    let mut rng = JavaRandom::from_entropy();
    // Validity and distribution do not depend on the 3x3 search budget.
    let fast = ScrambleOptions {
        search_budget: SearchBudget::Probes {
            min: 0,
            max: 60_000,
        },
    };
    let mut flipped_edges = [0u64; 7];
    let mut corner_twist_sums = [0u64; 6];
    let mut edge_positions = [[0u64; 12]; 12];
    let mut corner_positions = [[0u64; 8]; 8];
    let mut odd_parity = 0;

    for _ in 0..SCRAMBLE_COUNT {
        let scramble = generate_wca_scramble(PuzzleType::Three, &mut rng, &fast);
        let (ca, ea) = cubies(&scramble);

        let flips = ea.iter().filter(|&&e| e & 1 == 1).count();
        flipped_edges[flips / 2] += 1;
        let twist: u8 = ca.iter().map(|&c| c >> 3).sum();
        corner_twist_sums[usize::from(twist / 3)] += 1;
        for (position, &e) in ea.iter().enumerate() {
            edge_positions[usize::from(e >> 1)][position] += 1;
        }
        for (position, &c) in ca.iter().enumerate() {
            corner_positions[usize::from(c & 7)][position] += 1;
        }
        let corners: Vec<u8> = ca.iter().map(|&c| c & 7).collect();
        if odd_permutation(&corners) {
            odd_parity += 1;
        }
    }

    let threshold = ALPHA / TEST_COUNT;
    let mut failures = Vec::new();
    let mut check = |name: String, p: f64| {
        println!("{name:<28} p = {p:.4}");
        if p < threshold {
            failures.push(format!("{name} (p = {p:.2e})"));
        }
    };

    check(
        "edge orientation".into(),
        chi_square_p(&expected_edge_orientation(), &flipped_edges),
    );
    for (edge, counts) in edge_positions.iter().enumerate() {
        check(
            format!("edge {edge} final position"),
            chi_square_p(&[1.0 / 12.0; 12], counts),
        );
    }
    check(
        "corner orientation".into(),
        chi_square_p(&expected_corner_orientation(), &corner_twist_sums),
    );
    for (corner, counts) in corner_positions.iter().enumerate() {
        check(
            format!("corner {corner} final position"),
            chi_square_p(&[1.0 / 8.0; 8], counts),
        );
    }
    check(
        "permutation parity".into(),
        fair_coin_p(SCRAMBLE_COUNT, odd_parity),
    );

    assert!(failures.is_empty(), "not uniformly random: {failures:?}");
}
