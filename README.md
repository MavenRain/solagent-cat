# solagent-cat

A Solana AI agent framework built on [comp-cat-rs](https://github.com/MavenRain/comp-cat-rs) and [rig-cat](https://github.com/MavenRain/rig-cat).  No async, no tokio.  All effects are `Io<Error, A>`, all concurrency is `Fiber`, all streaming is `Stream`.

Reimplements [solagent](https://github.com/zTgx/solagent) on a categorical effect system with typed errors, newtypes, immutability, and static dispatch.

## Installation

```toml
[dependencies]
solagent-cat = "0.1"
```

## Quick start

```rust
use std::sync::Arc;
use rig_cat::provider::openai::{OpenAiCompletion, ApiKey, ModelName};
use rig_cat::agent::AgentBuilder;
use solagent_cat::agent::SolAgent;
use solagent_cat::config::Config;
use solagent_cat::wallet::Wallet;
use solagent_cat::rpc::RpcClient;
use solagent_cat::types::{RpcUrl, SlippageBps, CommitmentLevel};
use solagent_cat::tools::all_tools;

let wallet = Wallet::from_base58("your-base58-private-key")?;
let rpc = RpcClient::new(&RpcUrl::new("https://api.devnet.solana.com".into()));
let config = Config::new(SlippageBps::new(50), CommitmentLevel::Confirmed);
let sol_agent = Arc::new(SolAgent::new(wallet, rpc, config));

let model = OpenAiCompletion::new(
    ApiKey::new("sk-...".into()),
    ModelName::new("gpt-4o".into()),
);

let agent = AgentBuilder::new(model)
    .preamble("You are a Solana trading assistant.")
    .tools(all_tools(sol_agent))
    .build();

// Nothing runs until .run()
let response = agent.prompt("What is my SOL balance?").run();
```

## Architecture

```
error           Hand-rolled Error enum, bridge to rig-cat errors
types           Newtypes: Pubkey, Lamports, Sol, MintAddress, Signature, API keys, etc.
config          Config with builder-style with_* methods
wallet          Wallet wrapping solana_sdk::Keypair, no key exposure
rpc             RpcClient wrapping sync solana-client in Io::suspend
agent           SolAgent (wallet + rpc + config), shared via Arc
tools/          15 rig-cat Tool implementations + SolanaTools enum
```

Every function that touches the network returns `Io<Error, A>`.  Composition happens via `map`, `flat_map`, `zip`.  Side effects only happen when you call `.run()`.

## Tools

15 tools are available, organized by domain:

### Token operations

| Tool | Description |
|------|-------------|
| `get_balance` | Get SOL or SPL token balance |
| `get_token_data` | Get token mint metadata (supply, decimals, authorities) |
| `transfer_tokens` | Transfer SOL or SPL tokens |
| `deploy_token` | Deploy a new SPL token mint |

### NFT operations

| Tool | Description |
|------|-------------|
| `mint_nft` | Mint an NFT (0-decimal, supply-1 token) |
| `deploy_collection` | Deploy an NFT collection mint |

### Trading (Jupiter)

| Tool | Description |
|------|-------------|
| `jupiter_swap` | Swap tokens via the Jupiter DEX aggregator |
| `jupiter_price` | Get a token price from Jupiter Price API |
| `stake_with_jup` | Stake SOL via Jupiter liquid staking (jupSOL) |

### Price feeds

| Tool | Description |
|------|-------------|
| `pyth_price` | Get a price from the Pyth oracle network |
| `birdeye_price` | Get a token price from Birdeye |
| `birdeye_token_overview` | Get full token analytics from Birdeye |

### Security

| Tool | Description |
|------|-------------|
| `goplus_token_security` | Token security analysis from `GoPlus` |
| `rugcheck_token` | Token safety and risk score from Rugcheck |

### DEX data

| Tool | Description |
|------|-------------|
| `dexscreener_pairs` | Look up DEX trading pairs via `DexScreener` |

## Tool dispatch

All tools are variants of the `SolanaTools` enum, which implements `rig_cat::tool::Tool` via static dispatch (no `dyn Trait`).  Use `all_tools()` to wire every tool to a shared `SolAgent`:

```rust
use std::sync::Arc;
use solagent_cat::tools::{all_tools, SolanaTools};
use rig_cat::tool::Toolbox;

let toolbox: Toolbox<SolanaTools> = all_tools(Arc::clone(&sol_agent));
let result = toolbox.invoke("get_balance", serde_json::json!({})).run();
```

For a subset of tools, construct the toolbox manually:

```rust
use solagent_cat::tools::{SolanaTools, token, price};

let toolbox = Toolbox::new()
    .with_tool(SolanaTools::GetBalance(token::GetBalance::new(Arc::clone(&sol_agent))))
    .with_tool(SolanaTools::BirdeyePrice(price::BirdeyePrice::new(Arc::clone(&sol_agent))));
```

## Configuration

API keys for external services are set via `Config`:

```rust
use solagent_cat::config::Config;
use solagent_cat::types::{SlippageBps, CommitmentLevel, BirdeyeApiKey, GoPlusApiKey};

let config = Config::new(SlippageBps::new(50), CommitmentLevel::Confirmed)
    .with_birdeye_api_key(BirdeyeApiKey::new("your-key".into()))
    .with_goplus_api_key(GoPlusApiKey::new("your-key".into()));
```

Tools that require a missing key return `Error::Config` at call time.

## Error handling

A single `Error` enum covers all failure modes.  The bridge `From<Error> for rig_cat::error::Error` converts at the `Tool::call` boundary via `Io::map_error`.  No `thiserror`, no `anyhow`.

```rust
use solagent_cat::error::Error;

// All these convert via From:
// ureq::Error       -> Error::Http
// serde_json::Error -> Error::Json
// std::io::Error    -> Error::Io
// bs58::decode::Error -> Error::Base58

// Domain errors:
// Error::Rpc { code, message }
// Error::Transaction { signature, message }
// Error::Api { service, status, message }
// Error::Config { field }
// Error::InvalidInput { field, reason }
```

## Why no async?

Solana RPC calls and external API requests are high-latency (100ms-10s) and low-concurrency.  Thread-per-request via `Fiber` is perfectly adequate.  The benefit: no tokio, no `Pin<Box<dyn Future>>`, no colored functions.  Everything composes with `flat_map`.

## The categorical foundation

This crate is built on the thesis proved in [comp-cat-theory](https://github.com/MavenRain/comp-cat-theory) (Lean 4, zero `sorry`s):

- `Io` is a monad, which is a pair of Kan extensions
- `Stream` is a colimit, which is a left Kan extension
- `Fiber::fork` is a coproduct, `Fiber::join` is a limit
- Every tool call is an `Io::suspend` wrapping a side effect, composed via monadic sequencing

## License

MIT OR Apache-2.0
