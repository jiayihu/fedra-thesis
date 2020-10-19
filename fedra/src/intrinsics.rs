#![allow(non_camel_case_types)]
#![allow(dead_code)]

// @TODO: most of these intrinsics could be avoid by setting the flags d_m3HasFloat=0 and
// d_m3FixedHeap (20 * 1024) to wasm3

#[no_mangle]
pub extern "C" fn ceil(x: f64) -> f64 {
    libm::ceil(x)
}

#[no_mangle]
pub extern "C" fn ceilf(x: f32) -> f32 {
    libm::ceilf(x)
}

#[no_mangle]
pub extern "C" fn floor(x: f64) -> f64 {
    libm::floor(x)
}

#[no_mangle]
pub extern "C" fn floorf(x: f32) -> f32 {
    libm::floorf(x)
}

#[no_mangle]
pub extern "C" fn trunc(x: f64) -> f64 {
    libm::trunc(x)
}

#[no_mangle]
pub extern "C" fn truncf(x: f32) -> f32 {
    libm::truncf(x)
}

#[no_mangle]
pub extern "C" fn sqrt(x: f64) -> f64 {
    libm::sqrt(x)
}

#[no_mangle]
pub extern "C" fn sqrtf(x: f32) -> f32 {
    libm::sqrtf(x)
}

#[no_mangle]
pub extern "C" fn rint(x: f64) -> f64 {
    // From https://github.com/nagisa/math.rs/blob/0e1ff50683e14618cf9920f1389e60e3f4912019/src/rint.rs
    libm::round(x)
}

#[no_mangle]
pub extern "C" fn rintf(x: f32) -> f32 {
    // From https://github.com/nagisa/math.rs/blob/0e1ff50683e14618cf9920f1389e60e3f4912019/src/rint.rs
    libm::roundf(x)
}

#[no_mangle]
pub extern "C" fn __aeabi_d2f(x: f64) -> f32 {
    // panic!("Conversion from double to float");

    // From https://github.com/tomaka/redshirt/blob/1b476ae7a6e6171d76e15a31ffa300625ebd8ba9/kernel/standalone/src/arch/arm/misc.rs
    libm::trunc(x) as f32
}

// Declare C types to simplify stubs for unimplemented intrinsics
type c_char = i8;
type c_schar = i8;
type c_uchar = u8;
type c_short = i16;
type c_ushort = u16;
type c_int = i32;
type c_uint = u32;
type c_long = i32;
type c_ulong = u32;
type c_longlong = i64;
type c_ulonglong = u64;
type c_float = f32;
type c_double = f64;
type size_t = u32;
use core::ffi::c_void;

#[no_mangle]
pub extern "C" fn __fpclassifyf(_x: f32) -> i32 {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn __fpclassifyd(_x: f64) -> i32 {
    unimplemented!();
}

// From https://github.com/Xiao-Cong-Cong/crosvm/blob/76148ae9828505f449ee015cfe864620027605e3/gpu_renderer/src/vsnprintf.rs
// @NOTE: This is a x86 impl
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct __va_list_tag {
    pub gp_offset: c_uint,
    pub fp_offset: c_uint,
    pub overflow_arg_area: *mut c_void,
    pub reg_save_area: *mut c_void,
}

#[no_mangle]
pub extern "C" fn vsnprintf(
    __s: *mut c_char,
    __maxlen: c_ulong,
    __format: *const c_char,
    __arg: *mut __va_list_tag, // Should be core::ffi::VaList
) -> c_int {
    unimplemented!()
}

// From https://github.com/rust-embedded-community/tinyrlibc/blob/master/src/strcmp.rs
#[no_mangle]
pub unsafe extern "C" fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int {
    for i in 0.. {
        let s1_i = s1.offset(i);
        let s2_i = s2.offset(i);

        let val = *s1_i as c_int - *s2_i as c_int;
        if val != 0 || *s1_i == 0 {
            return val;
        }
    }
    0
}

const ALIGN: usize = core::mem::size_of::<i32>();

#[no_mangle]
pub extern "C" fn calloc(num: size_t, size: size_t) -> *mut c_void {
    let size = (num * size) as usize;
    let layout =
        core::alloc::Layout::from_size_align(size, ALIGN).expect("Cannot create layout for calloc");

    unsafe { alloc::alloc::alloc_zeroed(layout) as *mut c_void }
}

#[no_mangle]
pub extern "C" fn free(_ptr: *mut u8) {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn realloc(ptr: *mut c_void, size: size_t) -> *mut c_void {
    if ptr.is_null() {
        return calloc(1, size);
    }

    unimplemented!()
}
