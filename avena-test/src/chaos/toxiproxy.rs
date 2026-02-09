use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, thiserror::Error)]
pub enum ToxiproxyError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Toxiproxy API error: {status} - {message}")]
    Api { status: u16, message: String },
}

pub struct Toxiproxy {
    base_url: String,
    client: reqwest::Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proxy {
    pub name: String,
    pub listen: String,
    pub upstream: String,
    #[serde(default)]
    pub enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Upstream,
    Downstream,
}

impl Default for Direction {
    fn default() -> Self {
        Self::Downstream
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Toxic {
    Latency {
        #[serde(flatten)]
        attributes: LatencyAttributes,
        #[serde(default)]
        stream: Direction,
    },
    Timeout {
        #[serde(flatten)]
        attributes: TimeoutAttributes,
        #[serde(default)]
        stream: Direction,
    },
    Bandwidth {
        #[serde(flatten)]
        attributes: BandwidthAttributes,
        #[serde(default)]
        stream: Direction,
    },
    SlowClose {
        #[serde(flatten)]
        attributes: SlowCloseAttributes,
        #[serde(default)]
        stream: Direction,
    },
    LimitData {
        #[serde(flatten)]
        attributes: LimitDataAttributes,
        #[serde(default)]
        stream: Direction,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyAttributes {
    pub latency: u32,
    #[serde(default)]
    pub jitter: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutAttributes {
    pub timeout: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthAttributes {
    pub rate: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlowCloseAttributes {
    pub delay: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitDataAttributes {
    pub bytes: u64,
}

#[derive(Debug, Deserialize)]
struct ToxicResponse {
    name: String,
}

impl Toxic {
    /// Create a latency toxic (adds delay to connections).
    pub fn latency(latency_ms: u32, jitter_ms: u32, direction: Direction) -> Self {
        Self::Latency {
            attributes: LatencyAttributes {
                latency: latency_ms,
                jitter: jitter_ms,
            },
            stream: direction,
        }
    }

    /// Create a timeout toxic (stops all data from flowing, simulating partition).
    /// Use timeout=0 for infinite timeout (complete partition).
    pub fn timeout(timeout_ms: u32, direction: Direction) -> Self {
        Self::Timeout {
            attributes: TimeoutAttributes { timeout: timeout_ms },
            stream: direction,
        }
    }

    /// Create a bandwidth toxic (limits throughput in KB/s).
    pub fn bandwidth(rate_kb: u32, direction: Direction) -> Self {
        Self::Bandwidth {
            attributes: BandwidthAttributes { rate: rate_kb },
            stream: direction,
        }
    }

    /// Create a slow_close toxic (delays closing connections).
    pub fn slow_close(delay_ms: u32, direction: Direction) -> Self {
        Self::SlowClose {
            attributes: SlowCloseAttributes { delay: delay_ms },
            stream: direction,
        }
    }

    /// Create a limit_data toxic (closes connection after N bytes).
    pub fn limit_data(bytes: u64, direction: Direction) -> Self {
        Self::LimitData {
            attributes: LimitDataAttributes { bytes },
            stream: direction,
        }
    }
}

impl Toxiproxy {
    /// Create a new Toxiproxy client.
    /// Default URL is http://localhost:8474
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client: reqwest::Client::new(),
        }
    }

    /// Create a new Toxiproxy client with default localhost URL.
    pub fn localhost() -> Self {
        Self::new("http://localhost:8474")
    }

    /// List all proxies.
    pub async fn list_proxies(&self) -> Result<HashMap<String, Proxy>, ToxiproxyError> {
        let resp = self
            .client
            .get(format!("{}/proxies", self.base_url))
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(ToxiproxyError::Api {
                status: resp.status().as_u16(),
                message: resp.text().await.unwrap_or_default(),
            });
        }

        Ok(resp.json().await?)
    }

    /// Create a new proxy.
    pub async fn create_proxy(&self, proxy: &Proxy) -> Result<Proxy, ToxiproxyError> {
        let resp = self
            .client
            .post(format!("{}/proxies", self.base_url))
            .json(proxy)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(ToxiproxyError::Api {
                status: resp.status().as_u16(),
                message: resp.text().await.unwrap_or_default(),
            });
        }

        Ok(resp.json().await?)
    }

    /// Get a proxy by name.
    pub async fn get_proxy(&self, name: &str) -> Result<Proxy, ToxiproxyError> {
        let resp = self
            .client
            .get(format!("{}/proxies/{}", self.base_url, name))
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(ToxiproxyError::Api {
                status: resp.status().as_u16(),
                message: resp.text().await.unwrap_or_default(),
            });
        }

        Ok(resp.json().await?)
    }

    /// Delete a proxy.
    pub async fn delete_proxy(&self, name: &str) -> Result<(), ToxiproxyError> {
        let resp = self
            .client
            .delete(format!("{}/proxies/{}", self.base_url, name))
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(ToxiproxyError::Api {
                status: resp.status().as_u16(),
                message: resp.text().await.unwrap_or_default(),
            });
        }

        Ok(())
    }

    /// Add a toxic to a proxy. Returns the toxic name.
    pub async fn add_toxic(&self, proxy_name: &str, toxic: Toxic) -> Result<String, ToxiproxyError> {
        let resp = self
            .client
            .post(format!("{}/proxies/{}/toxics", self.base_url, proxy_name))
            .json(&toxic)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(ToxiproxyError::Api {
                status: resp.status().as_u16(),
                message: resp.text().await.unwrap_or_default(),
            });
        }

        let toxic_resp: ToxicResponse = resp.json().await?;
        Ok(toxic_resp.name)
    }

    /// Remove a toxic from a proxy.
    pub async fn remove_toxic(&self, proxy_name: &str, toxic_name: &str) -> Result<(), ToxiproxyError> {
        let resp = self
            .client
            .delete(format!(
                "{}/proxies/{}/toxics/{}",
                self.base_url, proxy_name, toxic_name
            ))
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(ToxiproxyError::Api {
                status: resp.status().as_u16(),
                message: resp.text().await.unwrap_or_default(),
            });
        }

        Ok(())
    }

    /// Reset Toxiproxy - remove all proxies and toxics.
    pub async fn reset(&self) -> Result<(), ToxiproxyError> {
        let resp = self
            .client
            .post(format!("{}/reset", self.base_url))
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(ToxiproxyError::Api {
                status: resp.status().as_u16(),
                message: resp.text().await.unwrap_or_default(),
            });
        }

        Ok(())
    }

    /// Enable or disable a proxy.
    pub async fn set_proxy_enabled(&self, name: &str, enabled: bool) -> Result<(), ToxiproxyError> {
        #[derive(Serialize)]
        struct Update {
            enabled: bool,
        }

        let resp = self
            .client
            .post(format!("{}/proxies/{}", self.base_url, name))
            .json(&Update { enabled })
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(ToxiproxyError::Api {
                status: resp.status().as_u16(),
                message: resp.text().await.unwrap_or_default(),
            });
        }

        Ok(())
    }
}

impl Proxy {
    /// Create a new proxy configuration.
    pub fn new(name: impl Into<String>, listen: impl Into<String>, upstream: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            listen: listen.into(),
            upstream: upstream.into(),
            enabled: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toxic_serialization() {
        let toxic = Toxic::latency(100, 20, Direction::Downstream);
        let json = serde_json::to_string(&toxic).unwrap();
        assert!(json.contains("\"type\":\"latency\""));
        assert!(json.contains("\"latency\":100"));
        assert!(json.contains("\"jitter\":20"));
    }

    #[test]
    fn test_proxy_new() {
        let proxy = Proxy::new("test", "localhost:5555", "localhost:4222");
        assert_eq!(proxy.name, "test");
        assert_eq!(proxy.listen, "localhost:5555");
        assert_eq!(proxy.upstream, "localhost:4222");
        assert!(proxy.enabled);
    }
}
