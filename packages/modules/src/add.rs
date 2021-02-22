#[no_mangle]
pub extern "C" fn add(a: f32, b: f32, c: f32) -> f32 {
    a + b + c
}
