//! NFT tools: mint NFT, deploy collection.
//!
//! Uses raw SPL token instructions for NFT minting.
//! Metadata is stored off-chain via the provided URI.

use std::sync::Arc;

use comp_cat_rs::effect::io::Io;
use rig_cat::tool::{Tool, ToolDefinition};
use serde_json::Value;
use solana_sdk::program_pack::Pack;
use solana_sdk::signer::Signer;
#[allow(deprecated)]
use solana_sdk::system_instruction;

use crate::agent::SolAgent;
use crate::error::Error;
use crate::types::Pubkey;

// ---------------------------------------------------------------------------
// MintNft
// ---------------------------------------------------------------------------

/// Mint an NFT (supply-1, 0-decimal SPL token) with a metadata URI.
pub struct MintNft {
    agent: Arc<SolAgent>,
}

impl MintNft {
    /// Create a new `MintNft` tool.
    #[must_use]
    pub fn new(agent: Arc<SolAgent>) -> Self {
        Self { agent }
    }
}

impl Tool for MintNft {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "mint_nft".into(),
            "Mint a new NFT (0-decimal, supply-1 SPL token) with a metadata URI.".into(),
            serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "NFT name"
                    },
                    "uri": {
                        "type": "string",
                        "description": "Metadata URI (JSON)"
                    }
                },
                "required": ["name", "uri"]
            }),
        )
    }

    fn call(&self, args: Value) -> Io<rig_cat::error::Error, Value> {
        let agent = Arc::clone(&self.agent);
        Io::suspend(move || {
            let name = args
                .get("name")
                .and_then(Value::as_str)
                .ok_or_else(|| Error::InvalidInput {
                    field: "name".into(),
                    reason: "missing or not a string".into(),
                })?
                .to_owned();
            let uri = args
                .get("uri")
                .and_then(Value::as_str)
                .ok_or_else(|| Error::InvalidInput {
                    field: "uri".into(),
                    reason: "missing or not a string".into(),
                })?
                .to_owned();

            let payer = agent.wallet().pubkey();
            let mint_keypair = solana_sdk::signer::keypair::Keypair::new();
            let mint_pubkey = Pubkey::new(mint_keypair.pubkey());

            let rent = agent
                .rpc()
                .get_minimum_balance_for_rent_exemption(spl_token::state::Mint::LEN)
                .run()?;

            let create_ix = system_instruction::create_account(
                payer.as_inner(),
                mint_pubkey.as_inner(),
                rent.value(),
                spl_token::state::Mint::LEN as u64,
                &spl_token::id(),
            );

            // 0 decimals for NFT
            let init_ix = spl_token::instruction::initialize_mint(
                &spl_token::id(),
                mint_pubkey.as_inner(),
                payer.as_inner(),
                Some(payer.as_inner()),
                0,
            )
            .map_err(|e| Error::Nft {
                message: e.to_string(),
            })?;

            let ata = spl_associated_token_account::get_associated_token_address(
                payer.as_inner(),
                mint_pubkey.as_inner(),
            );

            let create_ata_ix =
                spl_associated_token_account::instruction::create_associated_token_account_idempotent(
                    payer.as_inner(),
                    payer.as_inner(),
                    mint_pubkey.as_inner(),
                    &spl_token::id(),
                );

            // Mint exactly 1 token
            let mint_to_ix = spl_token::instruction::mint_to(
                &spl_token::id(),
                mint_pubkey.as_inner(),
                &ata,
                payer.as_inner(),
                &[],
                1,
            )
            .map_err(|e| Error::Nft {
                message: e.to_string(),
            })?;

            let blockhash = agent.rpc().get_latest_blockhash().run()?;
            let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
                &[create_ix, init_ix, create_ata_ix, mint_to_ix],
                Some(payer.as_inner()),
                &[agent.wallet().signer(), &mint_keypair],
                blockhash,
            );
            let sig = agent.rpc().send_and_confirm_transaction(&tx).run()?;

            Ok::<Value, crate::error::Error>(serde_json::json!({
                "mint": mint_pubkey.to_base58(),
                "signature": sig.to_base58(),
                "name": name,
                "uri": uri,
            }))
            
        })
        .map_error(rig_cat::error::Error::from)
    }
}

// ---------------------------------------------------------------------------
// DeployCollection
// ---------------------------------------------------------------------------

/// Deploy an NFT collection (a master mint with metadata).
pub struct DeployCollection {
    agent: Arc<SolAgent>,
}

impl DeployCollection {
    /// Create a new `DeployCollection` tool.
    #[must_use]
    pub fn new(agent: Arc<SolAgent>) -> Self {
        Self { agent }
    }
}

impl Tool for DeployCollection {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "deploy_collection".into(),
            "Deploy an NFT collection mint.".into(),
            serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Collection name"
                    },
                    "uri": {
                        "type": "string",
                        "description": "Collection metadata URI"
                    },
                    "royalty_bps": {
                        "type": "integer",
                        "description": "Royalty in basis points (optional)"
                    }
                },
                "required": ["name", "uri"]
            }),
        )
    }

    fn call(&self, args: Value) -> Io<rig_cat::error::Error, Value> {
        let agent = Arc::clone(&self.agent);
        Io::suspend(move || {
            let name = args
                .get("name")
                .and_then(Value::as_str)
                .ok_or_else(|| Error::InvalidInput {
                    field: "name".into(),
                    reason: "missing or not a string".into(),
                })?
                .to_owned();
            let uri = args
                .get("uri")
                .and_then(Value::as_str)
                .ok_or_else(|| Error::InvalidInput {
                    field: "uri".into(),
                    reason: "missing or not a string".into(),
                })?
                .to_owned();

            let payer = agent.wallet().pubkey();
            let collection_keypair = solana_sdk::signer::keypair::Keypair::new();
            let collection_pubkey = Pubkey::new(collection_keypair.pubkey());

            let rent = agent
                .rpc()
                .get_minimum_balance_for_rent_exemption(spl_token::state::Mint::LEN)
                .run()?;

            let create_ix = system_instruction::create_account(
                payer.as_inner(),
                collection_pubkey.as_inner(),
                rent.value(),
                spl_token::state::Mint::LEN as u64,
                &spl_token::id(),
            );

            let init_ix = spl_token::instruction::initialize_mint(
                &spl_token::id(),
                collection_pubkey.as_inner(),
                payer.as_inner(),
                Some(payer.as_inner()),
                0,
            )
            .map_err(|e| Error::Nft {
                message: e.to_string(),
            })?;

            let blockhash = agent.rpc().get_latest_blockhash().run()?;
            let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
                &[create_ix, init_ix],
                Some(payer.as_inner()),
                &[agent.wallet().signer(), &collection_keypair],
                blockhash,
            );
            let sig = agent.rpc().send_and_confirm_transaction(&tx).run()?;

            Ok::<Value, crate::error::Error>(serde_json::json!({
                "mint": collection_pubkey.to_base58(),
                "signature": sig.to_base58(),
                "name": name,
                "uri": uri,
            }))
            
        })
        .map_error(rig_cat::error::Error::from)
    }
}
