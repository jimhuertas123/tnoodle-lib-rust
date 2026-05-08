#![allow(clippy::all)]
//! Port of cs.threephase.Util

/// Binomial coefficients C(n, k) for 0 <= n,k < 25.
/// Computed at compile time.
pub const CNK: [[i32; 25]; 25] = {
    let mut table = [[0i32; 25]; 25];
    let mut i = 0;
    while i < 25 {
        table[i][i] = 1;
        table[i][0] = 1;
        i += 1;
    }
    i = 1;
    while i < 25 {
        let mut j = 1;
        while j <= i {
            table[i][j] = table[i - 1][j] + table[i - 1][j - 1];
            j += 1;
        }
        i += 1;
    }
    table
};

/// Factorial lookup for 0..=12.
/// Computed at compile time.
pub const FACT: [i32; 13] = {
    let mut f = [0i32; 13];
    f[0] = 1;
    let mut i = 0;
    while i < 12 {
        f[i + 1] = f[i] * (i as i32 + 1);
        i += 1;
    }
    f
};

/// Cyclic permutation of four elements in an `i32` slice.
///
/// `key`:
///   - 0: d -> c -> b -> a -> (d's old value)  (i.e. cycle a<-b<-c<-d)
///   - 1: swap (a,c) and (b,d)
///   - 2: a -> b -> c -> d -> (a's old value)  (i.e. cycle a->b->c->d)
#[inline]
pub fn swap_i32(arr: &mut [i32], a: usize, b: usize, c: usize, d: usize, key: i32) {
    match key {
        0 => {
            let temp = arr[d];
            arr[d] = arr[c];
            arr[c] = arr[b];
            arr[b] = arr[a];
            arr[a] = temp;
        }
        1 => {
            let temp = arr[a];
            arr[a] = arr[c];
            arr[c] = temp;
            let temp = arr[b];
            arr[b] = arr[d];
            arr[d] = temp;
        }
        2 => {
            let temp = arr[a];
            arr[a] = arr[b];
            arr[b] = arr[c];
            arr[c] = arr[d];
            arr[d] = temp;
        }
        _ => {}
    }
}

/// Cyclic permutation of four elements in a `u8` slice.
///
/// Same semantics as [`swap_i32`].
#[inline]
pub fn swap_u8(arr: &mut [u8], a: usize, b: usize, c: usize, d: usize, key: i32) {
    match key {
        0 => {
            let temp = arr[d];
            arr[d] = arr[c];
            arr[c] = arr[b];
            arr[b] = arr[a];
            arr[a] = temp;
        }
        1 => {
            let temp = arr[a];
            arr[a] = arr[c];
            arr[c] = temp;
            let temp = arr[b];
            arr[b] = arr[d];
            arr[d] = temp;
        }
        2 => {
            let temp = arr[a];
            arr[a] = arr[b];
            arr[b] = arr[c];
            arr[c] = arr[d];
            arr[d] = temp;
        }
        _ => {}
    }
}

/// Decode a permutation index into an 8-element permutation array.
///
/// The encoding uses a factorial number system with a packed BCD-like
/// value register (`0x76543210`).
pub fn set8_perm(arr: &mut [u8], mut idx: i32) {
    let mut val: i32 = 0x76543210;
    for i in 0..7 {
        let p = FACT[7 - i];
        let v = idx / p;
        idx -= v * p;
        let v = v << 2;
        arr[i] = ((val >> v) & 0xf) as u8;
        let m = (1 << v) - 1;
        val = (val & m) + ((val >> 4) & !m);
    }
    arr[7] = val as u8;
}

/// Compute the parity of a permutation (0 = even, 1 = odd).
pub fn parity(arr: &[u8]) -> i32 {
    let mut parity = 0;
    let len = arr.len();
    for i in 0..len {
        for j in i..len {
            if arr[i] > arr[j] {
                parity ^= 1;
            }
        }
    }
    parity
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cnk_basic() {
        assert_eq!(CNK[0][0], 1);
        assert_eq!(CNK[4][2], 6);
        assert_eq!(CNK[10][5], 252);
        assert_eq!(CNK[24][12], 2704156);
    }

    #[test]
    fn test_cnk_boundary() {
        for i in 0..25 {
            assert_eq!(CNK[i][0], 1);
            assert_eq!(CNK[i][i], 1);
        }
    }

    #[test]
    fn test_fact() {
        assert_eq!(FACT[0], 1);
        assert_eq!(FACT[1], 1);
        assert_eq!(FACT[5], 120);
        assert_eq!(FACT[10], 3628800);
        assert_eq!(FACT[12], 479001600);
    }

    #[test]
    fn test_swap_i32_key0() {
        // key 0: cycle a <- b <- c <- d
        let mut arr = [0, 1, 2, 3];
        swap_i32(&mut arr, 0, 1, 2, 3, 0);
        assert_eq!(arr, [3, 0, 1, 2]);
    }

    #[test]
    fn test_swap_i32_key1() {
        // key 1: swap (a,c) and (b,d)
        let mut arr = [0, 1, 2, 3];
        swap_i32(&mut arr, 0, 1, 2, 3, 1);
        assert_eq!(arr, [2, 3, 0, 1]);
    }

    #[test]
    fn test_swap_i32_key2() {
        // key 2: cycle a -> b -> c -> d
        let mut arr = [0, 1, 2, 3];
        swap_i32(&mut arr, 0, 1, 2, 3, 2);
        assert_eq!(arr, [1, 2, 3, 0]);
    }

    #[test]
    fn test_swap_u8_key0() {
        let mut arr: [u8; 4] = [10, 20, 30, 40];
        swap_u8(&mut arr, 0, 1, 2, 3, 0);
        assert_eq!(arr, [40, 10, 20, 30]);
    }

    #[test]
    fn test_set8_perm_identity() {
        // idx=0 should give the identity permutation [0,1,2,3,4,5,6,7]
        let mut arr = [0u8; 8];
        set8_perm(&mut arr, 0);
        assert_eq!(arr, [0, 1, 2, 3, 4, 5, 6, 7]);
    }

    #[test]
    fn test_set8_perm_last() {
        // idx = 8! - 1 = 40319 should give [7,6,5,4,3,2,1,0]
        let mut arr = [0u8; 8];
        set8_perm(&mut arr, 40319);
        assert_eq!(arr, [7, 6, 5, 4, 3, 2, 1, 0]);
    }

    #[test]
    fn test_parity_identity() {
        assert_eq!(parity(&[0, 1, 2, 3]), 0);
    }

    #[test]
    fn test_parity_single_swap() {
        // single transposition => odd parity
        assert_eq!(parity(&[1, 0, 2, 3]), 1);
    }

    #[test]
    fn test_parity_double_swap() {
        // two transpositions => even parity
        assert_eq!(parity(&[1, 0, 3, 2]), 0);
    }
}
