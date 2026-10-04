//! Regression: D-Bus method handlers must run inside the tokio runtime.
//!
//! CheckForUpdates/StartUpdate spawn the package manager with `tokio::process`. While zbus used
//! its default async-io executor, handlers ran on a "zbus::Connection executor" thread and
//! panicked with "there is no reactor running, must be called from the context of a Tokio 1.x
//! runtime". The daemon now builds zbus with its `tokio` feature.

use zbus::{connection::Builder, interface, Guid};

struct ProcessProbe;

#[interface(name = "com.asuslinuxcontrol.TestProcessProbe")]
impl ProcessProbe {
    async fn run_child(&self) -> zbus::fdo::Result<i32> {
        let status = tokio::process::Command::new("true")
            .status()
            .await
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
        Ok(status.code().unwrap_or(-1))
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn interface_methods_can_spawn_tokio_processes() {
    let (server_socket, client_socket) = tokio::net::UnixStream::pair().unwrap();
    let server = Builder::unix_stream(server_socket)
        .server(Guid::generate())
        .unwrap()
        .p2p()
        .serve_at("/test", ProcessProbe)
        .unwrap()
        .build();
    let client = Builder::unix_stream(client_socket).p2p().build();
    let (_server, client) = tokio::try_join!(server, client).unwrap();

    let reply = client
        .call_method(
            None::<&str>,
            "/test",
            Some("com.asuslinuxcontrol.TestProcessProbe"),
            "RunChild",
            &(),
        )
        .await
        .expect("handler must not panic outside the tokio runtime");
    let exit_code: i32 = reply.body().deserialize().unwrap();

    assert_eq!(exit_code, 0);
}

#[test]
fn daemon_builds_zbus_on_tokio_instead_of_async_io() {
    let manifest = include_str!("../Cargo.toml");
    let dependencies = &manifest[manifest.find("[dependencies]").unwrap()..];
    let zbus = dependencies
        .lines()
        .find(|line| line.starts_with("zbus"))
        .expect("zbus dependency");

    assert!(zbus.contains("default-features = false"), "{zbus}");
    assert!(zbus.contains("\"tokio\""), "{zbus}");
}
