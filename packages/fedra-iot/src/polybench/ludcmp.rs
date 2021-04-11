#![allow(non_snake_case)]

fn init_array<const N: usize>(
    n: usize,
    A: &mut [[f32; N]; N],
    b: &mut [f32; N],
    x: &mut [f32; N],
    y: &mut [f32; N],
) {
    let float_n = n as f32;

    for i in 0..n {
        x[i] = 0.0;
        y[i] = 0.0;
        b[i] = (i + 1) as f32 / float_n / 2.0 + 4.0;
    }

    for i in 0..n {
        for j in 0..=i {
            A[i][j] = (-(j as isize) % n as isize) as f32 / n as f32 + 1.0;
        }
        for j in (i + 1)..n {
            A[i][j] = 0.0;
        }
        A[i][i] = 1.0;
    }

    super::utils::make_positive_semi_definite(A);
}

fn kernel_ludcmp<const N: usize>(
    n: usize,
    A: &mut [[f32; N]; N],
    b: &[f32; N],
    x: &mut [f32; N],
    y: &mut [f32; N],
) {
    let mut w;
    for i in 0..n {
        for j in 0..i {
            w = A[i][j];
            for k in 0..j {
                w -= A[i][k] * A[k][j];
            }
            A[i][j] = w / A[j][j];
        }
        for j in i..n {
            w = A[i][j];
            for k in 0..i {
                w -= A[i][k] * A[k][j];
            }
            A[i][j] = w;
        }
    }

    for i in 0..n {
        w = b[i];
        for j in 0..i {
            w -= A[i][j] * y[j];
        }
        y[i] = w;
    }

    for i in (0..n).rev() {
        w = y[i];
        for j in (i + 1)..n {
            w -= A[i][j] * x[j];
        }
        x[i] = w / A[i][i];
    }
}

pub fn bench() {
    const N: usize = 10;

    let mut A = [[0_f32; N]; N];
    let mut b = [0_f32; N];
    let mut x = [0_f32; N];
    let mut y = [0_f32; N];

    init_array(N, &mut A, &mut b, &mut x, &mut y);
    kernel_ludcmp(N, &mut A, &b, &mut x, &mut y);

    super::utils::consume(x);
}
