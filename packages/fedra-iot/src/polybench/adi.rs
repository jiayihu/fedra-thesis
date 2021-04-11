#![allow(non_snake_case)]

fn kernel_adi<const N: usize, const TSTEPS: usize>(
    tsteps: usize,
    n: usize,
    u: &mut [[f32; N]; N],
    v: &mut [[f32; N]; N],
    p: &mut [[f32; N]; N],
    q: &mut [[f32; N]; N],
) {
    let DX = 1.0 / n as f32;
    let DY = 1.0 / n as f32;
    let DT = 1.0 / tsteps as f32;
    let B1 = 2.0;
    let B2 = 1.0;
    let mul1 = B1 * DT / (DX * DX);
    let mul2 = B2 * DT / (DY * DY);

    let a = -mul1 / 2.0;
    let b = 1.0 + mul1;
    let c = a;
    let d = -mul2 / 2.0;
    let e = 1.0 + mul2;
    let f = d;

    for _ in 1..tsteps {
        for i in 1..(n - 1) {
            v[0][i] = 1.0;
            p[i][0] = 0.0;
            q[i][0] = v[0][i];
            for j in 1..(n - 1) {
                p[i][j] = -c / (a * p[i][j - 1] + b);
                q[i][j] = (-d * u[j][i - 1] + (1.0 + 2.0 * d) * u[j][i]
                    - f * u[j][i + 1]
                    - a * q[i][j - 1])
                    / (a * p[i][j - 1] + b);
            }

            v[n - 1][i] = 1.0;
            for j in (1..(n - 1)).rev() {
                v[j][i] = p[i][j] * v[j + 1][i] + q[i][j];
            }
        }

        for i in 1..(n - 1) {
            u[i][0] = 1.0;
            p[i][0] = 0.0;
            q[i][0] = u[i][0];
            for j in 1..(n - 1) {
                p[i][j] = -f / (d * p[i][j - 1] + e);
                q[i][j] = (-a * v[i - 1][j] + (1.0 + 2.0 * a) * v[i][j]
                    - c * v[i + 1][j]
                    - d * q[i][j - 1])
                    / (d * p[i][j - 1] + e);
            }
            u[i][n - 1] = 1.0;
            for j in (1..(n - 1)).rev() {
                u[i][j] = p[i][j] * u[i][j + 1] + q[i][j];
            }
        }
    }
}

pub fn bench() {
    const N: usize = 10;
    const TSTEPS: usize = 10;

    let mut u = [[0_f32; N]; N];
    let mut v = [[0_f32; N]; N];
    let mut p = [[0_f32; N]; N];
    let mut q = [[0_f32; N]; N];

    kernel_adi::<N, TSTEPS>(TSTEPS, N, &mut u, &mut v, &mut p, &mut q);

    super::utils::consume(u);
}
