#![allow(non_snake_case)]

fn init_array<const N: usize, const TSTEPS: usize>(
    n: usize,
    A: &mut [[f32; N]; N],
    B: &mut [[f32; N]; N],
) {
    for i in 0..n {
        for j in 0..n {
            A[i][j] = (i * (j + 2) + 2) as f32 / n as f32;
            B[i][j] = (i * (j + 3) + 3) as f32 / n as f32;
        }
    }
}

fn kernel_jacobi_2d<const N: usize, const TSTEPS: usize>(
    tsteps: usize,
    n: usize,
    A: &mut [[f32; N]; N],
    B: &mut [[f32; N]; N],
) {
    for _ in 0..tsteps {
        for i in 1..(n - 1) {
            for j in 1..(n - 1) {
                B[i][j] = 0.2 * (A[i][j] + A[i][j - 1] + A[i][1 + j] + A[1 + i][j] + A[i - 1][j]);
            }
        }
        for i in 1..(n - 1) {
            for j in 1..(n - 1) {
                A[i][j] = 0.2 * (B[i][j] + B[i][j - 1] + B[i][1 + j] + B[1 + i][j] + B[i - 1][j]);
            }
        }
    }
}

pub fn bench() {
    const N: usize = 10;
    const TSTEPS: usize = 10;

    let mut A = [[0_f32; N]; N];
    let mut B = [[0_f32; N]; N];

    init_array::<N, TSTEPS>(N, &mut A, &mut B);
    kernel_jacobi_2d::<N, TSTEPS>(TSTEPS, N, &mut A, &mut B);

    super::utils::consume(A);
    super::utils::consume(B);
}
