#![allow(non_snake_case)]

fn init_array<const N: usize>(n: usize, r: &mut [f32; N]) {
    for i in 0..n {
        r[i] = (n + 1 - i) as f32;
    }
}

fn kernel_durbin<const N: usize>(n: usize, r: &[f32; N], y: &mut [f32; N]) {
    let mut z: [f32; N] = unsafe { core::mem::MaybeUninit::uninit().assume_init() };

    y[0] = -r[0];
    let mut beta = 1.0;
    let mut alpha = -r[0];
    for k in 1..n {
        beta = (1.0 - alpha * alpha) * beta;
        let mut sum = 0.0;
        for i in 0..k {
            sum += r[k - i - 1] * y[i];
        }
        alpha = -(r[k] + sum) / beta;

        for i in 0..k {
            z[i] = y[i] + alpha * y[k - i - 1];
        }
        for i in 0..k {
            y[i] = z[i];
        }
        y[k] = alpha;
    }
}

pub fn bench() {
    const N: usize = 10;

    let mut r = [0_f32; N];
    let mut y = [0_f32; N];

    init_array(N, &mut r);
    kernel_durbin(N, &r, &mut y);

    super::utils::consume(y);
}
