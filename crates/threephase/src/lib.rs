//! Port of cs.threephase, the 4x4x4 random-state solver tnoodle-lib uses.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::cast_lossless,
    reason = "mirrors Java's int index arithmetic; every index is bounded by its table, and the Java cross-check proves the results identical"
)]
#![allow(
    clippy::similar_names,
    reason = "keeps Java's variable names for side-by-side reading"
)]

pub mod center1;
pub mod center2;
pub mod center3;
pub mod center_cube;
pub mod corner_cube;
pub mod edge3;
pub mod edge_cube;
pub mod full_cube;
pub mod moves;
pub mod search;
pub mod tools;
pub mod util;
