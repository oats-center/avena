mod systemd;

use std::time::Duration;

use data_encoding::BASE64URL_NOPAD;

use jsonwebtoken::{decode_header, Algorithm, Validation};
use serde::{Deserialize, Serialize};
use tokio::fs;

use color_print::cprintln;
use systemd::manager::{self, Systemd1ManagerProxy};

use zbus::Connection;

use color_eyre::Result;
//use nats::connect;

use askama::Template;

use futures::StreamExt;

#[derive(Template)]
#[template(path = "podman/template.service", escape = "none")]
struct PodmanServiceTemplate<'a> {
    description: &'a str,
    name: &'a str,
    image: &'a str,
    tag: &'a str,
    exec: &'a str,
    ports: Vec<PodmanServiceTemplatePort>,
    bind_mounts: Vec<PodmanServiceTemplateBindMount<'a>>,
    volume_mounts: Vec<PodmanServiceTemplateVolumeMount<'a>>,
}

struct PodmanServiceTemplatePort {
    inside: u16,
    outside: u16,
}

struct PodmanServiceTemplateBindMount<'a> {
    source: &'a str,
    path: &'a str,
}

struct PodmanServiceTemplateVolumeMount<'a> {
    name: &'a str,
    path: &'a str,
}

#[derive(Template)]
#[template(path = "nats/server.conf", escape = "none")]
struct NatsServerConfTemplate<'a> {
    hostname: &'a str,
    auth_issuer: &'a str,
    js_store_dir: &'a str,
    js_max_mem: &'a str,
    js_max_file: &'a str,
    js_domain: &'a str,
    remotes: Vec<NatsServerConfTemplateLeafNodeRemote<'a>>,
}

struct NatsServerConfTemplateLeafNodeRemote<'a> {
    url: &'a str,
    credentials: &'a str,
    account: &'a str,
}

#[derive(Debug, serde::Serialize)]
struct ServiceStatus {
    name: String,
    description: String,
    load_state: String,
    active_state: String,
    sub_state: String,
    followed_by: String,
    service_type: String,
    status: String,
    start_time: u64,
    exit_time: u64,
    pid: u32,
    memory_accounting: bool,
    memory_current: u64,
    memory_available: u64,
    cpu_accounting: bool,
    cpu_shares: u64,
    cpu_usage_n_sec: u64,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct ServiceStatusRequest {
    avena_only: bool,
}

impl Default for ServiceStatusRequest {
    fn default() -> Self {
        Self { avena_only: true }
    }
}

/*
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ServiceStartRequest {
    name: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ServiceStopRequest {
    name: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ServiceRestartRequest {
    name: String,
}
*/

// TODO: This should be a CMD line argument
static NATS: &str = "demo.nats.io";
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> Result<()> {
    greet();

    let systemd = connect_to_systemd().await?;

    // Key used to manage NATS auth
    let issuer = nkeys::KeyPair::new_account();

    // Make NATS systemd unit, template out nats config, start the service
    start_nats(systemd, &issuer.public_key()).await?;

    tokio::time::sleep(Duration::from_millis(2000)).await;

    /* NATS Auth */
    let nc = async_nats::ConnectOptions::with_user_and_password("auth".into(), "auth".into())
        .connect("localhost")
        .await?;
    // let nc = async_nats::connect_with_options("localhost").await?;
    println!("Connected to NATS!");

    let server_id = &nc.server_info().server_id;
    cprintln!("Server ID = {server_id}");
    let (_, server_key) = nkeys::from_public_key(server_id)?;

    let mut sub = nc.subscribe("$SYS.REQ.USER.AUTH").await?;
    while let Some(message) = sub.next().await {
        cprintln!("<r>Received message</r> <s>{:?}</s>", message);

        // let header = String::from_utf8(message.payload.into())?;
        // let parts = header.split(".").collect::<Vec<&str>>();
        //
        // cprintln!(
        //     "<r>parts[0]</r> = {}",
        //     String::from_utf8(BASE64URL_NOPAD.decode(parts[0].as_bytes())?)?
        // );
        // cprintln!(
        //     "<r>parts[1]</r> = {}",
        //     String::from_utf8(BASE64URL_NOPAD.decode(parts[1].as_bytes())?)?
        // );
        // // cprintln!(
        // //     "<r>parts[2]</r> = {}",
        // //     BASE64_STANDARD_NO_PAD.decode(parts[2])?
        // // );
        //
        // let sigval = format!(
        //     "{}.{}",
        //     String::from_utf8(BASE64URL_NOPAD.decode(parts[0].as_bytes())?)?,
        //     String::from_utf8(BASE64URL_NOPAD.decode(parts[1].as_bytes())?)?
        // );
        // cprintln!("sigval = {}", sigval);
        //
        // let sigval = format!("{}.{}", parts[0], parts[1]);
        // cprintln!("sigval = {}", sigval);
        //
        // // server_key.verify(
        // //     sigval.as_ref(),
        // //     BASE64URL_NOPAD.decode(parts[2].as_bytes())?.as_ref(),
        // // )?;
        // //
        #[derive(Debug, Serialize, Deserialize)]
        struct Claims {
            sub: String,
        }

        jsonwebtoken::decode::<Claims>(
            &String::from_utf8(message.payload.into())?,
            &jsonwebtoken::DecodingKey::from_ed_der(&server_key),
            &Validation::new(Algorithm::EdDSA),
        )?;

        cprintln!("<g>VALID!</g>");
    }

    /*
        let nc = connect(NATS)?;
        println!("Connected to {}", NATS);

        let sub = nc.subscribe("$avena.>")?;

        // TODO: Lack of proper error handling means all errors kill avenad
        for msg in sub {
            // TODO: Replace println! with env-logger or similar
            println!("Received request: {}", msg.subject);
            match msg.subject.as_str() {
                "$avena.services.status" => handle_service_status_request(&nc, &msg)?,
                "$avena.services.start" => handle_service_start(&nc, &msg)?,
                "$avena.services.stop" => handle_service_stop(&nc, &msg)?,
                "$avena.services.restart" => handle_service_restart(&nc, &msg)?,
                _ => {
                    println!("Unsupported API!");
                    if let Some(reply) = msg.reply {
                        nc.publish(&reply, "{\"error\": \"Unsupported API\"}")?;
                    };
                }
            }
        }

        Ok(())
    */

    Ok(())
}

fn greet() {
    const AVENA: &str = "
  __ ___   _____ _ __   __ _ 
 / _` \\ \\ / / _ \\ '_ \\ / _` |
| (_| |\\ V /  __/ | | | (_| |
 \\__,_| \\_/ \\___|_| |_|\\__,_|";

    cprintln!("<s><b>{}</b></s>", AVENA);
    cprintln!("        <s>Version {}</s>\n", VERSION);
}

async fn connect_to_systemd<'a>() -> Result<Systemd1ManagerProxy<'a>> {
    let connection = Connection::system().await?;
    cprintln!("<g>🎉 Connected to system Systemd via d-bus.</g>");

    let systemd = Systemd1ManagerProxy::new(&connection).await?;
    cprintln!("<s>Version = </s>{}", systemd.version().await?);
    cprintln!("<s>Architecture = </s>{}", systemd.architecture().await?);
    cprintln!("<s>Features = </s>{}", systemd.features().await?);
    cprintln!("<s>Progress = </s>{}", systemd.progress().await?);
    cprintln!("<s>Log Level = </s>{}", systemd.log_level().await?);
    cprintln!("<s>Log Target = </s>{}", systemd.log_target().await?);
    cprintln!(
        "<s>Default Standard Output = </s>{}",
        systemd.default_standard_output().await?
    );
    cprintln!(
        "<s>Default Standard Error = </s>{}",
        systemd.default_standard_error().await?
    );
    cprintln!("<s>Unit Paths:</s>");
    for path in systemd.unit_path().await? {
        cprintln!("\t<s>-</s> {}", path);
    }
    cprintln!("<s>Environment variables:</s>");
    for env in systemd.environment().await? {
        cprintln!("\t<s>-</s> {}", env);
    }

    Ok(systemd)
}

async fn start_nats<'a>(systemd: Systemd1ManagerProxy<'a>, issuer_pub_key: &'a str) -> Result<()> {
    // FIXME: Move to transient service?
    let nats_service = PodmanServiceTemplate {
        description: "Avena's node-local NATS",
        name: "avena-nats",
        image: "docker.io/library/nats",
        tag: "2.10.20",
        exec: "--config /server.conf",
        ports: vec![PodmanServiceTemplatePort {
            inside: 4222,
            outside: 4222,
        }],
        bind_mounts: vec![PodmanServiceTemplateBindMount {
            source: "./server.conf",
            path: "/server.conf",
        }],
        volume_mounts: vec![],
    };

    cprintln!("<g>Before</g>");
    for unit in systemd
        .list_units_by_names(vec!["avena-nats.service"])
        .await?
    {
        cprintln!("<s>-</s> {}", unit.name);
    }

    // FIXME: Change to /run/containers/systemd/* when podman > 5.2.2 is out
    fs::create_dir_all("/etc/containers/systemd/nats").await?;
    println!("Create nats folder");
    fs::write(
        format!(
            "/etc/containers/systemd/nats/{0}.container",
            nats_service.name
        ),
        nats_service.render()?,
    )
    .await?;

    let nats_conf = NatsServerConfTemplate {
        hostname: "avena",
        auth_issuer: issuer_pub_key,
        js_store_dir: "/js",
        js_max_mem: "1G",
        js_max_file: "10G",
        js_domain: "avena",
        remotes: vec![],
    };
    fs::write(
        "/etc/containers/systemd/nats/server.conf",
        nats_conf.render()?,
    )
    .await?;

    cprintln!("<g>Reloading</g>");
    systemd.reload().await?;

    let unit = systemd.get_unit("avena-nats.service").await?;

    cprintln!("ID = {}", unit.id().await?);
    cprintln!("Active State = {}", unit.active_state().await?);
    cprintln!("Need Daemon Reload = {}", unit.need_daemon_reload().await?);

    systemd.restart_unit("avena-nats.service", "fail").await?;

    Ok(())
}

/*
fn handle_service_status_request(nc: &nats::Connection, msg: &nats::Message) -> Result<()> {
    let request: ServiceStatusRequest = serde_json::from_slice(&msg.data)?;
    let manager = get_manager()?;

    let pattern = if request.avena_only {
        "avena-*.service"
    } else {
        "*.service"
    };

    let units = manager.list_units_by_patterns(vec!["active", "failed"], vec![pattern])?;

    let mut services = vec![];
    for unit in &units {
        let service = manager.get_unit(&unit.name)?;
        services.push(ServiceStatus {
            name: unit.name.clone(),
            description: unit.description.clone(),
            load_state: unit.load_state.clone(),
            active_state: unit.active_state.clone(),
            sub_state: unit.sub_state.clone(),
            followed_by: unit.followed_by.clone(),
            service_type: service.service_type()?,
            status: service.status_text()?,
            start_time: service.exec_main_start_timestamp()?,
            exit_time: service.exec_main_exit_timestamp()?,
            pid: service.exec_main_pid()?,
            memory_accounting: service.memory_accounting()?,
            memory_current: service.memory_current()?,
            memory_available: service.memory_available()?,
            cpu_accounting: service.cpu_accounting()?,
            cpu_shares: service.cpu_shares()?,
            cpu_usage_n_sec: service.cpu_usage_n_sec()?,
        });
    }

    nc.publish(
        msg.reply.as_ref().unwrap(),
        &serde_json::to_string(&services).unwrap(),
    )?;

    Ok(())
}

fn handle_service_start(nc: &nats::Connection, msg: &nats::Message) -> Result<()> {
    let request: ServiceStartRequest = serde_json::from_slice(&msg.data)?;
    let manager = get_manager()?;

    match manager.start_unit(&request.name, "replace") {
        Ok(_) => {
            nc.publish(msg.reply.as_ref().unwrap(), r#"{"result": "success"}"#)?;
        }
        Err(b) => {
            // FIXME: This likely leaks too much info
            nc.publish(
                msg.reply.as_ref().unwrap(),
                format!(
                    r#"{{
                        "result": "failure",
                        "error": "{b}"
                    }}"#
                ),
            )?;
        }
    };
    Ok(())
}

fn handle_service_stop(nc: &nats::Connection, msg: &nats::Message) -> Result<()> {
    let request: ServiceStopRequest = serde_json::from_slice(&msg.data)?;
    let manager = get_manager()?;

    match manager.stop_unit(&request.name, "replace") {
        Ok(_) => {
            nc.publish(msg.reply.as_ref().unwrap(), r#"{"result": "success"}"#)?;
        }
        Err(b) => {
            // FIXME: This likely leaks too much info
            nc.publish(
                msg.reply.as_ref().unwrap(),
                format!(
                    r#"{{
                        "result": "failure",
                        "error": "{b}"
                    }}"#
                ),
            )?;
        }
    };

    Ok(())
}

fn handle_service_restart(nc: &nats::Connection, msg: &nats::Message) -> Result<()> {
    let request: ServiceRestartRequest = serde_json::from_slice(&msg.data)?;
    let manager = get_manager()?;

    match manager.restart_unit(&request.name, "replace") {
        Ok(_) => {
            nc.publish(msg.reply.as_ref().unwrap(), r#"{"result": "success"}"#)?;
        }
        Err(b) => {
            // FIXME: This likely leaks too much info
            nc.publish(
                msg.reply.as_ref().unwrap(),
                format!(
                    r#"{{
                        "result": "failure",
                        "error": "{b}"
                    }}"#
                ),
            )?;
        }
    };

    Ok(())
}
*/
