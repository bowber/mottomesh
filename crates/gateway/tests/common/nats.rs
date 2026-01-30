use testcontainers::{ContainerAsync, runners::AsyncRunner};
use testcontainers_modules::nats::Nats;
use tokio::sync::OnceCell;

/// Shared NATS container for all tests
static NATS_CONTAINER: OnceCell<NatsContainer> = OnceCell::const_new();

/// NATS container wrapper (container only, clients created on-demand)
struct NatsContainer {
    _container: ContainerAsync<Nats>,
    url: String,
}

impl NatsContainer {
    /// Start a new NATS container
    async fn start() -> Self {
        let container = Nats::default()
            .start()
            .await
            .expect("Failed to start NATS container");

        let host = container.get_host().await.expect("Failed to get host");
        let port = container
            .get_host_port_ipv4(4222)
            .await
            .expect("Failed to get port");
        let url = format!("{}:{}", host, port);

        // Verify NATS is ready by connecting
        let _ = async_nats::connect(&url)
            .await
            .expect("Failed to connect to NATS");

        Self {
            _container: container,
            url,
        }
    }
}

/// NATS test helper with fresh client
pub struct TestNats {
    url: String,
    client: async_nats::Client,
}

impl TestNats {
    /// Create a new test NATS helper with a fresh client
    async fn new(url: &str) -> Self {
        let client = async_nats::connect(url)
            .await
            .expect("Failed to connect to NATS");

        Self {
            url: url.to_string(),
            client,
        }
    }

    /// Get the NATS URL
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Get a reference to the NATS client
    pub fn client(&self) -> &async_nats::Client {
        &self.client
    }

    /// Publish a message to NATS
    pub async fn publish(&self, subject: &str, payload: &[u8]) {
        self.client
            .publish(subject.to_string(), payload.to_vec().into())
            .await
            .expect("Failed to publish to NATS");

        // Flush to ensure the message is sent
        self.client.flush().await.expect("Failed to flush NATS");
    }

    /// Subscribe to a NATS subject
    pub async fn subscribe(&self, subject: &str) -> async_nats::Subscriber {
        self.client
            .subscribe(subject.to_string())
            .await
            .expect("Failed to subscribe to NATS")
    }
}

/// Get a test NATS helper with a fresh client (container is shared)
pub async fn get_nats() -> TestNats {
    let container = NATS_CONTAINER
        .get_or_init(|| async { NatsContainer::start().await })
        .await;

    TestNats::new(&container.url).await
}

/// Generate a unique test prefix for subjects
#[allow(dead_code)]
pub fn test_subject_prefix(test_name: &str) -> String {
    format!("{}.v1", test_name)
}

/// Generate a full test subject
pub fn test_subject(test_name: &str, topic: &str) -> String {
    format!("{}.v1.{}", test_name, topic)
}
