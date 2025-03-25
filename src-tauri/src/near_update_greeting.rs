use near_primitives::types::{AccountId, Balance};
use near_jsonrpc_client::methods::broadcast_tx_commit::RpcBroadcastTxCommitRequest;
use near_primitives::transaction::{Action, FunctionCallAction, Transaction, SignedTransaction};
use near_primitives::types::Nonce;
use near_crypto::{InMemorySigner, SecretKey};
use std::fs;
use serde::Deserialize;
use std::str::FromStr;

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
    let account = AccountId::from_str(&account_id).map_err(|e| e.to_string())?;
    let secret_key = SecretKey::from_str(&private_key).map_err(|e| e.to_string())?;
    let signer = InMemorySigner::from_secret_key(account.clone(), secret_key);

    let contract_account_id = AccountId::from_str(contract_id)
        .map_err(|e| format!("Invalid contract ID: {}", e))?;

    let args = serde_json::json!({ "greeting": new_greeting }).to_string().into_bytes();
    let function_call_action = Action::FunctionCall(Box::new(FunctionCallAction {
        method_name: "set_greeting".to_string(),
        args: args.into(),
        gas: 30_000_000_000_000, // 30 TGas
        deposit: Balance::from(0),
    }));

    let block_hash = provider
        .call(near_jsonrpc_client::methods::status::RpcStatusRequest)
        .await
        .map_err(|e| e.to_string())?
        .sync_info
        .latest_block_hash;

    // Get the current nonce for the account
    let access_key_query_response = provider
        .call(near_jsonrpc_client::methods::query::RpcQueryRequest {
            block_reference: near_primitives::types::Finality::Final.into(),
            request: near_primitives::views::QueryRequest::ViewAccessKey {
                account_id: account.clone(),
                public_key: signer.public_key(),
            },
        })
        .await
        .map_err(|e| e.to_string())?;

    let nonce = if let near_jsonrpc_client::methods::query::RpcQueryResponse {
        kind: near_jsonrpc_primitives::types::query::QueryResponseKind::AccessKey(access_key),
        ..
    } = access_key_query_response
    {
        access_key.nonce
    } else {
        return Err("Failed to get access key".to_string());
    };

    let transaction = Transaction {
        signer_id: account,
        public_key: signer.public_key(),
        nonce: nonce + 1,
        receiver_id: contract_account_id,
        block_hash,
        actions: vec![function_call_action],
    };

    let signed_transaction = SignedTransaction::sign(transaction, &signer);
    let result = provider
        .call(RpcBroadcastTxCommitRequest { signed_transaction })
        .await;

    match result {
        Ok(_) => Ok("Successfully updated greeting".to_string()),
        Err(e) => Err(format!("Failed to update greeting: {}", e))
    }
}