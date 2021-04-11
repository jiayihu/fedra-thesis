#![allow(non_snake_case)]

fn init_array<const N: usize>(n: usize, path: &mut [[f32; N]; N]) {
    for i in 0..n {
        for j in 0..n {
            path[i][j] = (i * j % 7 + 1) as f32;
            if (i + j) % 13 == 0 || (i + j) % 7 == 0 || (i + j) % 11 == 0 {
                path[i][j] = 999 as f32;
            }
        }
    }
}

fn kernel_floyd_warshall<const N: usize>(n: usize, path: &mut [[f32; N]; N]) {
    for k in 0..n {
        for i in 0..n {
            for j in 0..n {
                path[i][j] = if path[i][j] < path[i][k] + path[k][j] {
                    path[i][j]
                } else {
                    path[i][k] + path[k][j]
                };
            }
        }
    }
}

pub fn bench() {
    const N: usize = 10;

    let mut path = [[0_f32; N]; N];

    init_array(N, &mut path);
    kernel_floyd_warshall(N, &mut path);

    super::utils::consume(path);
}
