use std::process::Stdio;
use std::rc::Rc;
use std::time::Duration;

use nvim_rs::types::{Object, UiOptions};
use tokio::process::Command;
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

use nvim_rs::rpc::{message::Message, RpcReader};
use nvim_rs::{Client, NeovimApi};

#[tokio::test]
async fn smoke_test() {
    let mut cmd = Command::new("nvim")
        .arg("--headless")
        .arg("--cmd")
        .arg("call stdioopen({'rpc': v:true})")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let stdin = cmd.stdin.take().unwrap();
    let stdout = cmd.stdout.take().unwrap();

    let writer = stdin.compat_write();
    let mut reader: RpcReader<_> = stdout.compat().into();

    let client = Rc::new(Client::new(writer));

    let c = client.clone();
    let (result, _) = tokio::join!(client.nvim_get_vvar("argv"), async move {
        let v = reader.recv().await.unwrap();
        match v {
            Message::Response(response) => c.handle_response(response).unwrap(),
            v => panic!("unexpected message: {:?}", v),
        }
    });

    let vals = vec![
        rmpv::Value::from("nvim"),
        rmpv::Value::from("--headless"),
        rmpv::Value::from("--cmd"),
        rmpv::Value::from("call stdioopen({'rpc': v:true})"),
    ];

    assert_eq!(result.unwrap(), Object::new(vals));
}

#[tokio::test]
async fn smoke_test_ui_attach() {
    // Smoke test for deserializing _some_ of the UI events.

    let mut cmd = Command::new("nvim")
        .arg("--headless")
        .arg("--cmd")
        .arg("call stdioopen({'rpc': v:true})")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let stdin = cmd.stdin.take().unwrap();
    let stdout = cmd.stdout.take().unwrap();

    let writer = stdin.compat_write();
    let mut reader: RpcReader<_> = stdout.compat().into();

    let client = Rc::new(Client::new(writer));

    let c = client.clone();
    let res = c.nvim_ui_attach(10, 10, UiOptions::default());

    // Read what ever messages we manage get in a reasonalbe time and deserialize them.
    let read = async move {
        let mut i = 0;
        loop {
            tokio::select! {
                _sleep = tokio::time::sleep(Duration::from_secs(2)) => {
                    println!("Timeout");
                    // We should have some redraw calls.
                    assert!(i > 0);
                    break;
                },
                v = reader.recv() => {
                    match v.unwrap() {
                        Message::Response(response) => client.handle_response(response).unwrap(),
                        Message::Notification(notification) => {
                            match notification.method.as_ref() {
                                "redraw" => {
                                    i += 1;
                                    nvim_rs::types::decode_redraw_params(notification.params).unwrap();
                                }
                                _ => panic!("unexpected notification: {}", notification.method),
                            }
                        }
                        v => panic!("unexpected message: {:?}", v),
                    }
                },
            }
        }
    };

    let (res, _) = tokio::join!(res, read);
    assert!(res.is_ok());
}
