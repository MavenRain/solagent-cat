//! Project-wide error type.
//!
//! A single `Error` enum covers every failure mode in solagent-cat.
//! `From` impls enable `?` propagation from dependency errors.
//! The bridge `From<Error> for rig_cat::error::Error` converts at
//! the [`Tool::call`](rig_cat::tool::Tool::call) boundary.

/// All errors in solagent-cat.
#[derive(Debug)]
pub enum Error {
    /// HTTP request failed (ureq).
    Http(ureq::Error),
    /// JSON serialization or deserialization failed.
    Json(serde_json::Error),
    /// IO operation failed.
    Io(std::io::Error),
    /// Base58 decoding failed.
    Base58(bs58::decode::Error),
    /// Solana RPC returned an error.
    Rpc {
        /// JSON-RPC error code.
        code: i64,
        /// Human-readable message.
        message: String,
    },
    /// Transaction confirmation or simulation failed.
    Transaction {
        /// Transaction signature (if known).
        signature: String,
        /// Failure description.
        message: String,
    },
    /// Transaction signing failed.
    Signing {
        /// Failure description.
        message: String,
    },
    /// Missing required configuration field.
    Config {
        /// Name of the missing field.
        field: String,
    },
    /// External API returned an error (Jupiter, Birdeye, etc.).
    Api {
        /// Service name.
        service: String,
        /// HTTP status code.
        status: u16,
        /// Response body or message.
        message: String,
    },
    /// Invalid input parameter.
    InvalidInput {
        /// Parameter name.
        field: String,
        /// Why the value is invalid.
        reason: String,
    },
    /// SPL token operation failed.
    Token {
        /// Failure description.
        message: String,
    },
    /// NFT operation failed.
    Nft {
        /// Failure description.
        message: String,
    },
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Http(e) => write!(f, "HTTP error: {e}"),
            Self::Json(e) => write!(f, "JSON error: {e}"),
            Self::Io(e) => write!(f, "IO error: {e}"),
            Self::Base58(e) => write!(f, "base58 decode error: {e}"),
            Self::Rpc { code, message } => write!(f, "RPC error ({code}): {message}"),
            Self::Transaction { signature, message } => {
                write!(f, "transaction failed ({signature}): {message}")
            }
            Self::Signing { message } => write!(f, "signing error: {message}"),
            Self::Config { field } => write!(f, "missing config: {field}"),
            Self::Api {
                service,
                status,
                message,
            } => write!(f, "{service} API error ({status}): {message}"),
            Self::InvalidInput { field, reason } => {
                write!(f, "invalid input for {field}: {reason}")
            }
            Self::Token { message } => write!(f, "token error: {message}"),
            Self::Nft { message } => write!(f, "NFT error: {message}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Http(e) => Some(e),
            Self::Json(e) => Some(e),
            Self::Io(e) => Some(e),
            Self::Base58(e) => Some(e),
            Self::Rpc { .. }
            | Self::Transaction { .. }
            | Self::Signing { .. }
            | Self::Config { .. }
            | Self::Api { .. }
            | Self::InvalidInput { .. }
            | Self::Token { .. }
            | Self::Nft { .. } => None,
        }
    }
}

impl From<ureq::Error> for Error {
    fn from(e: ureq::Error) -> Self {
        Self::Http(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<bs58::decode::Error> for Error {
    fn from(e: bs58::decode::Error) -> Self {
        Self::Base58(e)
    }
}

/// Bridge solagent-cat errors into rig-cat's error type.
///
/// Used at the [`Tool::call`](rig_cat::tool::Tool::call) boundary
/// via [`Io::map_error`](comp_cat_rs::effect::io::Io::map_error).
impl From<Error> for rig_cat::error::Error {
    fn from(e: Error) -> Self {
        Self::Provider {
            status: 0,
            message: e.to_string(),
        }
    }
}
