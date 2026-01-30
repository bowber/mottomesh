use bitcode::{Decode, Encode};
use wasm_bindgen::prelude::*;

mod wasm;
pub use wasm::errors::CustomError;

#[wasm_bindgen]
#[derive(Encode, Decode, Debug, Clone)]
pub struct InnerData {
    id: Vec<u32>,
    name: Vec<String>,
}
#[wasm_bindgen]
#[derive(Encode, Decode, Debug, Clone)]
pub struct TestData {
    id: u32,
    name: String,
    inner_data: InnerData,
}

#[wasm_bindgen]
impl TestData {
    #[wasm_bindgen(constructor)]
    pub fn new(id: u32, name: &str) -> TestData {
        TestData {
            id,
            name: name.into(),
            inner_data: InnerData {
                id: (0..1000).map(|_| id).collect(),
                name: (0..1000).map(|_| name.to_string()).collect(),
            },
        }
    }

    #[wasm_bindgen]
    pub fn id(&self) -> u32 {
        self.id
    }

    #[wasm_bindgen]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    #[wasm_bindgen]
    pub fn encode(&self) -> Result<Vec<u8>, CustomError> {
        // let level = 3; // Compression level
        let source: &[u8] = &bitcode::encode(self);
        // zstd::stream::encode_all(source, level).map_err(|e| e.into())
        Ok(source.to_vec()) // No compression for now
    }

    #[wasm_bindgen]
    pub fn decode(data: &[u8]) -> Result<TestData, CustomError> {
        // let decompressed = zstd::stream::decode_all(data)?;
        let decompressed = data.to_vec(); // No decompression for now
        bitcode::decode(&decompressed).map_err(|e| e.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_test_data() {
        let data = TestData::new(42, "test_name");
        assert_eq!(data.id(), 42);
        assert_eq!(data.name(), "test_name");
    }

    #[test]
    fn test_inner_data_size() {
        let data = TestData::new(1, "inner");
        // Inner data should have 1000 entries
        assert_eq!(data.inner_data.id.len(), 1000);
        assert_eq!(data.inner_data.name.len(), 1000);
        // All IDs should match the outer ID
        assert!(data.inner_data.id.iter().all(|&id| id == 1));
        // All names should match the outer name
        assert!(data.inner_data.name.iter().all(|n| n == "inner"));
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        let original = TestData::new(123, "roundtrip_test");
        let encoded = original.encode().unwrap();
        let decoded = TestData::decode(&encoded).unwrap();

        assert_eq!(decoded.id(), original.id());
        assert_eq!(decoded.name(), original.name());
    }

    #[test]
    fn test_encode_produces_bytes() {
        let data = TestData::new(1, "bytes");
        let encoded = data.encode().unwrap();
        assert!(!encoded.is_empty());
    }

    #[test]
    fn test_decode_invalid_data() {
        let invalid_data = vec![0xFF, 0xFF, 0xFF];
        let result = TestData::decode(&invalid_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_empty_data() {
        let result = TestData::decode(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_large_id() {
        let data = TestData::new(u32::MAX, "max_id");
        let encoded = data.encode().unwrap();
        let decoded = TestData::decode(&encoded).unwrap();
        assert_eq!(decoded.id(), u32::MAX);
    }

    #[test]
    fn test_empty_name() {
        let data = TestData::new(1, "");
        let encoded = data.encode().unwrap();
        let decoded = TestData::decode(&encoded).unwrap();
        assert_eq!(decoded.name(), "");
    }

    #[test]
    fn test_unicode_name() {
        let data = TestData::new(1, "日本語テスト🎉");
        let encoded = data.encode().unwrap();
        let decoded = TestData::decode(&encoded).unwrap();
        assert_eq!(decoded.name(), "日本語テスト🎉");
    }

    #[test]
    fn test_long_name() {
        let long_name = "a".repeat(10000);
        let data = TestData::new(1, &long_name);
        let encoded = data.encode().unwrap();
        let decoded = TestData::decode(&encoded).unwrap();
        assert_eq!(decoded.name().len(), 10000);
    }

    #[test]
    fn test_multiple_roundtrips() {
        let original = TestData::new(999, "multi");

        // Encode and decode multiple times
        let mut current = original.clone();
        for _ in 0..5 {
            let encoded = current.encode().unwrap();
            current = TestData::decode(&encoded).unwrap();
        }

        assert_eq!(current.id(), 999);
        assert_eq!(current.name(), "multi");
    }

    #[test]
    fn test_clone() {
        let original = TestData::new(42, "clone_test");
        let cloned = original.clone();
        assert_eq!(cloned.id(), original.id());
        assert_eq!(cloned.name(), original.name());
    }

    #[test]
    fn test_debug_format() {
        let data = TestData::new(1, "debug");
        let debug_str = format!("{:?}", data);
        assert!(debug_str.contains("TestData"));
        assert!(debug_str.contains("id: 1"));
        assert!(debug_str.contains("name: \"debug\""));
    }
}
