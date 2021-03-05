mod ffi {
    extern "C" {
        pub fn get_rainfall() -> f32;
        pub fn set_rainfall(x: f32);
    }
}

pub fn set_rainfall(x: f32) {
    unsafe {
        ffi::set_rainfall(x);
    }
}

const RAINFALL_MIN: f32 = 0.;
const RAINFALL_MAX: f32 = 69.8;

const _FLOW_MIN: f32 = 2.143;
const _FLOW_MAX: f32 = 690.9;

// https://scikit-learn.org/stable/modules/generated/sklearn.preprocessing.MinMaxScaler.html
fn min_max_scale(x: f32, min: f32, max: f32) -> f32 {
    let x_scaled = (x - min) / (max - min);

    x_scaled
}

fn get_rainfall() -> f32 {
    unsafe { ffi::get_rainfall() }
}

#[no_mangle]
pub extern "C" fn preprocess_rainfall() {
    let rainfall = get_rainfall();
    let scaled = min_max_scale(rainfall, RAINFALL_MIN, RAINFALL_MAX);

    set_rainfall(scaled);
}
