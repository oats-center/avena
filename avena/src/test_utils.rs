#![cfg(any(test, feature = "test-utils"))]

use std::{
    net::{TcpListener, TcpStream},
    process::{Command, Stdio},
    thread::sleep,
    time::{Duration, Instant},
};

const NATS_IMAGE: &str = "docker.io/library/nats:2.10";

/// Handle to an ephemeral NATS container for tests.
pub struct NatsServer {
    pub url: String,
    container_id: String,
}

impl Drop for NatsServer {
    fn drop(&mut self) {
        let _ = Command::new("podman")
            .args(["rm", "-f", &self.container_id])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
}

/// Spawn a JetStream-enabled NATS container with basic auth for tests.
/// Uses a random localhost port and waits until the port is reachable.
pub fn start_nats_server() -> std::io::Result<NatsServer> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let port = listener.local_addr()?.port();
    drop(listener);

    let output = Command::new("podman")
        .args([
            "run",
            "-d",
            "--rm",
            "-p",
            &format!("127.0.0.1:{port}:4222"),
            NATS_IMAGE,
            "-js",
            "--user",
            "auth",
            "--pass",
            "auth",
        ])
        .stdin(Stdio::null())
        .stderr(Stdio::piped())
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("podman run failed: {stderr}"),
        ));
    }

    let container_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let url = format!("nats://127.0.0.1:{port}");

    wait_for_port(&url, Duration::from_secs(10)).map_err(|e| {
        let _ = Command::new("podman")
            .args(["rm", "-f", &container_id])
            .status();
        std::io::Error::new(std::io::ErrorKind::Other, e)
    })?;

    Ok(NatsServer { url, container_id })
}

fn wait_for_port(url: &str, timeout: Duration) -> Result<(), String> {
    let deadline = Instant::now() + timeout;
    let addr = url
        .strip_prefix("nats://")
        .ok_or_else(|| "invalid url".to_string())?;

    while Instant::now() < deadline {
        if TcpStream::connect(addr).is_ok() {
            return Ok(());
        }
        sleep(Duration::from_millis(100));
    }

    Err(format!("timed out waiting for {addr}"))
}
