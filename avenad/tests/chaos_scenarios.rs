//! Chaos testing scenarios.
//!
//! These tests exercise network fault tolerance without requiring
//! external Toxiproxy (those tests live in the Tier 1 harness).

use std::time::Duration;
use futures::StreamExt;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_message_delivery_with_latency() {
    let cluster = avena_test::cluster::TestCluster::with_hub(2).unwrap();

    let nc1 = cluster.connect_nats("node1").await.unwrap();
    let nc2 = cluster.connect_nats("node2").await.unwrap();

    let mut sub = nc2.subscribe("latency.test").await.unwrap();

    let start = std::time::Instant::now();
    nc1.publish("latency.test", "test message".into())
        .await
        .unwrap();
    nc1.flush().await.unwrap();

    let msg = tokio::time::timeout(Duration::from_secs(5), sub.next())
        .await
        .expect("timeout")
        .expect("no message");

    let elapsed = start.elapsed();
    assert_eq!(msg.payload.as_ref(), b"test message");
    assert!(
        elapsed < Duration::from_secs(1),
        "Without chaos, message should arrive quickly"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_request_timeout_handling() {
    let cluster = avena_test::cluster::TestCluster::with_hub(2).unwrap();

    let nc1 = cluster.connect_nats("node1").await.unwrap();

    let result = tokio::time::timeout(
        Duration::from_millis(500),
        nc1.request("nonexistent.service", "request".into()),
    )
    .await;

    assert!(result.is_err(), "Request to nonexistent service should timeout");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_reconnection_after_brief_disconnect() {
    let cluster = avena_test::cluster::TestCluster::with_hub(2).unwrap();

    let nc1 = cluster.connect_nats("node1").await.unwrap();
    let nc2 = cluster.connect_nats("node2").await.unwrap();

    let mut sub = nc2.subscribe("reconnect.test").await.unwrap();

    nc1.publish("reconnect.test", "before".into()).await.unwrap();
    nc1.flush().await.unwrap();

    let msg1 = tokio::time::timeout(Duration::from_secs(2), sub.next())
        .await
        .expect("timeout")
        .expect("no message");
    assert_eq!(msg1.payload.as_ref(), b"before");

    tokio::time::sleep(Duration::from_millis(100)).await;

    nc1.publish("reconnect.test", "after".into()).await.unwrap();
    nc1.flush().await.unwrap();

    let msg2 = tokio::time::timeout(Duration::from_secs(2), sub.next())
        .await
        .expect("timeout")
        .expect("no message");
    assert_eq!(msg2.payload.as_ref(), b"after");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_multiple_subscribers_receive_message() {
    let cluster = avena_test::cluster::TestCluster::with_hub(3).unwrap();

    let nc1 = cluster.connect_nats("node1").await.unwrap();
    let nc2 = cluster.connect_nats("node2").await.unwrap();
    let nc3 = cluster.connect_nats("node3").await.unwrap();

    let mut sub2 = nc2.subscribe("fanout.test").await.unwrap();
    let mut sub3 = nc3.subscribe("fanout.test").await.unwrap();

    tokio::time::sleep(Duration::from_millis(100)).await;

    nc1.publish("fanout.test", "broadcast".into()).await.unwrap();
    nc1.flush().await.unwrap();

    let msg2 = tokio::time::timeout(Duration::from_secs(2), sub2.next())
        .await
        .expect("timeout node2")
        .expect("no message node2");

    let msg3 = tokio::time::timeout(Duration::from_secs(2), sub3.next())
        .await
        .expect("timeout node3")
        .expect("no message node3");

    assert_eq!(msg2.payload.as_ref(), b"broadcast");
    assert_eq!(msg3.payload.as_ref(), b"broadcast");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_queue_group_load_balancing() {
    let cluster = avena_test::cluster::TestCluster::with_hub(3).unwrap();

    let nc1 = cluster.connect_nats("node1").await.unwrap();
    let nc2 = cluster.connect_nats("node2").await.unwrap();
    let nc3 = cluster.connect_nats("node3").await.unwrap();

    let mut sub2 = nc2
        .queue_subscribe("queue.test", "workers".to_string())
        .await
        .unwrap();
    let mut sub3 = nc3
        .queue_subscribe("queue.test", "workers".to_string())
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(100)).await;

    for i in 0..10 {
        nc1.publish("queue.test", format!("job-{}", i).into())
            .await
            .unwrap();
    }
    nc1.flush().await.unwrap();

    let mut count2 = 0;
    let mut count3 = 0;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(3);

    while tokio::time::Instant::now() < deadline && (count2 + count3) < 10 {
        tokio::select! {
            Some(_) = sub2.next() => count2 += 1,
            Some(_) = sub3.next() => count3 += 1,
            _ = tokio::time::sleep(Duration::from_millis(100)) => {}
        }
    }

    assert_eq!(count2 + count3, 10, "All messages should be received");
}
