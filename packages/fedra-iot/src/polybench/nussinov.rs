#![allow(non_snake_case)]

type Base = i8;

fn init_array<const N: usize>(n: usize, seq: &mut [Base; N], table: &mut [[f32; N]; N]) {
    for i in 0..n {
        seq[i] = ((i + 1) % 4) as Base;
    }

    for i in 0..n {
        for j in 0..n {
            table[i][j] = 0 as f32;
        }
    }
}

fn kernel_nussinov<const N: usize>(n: usize, seq: &[Base; N], table: &mut [[f32; N]; N]) {
    let match_base = |b1, b2| -> f32 {
        if b1 + b2 == 3 {
            1 as f32
        } else {
            0 as f32
        }
    };

    let max_score = |s1, s2| {
        if s1 >= s2 {
            s1
        } else {
            s2
        }
    };

    for i in (0..n).rev() {
        for j in (i as isize + 1)..(n as isize) {
            if j - 1 >= 0 {
                let j = j as usize;
                table[i][j] = max_score(table[i][j], table[i][j - 1]);
            }
            if i + 1 < n {
                let j = j as usize;
                table[i][j] = max_score(table[i][j], table[i + 1][j]);
            }

            if j - 1 >= 0 && i + 1 < n {
                let j = j as usize;
                if i < j - 1 {
                    table[i][j] = max_score(
                        table[i][j],
                        table[i + 1][j - 1] + match_base(seq[i], seq[j]),
                    );
                } else {
                    table[i][j] = max_score(table[i][j], table[i + 1][j - 1]);
                }
            }

            let j = j as usize;
            for k in (i + 1)..j {
                table[i][j] = max_score(table[i][j], table[i][k] + table[k + 1][j]);
            }
        }
    }
}

pub fn bench() {
    const N: usize = 10;

    let mut seq = [0; N];
    let mut table = [[0_f32; N]; N];

    init_array(N, &mut seq, &mut table);
    kernel_nussinov(N, &seq, &mut table);

    super::utils::consume(table);
}
