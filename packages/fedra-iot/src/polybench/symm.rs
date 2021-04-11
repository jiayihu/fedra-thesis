#![allow(non_snake_case)]

fn init_array<const M: usize, const N: usize>(
    m: usize,
    n: usize,
    alpha: &mut f32,
    beta: &mut f32,
    C: &mut [[f32; N]; M],
    A: &mut [[f32; M]; M],
    B: &mut [[f32; N]; M],
) {
    *alpha = 1.5;
    *beta = 1.2;
    for i in 0..m {
        for j in 0..n {
            C[i][j] = ((i + j) % 100) as f32 / m as f32;
            B[i][j] = ((n + i - j) % 100) as f32 / m as f32;
        }
    }

    for i in 0..m {
        for j in 0..=i {
            A[i][j] = ((i + j) % 100) as f32 / m as f32;
        }
        for j in (i + 1)..m {
            A[i][j] = -999 as f32;
        }
    }
}

fn kernel_symm<const M: usize, const N: usize>(
    m: usize,
    n: usize,
    alpha: f32,
    beta: f32,
    C: &mut [[f32; N]; M],
    A: &[[f32; M]; M],
    B: &[[f32; N]; M],
) {
    for i in 0..m {
        for j in 0..n {
            let mut temp2 = 0.0;
            for k in 0..i {
                C[k][j] += alpha * B[i][j] * A[i][k];
                temp2 += B[k][j] * A[i][k];
            }
            C[i][j] = beta * C[i][j] + alpha * B[i][j] * A[i][i] + alpha * temp2;
        }
    }
}

pub fn bench() {
    const M: usize = 10;
    const N: usize = 10;

    let mut alpha = 0.0;
    let mut beta = 0.0;
    let mut C = [[0_f32; N]; M];
    let mut A = [[0_f32; M]; M];
    let mut B = [[0_f32; N]; M];

    init_array(M, N, &mut alpha, &mut beta, &mut C, &mut A, &mut B);
    kernel_symm(M, N, alpha, beta, &mut C, &A, &B);

    super::utils::consume(C);
}
