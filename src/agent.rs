//! Core Solana agent.
//!
//! [`SolAgent`] holds the wallet, RPC client, and configuration.
//! It is constructed once and shared via `Rc<SolAgent>` among
//! tool instances.

use crate::config::Config;
use crate::rpc::RpcClient;
use crate::wallet::Wallet;

/// The core Solana agent: holds wallet, RPC connection, and configuration.
///
/// Immutable after construction.  Shared via `Rc<SolAgent>` among
/// tool instances for the lifetime of a session.
#[derive(Debug)]
pub struct SolAgent {
    wallet: Wallet,
    rpc: RpcClient,
    config: Config,
}

impl SolAgent {
    /// Create a new agent.
    #[must_use]
    pub fn new(wallet: Wallet, rpc: RpcClient, config: Config) -> Self {
        Self {
            wallet,
            rpc,
            config,
        }
    }

    /// The agent's wallet.
    #[must_use]
    pub fn wallet(&self) -> &Wallet {
        &self.wallet
    }

    /// The RPC client.
    #[must_use]
    pub fn rpc(&self) -> &RpcClient {
        &self.rpc
    }

    /// The agent configuration.
    #[must_use]
    pub fn config(&self) -> &Config {
        &self.config
    }
}
