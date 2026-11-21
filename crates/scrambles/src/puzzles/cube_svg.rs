//! SVG renderer for NxN cube puzzles.
//!
//! Renders a cube net (cross pattern) showing the state after applying a scramble.
//! Port of the Java `CubePuzzle` SVG generation.

use tnoodle_svglite::{Color, Dimension, Element, Svg};

// ---------------------------------------------------------------------------
// Face indices (matches Java: R=0, U=1, F=2, L=3, D=4, B=5)
// ---------------------------------------------------------------------------

const R: u8 = 0;
const U: u8 = 1;
const F: u8 = 2;
const L: u8 = 3;
const D: u8 = 4;
const B: u8 = 5;
pub(crate) const NUM_FACES: usize = 6;

fn opposite_face(f: u8) -> u8 {
    (f + 3) % 6
}

// ---------------------------------------------------------------------------
// Layout constants
// ---------------------------------------------------------------------------

const GAP: usize = 2;
const CUBIE_SIZE: usize = 10;

/// Colors for each face index. R=red, U=white, F=green, D=yellow, L=orange, B=blue.
const FACE_COLORS: [Color; NUM_FACES] = [
    Color::new(0xEC, 0x00, 0x00), // R = red    #EC0000
    Color::new(0xFF, 0xFF, 0xFF), // U = white  #FFFFFF
    Color::new(0x02, 0xD0, 0x40), // F = green  #02D040
    Color::new(0xFF, 0x8B, 0x24), // L = orange #FF8B24
    Color::new(0xFD, 0xD8, 0x35), // D = yellow #FDD835
    Color::new(0x30, 0x4F, 0xFE), // B = blue   #304FFE
];

// ---------------------------------------------------------------------------
// Cube state
// ---------------------------------------------------------------------------

/// A cube image: `image[face][row][col]` stores the face-index whose color is shown.
type CubeImage = Vec<Vec<Vec<u8>>>;

fn solved_image(size: usize) -> CubeImage {
    (0..NUM_FACES)
        .map(|f| vec![vec![f as u8; size]; size])
        .collect()
}

// ---------------------------------------------------------------------------
// Move application
// ---------------------------------------------------------------------------

/// Rotate a 2-D grid 90 degrees clockwise `count` times (count in 1..3).
fn rotate_face_cw<T: Copy>(grid: &mut Vec<Vec<T>>, count: usize) {
    let n = grid.len();
    for _ in 0..count {
        let old = grid.clone();
        for r in 0..n {
            for c in 0..n {
                grid[c][n - 1 - r] = old[r][c];
            }
        }
    }
}

/// Apply one slice turn to the cube image.
///
/// `face`  : the face being turned (R=0..B=5)
/// `slice` : 0 = outermost layer of that face, 1 = next layer inward, ...
/// `dir`   : 1 = CW (90), 2 = 180, 3 = CCW (270)
pub(crate) fn apply_slice<T: Copy>(
    image: &mut [Vec<Vec<T>>],
    size: usize,
    face: u8,
    slice: usize,
    dir: usize,
) {
    // Save originals before canonicalization (Java uses originals for face rotation)
    let orig_face = face;
    let orig_slice = slice;
    let orig_dir = dir;

    // Canonicalize: we only implement L, D, B directly.
    let (sface, sslice, sdir) = if face != L && face != D && face != B {
        let opp = opposite_face(face);
        let new_slice = size - 1 - slice;
        let new_dir = 4 - dir;
        (opp, new_slice, new_dir)
    } else {
        (face, slice, dir)
    };

    // Apply band cycling: sdir repetitions of one CW turn
    for _ in 0..sdir {
        single_slice_cw(image, size, sface, sslice);
    }

    // Face grid rotation uses ORIGINAL face/slice/dir.
    // Java's face swap does CCW per step; our rotate_face_cw does CW per step.
    // Conversion: java_ccw_count N → our_cw_count = 4 - N
    // Java slice==0: sdir = 4-dir CCW steps → we need dir CW steps
    // Java slice==size-1: sdir = dir CCW steps → we need 4-dir CW steps
    if orig_slice == 0 || orig_slice == size - 1 {
        if orig_slice == 0 {
            rotate_face_cw(&mut image[orig_face as usize], orig_dir);
        } else {
            let opp = opposite_face(orig_face);
            rotate_face_cw(&mut image[opp as usize], 4 - orig_dir);
        }
    }
}

/// One CW turn of a single slice band — exact port of Java's `swap` calls.
///
/// Java `swap(f1,x1,y1, f2,x2,y2, f3,x3,y3, f4,x4,y4, dir=1)` does:
///   `temp=f1; f1=f2; f2=f3; f3=f4; f4=temp`
///
/// We only implement L, D, B (R/U/F are canonicalized to these).
fn single_slice_cw<T: Copy>(image: &mut [Vec<Vec<T>>], size: usize, face: u8, slice: usize) {
    let n = size;
    let s = slice;

    match face {
        // Java L: swap(U[j,s], B[n-1-j, n-1-s], D[j,s], F[j,s], dir)
        L_FACE => {
            for j in 0..n {
                let tmp = image[1][j][s]; // U
                image[1][j][s] = image[5][n - 1 - j][n - 1 - s]; // U = B
                image[5][n - 1 - j][n - 1 - s] = image[4][j][s]; // B = D
                image[4][j][s] = image[2][j][s]; // D = F
                image[2][j][s] = tmp; // F = old U
            }
        }
        // Java D: swap(L[n-1-s, j], B[n-1-s, j], R[n-1-s, j], F[n-1-s, j], dir)
        D_FACE => {
            let r = n - 1 - s;
            for j in 0..n {
                let tmp = image[3][r][j]; // L
                image[3][r][j] = image[5][r][j]; // L = B
                image[5][r][j] = image[0][r][j]; // B = R
                image[0][r][j] = image[2][r][j]; // R = F
                image[2][r][j] = tmp; // F = old L
            }
        }
        // Java B: swap(U[s,j], R[j, n-1-s], D[n-1-s, n-1-j], L[n-1-j, s], dir)
        B_FACE => {
            for j in 0..n {
                let tmp = image[1][s][j]; // U
                image[1][s][j] = image[0][j][n - 1 - s]; // U = R
                image[0][j][n - 1 - s] = image[4][n - 1 - s][n - 1 - j]; // R = D
                image[4][n - 1 - s][n - 1 - j] = image[3][n - 1 - j][s]; // D = L
                image[3][n - 1 - j][s] = tmp; // L = old U
            }
        }
        _ => {}
    }
}

const L_FACE: u8 = L;
const D_FACE: u8 = D;
const B_FACE: u8 = B;

// ---------------------------------------------------------------------------
// Move parsing
// ---------------------------------------------------------------------------

/// A parsed move: face, slice range, and direction.
struct CubeMove {
    face: u8,
    /// Inclusive range of slices to turn.
    slice_start: usize,
    slice_end: usize, // inclusive
    dir: usize,       // 1=CW, 2=180, 3=CCW
}

/// Parse a scramble string into a list of moves.
fn parse_scramble(scramble: &str, size: usize) -> Vec<CubeMove> {
    let mut moves = Vec::new();
    for token in scramble.split_whitespace() {
        if let Some(m) = parse_move(token, size) {
            moves.push(m);
        }
    }
    moves
}

fn parse_move(token: &str, size: usize) -> Option<CubeMove> {
    let chars: Vec<char> = token.chars().collect();
    if chars.is_empty() {
        return None;
    }

    // Check for rotation moves: x, y, z
    let (face, slice_start, slice_end, rest_start) = match chars[0] {
        'x' => (R, 0, size - 1, 1),
        'y' => (U, 0, size - 1, 1),
        'z' => (F, 0, size - 1, 1),
        _ => {
            // Check for optional leading number (wide-layer count): e.g., "3Rw"
            let mut idx = 0;
            let mut wide_count: Option<usize> = None;
            if chars[idx].is_ascii_digit() {
                let mut num_str = String::new();
                while idx < chars.len() && chars[idx].is_ascii_digit() {
                    num_str.push(chars[idx]);
                    idx += 1;
                }
                wide_count = num_str.parse().ok();
            }

            if idx >= chars.len() {
                return None;
            }

            let face_char = chars[idx];
            let face = match face_char {
                'R' => R,
                'U' => U,
                'F' => F,
                'L' => L,
                'D' => D,
                'B' => B,
                _ => return None,
            };
            idx += 1;

            // Check for 'w' (wide move)
            let is_wide = idx < chars.len() && chars[idx] == 'w';
            if is_wide {
                idx += 1;
            }

            let (slice_start, slice_end) = if is_wide {
                let depth = wide_count.unwrap_or(2);
                (0, depth - 1)
            } else if wide_count.is_some() {
                // A number before a face without 'w' is not standard; treat as single slice
                (0, 0)
            } else {
                (0, 0)
            };

            (face, slice_start, slice_end, idx)
        }
    };

    // Parse direction suffix
    let rest: String = chars[rest_start..].iter().collect();
    let dir = if rest.is_empty() {
        1 // CW
    } else if rest == "2" {
        2 // 180
    } else if rest == "'" || rest == "\u{2019}" || rest == "`" {
        3 // CCW — accept ASCII ', Unicode ', and backtick
    } else if rest == "'2" || rest == "2'" || rest == "\u{2019}2" || rest == "2\u{2019}" {
        2 // 180
    } else {
        1
    };

    Some(CubeMove {
        face,
        slice_start,
        slice_end,
        dir,
    })
}

// ---------------------------------------------------------------------------
// SVG rendering
// ---------------------------------------------------------------------------

/// Compute the top-left (x, y) pixel position for a face in the net layout.
fn face_position(face: u8, size: usize) -> (usize, usize) {
    let cs = size * CUBIE_SIZE;
    match face {
        0 /* R */ => (3 * GAP + 2 * cs, 2 * GAP + cs),
        1 /* U */ => (2 * GAP + cs, GAP),
        2 /* F */ => (2 * GAP + cs, 2 * GAP + cs),
        3 /* L */ => (GAP, 2 * GAP + cs),
        4 /* D */ => (2 * GAP + cs, 3 * GAP + 2 * cs),
        5 /* B */ => (4 * GAP + 3 * cs, 2 * GAP + cs),
        _ => (0, 0),
    }
}

/// Render a cube scramble as an SVG string.
///
/// `size`: cube dimension (2-7)
/// `scramble`: space-separated moves like "R U R' F2 Rw"
pub fn render_cube_svg(size: usize, scramble: &str) -> String {
    let size = size.clamp(2, 7);
    let mut image = solved_image(size);

    // Apply scramble
    let moves = parse_scramble(scramble, size);
    for m in &moves {
        for slice in m.slice_start..=m.slice_end {
            apply_slice(&mut image, size, m.face, slice, m.dir);
        }
    }

    render_image(size, &image).to_string()
}

/// Draw a cube image (`image[face][row][col]` = face index of the sticker color).
pub(crate) fn render_image(size: usize, image: &[Vec<Vec<u8>>]) -> Svg {
    let cs = size * CUBIE_SIZE;
    let width = (cs + GAP) * 4 + GAP;
    let height = (cs + GAP) * 3 + GAP;
    let mut svg = Svg::new(Dimension::new(width as i32, height as i32));

    // Draw each face (no background — transparent)
    for face_idx in 0..NUM_FACES {
        let (fx, fy) = face_position(face_idx as u8, size);
        for row in 0..size {
            for col in 0..size {
                let color_idx = image[face_idx][row][col] as usize;
                let color = FACE_COLORS[color_idx];
                let x = fx + col * CUBIE_SIZE;
                let y = fy + row * CUBIE_SIZE;
                svg.append(
                    Element::rect(x as f64, y as f64, CUBIE_SIZE as f64, CUBIE_SIZE as f64)
                        .fill(color)
                        .stroke(Color::BLACK)
                        .stroke_width(1.0),
                );
            }
        }
    }

    svg
}

/// Size of the SVG drawn by [`render_image`].
pub(crate) fn preferred_size(size: usize) -> Dimension {
    let cs = size * CUBIE_SIZE;
    Dimension::new(((cs + GAP) * 4 + GAP) as i32, ((cs + GAP) * 3 + GAP) as i32)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Apply a scramble to a solved 3x3 and return the state as a URFDLB facelet string.
/// Our face order is R=0,U=1,F=2,L=3,D=4,B=5.
/// The standard facelet string is URFDLB order, 9 stickers per face, reading order.
#[cfg(test)]
fn cube_state_to_facelets(scramble: &str) -> String {
    let size = 3;
    let mut image = solved_image(size);
    let moves = parse_scramble(scramble, size);
    for m in &moves {
        for slice in m.slice_start..=m.slice_end {
            apply_slice(&mut image, size, m.face, slice, m.dir);
        }
    }
    // Convert to URFDLB string
    // Face order in string: U(1), R(0), F(2), D(4), L(3), B(5)
    let face_order = [1, 0, 2, 4, 3, 5]; // U R F D L B
    let face_chars = ['R', 'U', 'F', 'L', 'D', 'B']; // index -> char
    let mut s = String::with_capacity(54);
    for &fi in &face_order {
        for row in 0..size {
            for col in 0..size {
                s.push(face_chars[image[fi][row][col] as usize]);
            }
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solved_cube_facelets() {
        let facelets = cube_state_to_facelets("");
        assert_eq!(
            facelets,
            "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB"
        );
    }

    /// Cross-check our move application against the min2phase solver's
    /// `from_scramble`, which is proven correct by the two-phase solver.
    #[test]
    fn verify_r_against_min2phase() {
        let ours = cube_state_to_facelets("R");
        let theirs = tnoodle_min2phase::tools::from_scramble("R");
        assert_eq!(
            ours, theirs,
            "R move mismatch:\n  ours:   {ours}\n  theirs: {theirs}"
        );
    }

    #[test]
    fn verify_u_against_min2phase() {
        let ours = cube_state_to_facelets("U");
        let theirs = tnoodle_min2phase::tools::from_scramble("U");
        assert_eq!(
            ours, theirs,
            "U move mismatch:\n  ours:   {ours}\n  theirs: {theirs}"
        );
    }

    #[test]
    fn verify_f_against_min2phase() {
        let ours = cube_state_to_facelets("F");
        let theirs = tnoodle_min2phase::tools::from_scramble("F");
        assert_eq!(
            ours, theirs,
            "F move mismatch:\n  ours:   {ours}\n  theirs: {theirs}"
        );
    }

    #[test]
    fn verify_d_against_min2phase() {
        let ours = cube_state_to_facelets("D");
        let theirs = tnoodle_min2phase::tools::from_scramble("D");
        assert_eq!(
            ours, theirs,
            "D move mismatch:\n  ours:   {ours}\n  theirs: {theirs}"
        );
    }

    #[test]
    fn verify_l_against_min2phase() {
        let ours = cube_state_to_facelets("L");
        let theirs = tnoodle_min2phase::tools::from_scramble("L");
        assert_eq!(
            ours, theirs,
            "L move mismatch:\n  ours:   {ours}\n  theirs: {theirs}"
        );
    }

    #[test]
    fn verify_b_against_min2phase() {
        let ours = cube_state_to_facelets("B");
        let theirs = tnoodle_min2phase::tools::from_scramble("B");
        assert_eq!(
            ours, theirs,
            "B move mismatch:\n  ours:   {ours}\n  theirs: {theirs}"
        );
    }

    #[test]
    fn verify_primes_against_min2phase() {
        for mv in ["R'", "U'", "F'", "D'", "L'", "B'"] {
            let ours = cube_state_to_facelets(mv);
            let theirs = tnoodle_min2phase::tools::from_scramble(mv);
            assert_eq!(
                ours, theirs,
                "{mv} mismatch:\n  ours:   {ours}\n  theirs: {theirs}"
            );
        }
    }

    #[test]
    fn verify_doubles_against_min2phase() {
        for mv in ["R2", "U2", "F2", "D2", "L2", "B2"] {
            let ours = cube_state_to_facelets(mv);
            let theirs = tnoodle_min2phase::tools::from_scramble(mv);
            assert_eq!(
                ours, theirs,
                "{mv} mismatch:\n  ours:   {ours}\n  theirs: {theirs}"
            );
        }
    }

    #[test]
    fn verify_sequence_r_u_against_min2phase() {
        let ours = cube_state_to_facelets("R U");
        let theirs = tnoodle_min2phase::tools::from_scramble("R U");
        assert_eq!(
            ours, theirs,
            "R U mismatch:\n  ours:   {ours}\n  theirs: {theirs}"
        );
    }

    #[test]
    fn verify_sequence_r_u_r_prime_against_min2phase() {
        let ours = cube_state_to_facelets("R U R'");
        let theirs = tnoodle_min2phase::tools::from_scramble("R U R'");
        assert_eq!(
            ours, theirs,
            "R U R' mismatch:\n  ours:   {ours}\n  theirs: {theirs}"
        );
    }

    #[test]
    fn verify_sexy_move_against_min2phase() {
        let ours = cube_state_to_facelets("R U R' U'");
        let theirs = tnoodle_min2phase::tools::from_scramble("R U R' U'");
        assert_eq!(
            ours, theirs,
            "sexy move mismatch:\n  ours:   {ours}\n  theirs: {theirs}"
        );
    }

    #[test]
    fn verify_long_scramble_against_min2phase() {
        let scramble = "R U R' F2 D B' L U2 R D' F L2 B2 U R2 D2 L' F B";
        let ours = cube_state_to_facelets(scramble);
        let theirs = tnoodle_min2phase::tools::from_scramble(scramble);
        assert_eq!(
            ours, theirs,
            "long scramble mismatch:\n  ours:   {ours}\n  theirs: {theirs}"
        );
    }

    #[test]
    fn verify_t_perm_against_min2phase() {
        // T-perm: a well-known algorithm
        let scramble = "R U R' U' R' F R2 U' R' U' R U R' F'";
        let ours = cube_state_to_facelets(scramble);
        let theirs = tnoodle_min2phase::tools::from_scramble(scramble);
        assert_eq!(
            ours, theirs,
            "T-perm mismatch:\n  ours:   {ours}\n  theirs: {theirs}"
        );
    }

    #[test]
    fn four_r_moves_return_to_solved() {
        let facelets = cube_state_to_facelets("R R R R");
        assert_eq!(
            facelets, "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB",
            "4x R should return to solved"
        );
    }

    #[test]
    fn solved_cube_svg_contains_correct_colors() {
        let svg = render_cube_svg(3, "");
        assert!(svg.contains("#ec0000"), "missing red (R face)");
        assert!(svg.contains("#ffffff"), "missing white (U face)");
        assert!(svg.contains("#02d040"), "missing green (F face)");
        assert!(svg.contains("#ff8b24"), "missing orange (L face)");
        assert!(svg.contains("#fdd835"), "missing yellow (D face)");
        assert!(svg.contains("#304ffe"), "missing blue (B face)");
    }

    #[test]
    fn cube_after_r_move_has_changed_colors() {
        let solved_svg = render_cube_svg(3, "");
        let scrambled_svg = render_cube_svg(3, "R");
        // After an R move the SVG must differ from solved
        assert_ne!(
            solved_svg, scrambled_svg,
            "R move should change the cube state"
        );
        // The R face should still have red on it (R face rotates but stays red)
        assert!(
            scrambled_svg.contains("#ec0000"),
            "R face should still show red"
        );
    }

    #[test]
    fn svg_output_is_valid_xml_envelope() {
        let svg = render_cube_svg(3, "R U R' F2");
        let trimmed = svg.trim();
        assert!(
            trimmed.starts_with("<svg"),
            "SVG should start with <svg, got: {}",
            &trimmed[..40.min(trimmed.len())]
        );
        assert!(trimmed.ends_with("</svg>"), "SVG should end with </svg>");
    }

    #[test]
    fn different_sizes_produce_different_dimensions() {
        let svg2 = render_cube_svg(2, "");
        let svg3 = render_cube_svg(3, "");
        let svg5 = render_cube_svg(5, "");
        // Extract width from the SVG header
        fn extract_width(svg: &str) -> Option<&str> {
            let start = svg.find("width=\"")? + 7;
            let end = start + svg[start..].find('"')?;
            Some(&svg[start..end])
        }
        let w2 = extract_width(&svg2).unwrap();
        let w3 = extract_width(&svg3).unwrap();
        let w5 = extract_width(&svg5).unwrap();
        assert_ne!(w2, w3, "2x2 and 3x3 should have different widths");
        assert_ne!(w3, w5, "3x3 and 5x5 should have different widths");
    }

    #[test]
    fn wide_and_rotation_moves_parse() {
        // Should not panic
        let _svg = render_cube_svg(4, "Rw U2 3Rw' x y z");
        let _svg = render_cube_svg(5, "Rw Lw Uw Dw Fw Bw");
    }

    // ---------------------------------------------------------------
    // 4x4+ identity tests
    // ---------------------------------------------------------------

    /// Helper: get the image state for any NxN cube after a scramble.
    fn cube_image(size: usize, scramble: &str) -> CubeImage {
        let mut image = solved_image(size);
        let moves = parse_scramble(scramble, size);
        for m in &moves {
            for slice in m.slice_start..=m.slice_end {
                apply_slice(&mut image, size, m.face, slice, m.dir);
            }
        }
        image
    }

    fn is_solved(image: &CubeImage, size: usize) -> bool {
        for f in 0..6 {
            for r in 0..size {
                for c in 0..size {
                    if image[f][r][c] != f as u8 {
                        return false;
                    }
                }
            }
        }
        true
    }

    #[test]
    fn four_of_any_move_returns_to_solved_4x4() {
        for mv in [
            "R", "U", "F", "D", "L", "B", "Rw", "Uw", "Fw", "Dw", "Lw", "Bw",
        ] {
            let scramble = format!("{mv} {mv} {mv} {mv}");
            let img = cube_image(4, &scramble);
            assert!(is_solved(&img, 4), "4x {mv} on 4x4 should return to solved");
        }
    }

    #[test]
    fn four_of_any_move_returns_to_solved_5x5() {
        for mv in ["R", "U", "F", "D", "L", "B", "Rw", "Uw", "3Rw"] {
            let scramble = format!("{mv} {mv} {mv} {mv}");
            let img = cube_image(5, &scramble);
            assert!(is_solved(&img, 5), "4x {mv} on 5x5 should return to solved");
        }
    }

    #[test]
    fn move_and_inverse_cancel_4x4() {
        for (mv, inv) in [
            ("R", "R'"),
            ("U", "U'"),
            ("F", "F'"),
            ("D", "D'"),
            ("L", "L'"),
            ("B", "B'"),
            ("Rw", "Rw'"),
            ("Uw", "Uw'"),
            ("Fw", "Fw'"),
        ] {
            let scramble = format!("{mv} {inv}");
            let img = cube_image(4, &scramble);
            assert!(
                is_solved(&img, 4),
                "{mv} {inv} on 4x4 should cancel to solved"
            );
        }
    }

    #[test]
    fn double_move_equals_two_singles_4x4() {
        for mv in ["R", "U", "F", "Rw", "Uw"] {
            let two_singles = cube_image(4, &format!("{mv} {mv}"));
            let one_double = cube_image(4, &format!("{mv}2"));
            assert_eq!(
                two_singles, one_double,
                "{mv} {mv} should equal {mv}2 on 4x4"
            );
        }
    }

    #[test]
    fn size_clamped_to_valid_range() {
        // size=1 should be clamped to 2, size=100 to 7
        let svg_low = render_cube_svg(1, "");
        let svg_high = render_cube_svg(100, "");
        let svg2 = render_cube_svg(2, "");
        let svg7 = render_cube_svg(7, "");
        assert_eq!(svg_low, svg2);
        assert_eq!(svg_high, svg7);
    }
}
