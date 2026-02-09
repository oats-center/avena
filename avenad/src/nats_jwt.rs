use color_eyre::Result;
use nkeys::KeyPair;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::collections::HashMap;
use tokio::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct OperatorClaims {
    pub jti: String,
    pub iat: i64,
    pub iss: String,
    pub name: String,
    pub sub: String,
    pub nats: OperatorNats,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OperatorNats {
    #[serde(rename = "type")]
    pub claim_type: String,
    pub version: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_account: Option<String>,
    pub signing_keys: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountClaims {
    pub jti: String,
    pub iat: i64,
    pub iss: String,
    pub name: String,
    pub sub: String,
    pub nats: AccountNats,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountNats {
    #[serde(rename = "type")]
    pub claim_type: String,
    pub version: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limits: Option<AccountLimits>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_permissions: Option<Permissions>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountLimits {
    pub subs: i64,
    pub conn: i64,
    #[serde(rename = "leaf")]
    pub leaf_node_conn: i64,
    pub imports: i64,
    pub exports: i64,
    pub data: i64,
    pub payload: i64,
    pub wildcards: bool,
    #[serde(rename = "tiered_limits", skip_serializing_if = "Option::is_none")]
    pub tiered_limits: Option<HashMap<String, JetStreamLimits>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JetStreamLimits {
    pub mem_storage: i64,
    pub disk_storage: i64,
    pub streams: i64,
    pub consumer: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_ack_pending: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mem_max_stream_bytes: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disk_max_stream_bytes: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_bytes_required: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Permissions {
    pub publish: PermissionRules,
    pub subscribe: PermissionRules,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PermissionRules {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deny: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserClaims {
    pub jti: String,
    pub iat: i64,
    pub iss: String,
    pub name: String,
    pub sub: String,
    pub nats: UserNats,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserNats {
    #[serde(rename = "type")]
    pub claim_type: String,
    pub version: u8,
    #[serde(rename = "pub", skip_serializing_if = "Option::is_none")]
    pub pub_: Option<PermissionRules>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub: Option<PermissionRules>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resp: Option<ResponsePermission>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subs: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponsePermission {
    pub max: i32,
    pub ttl: i64,
}

pub struct NatsJwtManager {
    operator_kp: KeyPair,
}

impl NatsJwtManager {
    pub fn new() -> Result<Self> {
        let operator_kp = KeyPair::new_operator();
        Ok(Self { operator_kp })
    }

    pub fn from_keypair(operator_kp: KeyPair) -> Self {
        Self { operator_kp }
    }

    pub fn load_or_generate(cfg_dir: &Path) -> Result<Self> {
        let operator_seed_path = cfg_dir.join("operator.nk");
        let operator_kp = if operator_seed_path.exists() {
            let seed = std::fs::read_to_string(&operator_seed_path)?;
            KeyPair::from_seed(seed.trim())?
        } else {
            let kp = KeyPair::new_operator();
            std::fs::create_dir_all(cfg_dir)?;
            std::fs::write(&operator_seed_path, kp.seed()?)?;
            kp
        };

        Ok(Self { operator_kp })
    }

    pub fn operator_pubkey(&self) -> String {
        self.operator_kp.public_key()
    }

    pub fn generate_operator_jwt(&self, name: &str, system_account: Option<&str>) -> Result<String> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64;

        let pubkey = self.operator_kp.public_key();
        let system_account = system_account.map(|s| s.to_string());
        let claims = OperatorClaims {
            jti: uuid::Uuid::new_v4().to_string(),
            iat: now,
            iss: pubkey.clone(),
            name: name.to_string(),
            sub: pubkey.clone(),
            nats: OperatorNats {
                claim_type: "operator".to_string(),
                version: 2,
                system_account,
                signing_keys: vec![],
            },
        };

        self.sign_jwt(&claims)
    }

    pub fn generate_account_jwt(&self, name: &str, account_kp: &KeyPair, enable_jetstream: bool) -> Result<String> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64;

        let pubkey = account_kp.public_key();
        let issuer = self.operator_kp.public_key();

        let tiered_limits = if enable_jetstream {
            let mut map = HashMap::new();
            map.insert(
                "R1".to_string(),
                JetStreamLimits {
                    mem_storage: -1,
                    disk_storage: -1,
                    streams: -1,
                    consumer: -1,
                    max_ack_pending: Some(-1),
                    mem_max_stream_bytes: Some(-1),
                    disk_max_stream_bytes: Some(-1),
                    max_bytes_required: Some(false),
                },
            );
            Some(map)
        } else {
            None
        };

        let limits = Some(AccountLimits {
            subs: -1,
            conn: -1,
            leaf_node_conn: -1,
            imports: -1,
            exports: -1,
            data: -1,
            payload: -1,
            wildcards: true,
            tiered_limits,
        });

        let claims = AccountClaims {
            jti: uuid::Uuid::new_v4().to_string(),
            iat: now,
            iss: issuer,
            name: name.to_string(),
            sub: pubkey,
            nats: AccountNats {
                claim_type: "account".to_string(),
                version: 2,
                limits,
                default_permissions: None,
            },
        };

        let jwt = self.sign_jwt(&claims)?;
        Ok(jwt)
    }

    pub fn generate_user_jwt(
        &self,
        account_kp: &KeyPair,
        name: &str,
        pub_allow: Vec<String>,
        sub_allow: Vec<String>,
    ) -> Result<(String, KeyPair)> {
        let user_kp = KeyPair::new_user();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64;

        let pubkey = user_kp.public_key();
        let issuer = account_kp.public_key();

        let claims = UserClaims {
            jti: uuid::Uuid::new_v4().to_string(),
            iat: now,
            iss: issuer,
            name: name.to_string(),
            sub: pubkey,
            nats: UserNats {
                claim_type: "user".to_string(),
                version: 2,
                pub_: Some(PermissionRules {
                    allow: Some(pub_allow),
                    deny: None,
                }),
                sub: Some(PermissionRules {
                    allow: Some(sub_allow),
                    deny: None,
                }),
                resp: None,
                subs: Some(-1),
                data: Some(-1),
                payload: Some(-1),
            },
        };

        let jwt = self.sign_jwt_with_keypair(&claims, account_kp)?;
        Ok((jwt, user_kp))
    }

    fn sign_jwt<T: Serialize>(&self, claims: &T) -> Result<String> {
        self.sign_jwt_with_keypair(claims, &self.operator_kp)
    }

    fn sign_jwt_with_keypair<T: Serialize>(&self, claims: &T, kp: &KeyPair) -> Result<String> {
        let claims_json = serde_json::to_string(claims)?;
        let claims_b64 = data_encoding::BASE64URL_NOPAD.encode(claims_json.as_bytes());

        let header = r#"{"typ":"JWT","alg":"ed25519-nkey"}"#;
        let header_b64 = data_encoding::BASE64URL_NOPAD.encode(header.as_bytes());

        let signing_input = format!("{}.{}", header_b64, claims_b64);
        let signature = kp.sign(signing_input.as_bytes())?;
        let signature_b64 = data_encoding::BASE64URL_NOPAD.encode(&signature);

        Ok(format!("{}.{}", signing_input, signature_b64))
    }

    pub fn create_creds_file(jwt: &str, user_kp: &KeyPair) -> Result<String> {
        let seed = user_kp.seed()?;
        Ok(format!(
            "-----BEGIN NATS USER JWT-----\n{}\n------END NATS USER JWT------\n\n************************* IMPORTANT *************************\nNKEY Seed printed below can be used to sign and prove identity.\nNKEYs are sensitive and should be treated as secrets.\n\n-----BEGIN USER NKEY SEED-----\n{}\n------END USER NKEY SEED------\n\n*************************************************************\n",
            jwt, seed
        ))
    }
}

pub async fn setup_operator_mode(cfg_dir: &Path) -> Result<NatsJwtManager> {
    let mgr = NatsJwtManager::load_or_generate(cfg_dir)?;

    let sys_seed_path = cfg_dir.join("SYS.nk");
    let sys_kp = if sys_seed_path.exists() {
        let seed = std::fs::read_to_string(&sys_seed_path)?;
        KeyPair::from_seed(seed.trim())?
    } else {
        let kp = KeyPair::new_account();
        fs::write(&sys_seed_path, kp.seed()?).await?;
        kp
    };
    let sys_jwt = mgr.generate_account_jwt("SYS", &sys_kp, false)?;
    fs::write(cfg_dir.join("SYS.jwt"), &sys_jwt).await?;

    let operator_jwt = mgr.generate_operator_jwt("Avena", Some(&sys_kp.public_key()))?;
    fs::write(cfg_dir.join("operator.jwt"), &operator_jwt).await?;

    let (sys_admin_jwt, sys_admin_kp) = mgr.generate_user_jwt(
        &sys_kp,
        "sys-admin",
        vec![">".to_string()],
        vec![">".to_string()],
    )?;
    let sys_admin_creds = NatsJwtManager::create_creds_file(&sys_admin_jwt, &sys_admin_kp)?;
    fs::write(cfg_dir.join("sys-admin.creds"), &sys_admin_creds).await?;

    let avena_seed_path = cfg_dir.join("AVENA.nk");
    let avena_kp = if avena_seed_path.exists() {
        let seed = std::fs::read_to_string(&avena_seed_path)?;
        KeyPair::from_seed(seed.trim())?
    } else {
        let kp = KeyPair::new_account();
        fs::write(&avena_seed_path, kp.seed()?).await?;
        kp
    };
    let avena_jwt = mgr.generate_account_jwt("AVENA", &avena_kp, true)?;
    fs::write(cfg_dir.join("AVENA.jwt"), &avena_jwt).await?;

    let (avena_admin_jwt, avena_admin_kp) = mgr.generate_user_jwt(
        &avena_kp,
        "avena-admin",
        vec![">".to_string()],
        vec![">".to_string()],
    )?;
    let avena_admin_creds = NatsJwtManager::create_creds_file(&avena_admin_jwt, &avena_admin_kp)?;
    fs::write(cfg_dir.join("avena-admin.creds"), &avena_admin_creds).await?;

    Ok(mgr)
}
