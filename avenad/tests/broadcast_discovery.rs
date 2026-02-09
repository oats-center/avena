//! Broadcast ping and device discovery tests.
//!
//! Tests the avena client's broadcast_ping() and discover() methods
//! for finding devices across the NATS mesh.

use std::sync::Arc;
use std::time::{Duration, Instant};
use avena::hlc::HlcClock;
use avena::messages::{
    Announce, PingResponse, ANNOUNCE_SUBJECT, BROADCAST_PING_SUBJECT,
};
use futures::StreamExt;

async fn spawn_mock_device(
    nc: async_nats::Client,
    device_id: &str,
    announce_interval: Duration,
) -> tokio::task::JoinHandle<()> {
    let device_id = device_id.to_string();
    let hlc = Arc::new(HlcClock::new(&device_id));
    let started = Instant::now();

    let nc_ping = nc.clone();
    let device_id_ping = device_id.clone();
    let hlc_ping = hlc.clone();
    let ping_handle = tokio::spawn(async move {
        let subject = format!("avena.device.{}.ping", device_id_ping);
        let mut sub = nc_ping.subscribe(subject).await.unwrap();
        let mut broadcast_sub = nc_ping.subscribe(BROADCAST_PING_SUBJECT).await.unwrap();

        loop {
            tokio::select! {
                Some(msg) = sub.next() => {
                    if let Some(reply) = msg.reply {
                        hlc_ping.extract_and_merge(msg.headers.as_ref());
                        let resp = PingResponse {
                            device: device_id_ping.clone(),
                            avena_version: "0.1.0-test".to_string(),
                            uptime_ms: started.elapsed().as_millis() as u64,
                            nats_name: "test-nats".to_string(),
                        };
                        let mut headers = async_nats::HeaderMap::new();
                        hlc_ping.attach_to_headers(&mut headers);
                        let _ = nc_ping.publish_with_headers(reply, headers, Vec::from(resp).into()).await;
                    }
                }
                Some(msg) = broadcast_sub.next() => {
                    if let Some(reply) = msg.reply {
                        hlc_ping.extract_and_merge(msg.headers.as_ref());
                        let resp = PingResponse {
                            device: device_id_ping.clone(),
                            avena_version: "0.1.0-test".to_string(),
                            uptime_ms: started.elapsed().as_millis() as u64,
                            nats_name: "test-nats".to_string(),
                        };
                        let mut headers = async_nats::HeaderMap::new();
                        hlc_ping.attach_to_headers(&mut headers);
                        let _ = nc_ping.publish_with_headers(reply, headers, Vec::from(resp).into()).await;
                    }
                }
            }
        }
    });

    let nc_announce = nc.clone();
    let device_id_announce = device_id.clone();
    tokio::spawn(async move {
        let announce = Announce {
            device: device_id_announce.clone(),
            avena_version: "0.1.0-test".to_string(),
            uptime_ms: 0,
            nats_name: "test-nats".to_string(),
            pubkey: Some(format!("PUBKEY_{}", device_id_announce)),
        };
        nc_announce
            .publish(ANNOUNCE_SUBJECT, Vec::from(announce).into())
            .await
            .unwrap();

        let mut interval = tokio::time::interval(announce_interval);
        loop {
            interval.tick().await;
            let announce = Announce {
                device: device_id_announce.clone(),
                avena_version: "0.1.0-test".to_string(),
                uptime_ms: started.elapsed().as_millis() as u64,
                nats_name: "test-nats".to_string(),
                pubkey: Some(format!("PUBKEY_{}", device_id_announce)),
            };
            let _ = nc_announce
                .publish(ANNOUNCE_SUBJECT, Vec::from(announce).into())
                .await;
        }
    });

    ping_handle
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_broadcast_ping_finds_multiple_devices() {
    let cluster = avena_test::cluster::TestCluster::with_hub(3).unwrap();

    let nc1 = cluster.connect_nats("node1").await.unwrap();
    let nc2 = cluster.connect_nats("node2").await.unwrap();

    let _h1 = spawn_mock_device(nc1, "device-alpha", Duration::from_secs(60)).await;
    let _h2 = spawn_mock_device(nc2, "device-beta", Duration::from_secs(60)).await;

    tokio::time::sleep(Duration::from_millis(200)).await;

    let client = cluster.connect_avena("node3").await.unwrap();
    let responses = client.broadcast_ping(Duration::from_secs(2)).await;

    assert!(
        responses.len() >= 2,
        "Should find at least 2 devices, found {}",
        responses.len()
    );
    assert!(responses.contains_key("device-alpha"));
    assert!(responses.contains_key("device-beta"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_discover_receives_announcements() {
    let cluster = avena_test::cluster::TestCluster::with_hub(3).unwrap();

    let nc1 = cluster.connect_nats("node1").await.unwrap();
    let nc2 = cluster.connect_nats("node2").await.unwrap();

    let _h1 = spawn_mock_device(nc1, "device-one", Duration::from_millis(500)).await;
    let _h2 = spawn_mock_device(nc2, "device-two", Duration::from_millis(500)).await;

    let client = cluster.connect_avena("node3").await.unwrap();
    let discovered = client.discover(Duration::from_secs(2)).await;

    assert!(
        discovered.len() >= 2,
        "Should discover at least 2 devices, found {}",
        discovered.len()
    );
    assert!(discovered.contains_key("device-one"));
    assert!(discovered.contains_key("device-two"));

    let dev_one = &discovered["device-one"];
    assert_eq!(dev_one.pubkey, Some("PUBKEY_device-one".to_string()));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_direct_ping_specific_device() {
    let cluster = avena_test::cluster::TestCluster::with_hub(2).unwrap();

    let nc1 = cluster.connect_nats("node1").await.unwrap();
    let _h1 = spawn_mock_device(nc1, "target-device", Duration::from_secs(60)).await;

    tokio::time::sleep(Duration::from_millis(200)).await;

    let client = cluster.connect_avena("node2").await.unwrap();
    let response = client.ping("target-device").await.unwrap();

    assert_eq!(response.device, "target-device");
    assert_eq!(response.avena_version, "0.1.0-test");
}
