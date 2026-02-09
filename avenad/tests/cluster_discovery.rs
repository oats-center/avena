use std::time::Duration;
use avena::messages::{Announce, ANNOUNCE_SUBJECT};
use futures::StreamExt;

/// Test that messages can be routed between nodes via the hub.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_pub_sub_routing_via_hub() {
    let cluster = avena_test::cluster::TestCluster::with_hub(2).unwrap();

    let nc1 = cluster.connect_nats("node1").await.unwrap();
    let nc2 = cluster.connect_nats("node2").await.unwrap();

    let mut sub = nc2.subscribe("test.routing").await.unwrap();

    nc1.publish("test.routing", "hello from node1".into())
        .await
        .unwrap();
    nc1.flush().await.unwrap();

    let msg = tokio::time::timeout(Duration::from_secs(3), sub.next())
        .await
        .expect("timeout waiting for message")
        .expect("no message received");

    assert_eq!(msg.payload.as_ref(), b"hello from node1");
}

/// Test that announce messages propagate across the cluster.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_announce_propagation() {
    let cluster = avena_test::cluster::TestCluster::with_hub(2).unwrap();

    let nc1 = cluster.connect_nats("node1").await.unwrap();
    let nc2 = cluster.connect_nats("node2").await.unwrap();

    let mut sub = nc2.subscribe(ANNOUNCE_SUBJECT).await.unwrap();

    let announce = Announce {
        device: "device1".to_string(),
        avena_version: "0.1.0".to_string(),
        uptime_ms: 1000,
        nats_name: "test-nats".to_string(),
        pubkey: Some("PUBKEY123".to_string()),
    };

    nc1.publish(ANNOUNCE_SUBJECT, Vec::from(announce.clone()).into())
        .await
        .unwrap();
    nc1.flush().await.unwrap();

    let msg = tokio::time::timeout(Duration::from_secs(3), sub.next())
        .await
        .expect("timeout waiting for announce")
        .expect("no announce received");

    let received: Announce = msg.payload.as_ref().try_into().expect("parse announce");
    assert_eq!(received.device, "device1");
    assert_eq!(received.pubkey, Some("PUBKEY123".to_string()));
}

/// Test request/reply pattern across nodes.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_request_reply_across_nodes() {
    let cluster = avena_test::cluster::TestCluster::with_hub(2).unwrap();

    let nc1 = cluster.connect_nats("node1").await.unwrap();
    let nc2 = cluster.connect_nats("node2").await.unwrap();

    let mut sub = nc2.subscribe("test.request").await.unwrap();

    let nc2_clone = nc2.clone();
    tokio::spawn(async move {
        while let Some(msg) = sub.next().await {
            if let Some(reply) = msg.reply {
                nc2_clone
                    .publish(reply, "response from node2".into())
                    .await
                    .unwrap();
            }
        }
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let response = tokio::time::timeout(
        Duration::from_secs(3),
        nc1.request("test.request", "request from node1".into()),
    )
    .await
    .expect("timeout")
    .expect("request failed");

    assert_eq!(response.payload.as_ref(), b"response from node2");
}

/// Test that independent nodes (no hub) cannot communicate.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_isolated_nodes() {
    let cluster = avena_test::cluster::TestCluster::new(2).unwrap();

    let nc1 = cluster.connect_nats("node1").await.unwrap();
    let nc2 = cluster.connect_nats("node2").await.unwrap();

    let mut sub = nc2.subscribe("test.isolated").await.unwrap();

    nc1.publish("test.isolated", "should not arrive".into())
        .await
        .unwrap();
    nc1.flush().await.unwrap();

    let result = tokio::time::timeout(Duration::from_millis(500), sub.next()).await;

    assert!(result.is_err(), "Message should not propagate between isolated nodes");
}
