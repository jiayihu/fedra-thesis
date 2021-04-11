#![allow(non_snake_case)]

use core::intrinsics::sqrtf32;

fn init_array<const M: usize, const N: usize>(
    m: usize,
    n: usize,
    A: &mut [[f32; N]; M],
    R: &mut [[f32; N]; N],
    Q: &mut [[f32; N]; M],
) {
    for i in 0..m {
        for j in 0..n {
            A[i][j] = ((((i * j) % m) as f32 / m as f32) * 100.0) + 10.0;
            Q[i][j] = 0.0;
        }
    }
    for i in 0..n {
        for j in 0..n {
            R[i][j] = 0.0;
        }
    }
}

fn kernel_gramschmidt<const M: usize, const N: usize>(
    m: usize,
    n: usize,
    A: &mut [[f32; N]; M],
    R: &mut [[f32; N]; N],
    Q: &mut [[f32; N]; M],
) {
    for k in 0..n {
        let mut nrm = 0.0;
        for i in 0..m {
            nrm += A[i][k] * A[i][k];
        }
        R[k][k] = unsafe { sqrtf32(nrm) };
        for i in 0..m {
            Q[i][k] = A[i][k] / R[k][k];
        }
        for j in (k + 1)..n {
            R[k][j] = 0.0;
            for i in 0..m {
                R[k][j] += Q[i][k] * A[i][j];
            }
            for i in 0..m {
                A[i][j] = A[i][j] - Q[i][k] * R[k][j];
            }
        }
    }
}

pub fn bench() {
    const M: usize = 30;
    const N: usize = 35;

    let mut A = [[0_f32; N]; M];
    let mut R = [[0_f32; N]; N];
    let mut Q = [[0_f32; N]; M];

    init_array(M, N, &mut A, &mut R, &mut Q);
    kernel_gramschmidt(M, N, &mut A, &mut R, &mut Q);

    super::utils::consume(A);
}
