#![allow(non_snake_case)]

fn init_array<const N: usize, const TSTEPS: usize>(
    n: usize,
    A: &mut [[[f32; N]; N]; N],
    B: &mut [[[f32; N]; N]; N],
) {
    for i in 0..n {
        for j in 0..n {
            for k in 0..n {
                B[i][j][k] = (i + j + (n - k)) as f32 * (10 as f32) / n as f32;
                A[i][j][k] = B[i][j][k];
            }
        }
    }
}

fn kernel_heat_3d<const N: usize, const TSTEPS: usize>(
    tsteps: usize,
    n: usize,
    A: &mut [[[f32; N]; N]; N],
    B: &mut [[[f32; N]; N]; N],
) {
    for _ in 1..tsteps {
        for i in 1..(n - 1) {
            for j in 1..(n - 1) {
                for k in 1..(n - 1) {
                    B[i][j][k] = 0.125 * (A[i + 1][j][k] - 2.0 * A[i][j][k] + A[i - 1][j][k])
                        + 0.125 * (A[i][j + 1][k] - 2.0 * A[i][j][k] + A[i][j - 1][k])
                        + 0.125 * (A[i][j][k + 1] - 2.0 * A[i][j][k] + A[i][j][k - 1])
                        + A[i][j][k];
                }
            }
        }
        for i in 1..(n - 1) {
            for j in 1..(n - 1) {
                for k in 1..(n - 1) {
                    A[i][j][k] = 0.125 * (B[i + 1][j][k] - 2.0 * B[i][j][k] + B[i - 1][j][k])
                        + 0.125 * (B[i][j + 1][k] - 2.0 * B[i][j][k] + B[i][j - 1][k])
                        + 0.125 * (B[i][j][k + 1] - 2.0 * B[i][j][k] + B[i][j][k - 1])
                        + B[i][j][k];
                }
            }
        }
    }
}

pub fn bench() {
    const N: usize = 10;
    const TSTEPS: usize = 10;

    let mut A = [[[0_f32; N]; N]; N];
    let mut B = [[[0_f32; N]; N]; N];

    init_array::<N, TSTEPS>(N, &mut A, &mut B);
    kernel_heat_3d::<N, TSTEPS>(TSTEPS, N, &mut A, &mut B);

    super::utils::consume(A);
}
