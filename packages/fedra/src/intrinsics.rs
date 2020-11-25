#[no_mangle]
pub extern "C" fn fminf(a: f32, b: f32) -> f32 {
    if a < b {
        a
    } else {
        b
    }
}

#[no_mangle]
pub extern "C" fn fmaxf(a: f32, b: f32) -> f32 {
    if a > b {
        a
    } else {
        b
    }
}

#[no_mangle]
pub extern "C" fn fmin(_a: f64, _b: f64) -> f64 {
    panic!("Float 64 not supported");
}

#[no_mangle]
pub extern "C" fn fmax(_a: f64, _b: f64) -> f64 {
    panic!("Float 64 not supported");
}
