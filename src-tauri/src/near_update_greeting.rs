use near_jsonrpc_client::methods::broadcast_tx_commit::RpcBroadcastTxCommitRequest;
use near_primitives::transaction::{Action, FunctionCallAction, Transaction};
use near_primitives::types::{AccountId, BlockReference};
use std::str::FromStr;
use std::fs;
use serde::Deserialize;
use near_crypto::{InMemorySigner, SecretKey};

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

    let provider = near_jsonrpc_client::JsonRpcClient::connect(rpc_url);
    let signer_account_id = AccountId::from_str(&account_id)
        .map_err(|e| format!("Invalid account ID: {}", e))?;
    let contract_account_id = AccountId::from_str(&contract_id)
        .map_err(|e| format!("Invalid contract ID: {}", e))?;

    let secret_key = SecretKey::from_str(&private_key)
        .map_err(|e| format!("Invalid private key: {}", e))?;
    let signer = InMemorySigner::from_secret_key(signer_account_id.clone(), secret_key);

    let block_ref = provider
        .block(BlockReference::Finality(near_primitives::types::Finality::Final))
        .await
        .map_err(|e| e.to_string())?;

    let access_key_query_response = provider
        .query(
            near_primitives::views::QueryRequest::ViewAccessKey {
                account_id: signer_account_id.clone(),
                public_key: signer.public_key(),
            },
            BlockReference::BlockId(block_ref.header.hash.into()),
        )
        .await
        .map_err(|e| e.to_string())?;

    let nonce = if let near_jsonrpc_primitives::types::query::QueryResponseKind::AccessKey(access_key) =
        access_key_query_response.kind
    {
        access_key.nonce
    } else {
        return Err("Failed to get access key".to_string());
    };

    let transaction = Transaction {
        signer_id: signer_account_id.clone(),
        public_key: signer.public_key(),
        nonce: nonce + 1,
        receiver_id: contract_account_id,
        block_hash: block_ref.header.hash,
        actions: vec![Action::FunctionCall(FunctionCallAction {
            method_name: "set_greeting".to_string(),
            args: serde_json::json!({ "greeting": new_greeting })
                .to_string()
                .into_bytes(),
            gas: 30_000_000_000_000, // 30 TGas
            deposit: 0,
        })],
    };

    let signed_tx = transaction.sign(&signer);
    let tx_result = provider
        .call(RpcBroadcastTxCommitRequest { signed_transaction: signed_tx })
        .await
        .map_err(|e| e.to_string())?;

    if tx_result.status.is_success() {
        Ok("Successfully updated greeting".to_string())
    } else {
        Err(format!("Transaction failed: {:?}", tx_result.status))
    }
}