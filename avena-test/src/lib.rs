//! Test harness for Avena distributed device management.
//!
//! This crate provides infrastructure for testing Avena components:
//!
//! - [`cluster::TestCluster`] - Spawn multi-node NATS clusters for integration tests
//! - [`chaos`] - Toxiproxy client for fault injection (requires `chaos` feature)
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use avena_test::cluster::TestCluster;
//!
//! #[tokio::test]
//! async fn test_message_routing() {
//!     let cluster = TestCluster::with_hub(2).unwrap();
//!     let nc1 = cluster.connect_nats("node1").await.unwrap();
//!     let nc2 = cluster.connect_nats("node2").await.unwrap();
//!     // Messages from node1 reach node2 via hub
//! }
//! ```
//!
//! # Features
//!
//! - `chaos` - Enable Toxiproxy client for network fault injection

pub mod cluster;

#[cfg(feature = "chaos")]
pub mod chaos;
