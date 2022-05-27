use nats::{connect, jetstream, jetstream::JetStream, Connection};

pub mod messages;

pub mod devices;

pub struct Avena {
    nc: Connection,
    js: JetStream,
}

impl Avena {
    pub fn connect(connection_urls: &str) -> Self {
        // FIXME: Need library errors
        let nc = connect(connection_urls).unwrap();
        let js = jetstream::new(nc.clone());

        Avena { nc, js }
    }

    pub fn nc(&self) -> Connection {
        // NATS clone is fast
        self.nc.clone()
    }

    pub fn js(&self) -> JetStream {
        self.js.clone()
    }
}
