#![allow(non_snake_case)]

fn init_array<const M: usize, const N: usize>(
    m: usize,
    n: usize,
    float_n: &mut f32,
    data: &mut [[f32; N]; M],
) {
    *float_n = n as f32;
    for i in 0..m {
        for j in 0..n {
            data[i][j] = (i * j) as f32 / N as f32;
        }
    }
}

fn kernel_covariance<const M: usize, const N: usize>(
    m: usize,
    n: usize,
    float_n: f32,
    data: &mut [[f32; N]; M],
    cov: &mut [[f32; N]; N],
    mean: &mut [f32; N],
) {
    for j in 0..n {
        mean[j] = 0.0;
        for i in 0..m {
            mean[j] += data[i][j];
        }
        mean[j] /= float_n;
    }

    for i in 0..m {
        for j in 0..n {
            data[i][j] -= mean[j];
        }
    }

    for i in 0..n {
        for j in i..n {
            cov[i][j] = 0.0;
            for k in 0..m {
                cov[i][j] += data[k][i] * data[k][j];
            }
            cov[i][j] /= float_n - 1.0;
            cov[j][i] = cov[i][j];
        }
    }
}

pub fn bench() {
    const M: usize = 10;
    const N: usize = 10;

    let mut float_n = 0.0;
    let mut data = [[0_f32; N]; N];
    let mut cov = [[0_f32; N]; N];
    let mut mean = [0_f32; N];

    init_array(M, N, &mut float_n, &mut data);
    kernel_covariance(M, N, float_n, &mut data, &mut cov, &mut mean);

    super::utils::consume(cov);
}
