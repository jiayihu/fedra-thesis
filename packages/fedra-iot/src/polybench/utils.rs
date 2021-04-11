#![allow(non_snake_case)]

#[allow(dead_code)]
pub fn make_positive_semi_definite<const N: usize>(A: &mut [[f32; N]; N]) {
    let mut b = [[0_f32; N]; N];
    let n = N;

    for t in 0..n {
        for r in 0..n {
            for s in 0..n {
                b[r][s] += A[r][t] * A[s][t];
            }
        }
    }
    for r in 0..n {
        for s in 0..n {
            A[r][s] = b[r][s];
        }
    }
}

#[cfg(not(asm))]
#[inline(never)]
/// Mark a value as used to prevent it from being optimized away.
pub fn consume<T>(x: T) -> T {
    use core::mem;
    use core::ptr;

    unsafe {
        // Use a more expensive method when inline asm is unavailable.
        let ret = ptr::read_volatile(&x);
        mem::forget(x);
        ret
    }
}
