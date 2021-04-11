#![allow(non_snake_case)]

fn init_array<const NP: usize, const NQ: usize, const NR: usize>(
    nr: usize,
    nq: usize,
    np: usize,
    A: &mut [[[f32; NP]; NQ]; NR],
    C4: &mut [[f32; NP]; NP],
) {
    for i in 0..nr {
        for j in 0..nq {
            for k in 0..np {
                A[i][j][k] = ((i * j + k) % np) as f32 / np as f32;
            }
        }
    }
    for i in 0..np {
        for j in 0..np {
            C4[i][j] = (i * j % np) as f32 / np as f32;
        }
    }
}

fn kernel_doitgen<const NP: usize, const NQ: usize, const NR: usize>(
    nr: usize,
    nq: usize,
    np: usize,
    A: &mut [[[f32; NP]; NQ]; NR],
    C4: &[[f32; NP]; NP],
    sum: &mut [f32; NP],
) {
    for r in 0..nr {
        for q in 0..nq {
            for p in 0..np {
                sum[p] = 0.0;
                for s in 0..np {
                    sum[p] += A[r][q][s] * C4[s][p];
                }
            }
            for p in 0..np {
                A[r][q][p] = sum[p];
            }
        }
    }
}

pub fn bench() {
    const NR: usize = 10;
    const NQ: usize = 10;
    const NP: usize = 10;

    let mut A = [[[0_f32; NP]; NQ]; NR];
    let mut sum = [0_f32; NP];
    let mut C4 = [[0_f32; NP]; NP];

    init_array(NR, NQ, NP, &mut A, &mut C4);
    kernel_doitgen(NR, NQ, NP, &mut A, &C4, &mut sum);

    super::utils::consume(A);
}
