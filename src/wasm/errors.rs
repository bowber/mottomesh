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

#[cfg(not(target_arch = "wasm32"))]
impl From<async_nats::ConnectError> for CustomError {
    fn from(err: async_nats::ConnectError) -> Self {
        CustomError {
            message: err.to_string(),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<async_nats::SubscribeError> for CustomError {
    fn from(err: async_nats::SubscribeError) -> Self {
        CustomError {
            message: err.to_string(),
        }
    }
}
#[cfg(not(target_arch = "wasm32"))]
impl From<async_nats::PublishError> for CustomError {
    fn from(err: async_nats::PublishError) -> Self {
        CustomError {
            message: err.to_string(),
        }
    }
}
