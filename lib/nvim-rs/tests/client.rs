use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

use nvim_rs::rpc::{message::Message, RpcReader, RpcWriter};
use nvim_rs::{rpc::Caller, Client};

#[tokio::test]
async fn void_response_decodes_correctly() {
    let (client, server) = tokio::io::duplex(1024 * 64);

    let local = tokio::task::LocalSet::new();

    local
        .run_until(async move {
            let server_handle = tokio::task::spawn_local(async move {
                let (reader, writer) = tokio::io::split(server);
                let writer = writer.compat_write();
                let mut reader: RpcReader<_> = reader.compat().into();

                let got = reader.recv().await.unwrap();
                let req = match got {
                    Message::Request(req) => req,
                    _ => panic!("Unexpected message: {:?}", got),
                };

                assert_eq!(req.method, "get_nil");
                assert_eq!(req.params, rmpv::Value::Nil);

                writer
                    .write_rpc_response(req.msgid, None::<&rmpv::Value>, None::<&rmpv::Value>)
                    .await
                    .unwrap();
            });

            let client_handle = tokio::task::spawn_local(async move {
                let (reader, writer) = tokio::io::split(client);
                let writer = writer.compat_write();
                let mut reader: RpcReader<_> = reader.compat().into();

                let mut client = Client::new(writer);

                let res = client.call::<(), _, _>("get_nil", ()).await.unwrap();

                let handle = tokio::task::spawn(async move {
                    let v = reader.recv().await.unwrap();
                    match v {
                        Message::Response(response) => client.handle_response(response).unwrap(),
                        v => panic!("unexpected message: {:?}", v),
                    }
                });

                assert_eq!(res.await, Ok(()));

                handle.await.unwrap();
            });

            tokio::try_join!(server_handle, client_handle).unwrap();
        })
        .await;
}
