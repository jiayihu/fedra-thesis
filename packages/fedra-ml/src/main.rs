use tract_onnx::prelude::*;

fn main() -> TractResult<()> {
    let model = tract_onnx::onnx()
        .model_for_path("./flood.onnx")?
        .with_input_fact(0, InferenceFact::dt_shape(f32::datum_type(), tvec![5, 3]))?
        .into_optimized()?
        .into_runnable()?;

    let input = tract_ndarray::arr2(&[
        [0.31375358_f32, 0.13323782_f32, 0.1658887_f32],
        [0.35816619_f32, 0.01289398_f32, 0.08006743_f32],
        [0.18481375_f32, 0.13323782_f32, 0.28145921_f32],
        [0.42550143_f32, 0.1504298_f32, 0.21191364_f32],
        [0.1504298_f32, 0.18911175_f32, 0.54555816_f32],
    ]);

    let result = model.run(tvec![input.into()]).unwrap();
    let to_show = result[0].to_array_view::<f32>()?;

    println!("result: {:?}", to_show);

    Ok(())
}
