use ares_core::{GenerationMetadata, SliceError, SliceOptions};
use js_sys::Date;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub async fn slice_stl(input: Vec<u8>, options_json: String) -> Result<Vec<u8>, JsValue> {
    slice_stl_bytes(input, &options_json)
        .await
        .map_err(|error| JsValue::from_str(&error))
}

pub async fn slice_stl_bytes(input: Vec<u8>, options_json: &str) -> Result<Vec<u8>, String> {
    let options = parse_options(options_json)?;
    ares_core::slice(input, options)
        .await
        .map_err(|error| error.to_string())
}

#[wasm_bindgen(js_name = sliceProject)]
pub async fn slice_project(input: Vec<u8>) -> Result<Vec<u8>, JsValue> {
    slice_project_plate(input, None).await
}

#[wasm_bindgen(js_name = sliceProjectPlate)]
pub async fn slice_project_plate(input: Vec<u8>, plate: Option<u32>) -> Result<Vec<u8>, JsValue> {
    let metadata = local_generation_metadata().map_err(slice_error_js)?;
    ares_core::slice_project_plate(input, metadata, plate)
        .await
        .map_err(slice_error_js)
}

fn local_generation_metadata() -> Result<GenerationMetadata, SliceError> {
    let now = Date::new_0();
    GenerationMetadata::new_local(
        u16::try_from(now.get_full_year()).map_err(|_| invalid_date())?,
        u8::try_from(now.get_month() + 1).map_err(|_| invalid_date())?,
        u8::try_from(now.get_date()).map_err(|_| invalid_date())?,
        u8::try_from(now.get_hours()).map_err(|_| invalid_date())?,
        u8::try_from(now.get_minutes()).map_err(|_| invalid_date())?,
        u8::try_from(now.get_seconds()).map_err(|_| invalid_date())?,
    )
}

fn invalid_date() -> SliceError {
    SliceError::InvalidInput("browser local date is outside the supported range".to_owned())
}

fn slice_error_js(error: SliceError) -> JsValue {
    JsValue::from_str(&error.to_string())
}

fn parse_options(options_json: &str) -> Result<SliceOptions, String> {
    serde_json::from_str(options_json).map_err(|error| format!("invalid options JSON: {error}"))
}

#[cfg(test)]
mod tests;
