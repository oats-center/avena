use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PingRequest {}

impl From<PingRequest> for Vec<u8> {
    fn from(msg: PingRequest) -> Self {
        serde_json::to_vec(&msg).unwrap()
    }
}

impl TryFrom<&[u8]> for PingRequest {
    type Error = serde_json::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        serde_json::from_slice(value)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PingResponse {
    pub device: String,
    pub avena_version: String,
}

impl From<PingResponse> for Vec<u8> {
    fn from(msg: PingResponse) -> Self {
        serde_json::to_vec(&msg).unwrap()
    }
}

impl TryFrom<&[u8]> for PingResponse {
    type Error = serde_json::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        serde_json::from_slice(value)
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Device {
    pub id: String,
    pub version: String,
}

impl From<Device> for Vec<u8> {
    fn from(msg: Device) -> Self {
        serde_json::to_vec(&msg).unwrap()
    }
}

impl TryFrom<&[u8]> for Device {
    type Error = serde_json::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        serde_json::from_slice(value)
    }
}
