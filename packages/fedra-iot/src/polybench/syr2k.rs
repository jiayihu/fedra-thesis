#![allow(non_snake_case)]

fn init_array<const M: usize, const N: usize>(
    m: usize,
    n: usize,
    alpha: &mut f32,
    beta: &mut f32,
    C: &mut [[f32; M]; M],
    A: &mut [[f32; N]; M],
    B: &mut [[f32; N]; M],
) {
    *alpha = 1.5;
    *beta = 1.2;
    for i in 0..m {
        for j in 0..n {
            A[i][j] = ((i * j + 1) % m) as f32 / m as f32;
            B[i][j] = ((i * j + 2) % n) as f32 / n as f32;
        }
    }

    for i in 0..m {
        for j in 0..m {
            C[i][j] = ((i * j + 3) % m) as f32 / n as f32;
        }
    }
}

fn kernel_syr2k<const M: usize, const N: usize>(
    m: usize,
    n: usize,
    alpha: f32,
    beta: f32,
    C: &mut [[f32; M]; M],
    A: &[[f32; N]; M],
    B: &[[f32; N]; M],
) {
    for i in 0..m {
        for j in 0..=i {
            C[i][j] *= beta;
        }
        for k in 0..n {
            for j in 0..=i {
                C[i][j] += A[j][k] * alpha * B[i][k] + B[j][k] * alpha * A[i][k];
            }
        }
    }
}

pub fn bench() {
    const M: usize = 10;
    const N: usize = 10;

    let mut alpha = 0.0;
    let mut beta = 0.0;
    let mut C = [[0_f32; M]; M];
    let mut A = [[0_f32; N]; M];
    let mut B = [[0_f32; N]; M];

    init_array(M, N, &mut alpha, &mut beta, &mut C, &mut A, &mut B);
    kernel_syr2k(M, N, alpha, beta, &mut C, &A, &B);

    super::utils::consume(C);
}
