use near_api::*;
use std::str::FromStr;
use std::fs;
use serde::Deserialize;

#[derive(Deserialize)]
struct Config {
    mainnet: NetworkConfig,
    testnet: NetworkConfig,
}

#[derive(Deserialize)]
struct NetworkConfig {
    rpc_url: String,
    contract_id: String,
}

#[tauri::command]
pub async fn update_near_greeting(
    network: String,
    account_id: String,
    private_key: String,
    new_greeting: String,
) -> Result<String, String> {
    let config_str = fs::read_to_string("src/network_config.toml").map_err(|e| e.to_string())?;
    let config: Config = toml::from_str(&config_str).map_err(|e| e.to_string())?;

    let network_config = match network.as_str() {
        "mainnet" => &config.mainnet,
        "testnet" => &config.testnet,
        _ => return Err("Invalid network specified".to_string()),
    };

    let rpc_url = &network_config.rpc_url;
    let contract_id = &network_config.contract_id;

    let near = Near::new(rpc_url);
    let account = near.account_from_secret_key(&account_id, &private_key)
        .map_err(|e| format!("Failed to create account: {}", e))?;
    
    let result = account.function_call(
        contract_id.to_string(),
        "set_greeting".to_string(),
        serde_json::json!({ "greeting": new_greeting }).to_string().into_bytes(),
        30_000_000_000_000, // 30 TGas
        0, // deposit
    ).await;

    match result {
        Ok(_) => Ok("Successfully updated greeting".to_string()),
        Err(e) => Err(format!("Failed to update greeting: {}", e))
    }
}