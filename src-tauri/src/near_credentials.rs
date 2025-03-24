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

    let mut credentials = Vec::new();

    let entries = walkdir::WalkDir::new(near_credentials_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("json"));

    for entry in entries {
        let path = entry.path();
        let file_name = match path.file_stem().and_then(|s| s.to_str()) {
            Some(name) => name,
            None => {
                log::warn!("Could not get file stem for {}", path.display());
                continue;
            }
        };
        
        // Get the network from the parent directory name
        let parent_dir = path.parent().unwrap();
        let network = parent_dir.file_name().unwrap().to_str().unwrap();
        
        if network != "mainnet" && network != "testnet" {
            log::warn!("Skipping file in unsupported network directory: {}", path.display());
            continue;
        }

        // Use the full filename without .json extension as the account name
        let account_name = file_name.to_string();

        // Skip files that don't match the expected network
        let parent_dir = path.parent().unwrap();
        let dir_name = parent_dir.file_name().unwrap().to_str().unwrap();
        if dir_name != network {
            log::warn!("Skipping file with mismatched network directory: {}", path.display());
            continue;
        }

        log::info!("Attempting to read credentials file at {}", path.display());
        if let Ok(content) = fs::read_to_string(&path) {
            log::debug!("File content: {}", content);
            match serde_json::from_str::<RawCredential>(&content) {
                Ok(raw_cred) => {
                    log::info!("Found valid {} credential: {}", network, account_name);
                    credentials.push(NearCredential {
                        account_id: account_name,
                        public_key: raw_cred.public_key,
                        network: network.to_string(),
                        private_key: Some(raw_cred.private_key),
                    });
                }
                Err(e) => {
                    log::error!("Failed to parse {}: {}", path.display(), e);
                    log::warn!("Problematic file content: {}", content);
                }
            }
        } else {
            log::warn!("Failed to read credentials file at {}", path.display());
        }
    }

    CredentialResponse {
        credentials,
        error: None,
    }
}