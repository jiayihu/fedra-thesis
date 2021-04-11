#![allow(non_snake_case)]

fn init_array<const NI: usize, const NJ: usize, const NK: usize>(
    ni: usize,
    nj: usize,
    nk: usize,
    alpha: &mut f32,
    beta: &mut f32,
    C: &mut [[f32; NJ]; NI],
    A: &mut [[f32; NK]; NI],
    B: &mut [[f32; NJ]; NK],
) {
    *alpha = 1.5;
    *beta = 1.2;
    for i in 0..ni {
        for j in 0..nj {
            C[i][j] = ((i * j + 1) % ni) as f32 / ni as f32;
        }
    }
    for i in 0..ni {
        for j in 0..nk {
            A[i][j] = (i * (j + 1) % nk) as f32 / nk as f32;
        }
    }
    for i in 0..nk {
        for j in 0..nj {
            B[i][j] = (i * (j + 2) % nj) as f32 / nj as f32;
        }
    }
}

fn kernel_gemm<const NI: usize, const NJ: usize, const NK: usize>(
    ni: usize,
    nj: usize,
    nk: usize,
    alpha: f32,
    beta: f32,
    C: &mut [[f32; NJ]; NI],
    A: &[[f32; NK]; NI],
    B: &[[f32; NJ]; NK],
) {
    for i in 0..ni {
        for j in 0..nj {
            C[i][j] *= beta;
            for k in 0..nk {
                C[i][j] += alpha * A[i][k] * B[k][j];
            }
        }
    }
}

pub fn bench() {
    const NI: usize = 10;
    const NJ: usize = 10;
    const NK: usize = 10;

    let mut alpha = 0.0;
    let mut beta = 0.0;
    let mut C = [[0_f32; NJ]; NI];
    let mut A = [[0_f32; NK]; NI];
    let mut B = [[0_f32; NJ]; NK];

    init_array(NI, NJ, NK, &mut alpha, &mut beta, &mut C, &mut A, &mut B);
    kernel_gemm(NI, NJ, NK, alpha, beta, &mut C, &A, &B);

    super::utils::consume(C);
}
