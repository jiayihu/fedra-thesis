#![allow(non_snake_case)]

fn init_array<const N: usize, const TSTEPS: usize>(n: usize, A: &mut [f32; N], B: &mut [f32; N]) {
    for i in 0..n {
        A[i] = (i + 2) as f32 / n as f32;
        B[i] = (i + 3) as f32 / n as f32;
    }
}

fn kernel_jacobi_1d<const N: usize, const TSTEPS: usize>(
    tsteps: usize,
    n: usize,
    A: &mut [f32; N],
    B: &mut [f32; N],
) {
    for _ in 0..tsteps {
        for i in 1..(n - 1) {
            B[i] = 0.33333 * (A[i - 1] + A[i] + A[i + 1]);
        }
        for i in 1..(n - 1) {
            A[i] = 0.33333 * (B[i - 1] + B[i] + B[i + 1]);
        }
    }
}

pub fn bench() {
    const N: usize = 10;
    const TSTEPS: usize = 10;

    let mut A = [0_f32; N];
    let mut B = [0_f32; N];

    init_array::<N, TSTEPS>(N, &mut A, &mut B);
    kernel_jacobi_1d::<N, TSTEPS>(TSTEPS, N, &mut A, &mut B);

    super::utils::consume(A);
    super::utils::consume(B);
}
