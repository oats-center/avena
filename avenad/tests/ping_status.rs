use std::sync::Arc;
use std::time::Duration;

use avena::hlc::HlcClock;
use avena::messages::{subject_ping, subject_status, PingRequest, PingResponse, StatusResponse};
use avena::test_utils::start_nats_server;
use tokio::task::JoinHandle;
use avenad::device::DeviceIdentity;

/// Spin up the avenad request handlers (ping/status) against an ephemeral NATS and assert round-trips.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ping_and_status_roundtrip() {
    let nats = match start_nats_server() {
        Ok(n) => n,
        Err(err) => {
            eprintln!("Skipping test: failed to start nats-server ({err})");
            return;
        }
    };
    let device_id = "test-device";

    let nc = async_nats::ConnectOptions::with_user_and_password("auth".into(), "auth".into())
        .connect(&nats.url)
        .await
        .expect("connect nats");

    // Reuse the handlers from main.
    let started = std::time::Instant::now();
    let ping_subject = subject_ping(device_id);
    let status_subject = subject_status(device_id);
    let identity = DeviceIdentity {
        id: device_id.to_string(),
        pubkey: "PUB".to_string(),
        seed: "S".to_string(),
        network_token: None,
    };
    let hlc = Arc::new(HlcClock::new(device_id));

    let mut handles: Vec<JoinHandle<()>> = Vec::new();

    {
        let nc = nc.clone();
        let nats_name = nc.server_info().server_name.clone();
        let device_id = device_id.to_string();
        let hlc = hlc.clone();
        handles.push(tokio::spawn(async move {
            avenad::serve_ping(nc, ping_subject, device_id, nats_name, started, hlc)
                .await
                .unwrap();
        }));
    }

    {
        let nc = nc.clone();
        let hlc = hlc.clone();
        handles.push(tokio::spawn(async move {
            avenad::serve_status(nc, status_subject, started, identity, hlc)
                .await
                .unwrap();
        }));
    }

    // Ping request
    let pr = nc
        .request(subject_ping(device_id), Vec::from(PingRequest {}).into())
        .await
        .expect("ping request");
    let pong: PingResponse = pr.payload.as_ref().try_into().expect("decode ping");
    assert_eq!(pong.device, device_id);

    // Status request
    let sr = nc
        .request(subject_status(device_id), Vec::from(PingRequest {}).into())
        .await
        .expect("status request");
    let status: StatusResponse = sr.payload.as_ref().try_into().expect("decode status");
    assert_eq!(status.device, device_id);

    // Clean shutdown
    nc.flush().await.expect("flush");
    drop(nc);
    tokio::time::sleep(Duration::from_millis(50)).await;
    for handle in handles {
        handle.abort();
    }
}
