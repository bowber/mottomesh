pub fn main() {
    // Only run server on native
    // This will not run in WASM
    #[cfg(not(target_arch = "wasm32"))]
    let _ = server::init_server();
}

#[cfg(not(target_arch = "wasm32"))]
mod server {

    use futures::StreamExt;
    use mottomesh::TestData;

    #[tokio::main]
    pub async fn init_server() -> Result<(), mottomesh::CustomError> {
        // Connect to the NATS server
        let client = async_nats::connect("localhost:4222").await?;

        // Subscribe to the "messages" subject
        let mut subscriber = client.subscribe("messages").await?;

        // Publish messages to the "messages" subject
        for i in 0..3 {
            let name = format!("Test {}", i);
            client
                .publish("messages", TestData::new(i, name.as_str()).encode()?.into())
                .await?;
            println!("Tx: Published message {}", name);
        }

        // Receive and process messages
        while let Some(message) = subscriber.next().await {
            match TestData::decode(&message.payload) {
                Ok(_) => println!("Rx: Received {:?} bytes message", message.length),
                Err(e) => eprintln!(
                    "Rx Failed to {:?} bytes decode message: {:#?}",
                    message.length,
                    e.message()
                ),
            }
        }

        Ok(())
    }
}
