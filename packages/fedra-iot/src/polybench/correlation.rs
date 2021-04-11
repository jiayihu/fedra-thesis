#![allow(non_snake_case)]

use core::intrinsics::sqrtf32;

pub fn init_array<const M: usize, const N: usize>(
    m: usize,
    n: usize,
    float_n: &mut f32,
    data: &mut [[f32; N]; M],
) {
    *float_n = n as f32;
    for i in 0..m {
        for j in 0..n {
            data[i][j] = (i * j) as f32 / (n + i) as f32;
        }
    }
}

pub fn kernel_correlation<const M: usize, const N: usize>(
    m: usize,
    n: usize,
    float_n: f32,
    data: &mut [[f32; N]; M],
    corr: &mut [[f32; N]; N],
    mean: &mut [f32; N],
    stddev: &mut [f32; N],
) {
    let eps = 0.1;

    for j in 0..n {
        mean[j] = 0.0;
        for i in 0..m {
            mean[j] += data[i][j];
        }
        mean[j] /= float_n;
    }

    for j in 0..n {
        stddev[j] = 0.0;
        for i in 0..m {
            stddev[j] += (data[i][j] - mean[j]) * (data[i][j] - mean[j]);
            stddev[j] /= float_n;
            stddev[j] = unsafe { sqrtf32(stddev[j]) };
            stddev[j] = if stddev[j] <= eps { 1.0 } else { stddev[j] };
        }
    }

    for i in 0..m {
        for j in 0..n {
            data[i][j] -= mean[j];
            data[i][j] /= unsafe { sqrtf32(float_n) } * stddev[j];
        }
    }

    for i in 0..(n - 1) {
        corr[i][i] = 1.0;
        for j in (i + 1)..n {
            corr[i][j] = 0.0;
            for k in 0..m {
                corr[i][j] += data[k][i] * data[k][j];
            }
            corr[j][i] = corr[i][j];
        }
    }
    corr[n - 1][n - 1] = 1.0;
}

#[no_mangle]

pub fn bench() {
    const M: usize = 30;
    const N: usize = 35;

    let mut float_n = 0.0;
    let mut data = [[0_f32; N]; M];
    let mut corr = [[0_f32; N]; N];
    let mut mean = [0_f32; N];
    let mut stddev = [0_f32; N];

    init_array(M, N, &mut float_n, &mut data);
    kernel_correlation(M, N, float_n, &mut data, &mut corr, &mut mean, &mut stddev);
    super::utils::consume(corr);
}
