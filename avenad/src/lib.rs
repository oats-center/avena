use std::env;
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use avena::hlc::HlcClock;
use avena::messages::{
    Announce, LinkRegisterRequest, LinkRegisterResponse, LinkUnregisterRequest,
    LinkUnregisterResponse, MountSpec, PermSpec, PingResponse, StatusResponse, WorkloadCommand,
    WorkloadCommandRequest, WorkloadCommandResponse, WorkloadDesiredState, WorkloadListItem,
    WorkloadSpec, WorkloadState, WorkloadStatus, WorkloadStatusLite, WorkloadsListResponse,
    ANNOUNCE_SUBJECT,
};
use color_eyre::Result;
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::process::Command;
use zbus::Connection;
use async_nats::jetstream::kv::Store as KvStore;
use async_nats::Client;
use tokio::fs;
use avena::messages::PortSpec;
use tracing::{info, warn, error};
pub mod device;
pub mod link;
pub mod nats_jwt;
pub mod workload;
pub mod systemd;
use crate::device::DeviceIdentity;
use crate::systemd::manager::Systemd1ManagerProxy;
use crate::workload::WorkloadDeployment;
use serde::{Deserialize, Serialize};
use askama::Template;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LinkEntry {
    pub url: String,
    pub creds_path: Option<String>,
    pub inline_creds: Option<String>,
}

pub const LINKS_BUCKET: &str = "avena_links";

fn nats_conf_path() -> std::path::PathBuf {
    directories::BaseDirs::new()
        .map(|d| d.config_dir().join("containers/systemd/server.conf"))
        .unwrap_or_else(|| std::path::PathBuf::from("~/.config/containers/systemd/server.conf"))
}

#[derive(Template)]
#[template(path = "nats/server.conf", escape = "none")]
struct NatsServerConfTemplate<'a> {
    hostname: &'a str,
    js_store_dir: &'a str,
    js_max_mem: &'a str,
    js_max_file: &'a str,
    js_domain: &'a str,
    sys_account_key: &'a str,
    avena_account_key: &'a str,
    sys_jwt: &'a str,
    avena_jwt: &'a str,
    remotes: Vec<NatsServerConfTemplateLeafNodeRemote<'a>>,
}

struct NatsServerConfTemplateLeafNodeRemote<'a> {
    url: &'a str,
    credentials: &'a str,
}

/// Handle link register requests (store remote targets in KV).
pub async fn serve_link_register(
    nc: Client,
    subject: String,
    kv: Arc<Mutex<KvStore>>,
    nats_url: String,
    issuer_pub_key: String,
    device: DeviceIdentity,
    hlc: Arc<HlcClock>,
) -> Result<()> {
    let mut sub = nc.subscribe(subject).await?;
    while let Some(msg) = sub.next().await {
        hlc.extract_and_merge(msg.headers.as_ref());

        if let Some(reply) = msg.reply {
            let req: LinkRegisterRequest = serde_json::from_slice(&msg.payload)?;
            let ok = link_offer_handshake(&req.remote_url, &device, &issuer_pub_key, &nats_url, &kv).await?;

            let mut headers = async_nats::HeaderMap::new();
            hlc.attach_to_headers(&mut headers);

            if ok {
                let guard = kv.lock().await;
                let _ = guard
                    .put(
                        format!("link:{}", req.remote_url),
                        serde_json::to_vec(&LinkEntry {
                            url: req.remote_url.clone(),
                            creds_path: None,
                            inline_creds: None,
                        })?
                        .into(),
                    )
                    .await;

                let resp = LinkRegisterResponse {
                    ok: true,
                    message: "stored link request".to_string(),
                };
                nc.publish_with_headers(reply, headers, Vec::from(resp).into()).await?;

                let _ = reconcile_leaves(&kv, &issuer_pub_key, &nats_url).await;
            } else {
                let resp = LinkRegisterResponse {
                    ok: false,
                    message: "link offer failed".to_string(),
                };
                nc.publish_with_headers(reply, headers, Vec::from(resp).into()).await?;
            }
        }
    }

    Ok(())
}

/// Handle link unregister requests (remove link from KV and reload NATS).
pub async fn serve_link_unregister(
    nc: Client,
    subject: String,
    kv: Arc<Mutex<KvStore>>,
    nats_url: String,
    issuer_pub_key: String,
    hlc: Arc<HlcClock>,
) -> Result<()> {
    let mut sub = nc.subscribe(subject).await?;
    while let Some(msg) = sub.next().await {
        hlc.extract_and_merge(msg.headers.as_ref());

        if let Some(reply) = msg.reply {
            let req: LinkUnregisterRequest = serde_json::from_slice(&msg.payload)?;
            let key = format!("link:{}", req.remote_url);

            let guard = kv.lock().await;
            let existed = guard.get(&key).await?.is_some();
            if existed {
                let _ = guard.delete(&key).await;
            }
            drop(guard);

            let mut headers = async_nats::HeaderMap::new();
            hlc.attach_to_headers(&mut headers);

            if existed {
                let _ = reconcile_leaves(&kv, &issuer_pub_key, &nats_url).await;
                let resp = LinkUnregisterResponse {
                    ok: true,
                    message: format!("removed link to {}", req.remote_url),
                };
                nc.publish_with_headers(reply, headers, Vec::from(resp).into()).await?;
            } else {
                let resp = LinkUnregisterResponse {
                    ok: false,
                    message: format!("no link found for {}", req.remote_url),
                };
                nc.publish_with_headers(reply, headers, Vec::from(resp).into()).await?;
            }
        }
    }

    Ok(())
}

/// Reply to ping requests on the given subject.
pub async fn serve_ping(
    nc: async_nats::Client,
    subject: String,
    device_id: String,
    nats_name: String,
    started: Instant,
    hlc: Arc<HlcClock>,
) -> Result<()> {
    let mut sub = nc.subscribe(subject).await?;

    while let Some(message) = sub.next().await {
        hlc.extract_and_merge(message.headers.as_ref());

        if let Some(reply) = message.reply {
            let resp = PingResponse {
                device: device_id.clone(),
                avena_version: env!("CARGO_PKG_VERSION").to_string(),
                uptime_ms: started.elapsed().as_millis() as u64,
                nats_name: nats_name.clone(),
            };
            let mut headers = async_nats::HeaderMap::new();
            hlc.attach_to_headers(&mut headers);
            nc.publish_with_headers(reply, headers, Vec::from(resp).into()).await?;
        }
    }

    Ok(())
}

/// Reply to status requests on the given subject.
pub async fn serve_status(
    nc: async_nats::Client,
    subject: String,
    started: Instant,
    device: DeviceIdentity,
    hlc: Arc<HlcClock>,
) -> Result<()> {
    let mut sub = nc.subscribe(subject).await?;

    while let Some(message) = sub.next().await {
        hlc.extract_and_merge(message.headers.as_ref());

        if let Some(reply) = message.reply {
            let resp = StatusResponse {
                device: device.id.clone(),
                avena_version: env!("CARGO_PKG_VERSION").to_string(),
                uptime_ms: started.elapsed().as_millis() as u64,
                workloads: current_workloads().await,
            };

            let mut headers = async_nats::HeaderMap::new();
            hlc.attach_to_headers(&mut headers);
            nc.publish_with_headers(reply, headers, Vec::from(resp).into()).await?;
        }
    }

    Ok(())
}

/// Reply to workloads list requests.
pub async fn serve_workloads_list(
    nc: async_nats::Client,
    subject: String,
    device: DeviceIdentity,
    hlc: Arc<HlcClock>,
) -> Result<()> {
    let mut sub = nc.subscribe(subject).await?;

    while let Some(message) = sub.next().await {
        hlc.extract_and_merge(message.headers.as_ref());

        if let Some(reply) = message.reply {
            let resp = WorkloadsListResponse {
                device: device.id.clone(),
                workloads: current_workloads()
                    .await
                    .into_iter()
                    .map(|state| WorkloadListItem {
                        name: state.name.clone(),
                        spec: WorkloadSpec {
                            image: state.image.clone(),
                            tag: None,
                            cmd: None,
                            args: vec![],
                            env: vec![],
                            mounts: vec![],
                            devices: vec![],
                            perms: PermSpec {
                                publish: vec![],
                                subscribe: vec![],
                            },
                            ports: vec![],
                            volumes: vec![],
                        },
                        state: WorkloadStatusLite {
                            status: state.state.clone(),
                            since: state.started_at,
                        },
                    })
                    .collect(),
            };

            let mut headers = async_nats::HeaderMap::new();
            hlc.attach_to_headers(&mut headers);
            nc.publish_with_headers(reply, headers, Vec::from(resp).into()).await?;
        }
    }

    Ok(())
}

/// Handle workload control commands.
pub async fn serve_workload_command(
    nc: async_nats::Client,
    subject: String,
    hlc: Arc<HlcClock>,
) -> Result<()> {
    let mut sub = nc.subscribe(subject).await?;

    while let Some(message) = sub.next().await {
        hlc.extract_and_merge(message.headers.as_ref());

        if let Some(reply) = message.reply {
            let req: WorkloadCommandRequest = serde_json::from_slice(&message.payload)?;
            info!("Workload command: {:?} for {}", req.command, req.workload);
            let resp =
                handle_workload_command(req)
                    .await
                    .unwrap_or_else(|e| WorkloadCommandResponse {
                        ok: false,
                        message: format!("{e:?}"),
                        logs: None,
                    });

            let mut headers = async_nats::HeaderMap::new();
            hlc.attach_to_headers(&mut headers);
            nc.publish_with_headers(reply, headers, Vec::from(resp).into()).await?;
        }
    }

    Ok(())
}

async fn handle_workload_command(req: WorkloadCommandRequest) -> Result<WorkloadCommandResponse> {
    let unit_name = format!("{}.service", req.workload);

    match req.command {
        WorkloadCommand::Start => {
            let conn = zbus::Connection::session().await?;
            let manager = Systemd1ManagerProxy::new(&conn).await?;
            manager.start_unit(&unit_name, "replace").await?;
            Ok(WorkloadCommandResponse {
                ok: true,
                message: format!("Started {}", req.workload),
                logs: None,
            })
        }
        WorkloadCommand::Stop => {
            let conn = zbus::Connection::session().await?;
            let manager = Systemd1ManagerProxy::new(&conn).await?;
            manager.stop_unit(&unit_name, "replace").await?;
            Ok(WorkloadCommandResponse {
                ok: true,
                message: format!("Stopped {}", req.workload),
                logs: None,
            })
        }
        WorkloadCommand::Restart => {
            let conn = zbus::Connection::session().await?;
            let manager = Systemd1ManagerProxy::new(&conn).await?;
            manager.restart_unit(&unit_name, "replace").await?;
            Ok(WorkloadCommandResponse {
                ok: true,
                message: format!("Restarted {}", req.workload),
                logs: None,
            })
        }
        WorkloadCommand::Logs { tail } => {
            let mut cmd = Command::new("journalctl");
            cmd.arg("-u").arg(&unit_name).arg("--no-pager");
            if let Some(lines) = tail {
                cmd.arg("-n").arg(lines.to_string());
            }
            let output = cmd.output().await?;
            let logs = String::from_utf8_lossy(&output.stdout).to_string();
            Ok(WorkloadCommandResponse {
                ok: output.status.success(),
                message: format!(
                    "journalctl exited with status {}",
                    output.status.code().unwrap_or(-1)
                ),
                logs: Some(logs),
            })
        }
    }
}
/// Periodically publish device announces.
pub async fn serve_announce(
    nc: async_nats::Client,
    device: DeviceIdentity,
    nats_name: String,
    started: Instant,
    interval_secs: u64,
    kv: Option<Arc<Mutex<KvStore>>>,
) -> Result<()> {
    let mut ticker = tokio::time::interval(Duration::from_secs(interval_secs));

    // Send one immediately for snappier discovery
    let initial = Announce {
        device: device.id.clone(),
        avena_version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_ms: started.elapsed().as_millis() as u64,
        nats_name: nats_name.clone(),
        pubkey: Some(device.pubkey.clone()),
    };
    nc.publish(ANNOUNCE_SUBJECT, Vec::from(initial).into())
        .await?;
    if let Some(kv) = kv.as_ref() {
        let guard = kv.lock().await;
        let _ = guard
            .put(
                device.id.clone(),
                serde_json::to_vec(&avena::messages::Device {
                    id: device.id.clone(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                    last_seen_ms: Some(now_millis()),
                    nats_name: Some(nats_name.clone()),
                    pubkey: Some(device.pubkey.clone()),
                })?
                .into(),
            )
            .await;
    }

    loop {
        ticker.tick().await;
        let announce = Announce {
            device: device.id.clone(),
            avena_version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_ms: started.elapsed().as_millis() as u64,
            nats_name: nats_name.clone(),
            pubkey: Some(device.pubkey.clone()),
        };

        nc.publish(ANNOUNCE_SUBJECT, Vec::from(announce).into())
            .await?;
        if let Some(kv) = kv.as_ref() {
            let guard = kv.lock().await;
            let _ = guard
                .put(
                    device.id.clone(),
                    serde_json::to_vec(&avena::messages::Device {
                        id: device.id.clone(),
                        version: env!("CARGO_PKG_VERSION").to_string(),
                        last_seen_ms: Some(now_millis()),
                        nats_name: Some(nats_name.clone()),
                        pubkey: Some(device.pubkey.clone()),
                    })?
                    .into(),
                )
                .await;
        }
    }
}

/// Subscribe to announce subjects and update local KV for seen devices.
pub async fn observe_announces(
    nc: async_nats::Client,
    kv: Arc<Mutex<KvStore>>,
) -> Result<()> {
    let mut sub = nc.subscribe(ANNOUNCE_SUBJECT).await?;
    while let Some(msg) = sub.next().await {
        if let Ok(announce) = Announce::try_from(msg.payload.as_ref()) {
            let guard = kv.lock().await;
            let _ = guard
                .put(
                    announce.device.clone(),
                    serde_json::to_vec(&avena::messages::Device {
                        id: announce.device.clone(),
                        version: announce.avena_version.clone(),
                        last_seen_ms: Some(now_millis()),
                        nats_name: Some(announce.nats_name.clone()),
                        pubkey: announce.pubkey.clone(),
                    })?
                    .into(),
                )
                .await;
        }
    }

    Ok(())
}

pub fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

async fn current_workloads() -> Vec<WorkloadState> {
    let conn = match Connection::session().await {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    let systemd = match Systemd1ManagerProxy::new(&conn).await {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    let mut workloads = Vec::new();
    if let Ok(units) = systemd.list_units().await {
        for unit in units {
            if !unit.name.starts_with("avena-") || !unit.name.ends_with(".service") {
                continue;
            }
            let name = unit.name.trim_end_matches(".service").to_string();
            let state = match unit.active_state.as_str() {
                "active" => WorkloadStatus::Running,
                "inactive" => WorkloadStatus::Stopped,
                "failed" => WorkloadStatus::Error,
                _ => WorkloadStatus::Unknown,
            };

            workloads.push(WorkloadState {
                name,
                state,
                exit_code: None,
                restart_count: 0,
                started_at: None,
                image: "unknown".to_string(),
            });
        }
    }

    workloads
}

pub async fn reconcile_leaves(
    kv: &Arc<Mutex<KvStore>>,
    issuer_pub_key: &str,
    nats_url: &str,
) -> Result<()> {
    let guard = kv.lock().await;
    let mut remotes = vec![];
    let mut keys = guard.keys().await?;
    while let Some(key) = keys.next().await {
        let key = key?;
        if let Some(val) = guard.get(&key).await? {
                if let Ok(link) = serde_json::from_slice::<LinkEntry>(val.as_ref()) {
                    remotes.push((link.url, link.creds_path.unwrap_or_default()));
                }
            }
        }
    drop(guard);

    render_nats_conf(issuer_pub_key, remotes.clone()).await?;
    reload_nats(nats_url).await?;

    Ok(())
}

pub async fn reconcile_workloads(
    kv: &Arc<Mutex<KvStore>>,
    device_id: &str,
    systemd_dir: &std::path::Path,
) -> Result<()> {
    let prefix = format!("device/{device_id}/");
    let guard = kv.lock().await;
    let mut keys = match tokio::time::timeout(Duration::from_secs(5), guard.keys()).await {
        Ok(Ok(k)) => k,
        Ok(Err(err)) => {
            warn!("Workload reconcile: unable to list KV keys: {err:?}");
            return Ok(());
        }
        Err(_) => {
            warn!("Workload reconcile: list KV keys timed out");
            return Ok(());
        }
    };
    info!("Workload reconcile: scanning KV with prefix {prefix}");
    let mut desired: HashMap<String, WorkloadSpec> = HashMap::new();
    while let Some(key) = tokio::time::timeout(Duration::from_secs(2), keys.next()).await.unwrap_or(None) {
        let key = key?;
        if !key.starts_with(&prefix) {
            continue;
        }
        if let Some(val) = guard.get(&key).await? {
            if let Ok(entry) = serde_json::from_slice::<WorkloadDesiredState>(val.as_ref()) {
                desired.insert(entry.name, entry.spec);
            }
        }
    }
    for req in required_workloads() {
        desired.entry(req.name.clone()).or_insert(req.spec);
    }
    info!("Workload reconcile: desired entries {}", desired.len());
    drop(guard);

    let conn = Connection::session().await?;
    let manager = Systemd1ManagerProxy::new(&conn).await?;

    // Deploy/update desired workloads
    let mut active: HashSet<String> = HashSet::new();
    for (name, spec) in desired {
        let unit_name = if name.starts_with("avena-") {
            name.clone()
        } else {
            format!("avena-{name}")
        };
        let deployment = workload::WorkloadDeployment {
            name: unit_name.clone(),
            spec: spec.clone(),
        };
        deployment.deploy(systemd_dir).await?;
        manager.reload().await?;
        let _ = manager.restart_unit(&format!("{unit_name}.service"), "replace").await;
        active.insert(format!("{unit_name}.service"));
        info!("Workload reconcile: deployed {unit_name}");
    }

    // Stop workloads no longer desired
    if let Ok(units) = manager.list_units().await {
        for unit in units {
            if unit.name.starts_with("avena-")
                && unit.name.ends_with(".service")
                && !active.contains(&unit.name)
                && !is_required_unit(&unit.name)
            {
                let _ = manager.stop_unit(&unit.name, "replace").await;
                info!("Workload reconcile: stopped {}", unit.name);
            }
        }
    }

    Ok(())
}

pub fn required_workloads() -> Vec<WorkloadDeployment> {
    let nats_cfg_dir = directories::ProjectDirs::from("", "", "avena")
        .map(|d| d.config_dir().join("nats"))
        .unwrap_or_else(|| std::path::PathBuf::from("~/.config/avena/nats"));
    let server_conf_path = directories::BaseDirs::new()
        .map(|d| d.config_dir().join("containers/systemd/server.conf"))
        .unwrap_or_else(|| std::path::PathBuf::from("~/.config/containers/systemd/server.conf"));

    vec![WorkloadDeployment {
        name: "avena-nats".to_string(),
        spec: WorkloadSpec {
            image: "docker.io/library/nats".to_string(),
            tag: Some("2.12.2".to_string()),
            cmd: Some("--config /server.conf".to_string()),
            args: vec![],
            env: vec![],
            mounts: vec![
                MountSpec {
                    host: server_conf_path.to_string_lossy().to_string(),
                    container: "/server.conf".to_string(),
                    readonly: false,
                },
                MountSpec {
                    host: nats_cfg_dir.to_string_lossy().to_string(),
                    container: "/nats/cfg".to_string(),
                    readonly: false,
                },
            ],
            devices: vec![],
            perms: PermSpec {
                publish: vec![],
                subscribe: vec![],
            },
            ports: vec![PortSpec {
                container: 4222,
                host: 4222,
            }],
            volumes: vec!["avena-nats-js".to_string()],
        },
    }]
}

fn is_required_unit(unit_name: &str) -> bool {
    unit_name == "avena-nats.service" || unit_name == "avena-nats-js-volume.service"
}

pub async fn observe_workloads(
    kv: Arc<Mutex<KvStore>>,
    device_id: String,
    systemd_dir: std::path::PathBuf,
) -> Result<()> {
    let prefix = format!("device/{device_id}/");
    let pattern = format!("{prefix}>");
    let mut watcher = {
        let guard = kv.lock().await;
        guard.watch(pattern).await?
    };

    while let Some(_update) = watcher.next().await {
        info!("Workload watch: change detected");
        if let Err(err) = reconcile_workloads(&kv, &device_id, &systemd_dir).await {
            error!("Workload reconcile error: {err:?}");
        }
    }

    Ok(())
}

pub async fn render_nats_conf(_issuer_pub_key: &str, remotes: Vec<(String, String)>) -> Result<()> {
    let nats_cfg_dir = directories::ProjectDirs::from("", "", "avena")
        .map(|d| d.config_dir().join("nats"))
        .unwrap_or_else(|| std::path::PathBuf::from("~/.config/avena/nats"));

    let sys_seed = fs::read_to_string(nats_cfg_dir.join("SYS.nk")).await?;
    let sys_kp = nkeys::KeyPair::from_seed(sys_seed.trim())?;
    let sys_account_key = sys_kp.public_key();

    let avena_seed = fs::read_to_string(nats_cfg_dir.join("AVENA.nk")).await?;
    let avena_kp = nkeys::KeyPair::from_seed(avena_seed.trim())?;
    let avena_account_key = avena_kp.public_key();

    let sys_jwt = fs::read_to_string(nats_cfg_dir.join("SYS.jwt")).await?;
    let avena_jwt = fs::read_to_string(nats_cfg_dir.join("AVENA.jwt")).await?;

    let remotes = remotes
        .iter()
        .map(|(url, creds)| NatsServerConfTemplateLeafNodeRemote {
            url: url.as_str(),
            credentials: creds.as_str(),
        })
        .collect();

    let nats_conf = NatsServerConfTemplate {
        hostname: "avena",
        js_store_dir: "/data/jetstream",
        js_max_mem: "1G",
        js_max_file: "10G",
        js_domain: "avena",
        sys_account_key: &sys_account_key,
        avena_account_key: &avena_account_key,
        sys_jwt: sys_jwt.trim(),
        avena_jwt: avena_jwt.trim(),
        remotes,
    };

    let conf_path = nats_conf_path();
    if let Some(parent) = conf_path.parent() {
        fs::create_dir_all(parent).await?;
    }
    fs::write(&conf_path, nats_conf.render()?).await?;
    Ok(())
}

async fn reload_nats(nats_url: &str) -> Result<()> {
    let creds_path = directories::ProjectDirs::from("", "", "avena")
        .map(|d| d.config_dir().join("nats/sys-admin.creds"))
        .unwrap_or_else(|| std::path::PathBuf::from("~/.config/avena/nats/sys-admin.creds"));
    let sys_admin_creds = fs::read_to_string(&creds_path).await?;
    let sys = async_nats::ConnectOptions::with_credentials(&sys_admin_creds)?
        .connect(nats_url)
        .await?;
    let server_id = sys.server_info().server_id.clone();
    let subject = format!("$SYS.REQ.SERVER.{server_id}.RELOAD");
    let _ = sys.request(subject, "".into()).await?;
    Ok(())
}

async fn link_offer_handshake(
    remote_url: &str,
    device: &DeviceIdentity,
    issuer_pub: &str,
    nats_url: &str,
    kv: &Arc<Mutex<KvStore>>,
) -> Result<bool> {
    // Connect to remote
    let nc = async_nats::connect(remote_url).await?;
    let nonce: String = uuid::Uuid::new_v4().to_string();
    let msg = format!("{nonce}|{}", device.id);
    let sig = device.sign(msg.as_bytes())?;

    let offer = avena::messages::LinkOffer {
        from_id: device.id.clone(),
        from_pubkey: device.pubkey.clone(),
        nonce: nonce.clone(),
        leaf_url: String::new(),
        signature: sig,
        token: device.network_token.clone(),
    };

    let resp = nc
        .request(
            avena::messages::LINK_OFFER_SUBJECT,
            Vec::from(offer).into(),
        )
        .await?;

    let accept: avena::messages::LinkAccept = resp.payload.as_ref().try_into()?;

    // Verify nonce and signature
    if accept.nonce_response != nonce {
        return Ok(false);
    }
    let msg = format!("ACCEPT|{nonce}");
    let valid = DeviceIdentity::verify(&accept.to_pubkey, msg.as_bytes(), &accept.signature)?;
    if !valid {
        return Ok(false);
    }

    // Store creds if provided
    if let Some(creds) = accept.creds_inline {
        let links_dir = directories::ProjectDirs::from("", "", "avena")
            .map(|d| d.data_dir().join("links"))
            .unwrap_or_else(|| std::path::PathBuf::from("~/.local/share/avena/links"));
        let path = links_dir.join(format!("{}.creds", accept.to_id));
        tokio::fs::create_dir_all(&links_dir).await?;
        tokio::fs::write(&path, creds).await?;

        let guard = kv.lock().await;
        let _ = guard
            .put(
                format!("link:{}", remote_url),
                serde_json::to_vec(&LinkEntry {
                    url: remote_url.to_string(),
                    creds_path: Some(path.to_string_lossy().to_string()),
                    inline_creds: None,
                })?
                .into(),
            )
            .await;

        // Re-render config to include new creds
        let remotes = {
            let mut list = vec![];
            let mut iter = guard.keys().await?;
            while let Some(key) = iter.next().await {
                let key = key?;
                if let Some(val) = guard.get(&key).await? {
                    if let Ok(link) = serde_json::from_slice::<LinkEntry>(val.as_ref()) {
                        list.push((link.url, link.creds_path.unwrap_or_default()));
                    }
                }
            }
            list
        };
        render_nats_conf(issuer_pub, remotes).await?;
        reload_nats(nats_url).await?;
    }

    Ok(true)
}
