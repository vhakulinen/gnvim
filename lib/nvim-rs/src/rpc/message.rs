#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
pub enum Message {
    Request(Request),
    Response(Response),
    Notification(Notification),
}

impl Message {
    pub fn as_response_ref(&self) -> Option<&Response> {
        match self {
            Message::Response(ref request) => Some(request),
            _ => None,
        }
    }

    pub fn as_response(self) -> Option<Response> {
        match self {
            Message::Response(request) => Some(request),
            _ => None,
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct Request {
    // NOTE(ville): Required for deserialization.
    #[allow(dead_code)]
    r#type: u32,

    pub msgid: u32,
    pub method: String,
    pub params: rmpv::Value,
}

#[derive(Debug, serde::Deserialize)]
pub struct Response {
    // NOTE(ville): Required for deserialization.
    #[allow(dead_code)]
    r#type: u32,

    pub msgid: u32,
    pub error: Option<rmpv::Value>,
    pub result: Option<rmpv::Value>,
}

#[derive(Debug, serde::Deserialize)]
pub struct Notification {
    // NOTE(ville): Required for deserialization.
    #[allow(dead_code)]
    r#type: u32,

    pub method: String,
    pub params: rmpv::Value,
}
