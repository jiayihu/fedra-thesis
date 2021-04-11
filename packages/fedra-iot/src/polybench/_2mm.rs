#![allow(non_snake_case)]

fn init_array<const NI: usize, const NJ: usize, const NK: usize, const NL: usize>(
    ni: usize,
    nj: usize,
    nk: usize,
    nl: usize,
    alpha: &mut f32,
    beta: &mut f32,
    A: &mut [[f32; NK]; NI],
    B: &mut [[f32; NK]; NJ],
    C: &mut [[f32; NL]; NJ],
    D: &mut [[f32; NL]; NI],
) {
    *alpha = 1.5;
    *beta = 1.2;
    for i in 0..ni {
        for j in 0..nk {
            A[i][j] = ((i * j + 1) % ni) as f32 / ni as f32;
        }
    }
    for i in 0..nk {
        for j in 0..nj {
            B[i][j] = (i * (j + 1) % nj) as f32 / nj as f32;
        }
    }
    for i in 0..nj {
        for j in 0..nl {
            C[i][j] = ((i * (j + 3) + 1) % nl) as f32 / nl as f32;
        }
    }
    for i in 0..ni {
        for j in 0..nl {
            D[i][j] = (i * (j + 2) % nk) as f32 / nk as f32;
        }
    }
}

fn kernel_2mm<const NI: usize, const NJ: usize, const NK: usize, const NL: usize>(
    ni: usize,
    nj: usize,
    nk: usize,
    nl: usize,
    alpha: f32,
    beta: f32,
    tmp: &mut [[f32; NJ]; NI],
    A: &[[f32; NK]; NI],
    B: &[[f32; NK]; NJ],
    C: &[[f32; NL]; NJ],
    D: &mut [[f32; NL]; NI],
) {
    for i in 0..ni {
        for j in 0..nj {
            tmp[i][j] = 0.0;
            for k in 0..nk {
                tmp[i][j] += alpha * A[i][k] * B[k][j];
            }
        }
    }
    for i in 0..ni {
        for j in 0..nl {
            D[i][j] *= beta;
            for k in 0..nj {
                D[i][j] += tmp[i][k] * C[k][j];
            }
        }
    }
}

pub fn bench() {
    const NI: usize = 10;
    const NJ: usize = 10;
    const NK: usize = 10;
    const NL: usize = 10;

    let mut alpha = 0.0;
    let mut beta = 0.0;
    let mut tmp = [[0_f32; NJ]; NI];
    let mut A = [[0_f32; NK]; NI];
    let mut B = [[0_f32; NJ]; NK];
    let mut C = [[0_f32; NL]; NJ];
    let mut D = [[0_f32; NL]; NI];

    init_array(
        NI, NJ, NK, NL, &mut alpha, &mut beta, &mut A, &mut B, &mut C, &mut D,
    );
    kernel_2mm(NI, NJ, NK, NL, alpha, beta, &mut tmp, &A, &B, &C, &mut D);

    super::utils::consume(D);
}
