//! Java-identical randomness for tnoodle-rs.
//!
//! tnoodle-lib draws every random number from `SecureRandom.getInstance("SHA1PRNG", "SUN")`
//! through the `java.util.Random` API. [`JavaRandom`] reproduces both, so a seed gives the
//! same numbers, and therefore the same scrambles, as tnoodle-lib.

mod java_random;

pub use java_random::{InvalidBound, JavaRandom};
