use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

const HLC_HEADER: &str = "Avena-HLC";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HybridTimestamp {
    pub wall_time_ms: u64,
    pub counter: u32,
    pub node_id: String,
}

impl HybridTimestamp {
    pub fn now(node_id: &str, last: Option<&HybridTimestamp>) -> Self {
        let wall = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        match last {
            Some(prev) => {
                if wall > prev.wall_time_ms {
                    HybridTimestamp {
                        wall_time_ms: wall,
                        counter: 0,
                        node_id: node_id.to_string(),
                    }
                } else {
                    HybridTimestamp {
                        wall_time_ms: prev.wall_time_ms,
                        counter: prev.counter.saturating_add(1),
                        node_id: node_id.to_string(),
                    }
                }
            }
            None => HybridTimestamp {
                wall_time_ms: wall,
                counter: 0,
                node_id: node_id.to_string(),
            },
        }
    }

    pub fn is_newer_than(&self, other: &HybridTimestamp) -> bool {
        matches!(self.cmp(other), Ordering::Greater)
    }

    pub fn merge(&self, other: &HybridTimestamp, node_id: &str) -> Self {
        let wall = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        let max_wall = wall.max(self.wall_time_ms).max(other.wall_time_ms);

        let counter = if max_wall == wall && wall > self.wall_time_ms && wall > other.wall_time_ms {
            0
        } else if max_wall == self.wall_time_ms && self.wall_time_ms == other.wall_time_ms {
            self.counter.max(other.counter).saturating_add(1)
        } else if max_wall == self.wall_time_ms {
            self.counter.saturating_add(1)
        } else {
            other.counter.saturating_add(1)
        };

        HybridTimestamp {
            wall_time_ms: max_wall,
            counter,
            node_id: node_id.to_string(),
        }
    }
}

impl Ord for HybridTimestamp {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.wall_time_ms.cmp(&other.wall_time_ms) {
            Ordering::Equal => match self.counter.cmp(&other.counter) {
                Ordering::Equal => self.node_id.cmp(&other.node_id),
                ord => ord,
            },
            ord => ord,
        }
    }
}

impl PartialOrd for HybridTimestamp {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl std::fmt::Display for HybridTimestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}@{}", self.wall_time_ms, self.counter, self.node_id)
    }
}

impl std::str::FromStr for HybridTimestamp {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.splitn(3, |c| c == ':' || c == '@').collect();
        if parts.len() != 3 {
            return Err(format!("Invalid HLC format: {}", s));
        }
        Ok(HybridTimestamp {
            wall_time_ms: parts[0].parse().map_err(|e| format!("{}", e))?,
            counter: parts[1].parse().map_err(|e| format!("{}", e))?,
            node_id: parts[2].to_string(),
        })
    }
}

#[derive(Clone)]
pub struct HlcClock {
    node_id: String,
    state: Arc<Mutex<HybridTimestamp>>,
}

impl HlcClock {
    pub fn new(node_id: &str) -> Self {
        let initial = HybridTimestamp::now(node_id, None);
        HlcClock {
            node_id: node_id.to_string(),
            state: Arc::new(Mutex::new(initial)),
        }
    }

    pub fn from_persisted(node_id: &str, persisted: HybridTimestamp) -> Self {
        let merged = HybridTimestamp::now(node_id, Some(&persisted));
        HlcClock {
            node_id: node_id.to_string(),
            state: Arc::new(Mutex::new(merged)),
        }
    }

    pub fn load_or_new(node_id: &str, path: &std::path::Path) -> Self {
        if let Ok(contents) = std::fs::read_to_string(path) {
            if let Ok(persisted) = serde_json::from_str::<HybridTimestamp>(&contents) {
                return Self::from_persisted(node_id, persisted);
            }
        }
        Self::new(node_id)
    }

    pub fn save(&self, path: &std::path::Path) -> std::io::Result<()> {
        let state = self.state.lock().unwrap();
        let json = serde_json::to_string(&*state).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::Other, e)
        })?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, json)
    }

    pub fn tick(&self) -> HybridTimestamp {
        let mut state = self.state.lock().unwrap();
        let new_ts = HybridTimestamp::now(&self.node_id, Some(&state));
        *state = new_ts.clone();
        new_ts
    }

    pub fn receive(&self, remote: &HybridTimestamp) -> HybridTimestamp {
        let mut state = self.state.lock().unwrap();
        let merged = state.merge(remote, &self.node_id);
        *state = merged.clone();
        merged
    }

    pub fn current(&self) -> HybridTimestamp {
        self.state.lock().unwrap().clone()
    }

    pub fn node_id(&self) -> &str {
        &self.node_id
    }

    pub fn attach_to_headers(&self, headers: &mut async_nats::HeaderMap) {
        let ts = self.tick();
        headers.insert(HLC_HEADER, ts.to_string().as_str());
    }

    pub fn extract_and_merge(&self, headers: Option<&async_nats::HeaderMap>) -> Option<HybridTimestamp> {
        let headers = headers?;
        let value = headers.get(HLC_HEADER)?;
        let remote: HybridTimestamp = value.as_str().parse().ok()?;
        Some(self.receive(&remote))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_now_without_prior() {
        let ts = HybridTimestamp::now("node1", None);
        assert!(ts.wall_time_ms > 0);
        assert_eq!(ts.counter, 0);
        assert_eq!(ts.node_id, "node1");
    }

    #[test]
    fn test_now_with_prior_older() {
        let prior = HybridTimestamp {
            wall_time_ms: 1000,
            counter: 5,
            node_id: "old".to_string(),
        };
        let ts = HybridTimestamp::now("node1", Some(&prior));
        assert!(ts.wall_time_ms > prior.wall_time_ms);
        assert_eq!(ts.counter, 0);
    }

    #[test]
    fn test_now_with_prior_future() {
        let prior = HybridTimestamp {
            wall_time_ms: u64::MAX - 1000,
            counter: 5,
            node_id: "future".to_string(),
        };
        let ts = HybridTimestamp::now("node1", Some(&prior));
        assert_eq!(ts.wall_time_ms, prior.wall_time_ms);
        assert_eq!(ts.counter, 6);
    }

    #[test]
    fn test_ordering() {
        let ts1 = HybridTimestamp {
            wall_time_ms: 1000,
            counter: 0,
            node_id: "a".to_string(),
        };
        let ts2 = HybridTimestamp {
            wall_time_ms: 1000,
            counter: 1,
            node_id: "a".to_string(),
        };
        let ts3 = HybridTimestamp {
            wall_time_ms: 1001,
            counter: 0,
            node_id: "a".to_string(),
        };

        assert!(ts1 < ts2);
        assert!(ts2 < ts3);
        assert!(ts1 < ts3);
    }

    #[test]
    fn test_node_id_tiebreaker() {
        let ts1 = HybridTimestamp {
            wall_time_ms: 1000,
            counter: 0,
            node_id: "a".to_string(),
        };
        let ts2 = HybridTimestamp {
            wall_time_ms: 1000,
            counter: 0,
            node_id: "b".to_string(),
        };

        assert!(ts1 < ts2);
        assert!(ts2.is_newer_than(&ts1));
    }
}
