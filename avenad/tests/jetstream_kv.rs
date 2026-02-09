//! JetStream KV replication tests across distributed NATS clusters.
//!
//! These tests verify that KV operations replicate correctly across
//! leaf node connections, which is critical for workload and device
//! state synchronization.

use std::time::Duration;
use async_nats::jetstream::kv;
use futures::StreamExt;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_kv_put_replicated_to_other_nodes() {
    let cluster = avena_test::cluster::TestCluster::with_hub(2).unwrap();

    let nc1 = cluster.connect_nats("node1").await.unwrap();
    let js1 = async_nats::jetstream::new(nc1);

    let kv1 = js1
        .create_key_value(kv::Config {
            bucket: "test_replication".to_string(),
            history: 5,
            ..Default::default()
        })
        .await
        .unwrap();

    kv1.put("device1", "online".into()).await.unwrap();

    tokio::time::sleep(Duration::from_millis(500)).await;

    let nc2 = cluster.connect_nats("node2").await.unwrap();
    let js2 = async_nats::jetstream::new(nc2);

    let kv2 = js2.get_key_value("test_replication").await.unwrap();

    let entry = kv2.get("device1").await.unwrap();
    assert!(entry.is_some(), "KV entry should replicate to node2");
    assert_eq!(entry.unwrap().as_ref(), b"online");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_kv_watch_receives_updates() {
    let cluster = avena_test::cluster::TestCluster::with_hub(2).unwrap();

    let nc1 = cluster.connect_nats("node1").await.unwrap();
    let js1 = async_nats::jetstream::new(nc1);

    let kv1 = js1
        .create_key_value(kv::Config {
            bucket: "test_watch".to_string(),
            history: 5,
            ..Default::default()
        })
        .await
        .unwrap();

    let nc2 = cluster.connect_nats("node2").await.unwrap();
    let js2 = async_nats::jetstream::new(nc2);

    tokio::time::sleep(Duration::from_millis(200)).await;

    let kv2 = js2.get_key_value("test_watch").await.unwrap();
    let mut watcher = kv2.watch("device.>").await.unwrap();

    kv1.put("device.dev1", "state1".into()).await.unwrap();
    kv1.put("device.dev2", "state2".into()).await.unwrap();

    let mut updates = Vec::new();
    let deadline = tokio::time::Instant::now() + Duration::from_secs(3);

    while tokio::time::Instant::now() < deadline && updates.len() < 2 {
        match tokio::time::timeout(Duration::from_millis(500), watcher.next()).await {
            Ok(Some(Ok(entry))) => {
                updates.push((entry.key.clone(), entry.value.clone()));
            }
            _ => break,
        }
    }

    assert_eq!(updates.len(), 2, "Should receive both KV updates");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_kv_history_preserved() {
    let cluster = avena_test::cluster::TestCluster::with_hub(2).unwrap();

    let nc1 = cluster.connect_nats("node1").await.unwrap();
    let js1 = async_nats::jetstream::new(nc1);

    let kv1 = js1
        .create_key_value(kv::Config {
            bucket: "test_history".to_string(),
            history: 10,
            ..Default::default()
        })
        .await
        .unwrap();

    kv1.put("workload1", "v1".into()).await.unwrap();
    kv1.put("workload1", "v2".into()).await.unwrap();
    kv1.put("workload1", "v3".into()).await.unwrap();

    tokio::time::sleep(Duration::from_millis(300)).await;

    let nc2 = cluster.connect_nats("node2").await.unwrap();
    let js2 = async_nats::jetstream::new(nc2);
    let kv2 = js2.get_key_value("test_history").await.unwrap();

    let mut history = kv2.history("workload1").await.unwrap();
    let mut versions = Vec::new();

    while let Some(Ok(entry)) = history.next().await {
        versions.push(String::from_utf8_lossy(&entry.value).to_string());
    }

    assert_eq!(versions.len(), 3, "Should have 3 history entries");
    assert!(versions.contains(&"v1".to_string()));
    assert!(versions.contains(&"v2".to_string()));
    assert!(versions.contains(&"v3".to_string()));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_kv_delete_propagates() {
    let cluster = avena_test::cluster::TestCluster::with_hub(2).unwrap();

    let nc1 = cluster.connect_nats("node1").await.unwrap();
    let js1 = async_nats::jetstream::new(nc1);

    let kv1 = js1
        .create_key_value(kv::Config {
            bucket: "test_delete".to_string(),
            ..Default::default()
        })
        .await
        .unwrap();

    kv1.put("temp_key", "value".into()).await.unwrap();

    tokio::time::sleep(Duration::from_millis(200)).await;

    let nc2 = cluster.connect_nats("node2").await.unwrap();
    let js2 = async_nats::jetstream::new(nc2);
    let kv2 = js2.get_key_value("test_delete").await.unwrap();

    let before = kv2.get("temp_key").await.unwrap();
    assert!(before.is_some(), "Key should exist before delete");

    kv1.delete("temp_key").await.unwrap();

    tokio::time::sleep(Duration::from_millis(300)).await;

    let after = kv2.get("temp_key").await.unwrap();
    assert!(after.is_none(), "Key should be deleted on node2");
}
