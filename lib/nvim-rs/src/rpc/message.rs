use std::borrow::Cow;

use serde::ser::SerializeTuple;

#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
pub enum Message {
    Request(Request<'static, rmpv::Value>),
    Response(Response<rmpv::Value, rmpv::Value>),
    Notification(Notification<'static, rmpv::Value>),
}

#[derive(Debug, serde::Deserialize)]
#[serde(bound = "P: serde::Deserialize<'de>")]
pub struct Request<'a, P> {
    r#type: u32,
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
        tup.serialize_element(&self.r#type)?;
        tup.serialize_element(&self.msgid)?;
        tup.serialize_element(&self.method)?;
        tup.serialize_element(&self.params)?;
        tup.end()
    }
}

impl<'a, P> Request<'a, P> {
    pub fn new<S: Into<Cow<'a, str>>>(msgid: u32, method: S, params: P) -> Self {
        Self {
            r#type: 0,
            msgid,
            method: method.into(),
            params,
        }
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(bound = "R: serde::Deserialize<'de>, E: serde::Deserialize<'de>")]
pub struct Response<R, E> {
    r#type: u32,

    pub msgid: u32,
    pub error: Option<E>,
    pub result: Option<R>,
}

impl<R, E> Response<R, E> {
    pub fn new(msgid: u32, error: Option<E>, result: Option<R>) -> Self {
        Self {
            r#type: 1,
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
        tup.serialize_element(&self.r#type)?;
        tup.serialize_element(&self.msgid)?;
        tup.serialize_element(&self.error)?;
        tup.serialize_element(&self.result)?;
        tup.end()
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(bound = "P: serde::Deserialize<'de>")]
pub struct Notification<'a, P> {
    r#type: u32,

    pub method: Cow<'a, str>,
    pub params: P,
}

impl<'a, P> Notification<'a, P> {
    pub fn new<S: Into<Cow<'a, str>>>(method: S, params: P) -> Self {
        Self {
            r#type: 2,
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
        tup.serialize_element(&self.r#type)?;
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
