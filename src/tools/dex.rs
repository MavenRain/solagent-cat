//! DEX data tools: `DexScreener` pairs.

use std::sync::Arc;

use comp_cat_rs::effect::io::Io;
use rig_cat::tool::{Tool, ToolDefinition};
use serde::Deserialize;
use serde_json::Value;

use crate::agent::SolAgent;
use crate::error::Error;

// ---------------------------------------------------------------------------
// DexScreener response types (private)
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct DexScreenerPair {
    pair_address: String,
    #[serde(default)]
    dex_id: Option<String>,
    #[serde(default)]
    price_usd: Option<String>,
    #[serde(default)]
    price_native: Option<String>,
    #[serde(default)]
    volume: Option<DexScreenerVolume>,
    #[serde(default)]
    liquidity: Option<DexScreenerLiquidity>,
    #[serde(default)]
    price_change: Option<DexScreenerPriceChange>,
    #[serde(default)]
    base_token: Option<DexScreenerToken>,
    #[serde(default)]
    quote_token: Option<DexScreenerToken>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct DexScreenerVolume {
    #[serde(default)]
    h24: Option<f64>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct DexScreenerLiquidity {
    #[serde(default)]
    usd: Option<f64>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct DexScreenerPriceChange {
    #[serde(default)]
    h24: Option<f64>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct DexScreenerToken {
    #[serde(default)]
    address: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    symbol: Option<String>,
}

#[derive(Deserialize)]
struct DexScreenerResponse {
    #[serde(default)]
    pairs: Option<Vec<DexScreenerPair>>,
}

// ---------------------------------------------------------------------------
// DexScreenerPairs
// ---------------------------------------------------------------------------

/// Look up DEX trading pairs for a token via `DexScreener`.
pub struct DexScreenerPairs {
    agent: Arc<SolAgent>,
}

impl DexScreenerPairs {
    /// Create a new `DexScreenerPairs` tool.
    #[must_use]
    pub fn new(agent: Arc<SolAgent>) -> Self {
        Self { agent }
    }
}

impl Tool for DexScreenerPairs {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "dexscreener_pairs".into(),
            "Look up DEX trading pairs for a token via DexScreener.".into(),
            serde_json::json!({
                "type": "object",
                "properties": {
                    "token_address": {
                        "type": "string",
                        "description": "Token address to look up pairs for"
                    }
                },
                "required": ["token_address"]
            }),
        )
    }

    fn call(&self, args: Value) -> Io<rig_cat::error::Error, Value> {
        let _agent = Arc::clone(&self.agent);
        Io::suspend(move || {
            let token_address = args
                .get("token_address")
                .and_then(Value::as_str)
                .ok_or_else(|| Error::InvalidInput {
                    field: "token_address".into(),
                    reason: "missing or not a string".into(),
                })?;

            let url = format!(
                "https://api.dexscreener.com/latest/dex/tokens/{token_address}"
            );
            let resp: DexScreenerResponse = ureq::get(&url)
                .call()
                .map_err(Error::Http)?
                .into_body()
                .read_json()
                .map_err(|e| Error::Api { service: "api".into(), status: 0, message: e.to_string() })?;

            let pairs: Vec<Value> = resp
                .pairs
                .unwrap_or_default()
                .iter()
                .map(|p| {
                    serde_json::json!({
                        "pair_address": p.pair_address,
                        "dex": p.dex_id,
                        "price_usd": p.price_usd,
                        "volume_24h": p.volume.as_ref().and_then(|v| v.h24),
                        "liquidity_usd": p.liquidity.as_ref().and_then(|l| l.usd),
                        "price_change_24h_pct": p.price_change.as_ref().and_then(|pc| pc.h24),
                        "base_token": p.base_token.as_ref().map(|t| {
                            serde_json::json!({
                                "address": t.address,
                                "name": t.name,
                                "symbol": t.symbol,
                            })
                        }),
                        "quote_token": p.quote_token.as_ref().map(|t| {
                            serde_json::json!({
                                "address": t.address,
                                "name": t.name,
                                "symbol": t.symbol,
                            })
                        }),
                    })
                })
                .collect();

            Ok::<Value, crate::error::Error>(serde_json::json!({
                "token_address": token_address,
                "pairs": pairs,
            }))
            
        })
        .map_error(rig_cat::error::Error::from)
    }
}
