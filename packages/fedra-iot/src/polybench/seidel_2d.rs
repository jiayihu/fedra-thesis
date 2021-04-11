#![allow(non_snake_case)]

fn init_array<const N: usize>(n: usize, A: &mut [[f32; N]; N]) {
    for i in 0..n {
        for j in 0..n {
            A[i][j] = (i * (j + 2) + 2) as f32 / n as f32;
        }
    }
}

fn kernel_seidel_2d<const N: usize>(tsteps: usize, n: usize, A: &mut [[f32; N]; N]) {
    for _ in 0..tsteps {
        for i in 1..(n - 1) {
            for j in 1..(n - 1) {
                A[i][j] = (A[i - 1][j - 1]
                    + A[i - 1][j]
                    + A[i - 1][j + 1]
                    + A[i][j - 1]
                    + A[i][j]
                    + A[i][j + 1]
                    + A[i + 1][j - 1]
                    + A[i + 1][j]
                    + A[i + 1][j + 1])
                    / 9.0;
            }
        }
    }
}

pub fn bench() {
    const N: usize = 10;
    const TSTEPS: usize = 5;

    let mut A = [[0_f32; N]; N];

    init_array(N, &mut A);
    kernel_seidel_2d(TSTEPS, N, &mut A);

    super::utils::consume(A);
}
