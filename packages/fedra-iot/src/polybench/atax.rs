#![allow(non_snake_case)]

fn init_array<const M: usize, const N: usize>(
    m: usize,
    n: usize,
    A: &mut [[f32; N]; M],
    x: &mut [f32; N],
) {
    let float_n = n as f32;
    for i in 0..n {
        x[i] = 1.0 + (i as f32 / float_n);
    }
    for i in 0..m {
        for j in 0..n {
            A[i][j] = ((i + j) % n) as f32 / (5 * m) as f32;
        }
    }
}

fn kernel_atax<const M: usize, const N: usize>(
    m: usize,
    n: usize,
    A: &[[f32; N]; M],
    x: &[f32; N],
    y: &mut [f32; N],
    tmp: &mut [f32; M],
) {
    for i in 0..n {
        y[i] = 0.0;
    }
    for i in 0..m {
        tmp[i] = 0.0;
        for j in 0..n {
            tmp[i] = tmp[i] + A[i][j] * x[j];
        }
        for j in 0..n {
            y[j] = y[j] + A[i][j] * tmp[i];
        }
    }
}

pub fn bench() {
    const M: usize = 10;
    const N: usize = 10;

    let mut A = [[0_f32; N]; M];
    let mut x = [0_f32; N];
    let mut y = [0_f32; N];
    let mut tmp = [0_f32; M];

    init_array(M, N, &mut A, &mut x);
    kernel_atax(M, N, &A, &x, &mut y, &mut tmp);

    super::utils::consume(y);
}
