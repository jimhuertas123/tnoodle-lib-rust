//! Port of `cs.min2phase`, the two-phase 3x3x3 solver tnoodle-lib uses.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::cast_lossless,
    reason = "mirrors Java's int index arithmetic; every index is bounded by its table, and the Java cross-check proves the results identical"
)]
#![allow(
    clippy::similar_names,
    reason = "keeps Java's variable names (p2csym, p2esym, edgei, edgex) for side-by-side reading"
)]

pub mod coord_cube;
pub mod cubie_cube;
pub mod search;
pub mod tools;
pub mod util;
