//! Port of Sun's `sun.security.provider.SecureRandom` (`SHA1PRNG`) and of the
//! `java.util.Random` methods tnoodle-lib calls on it.

use sha1::{Digest, Sha1};

const DIGEST_SIZE: usize = 20;

/// The error `next_int_bounded` returns where Java throws `IllegalArgumentException`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidBound(pub i32);

impl std::fmt::Display for InvalidBound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "bound must be positive, got {}", self.0)
    }
}

impl std::error::Error for InvalidBound {}

/// `SecureRandom.getInstance("SHA1PRNG", "SUN")` with the `java.util.Random` methods.
///
/// Built with [`JavaRandom::from_seed`] it returns exactly the values Java returns after
/// `setSeed(seed)` on a fresh instance.
#[derive(Clone)]
pub struct JavaRandom {
    state: [u8; DIGEST_SIZE],
    remainder: [u8; DIGEST_SIZE],
    rem_count: usize,
}

impl std::fmt::Debug for JavaRandom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JavaRandom").finish_non_exhaustive()
    }
}

impl JavaRandom {
    /// A generator seeded like a fresh Java instance after `setSeed(seed)`.
    ///
    /// Port of `SecureRandom.engineSetSeed` on an unseeded `SHA1PRNG`.
    #[must_use]
    pub fn from_seed(seed: &[u8]) -> Self {
        Self {
            state: Sha1::digest(seed).into(),
            remainder: [0; DIGEST_SIZE],
            rem_count: 0,
        }
    }

    /// A generator seeded from operating-system entropy, like Java's self-seeding instance.
    #[must_use]
    pub fn from_entropy() -> Self {
        Self::from_seed(&rand::random::<[u8; DIGEST_SIZE]>())
    }

    /// Port of `SecureRandom.engineNextBytes`.
    pub fn next_bytes(&mut self, result: &mut [u8]) {
        let mut index = 0;
        let r = self.rem_count;
        if r > 0 {
            let todo = (result.len() - index).min(DIGEST_SIZE - r);
            for (i, byte) in result.iter_mut().take(todo).enumerate() {
                *byte = self.remainder[r + i];
                self.remainder[r + i] = 0;
            }
            self.rem_count += todo;
            index += todo;
        }
        while index < result.len() {
            let mut output: [u8; DIGEST_SIZE] = Sha1::digest(self.state).into();
            update_state(&mut self.state, &output);
            let todo = (result.len() - index).min(DIGEST_SIZE);
            for byte in output.iter_mut().take(todo) {
                result[index] = *byte;
                index += 1;
                *byte = 0;
            }
            self.rem_count += todo;
            self.remainder = output;
        }
        self.rem_count %= DIGEST_SIZE;
    }

    /// Port of `SecureRandom.next(int numBits)`.
    #[allow(
        clippy::cast_possible_wrap,
        reason = "Java reinterprets the 32 bits as an int, so 32-bit draws are negative half the time"
    )]
    fn next(&mut self, num_bits: u32) -> i32 {
        let num_bytes = num_bits.div_ceil(8);
        let mut bytes = [0u8; 4];
        let drawn = &mut bytes[..num_bytes as usize];
        self.next_bytes(drawn);
        let next = drawn.iter().fold(0u32, |acc, &b| (acc << 8) | u32::from(b));
        // Java's `>>>`: drop the extra low bits of the last byte.
        (next >> (num_bytes * 8 - num_bits)) as i32
    }

    /// Port of `Random.nextInt()`.
    pub fn next_int(&mut self) -> i32 {
        self.next(32)
    }

    /// Port of `Random.nextInt(int bound)`.
    ///
    /// # Errors
    ///
    /// [`InvalidBound`] when `bound <= 0`, where Java throws `IllegalArgumentException`.
    ///
    /// # Panics
    ///
    /// Never: the power-of-two branch yields a value below `bound`.
    pub fn next_int_bounded(&mut self, bound: i32) -> Result<i32, InvalidBound> {
        if bound <= 0 {
            return Err(InvalidBound(bound));
        }
        let mut r = self.next(31);
        let m = bound - 1;
        if bound & m == 0 {
            // Power of two: take the high bits.
            r = i32::try_from((i64::from(bound) * i64::from(r)) >> 31).expect("below bound");
        } else {
            // Reject values from the incomplete last band, as Java does with
            // `u - (r = u % bound) + m < 0` (an int overflow test).
            let mut u = r;
            loop {
                r = u % bound;
                if u.wrapping_sub(r).wrapping_add(m) >= 0 {
                    break;
                }
                u = self.next(31);
            }
        }
        Ok(r)
    }

    /// `Random.nextInt(bound)` for a bound known to be positive.
    ///
    /// # Panics
    ///
    /// If `bound <= 0`; callers pass a positive constant.
    pub fn next_index(&mut self, bound: usize) -> usize {
        let bound = i32::try_from(bound).expect("bound fits in an int");
        usize::try_from(self.next_int_bounded(bound).expect("bound is positive"))
            .expect("non-negative")
    }

    /// Port of `Random.nextLong()`.
    pub fn next_long(&mut self) -> i64 {
        let high = i64::from(self.next(32));
        let low = i64::from(self.next(32));
        high.wrapping_shl(32).wrapping_add(low)
    }

    /// Port of `Random.nextBoolean()`.
    pub fn next_boolean(&mut self) -> bool {
        self.next(1) != 0
    }

    /// Port of `Random.nextDouble()`.
    #[allow(
        clippy::cast_precision_loss,
        reason = "the value has 53 bits, which f64 holds exactly, as in Java"
    )]
    pub fn next_double(&mut self) -> f64 {
        let high = i64::from(self.next(26));
        let low = i64::from(self.next(27));
        ((high << 27) + low) as f64 * f64::from(0.5f32).powi(53)
    }
}

/// Port of `SecureRandom.updateState`: `state += output + 1` as a little-endian
/// number of signed bytes, forcing a change when the sum leaves it unchanged.
fn update_state(state: &mut [u8; DIGEST_SIZE], output: &[u8; DIGEST_SIZE]) {
    let mut last: i32 = 1;
    let mut changed = false;
    for (s, &o) in state.iter_mut().zip(output) {
        // Java adds the bytes as signed values.
        let v = i32::from(i8::from_ne_bytes([*s])) + i32::from(i8::from_ne_bytes([o])) + last;
        let t = v.to_le_bytes()[0];
        changed |= *s != t;
        *s = t;
        last = v >> 8;
    }
    if !changed {
        state[0] = state[0].wrapping_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_positive_bound_is_an_error() {
        let mut r = JavaRandom::from_seed(b"abc");
        assert_eq!(r.next_int_bounded(0), Err(InvalidBound(0)));
        assert_eq!(r.next_int_bounded(-5), Err(InvalidBound(-5)));
        // The generator still works afterwards.
        assert!((0..10).contains(&r.next_int_bounded(10).unwrap()));
    }

    #[test]
    fn entropy_generators_differ() {
        let a = JavaRandom::from_entropy().next_long();
        let b = JavaRandom::from_entropy().next_long();
        assert_ne!(a, b);
    }

    #[test]
    fn same_seed_gives_same_stream() {
        let mut a = JavaRandom::from_seed(b"tnoodle");
        let mut b = JavaRandom::from_seed(b"tnoodle");
        for _ in 0..1000 {
            assert_eq!(a.next_int(), b.next_int());
        }
    }
}
