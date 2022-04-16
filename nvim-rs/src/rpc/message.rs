use rmpv::Value;
use serde;

#[derive(Debug, serde::Serialize)]
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

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Request {
    r#type: u32,
    pub msgid: u32,
    pub method: String,
    pub params: Vec<rmpv::Value>,
}

impl Request {
    pub fn new<S: Into<String>>(msgid: u32, method: S, params: Vec<rmpv::Value>) -> Self {
        Self {
            r#type: 0,
            msgid,
            method: method.into(),
            params,
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Response {
    r#type: u32,
    pub msgid: u32,
    pub error: Option<rmpv::Value>,
    pub result: Option<rmpv::Value>,
}

impl Response {
    pub fn new(msgid: u32, result: Option<rmpv::Value>, error: Option<rmpv::Value>) -> Self {
        Self {
            r#type: 1,
            msgid,
            result,
            error,
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Notification {
    r#type: u32,
    pub method: String,
    pub params: rmpv::Value,
}

impl<'de> serde::Deserialize<'de> for Message {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let value = Value::deserialize(d)?;

        // TODO(ville): Error handling.
        Ok(
            match value
                .as_array()
                .and_then(|v| v.get(0))
                .and_then(|v| v.as_u64())
            {
                Some(0) => Message::Request(Request::deserialize(value).unwrap()),
                Some(1) => Message::Response(Response::deserialize(value).unwrap()),
                Some(2) => Message::Notification(Notification::deserialize(value).unwrap()),
                v => panic!("failed to decode message {:?}", v),
            },
        )
    }
}
