#![allow(non_snake_case)]

fn init_array<const M: usize, const N: usize>(
    m: usize,
    n: usize,
    alpha: &mut f32,
    beta: &mut f32,
    C: &mut [[f32; M]; M],
    A: &mut [[f32; N]; M],
) {
    *alpha = 1.5;
    *beta = 1.2;
    for i in 0..m {
        for j in 0..n {
            A[i][j] = ((i * j + 1) % m) as f32 / m as f32;
        }
    }

    for i in 0..m {
        for j in 0..m {
            C[i][j] = ((i * j + 2) % n) as f32 / n as f32;
        }
    }
}

fn kernel_syrk<const M: usize, const N: usize>(
    m: usize,
    n: usize,
    alpha: f32,
    beta: f32,
    C: &mut [[f32; M]; M],
    A: &[[f32; N]; M],
) {
    for i in 0..m {
        for j in 0..=i {
            C[i][j] *= beta;
        }
        for k in 0..n {
            for j in 0..=i {
                C[i][j] += alpha * A[i][k] * A[j][k];
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

    init_array(M, N, &mut alpha, &mut beta, &mut C, &mut A);
    kernel_syrk(M, N, alpha, beta, &mut C, &A);

    super::utils::consume(C);
}
