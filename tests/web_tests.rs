use mottomesh::TestData;
use wasm_bindgen_test::*;
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
pub fn addition_works() {
    assert_eq!(2 + 2, 4);
}

#[wasm_bindgen_test]
pub fn test_data_construction() {
    let data = TestData::new(1, "Test".into());
    let id = data.id();
    let name = data.name();
    assert_eq!(id, 1);
    assert_eq!(name, "Test");
}

#[wasm_bindgen_test]
pub fn test_data_encoding() {
    let data = TestData::new(1, "Test".into());
    let encoded = data.encode().unwrap();
    assert!(!encoded.is_empty());
}

#[wasm_bindgen_test]
pub fn test_data_decoding() {
    let data = TestData::new(1, "Test".into());
    let encoded = data.encode().unwrap();
    let decoded = TestData::decode(&encoded).unwrap();
    assert_eq!(data.id(), decoded.id());
    assert_eq!(data.name(), decoded.name());
}

#[wasm_bindgen_test]
pub fn test_data_fails_decoding() {
    let result = TestData::decode(&[0, 1, 2, 3]);
    assert!(result.is_err());
}
