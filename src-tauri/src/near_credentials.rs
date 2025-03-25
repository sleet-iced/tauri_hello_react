use serde::{Deserialize, Serialize};
use std::fs;
use log;


#[derive(Debug, Serialize, Deserialize)]
pub struct NearCredential {
    #[serde(rename = "accountId")]
    pub account_id: String,
    #[serde(rename = "publicKey")]
    pub public_key: String,
    pub network: String,
    // Note: Network value must match TypeScript's 'mainnet'|'testnet' specification
    #[serde(rename = "privateKey")]
    pub private_key: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CredentialResponse {
    pub credentials: Vec<NearCredential>,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawCredential {
    #[serde(rename = "public_key")]
    public_key: String,
    #[serde(rename = "private_key")]
    private_key: String,
    #[serde(skip)]
    _other: serde_json::Value,
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
    log::info!("Looking for credentials in: {}", near_credentials_dir.display());

    let mut credentials = Vec::new();

    let entries = match std::fs::read_dir(&near_credentials_dir) {
        Ok(entries) => entries
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().ok().map_or(false, |ft| ft.is_dir()))
            .filter(|entry| {
                let name = entry.file_name();
                let network = name.to_str().unwrap_or("");
                network == "mainnet" || network == "testnet"
            }),
        Err(_) => return CredentialResponse {
            credentials: Vec::new(),
            error: Some("Failed to read credentials directory".to_string()),
        },
    };

    for network_dir in entries {
        let network = network_dir.file_name().to_str().unwrap_or("").to_string();
        let network_path = network_dir.path();

        if let Ok(files) = std::fs::read_dir(&network_path) {
            for file in files {
                if let Ok(file) = file {
                    let path = file.path();
                    if !path.is_file() || path.extension().and_then(|s| s.to_str()) != Some("json") {
                        continue;
                    }

                    let account_id = match path.file_stem().and_then(|s| s.to_str()) {
                        Some(name) => name.to_string(),
                        None => {
                            log::warn!("Could not get file stem for {}", path.display());
                            continue;
                        }
                    };

                    log::info!("Reading credentials for {} in {}", account_id, network);
                    if let Ok(content) = fs::read_to_string(&path) {
                        match serde_json::from_str::<RawCredential>(&content) {
                            Ok(raw_cred) => {
                                log::info!("Found valid {} credential for {}", network, account_id);
                                credentials.push(NearCredential {
                                    account_id,
                                    public_key: raw_cred.public_key,
                                    network: network.clone(),
                                    private_key: Some(raw_cred.private_key),
                                });
                            }
                            Err(e) => {
                                log::error!("Failed to parse {}: {}", path.display(), e);
                            }
                        }
                    } else {
                        log::warn!("Failed to read credentials file at {}", path.display());
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