use tract_onnx::prelude::*;

fn predict(datum: [f32; 3]) -> TractResult<f32> {
    let file = include_bytes!("./flood.onnx");
    let model = tract_onnx::onnx()
        .model_for_read(&mut &file[..])?
        .with_input_fact(0, InferenceFact::dt_shape(f32::datum_type(), tvec![1, 3]))?
        .into_optimized()?
        .into_runnable()?;

    let input = tract_ndarray::arr2(&[datum]);
    let output = model.run(tvec![input.into()]).unwrap();

    let result = output[0].nth(0)?;
    let scalar = result.to_scalar::<f32>()?;

    Ok(scalar.clone())
}

#[no_mangle]
pub extern "C" fn run(rain_ystd: f32, rain_tdy: f32, flow_ystd: f32) -> f32 {
    let datum = [rain_ystd, rain_tdy, flow_ystd];
    let result = predict(datum).unwrap();

    result
}
