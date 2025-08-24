use bitcode::{Decode, Encode};
use wasm_bindgen::prelude::*;
mod wasm;
pub use wasm::errors::CustomError;

#[wasm_bindgen]
#[derive(Encode, Decode, Debug)]
pub struct InnerData {
    id: Vec<u32>,
    name: Vec<String>,
}
#[wasm_bindgen]
#[derive(Encode, Decode, Debug)]
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
                id: (0..1000).map(|i| i as u32).collect(),
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
        let level = 3; // Compression level
        let source: &[u8] = &bitcode::encode(self);
        zstd::stream::encode_all(source, level).map_err(|e| e.into())
        // Ok(source.to_vec()) // No compression for now
    }

    #[wasm_bindgen]
    pub fn decode(data: &[u8]) -> Result<TestData, CustomError> {
        let decompressed = zstd::stream::decode_all(data)?;
        // let decompressed = data.to_vec(); // No decompression for now
        bitcode::decode(&decompressed).map_err(|e| e.into())
    }
}
