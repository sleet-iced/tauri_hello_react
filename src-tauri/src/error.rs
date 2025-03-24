use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum NearError {
    ConfigError(String),
    ContractError(String),
    NetworkError(String),
    ParseError(String),
}

impl std::fmt::Display for NearError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NearError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            NearError::ContractError(msg) => write!(f, "Contract error: {}", msg),
            NearError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            NearError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl From<std::io::Error> for NearError {
    fn from(err: std::io::Error) -> Self {
        NearError::ConfigError(err.to_string())
    }
}

impl From<toml::de::Error> for NearError {
    fn from(err: toml::de::Error) -> Self {
        NearError::ConfigError(err.to_string())
    }
}

impl From<near_jsonrpc_client::errors::JsonRpcError> for NearError {
    fn from(err: near_jsonrpc_client::errors::JsonRpcError) -> Self {
        NearError::NetworkError(err.to_string())
    }
}

impl From<serde_json::Error> for NearError {
    fn from(err: serde_json::Error) -> Self {
        NearError::ParseError(err.to_string())
    }
}