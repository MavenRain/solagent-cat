//! Price tools: Pyth oracle, Birdeye analytics.

use std::sync::Arc;

use comp_cat_rs::effect::io::Io;
use rig_cat::tool::{Tool, ToolDefinition};
use serde::Deserialize;
use serde_json::Value;

use crate::agent::SolAgent;
use crate::error::Error;

// ---------------------------------------------------------------------------
// Pyth response types (private)
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct PythParsedPrice {
    price: String,
    conf: String,
    expo: i32,
    publish_time: i64,
}

#[derive(Deserialize)]
struct PythParsedData {
    id: String,
    price: PythParsedPrice,
}

#[derive(Deserialize)]
struct PythResponse {
    parsed: Vec<PythParsedData>,
}

// ---------------------------------------------------------------------------
// PythPrice
// ---------------------------------------------------------------------------

/// Get a price from the Pyth oracle network.
pub struct PythPrice {
    agent: Arc<SolAgent>,
}

impl PythPrice {
    /// Create a new `PythPrice` tool.
    #[must_use]
    pub fn new(agent: Arc<SolAgent>) -> Self {
        Self { agent }
    }
}

impl Tool for PythPrice {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "pyth_price".into(),
            "Get a price from the Pyth oracle network by feed ID.".into(),
            serde_json::json!({
                "type": "object",
                "properties": {
                    "feed_id": {
                        "type": "string",
                        "description": "Pyth price feed ID (hex)"
                    }
                },
                "required": ["feed_id"]
            }),
        )
    }

    fn call(&self, args: Value) -> Io<rig_cat::error::Error, Value> {
        let _agent = Arc::clone(&self.agent);
        Io::suspend(move || {
            let feed_id = args
                .get("feed_id")
                .and_then(Value::as_str)
                .ok_or_else(|| Error::InvalidInput {
                    field: "feed_id".into(),
                    reason: "missing or not a string".into(),
                })?;

            let url = format!(
                "https://hermes.pyth.network/v2/updates/price/latest?ids[]={feed_id}"
            );
            let resp: PythResponse = ureq::get(&url)
                .call()
                .map_err(Error::Http)?
                .into_body()
                .read_json()
                .map_err(|e| Error::Api { service: "api".into(), status: 0, message: e.to_string() })?;

            resp.parsed
                .first()
                .map(|data| {
                    serde_json::json!({
                        "feed_id": data.id,
                        "price": data.price.price,
                        "confidence": data.price.conf,
                        "exponent": data.price.expo,
                        "publish_time": data.price.publish_time,
                    })
                })
                .ok_or_else(|| Error::Api {
                    service: "pyth".into(),
                    status: 404,
                    message: format!("no data for feed {feed_id}"),
                })
                
        })
        .map_error(rig_cat::error::Error::from)
    }
}

// ---------------------------------------------------------------------------
// Birdeye response types (private)
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct BirdeyePriceData {
    value: f64,
    #[serde(rename = "updateUnixTime")]
    update_unix_time: i64,
}

#[derive(Deserialize)]
struct BirdeyePriceResponse {
    data: BirdeyePriceData,
    success: bool,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct BirdeyeOverviewData {
    price: f64,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    symbol: Option<String>,
    #[serde(default, rename = "mc")]
    market_cap: Option<f64>,
    #[serde(default, rename = "v24hUSD")]
    volume_24h: Option<f64>,
    #[serde(default)]
    liquidity: Option<f64>,
    #[serde(default)]
    holder: Option<u64>,
    #[serde(default, rename = "priceChange24hPercent")]
    price_change_24h_pct: Option<f64>,
}

#[derive(Deserialize)]
struct BirdeyeOverviewResponse {
    data: BirdeyeOverviewData,
    success: bool,
}

// ---------------------------------------------------------------------------
// BirdeyePrice
// ---------------------------------------------------------------------------

/// Get a token price from the Birdeye API.
pub struct BirdeyePrice {
    agent: Arc<SolAgent>,
}

impl BirdeyePrice {
    /// Create a new `BirdeyePrice` tool.
    #[must_use]
    pub fn new(agent: Arc<SolAgent>) -> Self {
        Self { agent }
    }
}

impl Tool for BirdeyePrice {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "birdeye_price".into(),
            "Get a token price from the Birdeye API.".into(),
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
            let mint = args
                .get("mint")
                .and_then(Value::as_str)
                .ok_or_else(|| Error::InvalidInput {
                    field: "mint".into(),
                    reason: "missing or not a string".into(),
                })?;

            let api_key = agent.config().require_birdeye_api_key()?;
            let url = format!(
                "https://public-api.birdeye.so/defi/price?address={mint}"
            );
            let resp: BirdeyePriceResponse = ureq::get(&url)
                .header("X-API-KEY", api_key.as_str())
                .header("x-chain", "solana")
                .call()
                .map_err(Error::Http)?
                .into_body()
                .read_json()
                .map_err(|e| Error::Api { service: "api".into(), status: 0, message: e.to_string() })?;

            if resp.success {
                Ok::<Value, crate::error::Error>(serde_json::json!({
                    "mint": mint,
                    "price_usd": resp.data.value,
                    "timestamp": resp.data.update_unix_time,
                }))
            } else {
                Err(Error::Api {
                    service: "birdeye".into(),
                    status: 200,
                    message: "request returned success=false".into(),
                })
            }

        })
        .map_error(rig_cat::error::Error::from)
    }
}

// ---------------------------------------------------------------------------
// BirdeyeTokenOverview
// ---------------------------------------------------------------------------

/// Get full token analytics from the Birdeye API.
pub struct BirdeyeTokenOverview {
    agent: Arc<SolAgent>,
}

impl BirdeyeTokenOverview {
    /// Create a new `BirdeyeTokenOverview` tool.
    #[must_use]
    pub fn new(agent: Arc<SolAgent>) -> Self {
        Self { agent }
    }
}

impl Tool for BirdeyeTokenOverview {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "birdeye_token_overview".into(),
            "Get full token analytics (price, market cap, volume, holders) from Birdeye.".into(),
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
            let mint = args
                .get("mint")
                .and_then(Value::as_str)
                .ok_or_else(|| Error::InvalidInput {
                    field: "mint".into(),
                    reason: "missing or not a string".into(),
                })?;

            let api_key = agent.config().require_birdeye_api_key()?;
            let url = format!(
                "https://public-api.birdeye.so/defi/token_overview?address={mint}"
            );
            let resp: BirdeyeOverviewResponse = ureq::get(&url)
                .header("X-API-KEY", api_key.as_str())
                .header("x-chain", "solana")
                .call()
                .map_err(Error::Http)?
                .into_body()
                .read_json()
                .map_err(|e| Error::Api { service: "api".into(), status: 0, message: e.to_string() })?;

            if resp.success {
                Ok::<Value, crate::error::Error>(serde_json::json!({
                    "mint": mint,
                    "name": resp.data.name,
                    "symbol": resp.data.symbol,
                    "price_usd": resp.data.price,
                    "market_cap": resp.data.market_cap,
                    "volume_24h": resp.data.volume_24h,
                    "liquidity": resp.data.liquidity,
                    "holder_count": resp.data.holder,
                    "price_change_24h_pct": resp.data.price_change_24h_pct,
                }))
            } else {
                Err(Error::Api {
                    service: "birdeye".into(),
                    status: 200,
                    message: "request returned success=false".into(),
                })
            }
            
        })
        .map_error(rig_cat::error::Error::from)
    }
}
