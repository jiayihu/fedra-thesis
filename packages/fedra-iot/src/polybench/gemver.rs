#![allow(non_snake_case)]

fn init_array<const N: usize>(
    n: usize,
    alpha: &mut f32,
    beta: &mut f32,
    A: &mut [[f32; N]; N],
    u1: &mut [f32; N],
    v1: &mut [f32; N],
    u2: &mut [f32; N],
    v2: &mut [f32; N],
    w: &mut [f32; N],
    x: &mut [f32; N],
    y: &mut [f32; N],
    z: &mut [f32; N],
) {
    *alpha = 1.5;
    *beta = 1.2;

    let float_n = n as f32;

    for i in 0..n {
        u1[i] = i as f32;
        u2[i] = ((i + 1) as f32 / float_n) / 2.0;
        v1[i] = ((i + 1) as f32 / float_n) / 4.0;
        v2[i] = ((i + 1) as f32 / float_n) / 6.0;
        y[i] = ((i + 1) as f32 / float_n) / 8.0;
        z[i] = ((i + 1) as f32 / float_n) / 9.0;
        x[i] = 0.0;
        w[i] = 0.0;
        for j in 0..n {
            A[i][j] = (i * j % n) as f32 / n as f32;
        }
    }
}

fn kernel_gemver<const N: usize>(
    n: usize,
    alpha: f32,
    beta: f32,
    A: &mut [[f32; N]; N],
    u1: &[f32; N],
    v1: &[f32; N],
    u2: &[f32; N],
    v2: &[f32; N],
    w: &mut [f32; N],
    x: &mut [f32; N],
    y: &[f32; N],
    z: &[f32; N],
) {
    for i in 0..n {
        for j in 0..n {
            A[i][j] = A[i][j] + u1[i] * v1[j] + u2[i] * v2[j];
        }
    }

    for i in 0..n {
        for j in 0..n {
            x[i] = x[i] + beta * A[j][i] * y[j];
        }
    }

    for i in 0..n {
        x[i] = x[i] + z[i];
    }

    for i in 0..n {
        for j in 0..n {
            w[i] = w[i] + alpha * A[i][j] * x[j];
        }
    }
}

pub fn bench() {
    const N: usize = 10;

    let mut alpha = 0.0;
    let mut beta = 0.0;
    let mut A = [[0_f32; N]; N];
    let mut u1 = [0_f32; N];
    let mut v1 = [0_f32; N];
    let mut u2 = [0_f32; N];
    let mut v2 = [0_f32; N];
    let mut w = [0_f32; N];
    let mut x = [0_f32; N];
    let mut y = [0_f32; N];
    let mut z = [0_f32; N];

    init_array(
        N, &mut alpha, &mut beta, &mut A, &mut u1, &mut v1, &mut u2, &mut v2, &mut w, &mut x,
        &mut y, &mut z,
    );

    kernel_gemver(
        N, alpha, beta, &mut A, &u1, &v1, &u2, &v2, &mut w, &mut x, &y, &z,
    );

    super::utils::consume(w);
}
