#![allow(non_snake_case)]

fn init_array<const N: usize>(n: usize, L: &mut [[f32; N]; N], x: &mut [f32; N], b: &mut [f32; N]) {
    for i in 0..n {
        x[i] = -999.0;
        b[i] = i as f32;
        for j in 0..=i {
            L[i][j] = (i + n - j + 1) as f32 * 2.0 / n as f32;
        }
    }
}

fn kernel_trisolv<const N: usize>(n: usize, L: &[[f32; N]; N], x: &mut [f32; N], b: &[f32; N]) {
    for i in 0..n {
        x[i] = b[i];
        for j in 0..i {
            x[i] -= L[i][j] * x[j];
        }
        x[i] = x[i] / L[i][i];
    }
}

pub fn bench() {
    const N: usize = 10;

    let mut L = [[0_f32; N]; N];
    let mut x = [0_f32; N];
    let mut b = [0_f32; N];

    init_array(N, &mut L, &mut x, &mut b);
    kernel_trisolv(N, &L, &mut x, &b);

    super::utils::consume(x);
}
