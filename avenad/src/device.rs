use std::{fs, path::PathBuf};

use color_eyre::Result;
use serde::{Deserialize, Serialize};
use data_encoding::BASE64URL_NOPAD;
use ed25519_dalek::{PublicKey, Signature, Verifier};

fn device_state_path() -> PathBuf {
    directories::ProjectDirs::from("", "", "avena")
        .map(|d| d.data_dir().join("device.json"))
        .unwrap_or_else(|| PathBuf::from("~/.local/share/avena/device.json"))
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeviceIdentity {
    pub id: String,
    pub pubkey: String,
    #[serde(skip)]
    pub seed: String,
    /// Owner-signed network token presented during link offers
    #[serde(skip)]
    pub network_token: Option<String>,
}

impl DeviceIdentity {
    pub fn load_or_generate() -> Result<Self> {
        let path = device_state_path();
        if path.exists() {
            let data = fs::read_to_string(&path)?;
            let mut persisted: PersistedIdentity = serde_json::from_str(&data)?;

            // Validate seed; if invalid or empty, regenerate and update file.
            let seed_valid = !persisted.seed.trim().is_empty()
                && nkeys::KeyPair::from_seed(&persisted.seed).is_ok();
            if !seed_valid {
                let kp = nkeys::KeyPair::new_user();
                persisted.seed = kp.seed()?;
                persisted.pubkey = kp.public_key();
                fs::write(&path, serde_json::to_string_pretty(&persisted)?)?;
            }

            let kp = nkeys::KeyPair::from_seed(&persisted.seed)?;
            Ok(DeviceIdentity {
                id: persisted.id,
                pubkey: kp.public_key(),
                seed: kp.seed()?,
                network_token: None,
            })
        } else {
            fs::create_dir_all(path.parent().unwrap())?;
            let kp = nkeys::KeyPair::new_user();
            let seed = kp.seed()?;
            let pubkey = kp.public_key();
            let id = uuid::Uuid::new_v4().to_string();
            let me = DeviceIdentity {
                id,
                pubkey,
                seed,
                network_token: None,
            };
            let persist = PersistedIdentity {
                id: me.id.clone(),
                pubkey: me.pubkey.clone(),
                seed: me.seed.clone(),
            };
            fs::write(&path, serde_json::to_string_pretty(&persist)?)?;
            Ok(me)
        }
    }

    pub fn sign(&self, msg: &[u8]) -> Result<String> {
        let kp = nkeys::KeyPair::from_seed(&self.seed)?;
        let sig = kp.sign(msg)?;
        Ok(BASE64URL_NOPAD.encode(&sig))
    }

    pub fn verify(pubkey: &str, msg: &[u8], sig_b64: &str) -> Result<bool> {
        let sig_bytes = BASE64URL_NOPAD.decode(sig_b64.as_bytes())?;
        let sig = Signature::from_bytes(&sig_bytes)?;
        let pk = PublicKey::from_bytes(nkeys::from_public_key(pubkey)?.1.as_ref())?;
        Ok(pk.verify(msg, &sig).is_ok())
    }

    pub fn load_token(&mut self) {
        if self.network_token.is_none() {
            if let Ok(token) = std::env::var("AVENA_NETWORK_TOKEN") {
                self.network_token = Some(token);
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct PersistedIdentity {
    pub id: String,
    pub pubkey: String,
    pub seed: String,
}
