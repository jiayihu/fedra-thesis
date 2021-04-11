#![allow(non_snake_case)]

fn init_array<const N: usize>(
    n: usize,
    x1: &mut [f32; N],
    x2: &mut [f32; N],
    y_1: &mut [f32; N],
    y_2: &mut [f32; N],
    A: &mut [[f32; N]; N],
) {
    for i in 0..n {
        x1[i] = (i % n) as f32 / n as f32;
        x2[i] = ((i + 1) % n) as f32 / n as f32;
        y_1[i] = ((i + 3) % n) as f32 / n as f32;
        y_2[i] = ((i + 4) % n) as f32 / n as f32;
        for j in 0..n {
            A[i][j] = (i * j % n) as f32 / n as f32;
        }
    }
}

fn kernel_mvt<const N: usize>(
    n: usize,
    x1: &mut [f32; N],
    x2: &mut [f32; N],
    y_1: &[f32; N],
    y_2: &[f32; N],
    A: &[[f32; N]; N],
) {
    for i in 0..n {
        for j in 0..n {
            x1[i] = x1[i] + A[i][j] * y_1[j];
        }
    }
    for i in 0..n {
        for j in 0..n {
            x2[i] = x2[i] + A[j][i] * y_2[j];
        }
    }
}

pub fn bench() {
    const N: usize = 10;

    let mut A = [[0_f32; N]; N];
    let mut x1 = [0_f32; N];
    let mut x2 = [0_f32; N];
    let mut y_1 = [0_f32; N];
    let mut y_2 = [0_f32; N];

    init_array(N, &mut x1, &mut x2, &mut y_1, &mut y_2, &mut A);
    kernel_mvt(N, &mut x1, &mut x2, &y_1, &y_2, &A);

    super::utils::consume(x2);
}
