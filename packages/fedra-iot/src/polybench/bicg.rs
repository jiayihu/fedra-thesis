#![allow(non_snake_case)]

fn init_array<const M: usize, const N: usize>(
    A: &mut [[f32; N]; M],
    r: &mut [f32; M],
    p: &mut [f32; N],
) {
    for i in 0..N {
        p[i] = (i % N) as f32 / N as f32;
    }
    for i in 0..M {
        r[i] = (i % M) as f32 / M as f32;
        for j in 0..N {
            A[i][j] = (i * (j + 1) % M) as f32 / M as f32;
        }
    }
}

fn kernel_bicg<const M: usize, const N: usize>(
    A: &[[f32; N]; M],
    s: &mut [f32; N],
    q: &mut [f32; M],
    p: &[f32; N],
    r: &[f32; M],
) {
    for i in 0..N {
        s[i] = 0.0;
    }
    for i in 0..M {
        q[i] = 0.0;
        for j in 0..N {
            s[j] = s[j] + r[i] * A[i][j];
            q[i] = q[i] + A[i][j] * p[j];
        }
    }
}

pub fn bench() {
    const M: usize = 25;
    const N: usize = 25;

    let mut A = [[0_f32; N]; M];
    let mut s = [0_f32; N];
    let mut q = [0_f32; M];
    let mut p = [0_f32; N];
    let mut r = [0_f32; M];

    init_array(&mut A, &mut r, &mut p);
    kernel_bicg(&A, &mut s, &mut q, &p, &r);

    super::utils::consume(s);
    super::utils::consume(q);
}
