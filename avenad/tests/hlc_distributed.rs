use std::time::Duration;
use avena::hlc::HlcClock;
use futures::StreamExt;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_hlc_sync_across_nodes() {
    let cluster = avena_test::cluster::TestCluster::with_hub(2).unwrap();

    let nc1 = cluster.connect_nats("node1").await.unwrap();
    let nc2 = cluster.connect_nats("node2").await.unwrap();

    let hlc1 = HlcClock::new("node1");
    let hlc2 = HlcClock::new("node2");

    let mut sub = nc2.subscribe("hlc.test").await.unwrap();

    for i in 0..5 {
        let mut headers = async_nats::HeaderMap::new();
        hlc1.attach_to_headers(&mut headers);

        nc1.publish_with_headers("hlc.test", headers, format!("msg-{}", i).into())
            .await
            .unwrap();
    }
    nc1.flush().await.unwrap();

    let mut received_count = 0;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(3);

    while tokio::time::Instant::now() < deadline {
        match tokio::time::timeout(Duration::from_millis(500), sub.next()).await {
            Ok(Some(msg)) => {
                hlc2.extract_and_merge(msg.headers.as_ref());
                received_count += 1;
                if received_count >= 5 {
                    break;
                }
            }
            _ => break,
        }
    }

    assert_eq!(received_count, 5, "Should receive all 5 messages");

    let ts1 = hlc1.current();
    let ts2 = hlc2.current();

    assert!(
        ts2.wall_time_ms >= ts1.wall_time_ms || ts2.counter > 0,
        "node2 HLC should have advanced from syncing with node1"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_hlc_causal_ordering() {
    let cluster = avena_test::cluster::TestCluster::with_hub(3).unwrap();

    let nc1 = cluster.connect_nats("node1").await.unwrap();
    let nc2 = cluster.connect_nats("node2").await.unwrap();
    let nc3 = cluster.connect_nats("node3").await.unwrap();

    let hlc1 = HlcClock::new("node1");
    let hlc2 = HlcClock::new("node2");
    let hlc3 = HlcClock::new("node3");

    let mut sub2 = nc2.subscribe("chain.>").await.unwrap();
    let mut sub3 = nc3.subscribe("chain.>").await.unwrap();

    let mut headers1 = async_nats::HeaderMap::new();
    hlc1.attach_to_headers(&mut headers1);
    nc1.publish_with_headers("chain.step1", headers1, "from-node1".into())
        .await
        .unwrap();
    nc1.flush().await.unwrap();

    let msg_at_2 = tokio::time::timeout(Duration::from_secs(2), sub2.next())
        .await
        .expect("timeout")
        .expect("no message");
    hlc2.extract_and_merge(msg_at_2.headers.as_ref());

    let mut headers2 = async_nats::HeaderMap::new();
    hlc2.attach_to_headers(&mut headers2);
    nc2.publish_with_headers("chain.step2", headers2, "from-node2".into())
        .await
        .unwrap();
    nc2.flush().await.unwrap();

    let msg_at_3 = tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            if let Some(msg) = sub3.next().await {
                if msg.subject.as_str() == "chain.step2" {
                    return msg;
                }
            }
        }
    })
    .await
    .expect("timeout waiting for chain.step2");
    hlc3.extract_and_merge(msg_at_3.headers.as_ref());

    let ts1 = hlc1.current();
    let ts3 = hlc3.current();

    assert!(
        ts3.is_newer_than(&ts1) || ts3.wall_time_ms > ts1.wall_time_ms,
        "node3 timestamp should reflect causal chain: {:?} vs {:?}",
        ts3,
        ts1
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_hlc_concurrent_events() {
    let cluster = avena_test::cluster::TestCluster::with_hub(2).unwrap();

    let _nc1 = cluster.connect_nats("node1").await.unwrap();
    let _nc2 = cluster.connect_nats("node2").await.unwrap();

    let hlc1 = HlcClock::new("node1");
    let hlc2 = HlcClock::new("node2");

    let ts1 = hlc1.tick();
    let ts2 = hlc2.tick();

    let ordering_defined = ts1.is_newer_than(&ts2) || ts2.is_newer_than(&ts1) || ts1 == ts2;
    assert!(
        ordering_defined,
        "HLC should provide total ordering even for concurrent events"
    );

    if ts1.wall_time_ms == ts2.wall_time_ms && ts1.counter == ts2.counter {
        assert_ne!(
            ts1.node_id, ts2.node_id,
            "Node IDs should differ for concurrent timestamps"
        );
    }
}
