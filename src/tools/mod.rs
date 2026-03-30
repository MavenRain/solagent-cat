//! Solana tools for rig-cat agents.
//!
//! Each tool implements [`rig_cat::tool::Tool`] and holds an
//! `Arc<SolAgent>` for shared access to wallet, RPC, and config.
//!
//! The [`SolanaTools`] enum provides heterogeneous dispatch
//! (no `dyn Trait`) for use with [`rig_cat::tool::Toolbox`].
//!
//! Use [`all_tools`] to construct a toolbox with every tool wired
//! to a shared agent.

pub mod dex;
pub mod nft;
pub mod price;
pub mod security;
pub mod token;
pub mod trading;

use std::sync::Arc;

use comp_cat_rs::effect::io::Io;
use rig_cat::tool::{Tool, ToolDefinition, Toolbox};
use serde_json::Value;

use crate::agent::SolAgent;

/// All Solana tools as a single enum for use with `Toolbox<SolanaTools>`.
pub enum SolanaTools {
    // Token
    /// Get SOL or SPL token balance.
    GetBalance(token::GetBalance),
    /// Get SPL token mint metadata.
    GetTokenData(token::GetTokenData),
    /// Transfer SOL or SPL tokens.
    TransferTokens(token::TransferTokens),
    /// Deploy a new SPL token mint.
    DeployToken(token::DeployToken),
    // NFT
    /// Mint a new NFT.
    MintNft(nft::MintNft),
    /// Deploy an NFT collection.
    DeployCollection(nft::DeployCollection),
    // Trading
    /// Swap tokens via Jupiter.
    JupiterSwap(trading::JupiterSwap),
    /// Get token price from Jupiter.
    JupiterPrice(trading::JupiterPrice),
    /// Stake SOL via Jupiter liquid staking.
    StakeWithJup(trading::StakeWithJup),
    // Price
    /// Get price from Pyth oracle.
    PythPrice(price::PythPrice),
    /// Get token price from Birdeye.
    BirdeyePrice(price::BirdeyePrice),
    /// Get full token analytics from Birdeye.
    BirdeyeTokenOverview(price::BirdeyeTokenOverview),
    // Security
    /// Get token security analysis from `GoPlus`.
    GoPlusTokenSecurity(security::GoPlusTokenSecurity),
    /// Check token safety via Rugcheck.
    RugcheckToken(security::RugcheckToken),
    // DEX
    /// Look up DEX trading pairs via `DexScreener`.
    DexScreenerPairs(dex::DexScreenerPairs),
}

impl Tool for SolanaTools {
    fn definition(&self) -> ToolDefinition {
        match self {
            Self::GetBalance(t) => t.definition(),
            Self::GetTokenData(t) => t.definition(),
            Self::TransferTokens(t) => t.definition(),
            Self::DeployToken(t) => t.definition(),
            Self::MintNft(t) => t.definition(),
            Self::DeployCollection(t) => t.definition(),
            Self::JupiterSwap(t) => t.definition(),
            Self::JupiterPrice(t) => t.definition(),
            Self::StakeWithJup(t) => t.definition(),
            Self::PythPrice(t) => t.definition(),
            Self::BirdeyePrice(t) => t.definition(),
            Self::BirdeyeTokenOverview(t) => t.definition(),
            Self::GoPlusTokenSecurity(t) => t.definition(),
            Self::RugcheckToken(t) => t.definition(),
            Self::DexScreenerPairs(t) => t.definition(),
        }
    }

    fn call(&self, args: Value) -> Io<rig_cat::error::Error, Value> {
        match self {
            Self::GetBalance(t) => t.call(args),
            Self::GetTokenData(t) => t.call(args),
            Self::TransferTokens(t) => t.call(args),
            Self::DeployToken(t) => t.call(args),
            Self::MintNft(t) => t.call(args),
            Self::DeployCollection(t) => t.call(args),
            Self::JupiterSwap(t) => t.call(args),
            Self::JupiterPrice(t) => t.call(args),
            Self::StakeWithJup(t) => t.call(args),
            Self::PythPrice(t) => t.call(args),
            Self::BirdeyePrice(t) => t.call(args),
            Self::BirdeyeTokenOverview(t) => t.call(args),
            Self::GoPlusTokenSecurity(t) => t.call(args),
            Self::RugcheckToken(t) => t.call(args),
            Self::DexScreenerPairs(t) => t.call(args),
        }
    }
}

/// Construct a [`Toolbox`] with all Solana tools wired to the given agent.
#[must_use]
#[allow(clippy::needless_pass_by_value)]
pub fn all_tools(agent: Arc<SolAgent>) -> Toolbox<SolanaTools> {
    Toolbox::new()
        .with_tool(SolanaTools::GetBalance(token::GetBalance::new(Arc::clone(&agent))))
        .with_tool(SolanaTools::GetTokenData(token::GetTokenData::new(Arc::clone(&agent))))
        .with_tool(SolanaTools::TransferTokens(token::TransferTokens::new(Arc::clone(&agent))))
        .with_tool(SolanaTools::DeployToken(token::DeployToken::new(Arc::clone(&agent))))
        .with_tool(SolanaTools::MintNft(nft::MintNft::new(Arc::clone(&agent))))
        .with_tool(SolanaTools::DeployCollection(nft::DeployCollection::new(Arc::clone(&agent))))
        .with_tool(SolanaTools::JupiterSwap(trading::JupiterSwap::new(Arc::clone(&agent))))
        .with_tool(SolanaTools::JupiterPrice(trading::JupiterPrice::new(Arc::clone(&agent))))
        .with_tool(SolanaTools::StakeWithJup(trading::StakeWithJup::new(Arc::clone(&agent))))
        .with_tool(SolanaTools::PythPrice(price::PythPrice::new(Arc::clone(&agent))))
        .with_tool(SolanaTools::BirdeyePrice(price::BirdeyePrice::new(Arc::clone(&agent))))
        .with_tool(SolanaTools::BirdeyeTokenOverview(price::BirdeyeTokenOverview::new(Arc::clone(&agent))))
        .with_tool(SolanaTools::GoPlusTokenSecurity(security::GoPlusTokenSecurity::new(Arc::clone(&agent))))
        .with_tool(SolanaTools::RugcheckToken(security::RugcheckToken::new(Arc::clone(&agent))))
        .with_tool(SolanaTools::DexScreenerPairs(dex::DexScreenerPairs::new(agent)))
}
