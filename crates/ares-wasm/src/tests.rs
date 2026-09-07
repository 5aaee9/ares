use super::*;

#[tokio::test]
async fn slice_stl_bytes_returns_gcode_bytes() {
    let output = slice_stl_bytes(square_ascii_stl(), r#"{"layer_height":0.2}"#)
        .await
        .unwrap();
    let gcode = String::from_utf8(output).unwrap();

    assert!(gcode.contains("; input_format = stl"));
    assert!(gcode.contains("; layer_height = 0.2"));
    assert!(gcode.ends_with("M2\n"));
}

#[tokio::test]
async fn slice_stl_bytes_rejects_invalid_options_json() {
    let error = slice_stl_bytes(square_ascii_stl(), "{").await.unwrap_err();

    assert!(error.starts_with("invalid options JSON:"));
}

#[tokio::test]
async fn slice_stl_bytes_rejects_invalid_model_bytes() {
    let error = slice_stl_bytes(Vec::new(), "{}").await.unwrap_err();

    assert_eq!(error, "slice input is empty");
}

fn square_ascii_stl() -> Vec<u8> {
    b"solid square\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex 1 0 0.2\nvertex 0 1 0.2\nendloop\nendfacet\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex 0 -1 0.2\nvertex 1 0 0.2\nendloop\nendfacet\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex -1 0 0.2\nvertex 0 -1 0.2\nendloop\nendfacet\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex 0 1 0.2\nvertex -1 0 0.2\nendloop\nendfacet\nendsolid square"
        .to_vec()
}
