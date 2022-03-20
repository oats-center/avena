use nats::{connect, Connection};

pub mod messages;

pub mod devices;

pub struct Avena {
    nc: Connection,
}

impl Avena {
    pub fn connect(connection_urls: &str) -> Self {
        Avena {
            // FIXME: Need library errors
            nc: connect(connection_urls).unwrap(),
        }
    }

    pub fn nc(&self) -> Connection {
        // NATS clone is fast
        self.nc.clone()
    }
}
