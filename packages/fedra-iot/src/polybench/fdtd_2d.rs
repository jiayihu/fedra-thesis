#![allow(non_snake_case)]

fn init_array<const NX: usize, const NY: usize, const TMAX: usize>(
    tmax: usize,
    nx: usize,
    ny: usize,
    ex: &mut [[f32; NY]; NX],
    ey: &mut [[f32; NY]; NX],
    hz: &mut [[f32; NY]; NX],
    fict: &mut [f32; TMAX],
) {
    for i in 0..tmax {
        fict[i] = i as f32;
    }
    for i in 0..nx {
        for j in 0..ny {
            ex[i][j] = (i * (j + 1)) as f32 / nx as f32;
            ey[i][j] = (i * (j + 2)) as f32 / ny as f32;
            hz[i][j] = (i * (j + 3)) as f32 / nx as f32;
        }
    }
}

fn kernel_fdtd_2d<const NX: usize, const NY: usize, const TMAX: usize>(
    tmax: usize,
    nx: usize,
    ny: usize,
    ex: &mut [[f32; NY]; NX],
    ey: &mut [[f32; NY]; NX],
    hz: &mut [[f32; NY]; NX],
    fict: &[f32; TMAX],
) {
    for t in 0..tmax {
        for j in 0..ny {
            ey[0][j] = fict[t];
        }
        for i in 1..nx {
            for j in 0..ny {
                ey[i][j] = ey[i][j] - 0.5 * (hz[i][j] - hz[i - 1][j]);
            }
        }
        for i in 0..nx {
            for j in 1..ny {
                ex[i][j] = ex[i][j] - 0.5 * (hz[i][j] - hz[i][j - 1]);
            }
        }
        for i in 0..(nx - 1) {
            for j in 0..(ny - 1) {
                hz[i][j] = hz[i][j] - 0.7 * (ex[i][j + 1] - ex[i][j] + ey[i + 1][j] - ey[i][j]);
            }
        }
    }
}

pub fn bench() {
    const TMAX: usize = 10;
    const NX: usize = 10;
    const NY: usize = 10;

    let mut ex = [[0_f32; NY]; NX];
    let mut ey = [[0_f32; NY]; NX];
    let mut hz = [[0_f32; NY]; NX];
    let mut fict = [0_f32; TMAX];

    init_array(TMAX, NX, NY, &mut ex, &mut ey, &mut hz, &mut fict);
    kernel_fdtd_2d(TMAX, NX, NY, &mut ex, &mut ey, &mut hz, &fict);

    super::utils::consume(hz);
}
