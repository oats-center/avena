use std::collections::HashMap;

use crate::messages::{Device, PingRequest, PingResponse};

use super::Avena;

const KV_DEVICES: &str = "avena_devices";

impl Avena {
    pub fn ping(&self, device: &str) -> PingResponse {
        let msg = self
            .nc
            .request(&format!("avena.ping.{}", device), Vec::from(PingRequest {}))
            .unwrap();

        msg.data.as_slice().try_into().unwrap()
    }

    pub fn get_devices(&self) -> HashMap<String, Device> {
        let kv = self.js.key_value(KV_DEVICES).unwrap();

        let mut devices = HashMap::new();
        for key in kv.keys().unwrap() {
            let device = kv.get(&key).unwrap();

            if let Some(device) = device {
                devices.insert(key, Device::try_from(device.as_slice()).unwrap());
            }
        }

        devices
    }
}
