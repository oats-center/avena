use nats::{connect, Message};

use color_eyre::Result;

use avena::messages::{PingRequest, PingResponse};

fn main() -> Result<()> {
    println!("Hello, world!");

    let nc = connect("demo.nats.io")?;

    let ping = nc.subscribe("avena.ping.test123")?;
    let ping_ch = ping.receiver();

    let ping_all = nc.subscribe("avena.ping")?;
    let ping_all_ch = ping_all.receiver();

    loop {
        crossbeam_channel::select! {
            recv(ping_ch) -> msg => handle_ping(msg?)?,
            recv(ping_all_ch) -> msg => handle_ping(msg?)?,
        }
    }
}

fn handle_ping(msg: Message) -> Result<()> {
    let msg3 = PingRequest::try_from(msg.data.as_slice())?;
    println!("PingRequest: {:#?}", msg3);

    let r: Vec<u8> = PingResponse {
        device: "test123".to_owned(),
        avena_version: env!("CARGO_PKG_VERSION").to_owned(),
    }
    .into();

    msg.respond(&r)?;

    Ok(())
}
