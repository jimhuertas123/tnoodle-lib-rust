use tnoodle_scrambles::puzzles::cube_svg::render_cube_svg;

fn main() {
    let scramble = "R U R' F2 D B' L U2";
    let svg = render_cube_svg(3, scramble);

    let path = "test_cube.svg";
    std::fs::write(path, &svg).unwrap();
    println!("Wrote {path} — open it in your browser or Finder");
    println!("Scramble: {scramble}");
}
