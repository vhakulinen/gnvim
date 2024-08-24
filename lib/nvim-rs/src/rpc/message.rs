use std::borrow::Cow;

use serde::{de::value::SeqAccessDeserializer, ser::SerializeTuple, Deserialize};

#[derive(Debug)]
pub enum Message {
    Request(Request<'static, rmpv::Value>),
    Response(Response<rmpv::Value, rmpv::Value>),
    Notification(Notification<'static, rmpv::Value>),
}

impl<'de> serde::Deserialize<'de> for Message {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = Message;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("valid msgpack-rpc message")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                match seq
                    .next_element::<u32>()?
                    .ok_or_else(|| serde::de::Error::invalid_length(0usize, &self))?
                {
                    0 => Ok(Message::Request(Request::deserialize(
                        SeqAccessDeserializer::new(seq),
                    )?)),
                    1 => Ok(Message::Response(Response::deserialize(
                        SeqAccessDeserializer::new(seq),
                    )?)),
                    2 => Ok(Message::Notification(Notification::deserialize(
                        SeqAccessDeserializer::new(seq),
                    )?)),
                    v => Err(serde::de::Error::custom(format!(
                        "unknown msgpack-rpc tag: {}",
                        v
                    ))),
                }
            }
        }

        deserializer.deserialize_seq(Visitor)
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(bound = "P: serde::Deserialize<'de>")]
pub struct Request<'a, P> {
    pub msgid: u32,
    pub method: Cow<'a, str>,
    pub params: P,
}

impl<'a, P> serde::Serialize for Request<'a, P>
where
    P: serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut tup = serializer.serialize_tuple(4)?;
        tup.serialize_element(&0)?;
        tup.serialize_element(&self.msgid)?;
        tup.serialize_element(&self.method)?;
        tup.serialize_element(&self.params)?;
        tup.end()
    }
}

impl<'a, P> Request<'a, P> {
    pub fn new<S: Into<Cow<'a, str>>>(msgid: u32, method: S, params: P) -> Self {
        Self {
            msgid,
            method: method.into(),
            params,
        }
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(bound = "R: serde::Deserialize<'de>, E: serde::Deserialize<'de>")]
pub struct Response<R, E> {
    pub msgid: u32,
    pub error: Option<E>,
    pub result: Option<R>,
}

impl<R, E> Response<R, E> {
    pub fn new(msgid: u32, error: Option<E>, result: Option<R>) -> Self {
        Self {
            msgid,
            error,
            result,
        }
    }
}

impl<R, E> serde::Serialize for Response<R, E>
where
    R: serde::Serialize,
    E: serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut tup = serializer.serialize_tuple(4)?;
        tup.serialize_element(&1)?;
        tup.serialize_element(&self.msgid)?;
        tup.serialize_element(&self.error)?;
        tup.serialize_element(&self.result)?;
        tup.end()
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(bound = "P: serde::Deserialize<'de>")]
pub struct Notification<'a, P> {
    pub method: Cow<'a, str>,
    pub params: P,
}

impl<'a, P> Notification<'a, P> {
    pub fn new<S: Into<Cow<'a, str>>>(method: S, params: P) -> Self {
        Self {
            method: method.into(),
            params,
        }
    }
}

impl<'a, P> serde::Serialize for Notification<'a, P>
where
    P: serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut tup = serializer.serialize_tuple(3)?;
        tup.serialize_element(&2)?;
        tup.serialize_element(&self.method)?;
        tup.serialize_element(&self.params)?;
        tup.end()
    }
}

#[cfg(test)]
mod tests {
    use super::{Notification, Request, Response};

    #[derive(serde::Serialize)]
    struct Foo {
        foo: u32,
        bar: u32,
    }

    #[derive(serde::Serialize)]
    struct Bar {
        foobar: u32,
        barfoo: u32,
    }

    #[test]
    fn request_serialize() {
        let mut expected = Vec::new();
        rmp_serde::encode::write_named(&mut expected, &(0, 5, "foobar", Foo { foo: 2, bar: 3 }))
            .expect("serialize test data");

        let mut got = Vec::new();
        rmp_serde::encode::write_named(
            &mut got,
            &Request::new(5, "foobar", Foo { foo: 2, bar: 3 }),
        )
        .expect("serialize test data");

        assert_eq!(got, expected);
    }

    #[test]
    fn response_serialize() {
        let mut expected = Vec::new();
        rmp_serde::encode::write_named(
            &mut expected,
            &(
                1,
                23,
                Foo { foo: 32, bar: 99 },
                Bar {
                    foobar: 2,
                    barfoo: 44,
                },
            ),
        )
        .expect("serialize test data");

        let mut got = Vec::new();
        rmp_serde::encode::write_named(
            &mut got,
            &Response::new(
                23,
                Some(Foo { foo: 32, bar: 99 }),
                Some(Bar {
                    foobar: 2,
                    barfoo: 44,
                }),
            ),
        )
        .expect("serialize test data");

        assert_eq!(got, expected);
    }

    #[test]
    fn notification_serialize() {
        let mut expected = Vec::new();
        rmp_serde::encode::write_named(&mut expected, &(2, "foo", Foo { foo: 3, bar: 0 }))
            .expect("serialize test data");

        let mut got = Vec::new();
        rmp_serde::encode::write_named(&mut got, &Notification::new("foo", Foo { foo: 3, bar: 0 }))
            .expect("serialize test data");

        assert_eq!(got, expected);
    }
}
