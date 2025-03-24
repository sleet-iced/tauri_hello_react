use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct NearCredential {
    pub account_id: String,
    pub public_key: String,
    pub network: String,
    #[serde(skip_serializing)]
    pub private_key: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CredentialResponse {
    pub credentials: Vec<NearCredential>,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawCredential {
    account_id: String,
    public_key: String,
    private_key: String,
}

#[tauri::command]
pub fn load_near_credentials() -> CredentialResponse {
    let home_dir = match dirs::home_dir() {
        Some(path) => path,
        None => return CredentialResponse {
            credentials: Vec::new(),
            error: Some("Could not find home directory".to_string()),
        },
    };

    let near_credentials_dir = home_dir.join(".near-credentials");
    let networks = vec!["mainnet", "testnet"];
    let mut credentials = Vec::new();

    for network in networks {
        let network_dir = near_credentials_dir.join(network);
        if !network_dir.exists() {
            continue;
        }

        if let Ok(entries) = fs::read_dir(&network_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(raw_cred) = serde_json::from_str::<RawCredential>(&content) {
                            credentials.push(NearCredential {
                                account_id: raw_cred.account_id,
                                public_key: raw_cred.public_key,
                                network: network.to_string(),
                                private_key: Some(raw_cred.private_key),
                            });
                        }
                    }
                }
            }
        }
    }

    CredentialResponse {
        credentials,
        error: None,
    }
}