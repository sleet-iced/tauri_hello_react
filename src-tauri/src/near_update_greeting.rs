use near_primitives::types::AccountId;
use near_primitives::views::{FinalExecutionStatus, QueryRequest};
use near_primitives::transaction::{Action, FunctionCallAction, Transaction};
use near_crypto::{InMemorySigner, SecretKey, Signer};
use near_jsonrpc_client::JsonRpcClient;
use std::str::FromStr;
use std::fs;
use serde::Deserialize;
use near_primitives::types::{BlockReference, Finality};
use near_jsonrpc_client::methods::broadcast_tx_commit::RpcBroadcastTxCommitRequest;
use near_jsonrpc_client::methods::query::RpcQueryRequest;

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

    let client = JsonRpcClient::connect(rpc_url);
    let signer_account_id = AccountId::from_str(&account_id)
        .map_err(|e| format!("Invalid account ID: {}", e))?;
    let contract_account_id = AccountId::from_str(&contract_id)
        .map_err(|e| format!("Invalid contract ID: {}", e))?;

    let secret_key = SecretKey::from_str(&private_key)
        .map_err(|e| format!("Invalid private key: {}", e))?;
    let signer = InMemorySigner::from_secret_key(signer_account_id.clone(), secret_key);

    let block_hash = client
        .call(near_jsonrpc_client::methods::block::RpcBlockRequest {
            block_reference: BlockReference::Finality(Finality::Final),
        })
        .await
        .map_err(|e| format!("Failed to fetch block hash: {}", e))?
        .header
        .hash;

    let access_key_query_response = client
        .call(near_jsonrpc_client::methods::query::RpcQueryRequest {
            block_reference: BlockReference::Finality(Finality::Final),
            request: near_primitives::views::QueryRequest::ViewAccessKey {
                account_id: signer_account_id.clone(),
                public_key: signer.public_key(),
            },
        })
        .await
        .map_err(|e| format!("Failed to fetch access key: {}", e))?;

    let current_nonce = if let near_jsonrpc_primitives::types::query::QueryResponseKind::AccessKey(access_key) = access_key_query_response.kind {
        access_key.nonce
    } else {
        return Err("Failed to get current nonce".to_string());
    };

    let transaction = Transaction {
        signer_id: signer_account_id.clone(),
        public_key: signer.public_key().clone(),
        nonce: current_nonce + 1,
        receiver_id: contract_account_id,
        block_hash,
        actions: vec![Action::FunctionCall(FunctionCallAction {
            method_name: "set_greeting".to_string(),
            args: serde_json::json!({ "greeting": new_greeting })
                .to_string()
                .into_bytes(),
            gas: 30_000_000_000_000, // 30 TGas
            deposit: 0,
        })],
    };
    let signed_transaction = transaction.sign(&signer);

    let request = near_jsonrpc_client::methods::broadcast_tx_commit::RpcBroadcastTxCommitRequest {
        signed_transaction,
    };

    match client.call(request).await {
        Ok(outcome) => {
            match outcome.status {
                FinalExecutionStatus::SuccessValue(_) => Ok("Successfully updated greeting".to_string()),
                status => Err(format!("Transaction failed: {:?}", status))
            }
        }
        Err(e) => Err(format!("Failed to send transaction: {}", e)),
    }
}