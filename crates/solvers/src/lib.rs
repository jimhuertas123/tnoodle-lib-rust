//! Port of tnoodle-lib's 2x2x2, Skewb and Pyraminx solvers.
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
#![allow(
    clippy::large_stack_arrays,
    reason = "the move and pruning tables are built once, as Java's arrays"
)]

pub mod pyraminx;
pub mod skewb;
pub mod two_by_two;
