//! Motto schema for the Mottomesh demo payload.

pub struct InnerData {
    pub id: Vec<u32>,
    pub name: Vec<String>,
}

pub struct TestData {
    pub id: u32,
    pub name: String,
    pub inner_data: InnerData,
}

pub enum ClientMessage {
    Auth {
        token: String,
    },
    Subscribe {
        subject: String,
        id: u64,
    },
    Unsubscribe {
        id: u64,
    },
    Publish {
        subject: String,
        payload: Vec<u8>,
    },
    Request {
        subject: String,
        payload: Vec<u8>,
        timeout_ms: u32,
        request_id: u64,
    },
    Ping,
}

pub enum ServerMessage {
    AuthOk {
        session_id: String,
    },
    AuthError {
        reason: String,
    },
    SubscribeOk {
        id: u64,
    },
    SubscribeError {
        id: u64,
        reason: String,
    },
    Message {
        subscription_id: u64,
        subject: String,
        payload: Vec<u8>,
    },
    Response {
        request_id: u64,
        payload: Vec<u8>,
    },
    RequestError {
        request_id: u64,
        reason: String,
    },
    Error {
        code: u32,
        message: String,
    },
    Pong,
}

pub struct ClientEnvelope {
    pub message: ClientMessage,
}

pub struct ServerEnvelope {
    pub message: ServerMessage,
}
