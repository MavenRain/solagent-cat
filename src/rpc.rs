//! Solana RPC client wrapped in [`Io`].
//!
//! [`RpcClient`] wraps `solana_client::rpc_client::RpcClient`
//! (which is already synchronous) inside [`Io::suspend`] to track
//! effects in the categorical effect system.

use std::sync::Arc;

use comp_cat_rs::effect::io::Io;

use crate::error::Error;
use crate::types::{CommitmentLevel, Lamports, Pubkey, RpcUrl, Signature};

/// A Solana JSON-RPC client.
///
/// Wraps the blocking `solana_client::rpc_client::RpcClient`.
/// All methods return [`Io<Error, _>`] for effect tracking.
/// The inner client is behind `Arc` so it can be cloned into
/// `Io::suspend` closures.
#[derive(Clone)]
pub struct RpcClient {
    inner: Arc<solana_client::rpc_client::RpcClient>,
}

impl RpcClient {
    /// Create a new RPC client for the given endpoint.
    #[must_use]
    pub fn new(url: &RpcUrl) -> Self {
        Self {
            inner: Arc::new(solana_client::rpc_client::RpcClient::new(
                url.as_str().to_owned(),
            )),
        }
    }

    /// Create with a specific commitment level.
    #[must_use]
    pub fn with_commitment(url: &RpcUrl, commitment: CommitmentLevel) -> Self {
        Self {
            inner: Arc::new(
                solana_client::rpc_client::RpcClient::new_with_commitment(
                    url.as_str().to_owned(),
                    commitment.into(),
                ),
            ),
        }
    }

    /// Get the SOL balance of an account in lamports.
    #[must_use]
    pub fn get_balance(&self, pubkey: &Pubkey) -> Io<Error, Lamports> {
        let inner = Arc::clone(&self.inner);
        let pk = *pubkey.as_inner();
        Io::suspend(move || {
            inner
                .get_balance(&pk)
                .map(Lamports::new)
                .map_err(|e| Error::Rpc {
                    code: 0,
                    message: e.to_string(),
                })
        })
    }

    /// Get the latest blockhash.
    #[must_use]
    pub fn get_latest_blockhash(&self) -> Io<Error, solana_sdk::hash::Hash> {
        let inner = Arc::clone(&self.inner);
        Io::suspend(move || {
            inner
                .get_latest_blockhash()
                .map_err(|e| Error::Rpc {
                    code: 0,
                    message: e.to_string(),
                })
        })
    }

    /// Send and confirm a transaction.
    ///
    /// Blocks until the transaction is confirmed at the client's
    /// commitment level.
    #[must_use]
    pub fn send_and_confirm_transaction(
        &self,
        transaction: &solana_sdk::transaction::Transaction,
    ) -> Io<Error, Signature> {
        let inner = Arc::clone(&self.inner);
        let tx = transaction.clone();
        Io::suspend(move || {
            inner
                .send_and_confirm_transaction(&tx)
                .map(Signature::new)
                .map_err(|e| Error::Transaction {
                    signature: String::new(),
                    message: e.to_string(),
                })
        })
    }

    /// Get the token balance for an SPL token account.
    #[must_use]
    pub fn get_token_account_balance(
        &self,
        token_account: &Pubkey,
    ) -> Io<Error, solana_client::rpc_response::RpcTokenAccountBalance> {
        let inner = Arc::clone(&self.inner);
        let pk = *token_account.as_inner();
        Io::suspend(move || {
            inner
                .get_token_account_balance(&pk)
                .map(|ui_amount| solana_client::rpc_response::RpcTokenAccountBalance {
                    address: String::new(),
                    amount: ui_amount,
                })
                .map_err(|e| Error::Rpc {
                    code: 0,
                    message: e.to_string(),
                })
        })
    }

    /// Get account info.
    #[must_use]
    pub fn get_account(
        &self,
        pubkey: &Pubkey,
    ) -> Io<Error, solana_sdk::account::Account> {
        let inner = Arc::clone(&self.inner);
        let pk = *pubkey.as_inner();
        Io::suspend(move || {
            inner
                .get_account(&pk)
                .map_err(|e| Error::Rpc {
                    code: 0,
                    message: e.to_string(),
                })
        })
    }

    /// Get the minimum balance required for rent exemption
    /// for an account of the given data length.
    #[must_use]
    pub fn get_minimum_balance_for_rent_exemption(
        &self,
        data_len: usize,
    ) -> Io<Error, Lamports> {
        let inner = Arc::clone(&self.inner);
        Io::suspend(move || {
            inner
                .get_minimum_balance_for_rent_exemption(data_len)
                .map(Lamports::new)
                .map_err(|e| Error::Rpc {
                    code: 0,
                    message: e.to_string(),
                })
        })
    }
}

impl core::fmt::Debug for RpcClient {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RpcClient").finish_non_exhaustive()
    }
}
