#![allow(non_snake_case)]

fn init_array<const N: usize>(n: usize, A: &mut [[f32; N]; N]) {
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

fn kernel_lu<const N: usize>(n: usize, A: &mut [[f32; N]; N]) {
    for i in 0..n {
        for j in 0..i {
            for k in 0..j {
                A[i][j] -= A[i][k] * A[k][j];
            }
            A[i][j] /= A[j][j];
        }
        for j in i..n {
            for k in 0..i {
                A[i][j] -= A[i][k] * A[k][j];
            }
        }
    }
}

pub fn bench() {
    const N: usize = 10;

    let mut A = [[0_f32; N]; N];

    init_array(N, &mut A);
    kernel_lu(N, &mut A);

    super::utils::consume(A);
}
