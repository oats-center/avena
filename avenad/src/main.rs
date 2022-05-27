mod systemd;

use color_eyre::Result;
use nats::connect;

use systemd::manager::get_manager;

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

// TODO: This should be a CMD line argument
static NATS: &str = "demo.nats.io";

fn main() -> Result<()> {
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
}

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
