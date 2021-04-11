#![allow(non_snake_case)]

fn init_array<
    const NI: usize,
    const NJ: usize,
    const NK: usize,
    const NL: usize,
    const NM: usize,
>(
    ni: usize,
    nj: usize,
    nk: usize,
    nl: usize,
    nm: usize,
    A: &mut [[f32; NK]; NI],
    B: &mut [[f32; NK]; NJ],
    C: &mut [[f32; NM]; NJ],
    D: &mut [[f32; NL]; NM],
) {
    for i in 0..ni {
        for j in 0..nk {
            A[i][j] = ((i * j + 1) % ni) as f32 / (5 * ni) as f32;
        }
    }
    for i in 0..nk {
        for j in 0..nj {
            B[i][j] = ((i * (j + 1) + 2) % nj) as f32 / (5 * nj) as f32;
        }
    }
    for i in 0..nj {
        for j in 0..nm {
            C[i][j] = (i * (j + 3) % nl) as f32 / (5 * nl) as f32;
        }
    }
    for i in 0..nm {
        for j in 0..nl {
            D[i][j] = ((i * (j + 2) + 2) % nk) as f32 / (5 * nk) as f32;
        }
    }
}

fn kernel_3mm<
    const NI: usize,
    const NJ: usize,
    const NK: usize,
    const NL: usize,
    const NM: usize,
>(
    ni: usize,
    nj: usize,
    nk: usize,
    nl: usize,
    nm: usize,
    E: &mut [[f32; NK]; NI],
    A: &[[f32; NK]; NI],
    B: &[[f32; NK]; NJ],
    F: &mut [[f32; NL]; NJ],
    C: &[[f32; NM]; NJ],
    D: &[[f32; NL]; NM],
    G: &mut [[f32; NL]; NI],
) {
    for i in 0..ni {
        for j in 0..nj {
            E[i][j] = 0.0;
            for k in 0..nk {
                E[i][j] += A[i][k] * B[k][j];
            }
        }
    }
    for i in 0..nj {
        for j in 0..nl {
            F[i][j] = 0.0;
            for k in 0..nm {
                F[i][j] += C[i][k] * D[k][j];
            }
        }
    }
    for i in 0..ni {
        for j in 0..nl {
            G[i][j] = 0.0;
            for k in 0..nj {
                G[i][j] += E[i][k] * F[k][j];
            }
        }
    }
}

pub fn bench() {
    const NI: usize = 10;
    const NJ: usize = 10;
    const NK: usize = 10;
    const NL: usize = 10;
    const NM: usize = 10;

    let mut E = [[0_f32; NJ]; NI];
    let mut A = [[0_f32; NK]; NI];
    let mut B = [[0_f32; NJ]; NK];
    let mut F = [[0_f32; NL]; NJ];
    let mut C = [[0_f32; NM]; NJ];
    let mut D = [[0_f32; NL]; NM];
    let mut G = [[0_f32; NL]; NI];

    init_array(NI, NJ, NK, NL, NM, &mut A, &mut B, &mut C, &mut D);
    kernel_3mm(NI, NJ, NK, NL, NM, &mut E, &A, &B, &mut F, &C, &D, &mut G);

    super::utils::consume(G);
}
