mod ffi {
    extern "C" {
        pub fn set_temp(x: i32);
        pub fn get_temp() -> i32;
    }
}

pub fn set_temp(x: i32) {
    unsafe {
        ffi::set_temp(x);
    }
}

fn get_temp() -> i32 {
    unsafe { ffi::get_temp() }
}

#[no_mangle]
pub extern "C" fn main() {
    let temp = get_temp();

    set_temp(temp + 1);
}
