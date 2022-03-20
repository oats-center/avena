use std::collections::HashMap;

use crate::messages::{Device, PingRequest, PingResponse};

use super::Avena;

impl Avena {
    pub fn ping(&self, device: &str) -> PingResponse {
        let msg = self
            .nc
            .request(&format!("avena.ping.{}", device), Vec::from(PingRequest {}))
            .unwrap();

        msg.data.as_slice().try_into().unwrap()
    }

    pub fn get_devices(&self) -> HashMap<String, Device> {
        let context = nats::jetstream::new(self.nc.clone());

        let kv = context.key_value("avena_devices").unwrap();

        let mut devices = HashMap::new();

        for key in kv.keys().unwrap() {
            let a = kv.get(&key).unwrap().unwrap();

            devices.insert(key, Device::try_from(a.as_slice()).unwrap());
        }

        devices
    }
}
