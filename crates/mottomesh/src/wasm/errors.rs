use std::fmt;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug)]
pub struct CustomError {
    message: String,
}

#[wasm_bindgen]
impl CustomError {
    #[wasm_bindgen(constructor)]
    pub fn new(message: String) -> CustomError {
        CustomError { message }
    }

    #[wasm_bindgen]
    pub fn message(&self) -> String {
        self.message.clone()
    }
}

impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for CustomError {}

impl From<bitcode::Error> for CustomError {
    fn from(err: bitcode::Error) -> Self {
        CustomError {
            message: err.to_string(),
        }
    }
}

impl From<std::io::Error> for CustomError {
    fn from(err: std::io::Error) -> Self {
        CustomError {
            message: err.to_string(),
        }
    }
}
