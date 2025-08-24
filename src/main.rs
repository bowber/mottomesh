pub fn main() {
    // Only run server on native
    // This will not run in WASM
    #[cfg(not(target_arch = "wasm32"))]
    // Spawn the server
    {
        use tokio::runtime::Runtime;

        let rt = Runtime::new().unwrap();
        println!("Runtime created");
        rt.block_on(async {
            println!("Starting server...");
            let r = server::init_server().await;

            println!("Server stopped {:?}", r);
        });
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod server {

    use std::io::Read;

    use flate2::{Compression, bufread::GzDecoder, read::GzEncoder};
    use futures::StreamExt;
    use mottomesh::{CustomError, TestData};

    fn compress(data: &[u8]) -> Result<Vec<u8>, CustomError> {
        let mut ret_vec = Vec::new();
        let mut gz = GzEncoder::new(&data[..], Compression::fast());
        gz.read_to_end(&mut ret_vec)?;
        Ok(ret_vec)
    }

    fn decompress(bytes: Vec<u8>) -> Result<Vec<u8>, CustomError> {
        let mut gz = GzDecoder::new(&bytes[..]);
        let mut b = Vec::new();
        gz.read_to_end(&mut b)?;
        Ok(b)
    }

    pub async fn init_server() -> Result<(), mottomesh::CustomError> {
        // Connect to the NATS server
        let client = async_nats::connect("localhost:4222").await?;

        // Subscribe to the "messages" subject
        let mut subscriber = client.subscribe("messages").await?;

        // Publish messages to the "messages" subject
        for i in 0..3 {
            let name = format!("Test {}", i);
            let data = TestData::new(i, name.as_str());
            let compresses = compress(&data.encode()?)?;
            // let compresses = data.encode()?; // No compression for now
            client.publish("messages", compresses.into()).await?;
            println!("Tx: Published message {}", name);
        }

        // Receive and process messages
        while let Some(message) = subscriber.next().await {
            let decompressed = match decompress(message.payload.to_vec()) {
                Ok(data) => data,
                Err(e) => {
                    eprintln!(
                        "Rx Failed to {:?} bytes decompress message: {:#?}",
                        message.length,
                        e.message()
                    );
                    continue;
                }
            };
            // let decompressed = message.payload.to_vec(); // No decompression for now
            match TestData::decode(&decompressed) {
                Ok(data) => println!(
                    "Rx: Received {:?} bytes message: {:?}",
                    message.length,
                    data.name()
                ),
                Err(e) => eprintln!(
                    "Rx Failed to decode {:?} bytes message: {:#?}",
                    message.length,
                    e.message()
                ),
            }
        }

        Ok(())
    }
}
