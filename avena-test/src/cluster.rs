//! Multi-node NATS cluster management for integration tests.
//!
//! Provides [`TestCluster`] for spawning ephemeral NATS server containers via podman.

use std::{
    collections::HashMap,
    io::{self, Write},
    net::TcpListener,
    process::{Command, Stdio},
    thread::sleep,
    time::{Duration, Instant},
};
use tempfile::NamedTempFile;

const NATS_IMAGE: &str = "docker.io/library/nats:2.10";

struct ContainerHandle(String);

impl Drop for ContainerHandle {
    fn drop(&mut self) {
        let _ = Command::new("podman")
            .args(["rm", "-f", &self.0])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
}

pub struct NatsServer {
    pub url: String,
    pub port: u16,
    _handle: ContainerHandle,
    #[allow(dead_code)]
    config_file: Option<NamedTempFile>,
}

impl std::fmt::Debug for NatsServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NatsServer")
            .field("url", &self.url)
            .field("port", &self.port)
            .finish()
    }
}

pub struct TestNode {
    pub id: String,
    pub nats: NatsServer,
}

impl std::fmt::Debug for TestNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TestNode")
            .field("id", &self.id)
            .field("url", &self.nats.url)
            .finish()
    }
}

impl TestNode {
    pub fn url(&self) -> &str {
        &self.nats.url
    }
}

pub struct TestCluster {
    nodes: HashMap<String, TestNode>,
    hub: Option<NatsServer>,
}

impl std::fmt::Debug for TestCluster {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TestCluster")
            .field("nodes", &self.nodes.keys().collect::<Vec<_>>())
            .field("has_hub", &self.hub.is_some())
            .finish()
    }
}

impl TestCluster {
    pub fn new(count: usize) -> io::Result<Self> {
        let mut nodes = HashMap::new();
        for i in 1..=count {
            let id = format!("node{}", i);
            let nats = start_nats_server()?;
            nodes.insert(id.clone(), TestNode { id, nats });
        }
        Ok(Self { nodes, hub: None })
    }

    pub fn with_hub(leaf_count: usize) -> io::Result<Self> {
        let hub = start_nats_hub()?;
        let hub_port = hub.port;

        let mut nodes = HashMap::new();
        for i in 1..=leaf_count {
            let id = format!("node{}", i);
            let nats = start_nats_leaf(hub_port)?;
            nodes.insert(id.clone(), TestNode { id, nats });
        }

        Ok(Self {
            nodes,
            hub: Some(hub),
        })
    }

    pub fn node(&self, id: &str) -> Option<&TestNode> {
        self.nodes.get(id)
    }

    pub fn hub(&self) -> Option<&NatsServer> {
        self.hub.as_ref()
    }

    pub fn node_ids(&self) -> Vec<&str> {
        self.nodes.keys().map(|s| s.as_str()).collect()
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub async fn connect_nats(
        &self,
        node_id: &str,
    ) -> Result<async_nats::Client, async_nats::ConnectError> {
        let node = self
            .node(node_id)
            .unwrap_or_else(|| panic!("node {} not found", node_id));
        async_nats::ConnectOptions::with_user_and_password("auth".into(), "auth".into())
            .connect(&node.nats.url)
            .await
    }

    pub async fn connect_avena(&self, node_id: &str) -> Result<avena::Avena, avena::ConnectError> {
        let node = self
            .node(node_id)
            .unwrap_or_else(|| panic!("node {} not found", node_id));
        avena::Avena::connect_with_auth(&node.nats.url, "auth", "auth").await
    }
}

fn find_available_port() -> io::Result<u16> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let port = listener.local_addr()?.port();
    drop(listener);
    Ok(port)
}

fn wait_for_port(port: u16, timeout: Duration) -> io::Result<()> {
    let addr = format!("127.0.0.1:{}", port);
    let deadline = Instant::now() + timeout;

    while Instant::now() < deadline {
        if std::net::TcpStream::connect(&addr).is_ok() {
            return Ok(());
        }
        sleep(Duration::from_millis(50));
    }

    Err(io::Error::new(
        io::ErrorKind::TimedOut,
        format!("timed out waiting for port {}", port),
    ))
}

pub fn start_nats_server() -> io::Result<NatsServer> {
    let port = find_available_port()?;
    start_nats_container(port, None)
}

fn start_nats_container(port: u16, config: Option<&NamedTempFile>) -> io::Result<NatsServer> {
    let mut args = vec![
        "run".to_string(),
        "-d".to_string(),
        "--rm".to_string(),
        "-p".to_string(),
        format!("127.0.0.1:{}:4222", port),
    ];

    if let Some(cfg) = config {
        let path = cfg.path().to_str().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "Invalid config path")
        })?;
        args.push("-v".to_string());
        args.push(format!("{}:/nats.conf:ro,Z", path));
        args.push(NATS_IMAGE.to_string());
        args.push("-c".to_string());
        args.push("/nats.conf".to_string());
    } else {
        args.push(NATS_IMAGE.to_string());
        args.push("-js".to_string());
        args.push("--user".to_string());
        args.push("auth".to_string());
        args.push("--pass".to_string());
        args.push("auth".to_string());
    }

    let output = Command::new("podman")
        .args(&args)
        .output()?;

    if !output.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("podman run failed: {}", String::from_utf8_lossy(&output.stderr)),
        ));
    }

    let container_id = String::from_utf8_lossy(&output.stdout).trim().to_string();

    wait_for_port(port, Duration::from_secs(10))?;

    Ok(NatsServer {
        url: format!("nats://127.0.0.1:{}", port),
        port,
        _handle: ContainerHandle(container_id),
        config_file: None,
    })
}

fn start_nats_hub() -> io::Result<NatsServer> {
    let client_port = find_available_port()?;
    let leaf_port = find_available_port()?;

    let config = format!(
        r#"
port: 4222
jetstream: enabled
authorization {{
    user: auth
    password: auth
}}
leafnodes {{
    port: 7422
    authorization {{
        user: leaf
        password: leaf
    }}
}}
"#
    );

    let mut config_file = NamedTempFile::new()?;
    config_file.write_all(config.as_bytes())?;
    config_file.flush()?;

    let output = Command::new("podman")
        .args([
            "run", "-d", "--rm",
            "-p", &format!("127.0.0.1:{}:4222", client_port),
            "-p", &format!("{}:7422", leaf_port),
            "-v", &format!("{}:/nats.conf:ro,Z", config_file.path().to_str().unwrap()),
            NATS_IMAGE,
            "-c", "/nats.conf",
        ])
        .output()?;

    if !output.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("podman run failed: {}", String::from_utf8_lossy(&output.stderr)),
        ));
    }

    let container_id = String::from_utf8_lossy(&output.stdout).trim().to_string();

    wait_for_port(client_port, Duration::from_secs(10))?;
    wait_for_port(leaf_port, Duration::from_secs(10))?;

    Ok(NatsServer {
        url: format!("nats://127.0.0.1:{}", client_port),
        port: leaf_port,
        _handle: ContainerHandle(container_id),
        config_file: Some(config_file),
    })
}

fn start_nats_leaf(hub_leaf_port: u16) -> io::Result<NatsServer> {
    let client_port = find_available_port()?;

    let config = format!(
        r#"
port: 4222
jetstream: enabled
authorization {{
    user: auth
    password: auth
}}
leafnodes {{
    remotes [
        {{
            url: "nats://leaf:leaf@host.containers.internal:{}"
        }}
    ]
}}
"#,
        hub_leaf_port
    );

    let mut config_file = NamedTempFile::new()?;
    config_file.write_all(config.as_bytes())?;
    config_file.flush()?;

    let output = Command::new("podman")
        .args([
            "run", "-d", "--rm",
            "-p", &format!("127.0.0.1:{}:4222", client_port),
            "-v", &format!("{}:/nats.conf:ro,Z", config_file.path().to_str().unwrap()),
            NATS_IMAGE,
            "-c", "/nats.conf",
        ])
        .output()?;

    if !output.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("podman run failed: {}", String::from_utf8_lossy(&output.stderr)),
        ));
    }

    let container_id = String::from_utf8_lossy(&output.stdout).trim().to_string();

    wait_for_port(client_port, Duration::from_secs(10))?;
    sleep(Duration::from_millis(1000));

    Ok(NatsServer {
        url: format!("nats://127.0.0.1:{}", client_port),
        port: client_port,
        _handle: ContainerHandle(container_id),
        config_file: Some(config_file),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;

    #[test]
    fn test_cluster_new() {
        let cluster = TestCluster::new(3).unwrap();
        assert_eq!(cluster.len(), 3);
        assert!(cluster.node("node1").is_some());
        assert!(cluster.node("node2").is_some());
        assert!(cluster.node("node3").is_some());
        assert!(cluster.hub().is_none());
    }

    #[test]
    fn test_cluster_with_hub() {
        let cluster = TestCluster::with_hub(2).unwrap();
        assert_eq!(cluster.len(), 2);
        assert!(cluster.hub().is_some());
    }

    #[tokio::test]
    async fn test_message_routing_via_hub() {
        let cluster = TestCluster::with_hub(2).unwrap();

        tokio::time::sleep(Duration::from_millis(500)).await;

        let nc1 = cluster.connect_nats("node1").await.unwrap();
        let nc2 = cluster.connect_nats("node2").await.unwrap();

        let mut sub = nc2.subscribe("test.subject").await.unwrap();

        tokio::time::sleep(Duration::from_millis(200)).await;

        nc1.publish("test.subject", "hello".into()).await.unwrap();
        nc1.flush().await.unwrap();

        let msg = tokio::time::timeout(Duration::from_secs(5), sub.next())
            .await
            .expect("timeout")
            .expect("message");

        assert_eq!(msg.payload.as_ref(), b"hello");
    }
}
