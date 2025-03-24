use near_primitives::types::AccountId;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::fs;
use thiserror::Error;
use toml;

#[derive(Debug, Error)]
pub enum NearError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("TOML parsing error: {0}")]
    TomlError(#[from] toml::de::Error),
    #[error("NEAR RPC error: {0}")]
    RpcError(#[from] near_jsonrpc_client::errors::JsonRpcError),
    #[error("Contract error: {0}")]
    ContractError(String),
    #[error("Response parsing error: {0}")]
    ResponseError(String)
}

#[derive(Debug, Deserialize)]
struct NetworkConfig {
    rpc_url: String,
    contract_id: String,
}

#[derive(Debug, Deserialize)]
struct Config {
    testnet: NetworkConfig,
    mainnet: NetworkConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GreetingResponse {
    greeting: String,
}

#[tauri::command]
pub async fn get_near_greeting() -> Result<String, String> {
    let config_str = fs::read_to_string("src/network_config.toml").map_err(|e| e.to_string())?;
    let config: Config = toml::from_str(&config_str).map_err(|e| e.to_string())?;

    // Using testnet configuration
    let rpc_url = config.testnet.rpc_url;
    let contract_id = config.testnet.contract_id;

    let provider = near_jsonrpc_client::JsonRpcClient::connect(rpc_url);
    let account_id = AccountId::from_str(&contract_id)
        .map_err(|e| format!("Invalid account ID: {}", e))?;

    let args = serde_json::json!({});
    let query_response = provider
        .call(near_jsonrpc_client::methods::query::RpcQueryRequest {
            block_reference: near_primitives::types::Finality::Final.into(),
            request: near_primitives::views::QueryRequest::CallFunction {
                account_id,
                method_name: "get_greeting".to_string(),
                args: args.to_string().into_bytes().into(),
            },
        })
        .await
        .map_err(|e| e.to_string())?;

    if let near_jsonrpc_client::methods::query::RpcQueryResponse {
        kind: near_jsonrpc_primitives::types::query::QueryResponseKind::CallResult(result),
        ..
    } = query_response
    {
        let result: GreetingResponse = serde_json::from_slice(&result.result)
            .map_err(|e| format!("Failed to parse response: {}", e))?;
        Ok(result.greeting)
    } else {
        Err("Unexpected response type".to_string())
    }
}