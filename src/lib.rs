//! # solagent-cat
//!
//! A Solana AI agent framework built on [`comp_cat_rs`] and [`rig_cat`].
//!
//! No async, no tokio.  All effects are `Io<Error, A>`, all
//! concurrency is `Fiber`, all streaming is `Stream`.
//!
//! ## Quick start
//!
//! ```rust,ignore
//! use std::sync::Arc;
//! use rig_cat::provider::openai::{OpenAiCompletion, ApiKey, ModelName};
//! use rig_cat::agent::AgentBuilder;
//! use solagent_cat::agent::SolAgent;
//! use solagent_cat::config::Config;
//! use solagent_cat::wallet::Wallet;
//! use solagent_cat::rpc::RpcClient;
//! use solagent_cat::types::{RpcUrl, SlippageBps, CommitmentLevel};
//! use solagent_cat::tools::all_tools;
//!
//! let wallet = Wallet::from_base58("your-base58-private-key")?;
//! let rpc = RpcClient::new(&RpcUrl::new("https://api.devnet.solana.com".into()));
//! let config = Config::new(SlippageBps::new(50), CommitmentLevel::Confirmed);
//! let sol_agent = Arc::new(SolAgent::new(wallet, rpc, config));
//!
//! let model = OpenAiCompletion::new(
//!     ApiKey::new(std::env::var("OPENAI_API_KEY")?),
//!     ModelName::new("gpt-4o".into()),
//! );
//!
//! let agent = AgentBuilder::new(model)
//!     .preamble("You are a Solana trading assistant.")
//!     .tools(all_tools(sol_agent))
//!     .build();
//!
//! let response = agent.prompt("What is my SOL balance?").run()?;
//! println!("{response}");
//! ```

pub mod error;
pub mod types;
pub mod config;
pub mod wallet;
pub mod rpc;
pub mod agent;
pub mod tools;
