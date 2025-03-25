use near_primitives::types::AccountId;
use near_jsonrpc_client::methods::query::RpcQueryRequest;
use near_primitives::views::QueryRequest;
use near_primitives::types::Finality;
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
    RpcError(#[from] near_jsonrpc_client::errors::JsonRpcError<near_jsonrpc_primitives::types::query::RpcQueryError>),
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
pub async fn get_near_greeting(network: String) -> Result<String, String> {
    let config_str = fs::read_to_string("src/network_config.toml").map_err(|e| e.to_string())?;
    let config: Config = toml::from_str(&config_str).map_err(|e| e.to_string())?;

    let network_config = match network.as_str() {
        "mainnet" => &config.mainnet,
        "testnet" => &config.testnet,
        _ => return Err("Invalid network specified".to_string()),
    };

    let rpc_url = &network_config.rpc_url;
    let contract_id = &network_config.contract_id;

    let provider = near_jsonrpc_client::JsonRpcClient::connect(rpc_url);
    let account_id = AccountId::from_str(&contract_id)
        .map_err(|e| format!("Invalid account ID: {}", e))?;

    let args = serde_json::json!({});
    let query_response = provider
        .call(RpcQueryRequest {
            block_reference: near_primitives::types::BlockReference::Finality(Finality::Final),
            request: QueryRequest::CallFunction {
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
        match serde_json::from_slice::<GreetingResponse>(&result.result) {
            Ok(response) => Ok(response.greeting),
            Err(_) => {
                match String::from_utf8(result.result.to_vec()) {
                    Ok(greeting) => Ok(greeting.trim_matches('"').to_string()),
                    Err(e) => Err(format!("Failed to parse response as string: {}", e))
                }
            }
        }
    } else {
        Err("Unexpected response type".to_string())
    }
}