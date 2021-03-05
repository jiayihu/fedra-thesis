use rand::Rng;
use stm32f4xx_hal::rng::Rng as StmRng;

const RAINFALL_MEAN: f32 = 3.47;
const RAINFALL_SIGMA: f32 = 5.58;

#[allow(dead_code)]
const FLOW_MEAN: f32 = 48.42;
#[allow(dead_code)]
const FLOW_SIGMA: f32 = 58.69;

pub fn gen_rainfall(rand_source: &mut StmRng) -> f32 {
    let std_rainfall: f32 = rand_source.gen();
    let sample = (std_rainfall * RAINFALL_SIGMA) + RAINFALL_MEAN;

    if sample < 0f32 {
        -sample
    } else {
        sample
    }
}

#[allow(dead_code)]
pub fn gen_flow(rand_source: &mut StmRng) -> f32 {
    let std_rainfall: f32 = rand_source.gen();
    let sample = (std_rainfall * FLOW_SIGMA) + FLOW_MEAN;

    if sample < 0f32 {
        -sample
    } else {
        sample
    }
}
