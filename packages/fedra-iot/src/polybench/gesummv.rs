#![allow(non_snake_case)]

fn init_array<const N: usize>(
    n: usize,
    alpha: &mut f32,
    beta: &mut f32,
    A: &mut [[f32; N]; N],
    B: &mut [[f32; N]; N],
    x: &mut [f32; N],
) {
    *alpha = 1.5;
    *beta = 1.2;
    for i in 0..n {
        x[i] = (i % n) as f32 / n as f32;
        for j in 0..n {
            A[i][j] = ((i * j + 1) % n) as f32 / n as f32;
            B[i][j] = ((i * j + 2) % n) as f32 / n as f32;
        }
    }
}

fn kernel_gesummv<const N: usize>(
    n: usize,
    alpha: f32,
    beta: f32,
    A: &[[f32; N]; N],
    B: &[[f32; N]; N],
    tmp: &mut [f32; N],
    x: &[f32; N],
    y: &mut [f32; N],
) {
    for i in 0..n {
        tmp[i] = 0.0;
        y[i] = 0.0;
        for j in 0..n {
            tmp[i] = A[i][j] * x[j] + tmp[i];
            y[i] = B[i][j] * x[j] + y[i];
        }
        y[i] = alpha * tmp[i] + beta * y[i];
    }
}

pub fn bench() {
    const N: usize = 10;

    let mut alpha = 0.0;
    let mut beta = 0.0;
    let mut A = [[0_f32; N]; N];
    let mut B = [[0_f32; N]; N];
    let mut tmp = [0_f32; N];
    let mut x = [0_f32; N];
    let mut y = [0_f32; N];

    init_array(N, &mut alpha, &mut beta, &mut A, &mut B, &mut x);
    kernel_gesummv(N, alpha, beta, &A, &B, &mut tmp, &x, &mut y);

    super::utils::consume(y);
}
