//! Token tools: balance, token data, transfer, deploy.

use std::sync::Arc;

use comp_cat_rs::effect::io::Io;
use rig_cat::tool::{Tool, ToolDefinition};
use serde_json::Value;
use solana_sdk::program_pack::Pack;
#[allow(deprecated)]
use solana_sdk::system_instruction;
use solana_sdk::signer::Signer;

use crate::agent::SolAgent;
use crate::error::Error;
use crate::types::{Lamports, MintAddress, Pubkey, Sol, TokenAmount, TokenDecimals};

// ---------------------------------------------------------------------------
// GetBalance
// ---------------------------------------------------------------------------

/// Get the SOL or SPL token balance of an address.
pub struct GetBalance {
    agent: Arc<SolAgent>,
}

impl GetBalance {
    /// Create a new `GetBalance` tool.
    #[must_use]
    pub fn new(agent: Arc<SolAgent>) -> Self {
        Self { agent }
    }
}

impl Tool for GetBalance {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "get_balance".into(),
            "Get the SOL or SPL token balance of an address. Defaults to the agent wallet."
                .into(),
            serde_json::json!({
                "type": "object",
                "properties": {
                    "address": {
                        "type": "string",
                        "description": "Base58 public key. Omit for agent wallet."
                    },
                    "mint": {
                        "type": "string",
                        "description": "Token mint address (base58). Omit for SOL balance."
                    }
                },
                "required": []
            }),
        )
    }

    fn call(&self, args: Value) -> Io<rig_cat::error::Error, Value> {
        let agent = Arc::clone(&self.agent);
        Io::suspend(move || {
            let address = args
                .get("address")
                .and_then(Value::as_str)
                .map(str::parse::<Pubkey>)
                .transpose()?
                .unwrap_or_else(|| agent.wallet().pubkey());

            match args.get("mint").and_then(Value::as_str) {
                Some(mint_str) => {
                    let mint: MintAddress = mint_str.parse()?;
                    let ata = spl_associated_token_account::get_associated_token_address(
                        address.as_inner(),
                        mint.pubkey().as_inner(),
                    );
                    let ata_pubkey = Pubkey::new(ata);
                    agent
                        .rpc()
                        .get_token_account_balance(&ata_pubkey)
                        .run()
                        .map(|bal| {
                            serde_json::json!({
                                "address": address.to_base58(),
                                "mint": mint.to_base58(),
                                "balance": bal.amount.ui_amount_string,
                                "raw_amount": bal.amount.amount,
                                "decimals": bal.amount.decimals,
                            })
                        })
                        
                }
                None => agent
                    .rpc()
                    .get_balance(&address)
                    .run()
                    .map(|lamports| {
                        let sol: Sol = lamports.into();
                        serde_json::json!({
                            "address": address.to_base58(),
                            "balance": sol.value(),
                            "raw_amount": lamports.value(),
                            "decimals": 9,
                        })
                    })
                    ,
            }
        })
        .map_error(rig_cat::error::Error::from)
    }
}

// ---------------------------------------------------------------------------
// GetTokenData
// ---------------------------------------------------------------------------

/// Get metadata for an SPL token mint (supply, decimals, authorities).
pub struct GetTokenData {
    agent: Arc<SolAgent>,
}

impl GetTokenData {
    /// Create a new `GetTokenData` tool.
    #[must_use]
    pub fn new(agent: Arc<SolAgent>) -> Self {
        Self { agent }
    }
}

impl Tool for GetTokenData {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "get_token_data".into(),
            "Get metadata for an SPL token mint: supply, decimals, authorities.".into(),
            serde_json::json!({
                "type": "object",
                "properties": {
                    "mint": {
                        "type": "string",
                        "description": "Token mint address (base58)"
                    }
                },
                "required": ["mint"]
            }),
        )
    }

    fn call(&self, args: Value) -> Io<rig_cat::error::Error, Value> {
        let agent = Arc::clone(&self.agent);
        Io::suspend(move || {
            let mint_str = args
                .get("mint")
                .and_then(Value::as_str)
                .ok_or_else(|| Error::InvalidInput {
                    field: "mint".into(),
                    reason: "missing or not a string".into(),
                })?;
            let mint: MintAddress = mint_str.parse()?;
            agent
                .rpc()
                .get_account(mint.pubkey())
                .run()
                .and_then(|account| {
                    spl_token::state::Mint::unpack(&account.data).map_err(|e| Error::Token {
                        message: e.to_string(),
                    })
                })
                .map(|mint_data| {
                    let freeze_auth: Option<String> =
                        Option::<solana_sdk::pubkey::Pubkey>::from(mint_data.freeze_authority)
                            .map(|pk| pk.to_string());
                    let mint_auth: Option<String> =
                        Option::<solana_sdk::pubkey::Pubkey>::from(mint_data.mint_authority)
                            .map(|pk| pk.to_string());
                    serde_json::json!({
                        "mint": mint.to_base58(),
                        "decimals": mint_data.decimals,
                        "supply": mint_data.supply.to_string(),
                        "is_initialized": mint_data.is_initialized,
                        "freeze_authority": freeze_auth,
                        "mint_authority": mint_auth,
                    })
                })
                
        })
        .map_error(rig_cat::error::Error::from)
    }
}

// ---------------------------------------------------------------------------
// TransferTokens
// ---------------------------------------------------------------------------

/// Transfer SOL or SPL tokens to a recipient.
pub struct TransferTokens {
    agent: Arc<SolAgent>,
}

impl TransferTokens {
    /// Create a new `TransferTokens` tool.
    #[must_use]
    pub fn new(agent: Arc<SolAgent>) -> Self {
        Self { agent }
    }
}

impl Tool for TransferTokens {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "transfer_tokens".into(),
            "Transfer SOL or SPL tokens to a recipient address.".into(),
            serde_json::json!({
                "type": "object",
                "properties": {
                    "to": {
                        "type": "string",
                        "description": "Recipient address (base58)"
                    },
                    "amount": {
                        "type": "number",
                        "description": "Amount in human-readable units (SOL or token units)"
                    },
                    "mint": {
                        "type": "string",
                        "description": "Token mint address (base58). Omit for SOL transfer."
                    }
                },
                "required": ["to", "amount"]
            }),
        )
    }

    fn call(&self, args: Value) -> Io<rig_cat::error::Error, Value> {
        let agent = Arc::clone(&self.agent);
        Io::suspend(move || {
            let to_str = args
                .get("to")
                .and_then(Value::as_str)
                .ok_or_else(|| Error::InvalidInput {
                    field: "to".into(),
                    reason: "missing or not a string".into(),
                })?;
            let to: Pubkey = to_str.parse()?;
            let amount = args
                .get("amount")
                .and_then(Value::as_f64)
                .ok_or_else(|| Error::InvalidInput {
                    field: "amount".into(),
                    reason: "missing or not a number".into(),
                })?;

            match args.get("mint").and_then(Value::as_str) {
                Some(mint_str) => transfer_spl(&agent, &to, amount, mint_str),
                None => transfer_sol(&agent, &to, amount),
            }
            
        })
        .map_error(rig_cat::error::Error::from)
    }
}

fn transfer_sol(agent: &SolAgent, to: &Pubkey, amount_sol: f64) -> Result<Value, Error> {
    let lamports: Lamports = Sol::new(amount_sol).into();
    let from = agent.wallet().pubkey();
    let ix = system_instruction::transfer(from.as_inner(), to.as_inner(), lamports.value());
    let blockhash = agent.rpc().get_latest_blockhash().run()?;
    let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
        &[ix],
        Some(from.as_inner()),
        &[agent.wallet().signer()],
        blockhash,
    );
    let sig = agent.rpc().send_and_confirm_transaction(&tx).run()?;
    Ok::<Value, crate::error::Error>(serde_json::json!({
        "signature": sig.to_base58(),
        "from": from.to_base58(),
        "to": to.to_base58(),
        "amount_sol": amount_sol,
        "lamports": lamports.value(),
    }))
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn transfer_spl(
    agent: &SolAgent,
    to: &Pubkey,
    amount: f64,
    mint_str: &str,
) -> Result<Value, Error> {
    let mint: MintAddress = mint_str.parse()?;
    let from_pubkey = agent.wallet().pubkey();

    let mint_account = agent.rpc().get_account(mint.pubkey()).run()?;
    let mint_data =
        spl_token::state::Mint::unpack(&mint_account.data).map_err(|e| Error::Token {
            message: e.to_string(),
        })?;
    let decimals = TokenDecimals::new(mint_data.decimals);
    let raw_amount =
        TokenAmount::new((amount * 10_f64.powi(i32::from(decimals.value()))) as u64);

    let from_ata = spl_associated_token_account::get_associated_token_address(
        from_pubkey.as_inner(),
        mint.pubkey().as_inner(),
    );
    let to_ata = spl_associated_token_account::get_associated_token_address(
        to.as_inner(),
        mint.pubkey().as_inner(),
    );

    let create_ata_ix =
        spl_associated_token_account::instruction::create_associated_token_account_idempotent(
            from_pubkey.as_inner(),
            to.as_inner(),
            mint.pubkey().as_inner(),
            &spl_token::id(),
        );

    let transfer_ix = spl_token::instruction::transfer(
        &spl_token::id(),
        &from_ata,
        &to_ata,
        from_pubkey.as_inner(),
        &[],
        raw_amount.value(),
    )
    .map_err(|e| Error::Token {
        message: e.to_string(),
    })?;

    let blockhash = agent.rpc().get_latest_blockhash().run()?;
    let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
        &[create_ata_ix, transfer_ix],
        Some(from_pubkey.as_inner()),
        &[agent.wallet().signer()],
        blockhash,
    );
    let sig = agent.rpc().send_and_confirm_transaction(&tx).run()?;
    Ok::<Value, crate::error::Error>(serde_json::json!({
        "signature": sig.to_base58(),
        "from": from_pubkey.to_base58(),
        "to": to.to_base58(),
        "mint": mint.to_base58(),
        "amount": amount,
        "raw_amount": raw_amount.value(),
        "decimals": decimals.value(),
    }))
}

// ---------------------------------------------------------------------------
// DeployToken
// ---------------------------------------------------------------------------

/// Deploy a new SPL token mint.
pub struct DeployToken {
    agent: Arc<SolAgent>,
}

impl DeployToken {
    /// Create a new `DeployToken` tool.
    #[must_use]
    pub fn new(agent: Arc<SolAgent>) -> Self {
        Self { agent }
    }
}

impl Tool for DeployToken {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "deploy_token".into(),
            "Deploy a new SPL token mint with the given decimals and optional initial supply."
                .into(),
            serde_json::json!({
                "type": "object",
                "properties": {
                    "decimals": {
                        "type": "integer",
                        "description": "Number of decimal places (0-18)",
                        "minimum": 0,
                        "maximum": 18
                    },
                    "initial_supply": {
                        "type": "number",
                        "description": "Initial supply in human-readable units. Omit for zero."
                    }
                },
                "required": ["decimals"]
            }),
        )
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn call(&self, args: Value) -> Io<rig_cat::error::Error, Value> {
        let agent = Arc::clone(&self.agent);
        Io::suspend(move || {
            let decimals_val = args
                .get("decimals")
                .and_then(Value::as_u64)
                .ok_or_else(|| Error::InvalidInput {
                    field: "decimals".into(),
                    reason: "missing or not an integer".into(),
                })?;
            let decimals = TokenDecimals::new(decimals_val as u8);
            let initial_supply = args.get("initial_supply").and_then(Value::as_f64);

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

            let init_ix = spl_token::instruction::initialize_mint(
                &spl_token::id(),
                mint_pubkey.as_inner(),
                payer.as_inner(),
                Some(payer.as_inner()),
                decimals.value(),
            )
            .map_err(|e| Error::Token {
                message: e.to_string(),
            })?;

            let blockhash = agent.rpc().get_latest_blockhash().run()?;

            let mut instructions = vec![create_ix, init_ix];

            let ata = spl_associated_token_account::get_associated_token_address(
                payer.as_inner(),
                mint_pubkey.as_inner(),
            );

            match initial_supply {
                Some(supply) if supply > 0.0 => {
                    let raw =
                        (supply * 10_f64.powi(i32::from(decimals.value()))) as u64;

                    let create_ata_ix = spl_associated_token_account::instruction::create_associated_token_account_idempotent(
                        payer.as_inner(),
                        payer.as_inner(),
                        mint_pubkey.as_inner(),
                        &spl_token::id(),
                    );

                    let mint_to_ix = spl_token::instruction::mint_to(
                        &spl_token::id(),
                        mint_pubkey.as_inner(),
                        &ata,
                        payer.as_inner(),
                        &[],
                        raw,
                    )
                    .map_err(|e| Error::Token {
                        message: e.to_string(),
                    })?;

                    instructions.push(create_ata_ix);
                    instructions.push(mint_to_ix);
                }
                _ => {}
            }

            let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
                &instructions,
                Some(payer.as_inner()),
                &[agent.wallet().signer(), &mint_keypair],
                blockhash,
            );
            let sig = agent.rpc().send_and_confirm_transaction(&tx).run()?;
            Ok::<Value, crate::error::Error>(serde_json::json!({
                "mint": mint_pubkey.to_base58(),
                "signature": sig.to_base58(),
                "decimals": decimals.value(),
            }))
            
        })
        .map_error(rig_cat::error::Error::from)
    }
}
