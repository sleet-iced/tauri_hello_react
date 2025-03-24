use serde::{Deserialize, Serialize};
use std::fs;
use log;


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
    #[serde(rename = "implicit_account_id")]
    account_id: String,
    #[serde(rename = "public_key")]
    public_key: String,
    #[serde(rename = "private_key")]
    private_key: String,
    #[serde(rename = "seed_phrase_hd_path")]
    _hd_path: Option<String>,
    #[serde(rename = "master_seed_phrase")]
    _seed_phrase: Option<String>,
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

    let networks = vec!["mainnet", "testnet", "implicit"];
    let mut credentials = Vec::new();


    for network in networks {
        let network_dir = near_credentials_dir.join(network);
        log::info!("Checking {} network directory: {}", network, network_dir.display());
        
        if !network_dir.exists() {
            log::warn!("{} directory does not exist", network_dir.display());
            continue;
        }

        for entry in walkdir::WalkDir::new(network_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("json")) 
        {
            let path = entry.path();
            if let Ok(content) = fs::read_to_string(&path) {
                log::info!("Reading credentials file at {}", path.display());
                match serde_json::from_str::<RawCredential>(&content) {
                    Ok(raw_cred) => {
                        let network_type = match network {
                            "mainnet" => "mainnet",
                            "testnet" => "testnet",
                            _ => continue,
                        };

                        credentials.push(NearCredential {
                            account_id: raw_cred.account_id,
                            public_key: raw_cred.public_key,
                            network: network_type.to_string(),
                            private_key: Some(raw_cred.private_key),
                        });
                    }
                    Err(e) => {
                        log::error!("Failed to parse {}: {}", entry.path().display(), e);
                    }
                }
            } else {
                log::warn!("Failed to read credentials file at {}", path.display());
            }
        }
    }

    CredentialResponse {
        credentials,
        error: None,
    }
}