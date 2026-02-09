//! Chaos engineering tools for network fault injection.
//!
//! Provides a [`Toxiproxy`] client for injecting network faults:
//! - Latency and jitter
//! - Packet loss and corruption
//! - Bandwidth throttling
//! - Connection timeouts (partitions)
//!
//! # Example
//!
//! ```rust,ignore
//! use avena_test::chaos::{Toxiproxy, Proxy, Toxic, Direction};
//!
//! let proxy = Toxiproxy::localhost();
//!
//! // Create proxy
//! proxy.create_proxy(&Proxy::new("my-proxy", "localhost:5555", "localhost:4222")).await?;
//!
//! // Add 100ms latency
//! proxy.add_toxic("my-proxy", Toxic::latency(100, 20, Direction::Downstream)).await?;
//!
//! // Simulate partition
//! proxy.add_toxic("my-proxy", Toxic::timeout(0, Direction::Upstream)).await?;
//!
//! // Reset everything
//! proxy.reset().await?;
//! ```

mod toxiproxy;

pub use toxiproxy::{Direction, Proxy, Toxic, Toxiproxy, ToxiproxyError};
