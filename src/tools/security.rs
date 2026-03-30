//! Security tools: `GoPlus` token security, Rugcheck.

use std::sync::Arc;

use comp_cat_rs::effect::io::Io;
use rig_cat::tool::{Tool, ToolDefinition};
use serde::Deserialize;
use serde_json::Value;

use crate::agent::SolAgent;
use crate::error::Error;

// ---------------------------------------------------------------------------
// GoPlus response types (private)
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct GoPlusResponse {
    result: std::collections::HashMap<String, GoPlusTokenData>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct GoPlusTokenData {
    #[serde(default)]
    is_open_source: Option<String>,
    #[serde(default)]
    is_proxy: Option<String>,
    #[serde(default)]
    is_mintable: Option<String>,
    #[serde(default)]
    owner_change_balance: Option<String>,
    #[serde(default)]
    buy_tax: Option<String>,
    #[serde(default)]
    sell_tax: Option<String>,
    #[serde(default)]
    holder_count: Option<String>,
}

// ---------------------------------------------------------------------------
// GoPlusTokenSecurity
// ---------------------------------------------------------------------------

/// Get token security analysis from `GoPlus`.
pub struct GoPlusTokenSecurity {
    agent: Arc<SolAgent>,
}

impl GoPlusTokenSecurity {
    /// Create a new `GoPlusTokenSecurity` tool.
    #[must_use]
    pub fn new(agent: Arc<SolAgent>) -> Self {
        Self { agent }
    }
}

impl Tool for GoPlusTokenSecurity {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "goplus_token_security".into(),
            "Get token security analysis from GoPlus (honeypot, taxes, mintable, etc.).".into(),
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
        let _agent = Arc::clone(&self.agent);
        Io::suspend(move || {
            let mint = args
                .get("mint")
                .and_then(Value::as_str)
                .ok_or_else(|| Error::InvalidInput {
                    field: "mint".into(),
                    reason: "missing or not a string".into(),
                })?;

            // GoPlus Solana chain ID is "solana"
            let url = format!(
                "https://api.gopluslabs.io/api/v1/solana/token_security?contract_addresses={mint}"
            );
            let resp: GoPlusResponse = ureq::get(&url)
                .call()
                .map_err(Error::Http)?
                .into_body()
                .read_json()
                .map_err(|e| Error::Api { service: "api".into(), status: 0, message: e.to_string() })?;

            resp.result
                .get(mint)
                .or_else(|| resp.result.get(&mint.to_lowercase()))
                .map(|data| {
                    serde_json::json!({
                        "mint": mint,
                        "is_open_source": data.is_open_source,
                        "is_proxy": data.is_proxy,
                        "is_mintable": data.is_mintable,
                        "owner_change_balance": data.owner_change_balance,
                        "buy_tax": data.buy_tax,
                        "sell_tax": data.sell_tax,
                        "holder_count": data.holder_count,
                    })
                })
                .ok_or_else(|| Error::Api {
                    service: "goplus".into(),
                    status: 404,
                    message: format!("no security data for {mint}"),
                })
                
        })
        .map_error(rig_cat::error::Error::from)
    }
}

// ---------------------------------------------------------------------------
// Rugcheck response types (private)
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct RugcheckResponse {
    #[serde(default)]
    score: Option<f64>,
    #[serde(default)]
    risks: Vec<RugcheckRisk>,
}

#[derive(Deserialize)]
struct RugcheckRisk {
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    level: Option<String>,
}

// ---------------------------------------------------------------------------
// RugcheckToken
// ---------------------------------------------------------------------------

/// Check token safety via Rugcheck.
pub struct RugcheckToken {
    agent: Arc<SolAgent>,
}

impl RugcheckToken {
    /// Create a new `RugcheckToken` tool.
    #[must_use]
    pub fn new(agent: Arc<SolAgent>) -> Self {
        Self { agent }
    }
}

impl Tool for RugcheckToken {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "rugcheck_token".into(),
            "Check token safety and risk score via Rugcheck.".into(),
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
        let _agent = Arc::clone(&self.agent);
        Io::suspend(move || {
            let mint = args
                .get("mint")
                .and_then(Value::as_str)
                .ok_or_else(|| Error::InvalidInput {
                    field: "mint".into(),
                    reason: "missing or not a string".into(),
                })?;

            let url = format!("https://api.rugcheck.xyz/v1/tokens/{mint}/report");
            let resp: RugcheckResponse = ureq::get(&url)
                .call()
                .map_err(Error::Http)?
                .into_body()
                .read_json()
                .map_err(|e| Error::Api { service: "api".into(), status: 0, message: e.to_string() })?;

            let risk_level = resp.score.map_or("unknown", |s| match s {
                s if s >= 80.0 => "low",
                s if s >= 50.0 => "medium",
                _ => "high",
            });

            let risks: Vec<Value> = resp
                .risks
                .iter()
                .map(|r| {
                    serde_json::json!({
                        "name": r.name,
                        "description": r.description,
                        "level": r.level,
                    })
                })
                .collect();

            Ok::<Value, crate::error::Error>(serde_json::json!({
                "mint": mint,
                "score": resp.score,
                "risk_level": risk_level,
                "risks": risks,
            }))
            
        })
        .map_error(rig_cat::error::Error::from)
    }
}
