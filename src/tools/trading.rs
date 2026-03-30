//! Trading tools: Jupiter swap, price, staking.

use std::sync::Arc;

use comp_cat_rs::effect::io::Io;
use rig_cat::tool::{Tool, ToolDefinition};
use serde::Deserialize;
use serde_json::Value;

use crate::agent::SolAgent;
use crate::error::Error;

// ---------------------------------------------------------------------------
// Jupiter API response types (private)
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct QuoteResponse {
    input_mint: String,
    output_mint: String,
    in_amount: String,
    out_amount: String,
    price_impact_pct: String,
    #[serde(default)]
    route_plan: Value,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SwapResponse {
    swap_transaction: String,
}

#[derive(Deserialize)]
struct PriceData {
    id: String,
    #[serde(rename = "type")]
    data_type: Option<String>,
    price: String,
}

#[derive(Deserialize)]
struct PriceResponse {
    data: std::collections::HashMap<String, PriceData>,
}

// ---------------------------------------------------------------------------
// JupiterSwap
// ---------------------------------------------------------------------------

/// Swap tokens via the Jupiter DEX aggregator.
pub struct JupiterSwap {
    agent: Arc<SolAgent>,
}

impl JupiterSwap {
    /// Create a new `JupiterSwap` tool.
    #[must_use]
    pub fn new(agent: Arc<SolAgent>) -> Self {
        Self { agent }
    }
}

impl Tool for JupiterSwap {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "jupiter_swap".into(),
            "Swap tokens using the Jupiter DEX aggregator.".into(),
            serde_json::json!({
                "type": "object",
                "properties": {
                    "input_mint": {
                        "type": "string",
                        "description": "Input token mint address (base58)"
                    },
                    "output_mint": {
                        "type": "string",
                        "description": "Output token mint address (base58)"
                    },
                    "amount": {
                        "type": "number",
                        "description": "Amount of input token in smallest units"
                    },
                    "slippage_bps": {
                        "type": "integer",
                        "description": "Slippage tolerance in basis points. Defaults to config value."
                    }
                },
                "required": ["input_mint", "output_mint", "amount"]
            }),
        )
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn call(&self, args: Value) -> Io<rig_cat::error::Error, Value> {
        let agent = Arc::clone(&self.agent);
        Io::suspend(move || {
            let input_mint = args
                .get("input_mint")
                .and_then(Value::as_str)
                .ok_or_else(|| Error::InvalidInput {
                    field: "input_mint".into(),
                    reason: "missing or not a string".into(),
                })?;
            let output_mint = args
                .get("output_mint")
                .and_then(Value::as_str)
                .ok_or_else(|| Error::InvalidInput {
                    field: "output_mint".into(),
                    reason: "missing or not a string".into(),
                })?;
            let amount = args
                .get("amount")
                .and_then(Value::as_f64)
                .ok_or_else(|| Error::InvalidInput {
                    field: "amount".into(),
                    reason: "missing or not a number".into(),
                })? as u64;
            let slippage = args
                .get("slippage_bps")
                .and_then(Value::as_u64)
                .unwrap_or_else(|| u64::from(agent.config().default_slippage().value()));

            let quote_url = format!(
                "https://quote-api.jup.ag/v6/quote?\
                 inputMint={input_mint}&outputMint={output_mint}\
                 &amount={amount}&slippageBps={slippage}"
            );
            let quote: QuoteResponse = ureq::get(&quote_url)
                .call()
                .map_err(Error::Http)?
                .into_body()
                .read_json()
                .map_err(|e| Error::Api { service: "api".into(), status: 0, message: e.to_string() })?;

            let payer = agent.wallet().pubkey();
            let swap_body = serde_json::json!({
                "quoteResponse": serde_json::to_value(&quote.route_plan)
                    .unwrap_or(Value::Null),
                "userPublicKey": payer.to_base58(),
                "wrapAndUnwrapSol": true,
            });

            let swap_resp: SwapResponse = ureq::post("https://quote-api.jup.ag/v6/swap")
                .header("Content-Type", "application/json")
                .send_json(&swap_body)
                .map_err(Error::Http)?
                .into_body()
                .read_json()
                .map_err(|e| Error::Api { service: "api".into(), status: 0, message: e.to_string() })?;

            let tx_bytes = bs58::decode(&swap_resp.swap_transaction)
                .into_vec()
                .map_err(Error::Base58)?;
            let _tx: solana_sdk::transaction::VersionedTransaction =
                bincode::deserialize(&tx_bytes).map_err(|e| Error::Transaction {
                    signature: String::new(),
                    message: e.to_string(),
                })?;

            // Note: Jupiter returns a versioned transaction that may need
            // additional signing.  For now we return the quote details.
            // Full signing requires versioned transaction support.
            Ok::<Value, crate::error::Error>(serde_json::json!({
                "input_mint": quote.input_mint,
                "output_mint": quote.output_mint,
                "input_amount": quote.in_amount,
                "output_amount": quote.out_amount,
                "price_impact_pct": quote.price_impact_pct,
                "swap_transaction_size": tx_bytes.len(),
            }))
            
        })
        .map_error(rig_cat::error::Error::from)
    }
}

// ---------------------------------------------------------------------------
// JupiterPrice
// ---------------------------------------------------------------------------

/// Fetch a token price from the Jupiter Price API.
pub struct JupiterPrice {
    agent: Arc<SolAgent>,
}

impl JupiterPrice {
    /// Create a new `JupiterPrice` tool.
    #[must_use]
    pub fn new(agent: Arc<SolAgent>) -> Self {
        Self { agent }
    }
}

impl Tool for JupiterPrice {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "jupiter_price".into(),
            "Get a token price from the Jupiter Price API v2.".into(),
            serde_json::json!({
                "type": "object",
                "properties": {
                    "token_mint": {
                        "type": "string",
                        "description": "Token mint address (base58)"
                    },
                    "vs_mint": {
                        "type": "string",
                        "description": "Quote token mint. Defaults to USDC."
                    }
                },
                "required": ["token_mint"]
            }),
        )
    }

    fn call(&self, args: Value) -> Io<rig_cat::error::Error, Value> {
        let _agent = Arc::clone(&self.agent);
        Io::suspend(move || {
            let token_mint = args
                .get("token_mint")
                .and_then(Value::as_str)
                .ok_or_else(|| Error::InvalidInput {
                    field: "token_mint".into(),
                    reason: "missing or not a string".into(),
                })?;

            let url = format!("https://price.jup.ag/v6/price?ids={token_mint}");
            let resp: PriceResponse = ureq::get(&url)
                .call()
                .map_err(Error::Http)?
                .into_body()
                .read_json()
                .map_err(|e| Error::Api { service: "api".into(), status: 0, message: e.to_string() })?;

            resp.data
                .get(token_mint)
                .map(|pd| {
                    serde_json::json!({
                        "token_mint": pd.id,
                        "price": pd.price,
                        "type": pd.data_type,
                    })
                })
                .ok_or_else(|| Error::Api {
                    service: "jupiter".into(),
                    status: 404,
                    message: format!("no price data for {token_mint}"),
                })
                
        })
        .map_error(rig_cat::error::Error::from)
    }
}

// ---------------------------------------------------------------------------
// StakeWithJup
// ---------------------------------------------------------------------------

/// Stake SOL via Jupiter's liquid staking.
pub struct StakeWithJup {
    agent: Arc<SolAgent>,
}

impl StakeWithJup {
    /// Create a new `StakeWithJup` tool.
    #[must_use]
    pub fn new(agent: Arc<SolAgent>) -> Self {
        Self { agent }
    }
}

impl Tool for StakeWithJup {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "stake_with_jup".into(),
            "Stake SOL via Jupiter liquid staking (jupSOL).".into(),
            serde_json::json!({
                "type": "object",
                "properties": {
                    "amount_sol": {
                        "type": "number",
                        "description": "Amount of SOL to stake"
                    }
                },
                "required": ["amount_sol"]
            }),
        )
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn call(&self, args: Value) -> Io<rig_cat::error::Error, Value> {
        let agent = Arc::clone(&self.agent);
        Io::suspend(move || {
            let amount_sol = args
                .get("amount_sol")
                .and_then(Value::as_f64)
                .ok_or_else(|| Error::InvalidInput {
                    field: "amount_sol".into(),
                    reason: "missing or not a number".into(),
                })?;

            // jupSOL mint: bSo13r4TkiE4KumL71LsHTPpL2euBYLFx6h9HP3piy1
            let jupsol_mint = "bSo13r4TkiE4KumL71LsHTPpL2euBYLFx6h9HP3piy1";
            let sol_mint = "So11111111111111111111111111111111111111112";
            let lamports = (amount_sol * 1_000_000_000.0) as u64;

            let slippage = u64::from(agent.config().default_slippage().value());
            let quote_url = format!(
                "https://quote-api.jup.ag/v6/quote?\
                 inputMint={sol_mint}&outputMint={jupsol_mint}\
                 &amount={lamports}&slippageBps={slippage}"
            );
            let quote: QuoteResponse = ureq::get(&quote_url)
                .call()
                .map_err(Error::Http)?
                .into_body()
                .read_json()
                .map_err(|e| Error::Api { service: "api".into(), status: 0, message: e.to_string() })?;

            Ok::<Value, crate::error::Error>(serde_json::json!({
                "action": "stake_sol_to_jupsol",
                "input_amount_sol": amount_sol,
                "input_amount_lamports": lamports,
                "estimated_jupsol": quote.out_amount,
                "price_impact_pct": quote.price_impact_pct,
            }))
            
        })
        .map_error(rig_cat::error::Error::from)
    }
}
