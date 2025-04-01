use near_primitives::types::{AccountId, Balance};
use near_jsonrpc_client::methods::broadcast_tx_commit::RpcBroadcastTxCommitRequest;
use near_primitives::transaction::{Action, FunctionCallAction, SignedTransaction, Transaction};
use near_primitives::transaction::TransactionV0;
use near_primitives::borsh::{self, BorshSerialize};
use near_crypto::{InMemorySigner, SecretKey};
use std::fs;
use serde::{Deserialize, Serialize};
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

#[derive(Serialize)]
pub struct TransactionResult {
    transaction_hash: String,
    block_hash: String,
    status: String,
    gas_burnt: u64,
    message: String,
}

#[tauri::command]
pub async fn update_near_greeting(
    network: String,
    account_id: String,
    private_key: String,
    new_greeting: String,
) -> Result<TransactionResult, String> {
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
        deposit: Balance::from(0u128),
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

    let transaction = Transaction::V0(TransactionV0 {
        signer_id: account.clone(),
        public_key: signer.public_key(),
        nonce: nonce + 1,
        receiver_id: contract_account_id,
        block_hash,
        actions: vec![function_call_action],
    });

    let transaction_bytes = borsh::to_vec(&transaction).map_err(|e| e.to_string())?;
    let hash = near_primitives::hash::hash(&transaction_bytes);
    let signature = signer.sign(hash.as_ref());
    let signed_transaction = SignedTransaction::new(signature, transaction);
    let result = provider
        .call(RpcBroadcastTxCommitRequest { signed_transaction })
        .await;

    match result {
        Ok(outcome) => {
            let transaction_hash = format!("{}", outcome.transaction.hash);
            let block_hash = format!("{}", outcome.transaction_outcome.block_hash);
            let gas_burnt = outcome.transaction_outcome.outcome.gas_burnt;
            
            match outcome.status {
                near_primitives::views::FinalExecutionStatus::SuccessValue(_) => {
                    Ok(TransactionResult {
                        transaction_hash,
                        block_hash,
                        status: "Success".to_string(),
                        gas_burnt,
                        message: "Successfully updated greeting".to_string(),
                    })
                }
                near_primitives::views::FinalExecutionStatus::Failure(e) => {
                    let error_message = format!("Transaction failed: {:?}", e);
                    Ok(TransactionResult {
                        transaction_hash,
                        block_hash,
                        status: "Failed".to_string(),
                        gas_burnt,
                        message: error_message,
                    })
                }
                status => {
                    let error_message = format!("Unexpected transaction status: {:?}", status);
                    Ok(TransactionResult {
                        transaction_hash,
                        block_hash,
                        status: "Unknown".to_string(),
                        gas_burnt,
                        message: error_message,
                    })
                }
            }
        }
        Err(e) => Err(format!("Failed to submit transaction: {}", e))
    }
}