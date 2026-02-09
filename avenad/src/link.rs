use std::sync::Arc;

use async_nats::Client;
use color_eyre::Result;
use tokio::sync::Mutex;

use crate::device::DeviceIdentity;
use crate::nats_jwt::NatsJwtManager;
use crate::LinkEntry;
use async_nats::jetstream::kv::Store as KvStore;
use futures::StreamExt;
use nkeys::KeyPair;
use std::path::PathBuf;
/// Handle incoming link offers and respond with accepts, storing the peer URL.
pub async fn handle_link_offers(
    nc: Client,
    kv: Arc<Mutex<KvStore>>,
    identity: DeviceIdentity,
    leaf_url: String,
    creds_dir: &str,
    jwt_mgr: Arc<NatsJwtManager>,
    avena_account_kp: Arc<KeyPair>,
) -> Result<()> {
    let mut sub = nc.subscribe(avena::messages::LINK_OFFER_SUBJECT).await?;
    while let Some(msg) = sub.next().await {
        if let Some(reply) = msg.reply {
            if let Ok(offer) = avena::messages::LinkOffer::try_from(msg.payload.as_ref()) {
                let nonce = offer.nonce.clone();
                let msg = format!("{nonce}|{}", offer.from_id);
                let valid = DeviceIdentity::verify(&offer.from_pubkey, msg.as_bytes(), &offer.signature)?;
                let mut ok = valid;
                if ok {
                    // Optionally check network token against ours if set
                    if let Some(my_token) = &identity.network_token {
                        ok = offer.token.as_ref() == Some(my_token);
                    }
                }
                let mut auth_url = leaf_url.clone();
                let mut accept_creds: Option<String> = None;
                let creds_path_opt: Option<String> = if ok {
                    let (creds_inline, creds_path) =
                        generate_leaf_creds(&jwt_mgr, &avena_account_kp, &offer.from_id, creds_dir).await?;
                    auth_url = leaf_url.clone();
                    accept_creds = Some(creds_inline);
                    Some(creds_path)
                } else {
                    None
                };

                if ok {
                    let guard = kv.lock().await;
                    let _ = guard
                        .put(
                            format!("link:{}", offer.leaf_url),
                            serde_json::to_vec(&LinkEntry {
                                url: auth_url.clone(),
                                creds_path: creds_path_opt.clone(),
                                inline_creds: None,
                            })?
                            .into(),
                        )
                        .await;
                }

                // Respond
                let msg_resp = format!("ACCEPT|{nonce}");
                let sig = identity.sign(msg_resp.as_bytes())?;
                let accept = avena::messages::LinkAccept {
                    to_id: identity.id.clone(),
                    to_pubkey: identity.pubkey.clone(),
                    nonce_response: nonce,
                    leaf_url: auth_url,
                    creds_inline: accept_creds,
                    signature: sig,
                    token: identity.network_token.clone(),
                };
                nc.publish(reply, Vec::from(accept).into()).await?;
            }
        }
    }

    Ok(())
}

async fn generate_leaf_creds(
    jwt_mgr: &NatsJwtManager,
    avena_account_kp: &KeyPair,
    remote_device_id: &str,
    creds_dir: &str,
) -> Result<(String, String)> {
    let user_name = format!("leaf-{}", remote_device_id);
    let (jwt, user_kp) = jwt_mgr.generate_user_jwt(
        avena_account_kp,
        &user_name,
        vec![">".to_string()],
        vec![">".to_string()],
    )?;

    let creds_content = NatsJwtManager::create_creds_file(&jwt, &user_kp)?;

    let path = PathBuf::from(creds_dir).join(format!("{}.creds", remote_device_id));
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    tokio::fs::write(&path, &creds_content).await?;

    Ok((creds_content, path.to_string_lossy().to_string()))
}
