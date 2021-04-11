#![allow(non_snake_case)]

fn init_array<const M: usize, const N: usize>(
    m: usize,
    n: usize,
    alpha: &mut f32,
    A: &mut [[f32; M]; M],
    B: &mut [[f32; N]; M],
) {
    *alpha = 1.5;
    for i in 0..m {
        for j in 0..i {
            A[i][j] = ((i + j) % m) as f32 / m as f32;
        }
        A[i][i] = 1.0;
        for j in 0..n {
            B[i][j] = (((n + i) - j) % n) as f32 / n as f32;
        }
    }
}

fn kernel_trmm<const M: usize, const N: usize>(
    m: usize,
    n: usize,
    alpha: f32,
    A: &[[f32; M]; M],
    B: &mut [[f32; N]; M],
) {
    for i in 0..m {
        for j in 0..n {
            for k in (i + 1)..m {
                B[i][j] += A[k][i] * B[k][j];
            }
            B[i][j] = alpha * B[i][j];
        }
    }
}

pub fn bench() {
    const M: usize = 5;
    const N: usize = 6;

    let mut alpha = 0.0;
    let mut A = [[0_f32; M]; M];
    let mut B = [[0_f32; N]; M];

    init_array(M, N, &mut alpha, &mut A, &mut B);
    kernel_trmm(M, N, alpha, &A, &mut B);

    super::utils::consume(B);
}
