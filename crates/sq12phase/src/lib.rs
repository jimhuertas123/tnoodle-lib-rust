//! Port of cs.sq12phase, the Square-1 solver tnoodle-lib uses.
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

pub mod full_cube;
pub mod search;
pub mod shape;
pub mod square;
