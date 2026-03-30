//! Domain newtypes.
//!
//! Every domain primitive gets a newtype with a private field,
//! a constructor, and a getter.  No raw `String`, `u64`, or `f64`
//! crosses a function boundary when it carries domain meaning.

use crate::error::Error;

// ---------------------------------------------------------------------------
// Identity
// ---------------------------------------------------------------------------

/// A Solana public key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pubkey(solana_sdk::pubkey::Pubkey);

impl Pubkey {
    /// Wrap a `solana_sdk::pubkey::Pubkey`.
    #[must_use]
    pub fn new(inner: solana_sdk::pubkey::Pubkey) -> Self {
        Self(inner)
    }

    /// Access the inner `solana_sdk` pubkey.
    #[must_use]
    pub fn as_inner(&self) -> &solana_sdk::pubkey::Pubkey {
        &self.0
    }

    /// Base58 representation.
    #[must_use]
    pub fn to_base58(&self) -> String {
        self.0.to_string()
    }
}

impl core::fmt::Display for Pubkey {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl core::str::FromStr for Pubkey {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<solana_sdk::pubkey::Pubkey>()
            .map(Self)
            .map_err(|e| Error::InvalidInput {
                field: "pubkey".into(),
                reason: e.to_string(),
            })
    }
}

/// An SPL token mint address.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MintAddress(Pubkey);

impl MintAddress {
    /// Wrap a [`Pubkey`] as a mint address.
    #[must_use]
    pub fn new(pubkey: Pubkey) -> Self {
        Self(pubkey)
    }

    /// The underlying pubkey.
    #[must_use]
    pub fn pubkey(&self) -> &Pubkey {
        &self.0
    }

    /// Base58 representation.
    #[must_use]
    pub fn to_base58(&self) -> String {
        self.0.to_base58()
    }
}

impl core::fmt::Display for MintAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl core::str::FromStr for MintAddress {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<Pubkey>().map(Self)
    }
}

// ---------------------------------------------------------------------------
// Currency
// ---------------------------------------------------------------------------

/// SOL amount in lamports (1 SOL = `1_000_000_000` lamports).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Lamports(u64);

impl Lamports {
    /// Number of lamports per SOL.
    pub const PER_SOL: u64 = 1_000_000_000;

    /// Create from a raw lamport count.
    #[must_use]
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    /// The raw lamport count.
    #[must_use]
    pub fn value(self) -> u64 {
        self.0
    }
}

impl core::fmt::Display for Lamports {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Human-readable SOL amount.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Sol(f64);

impl Sol {
    /// Create from a floating-point SOL amount.
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(value)
    }

    /// The floating-point SOL amount.
    #[must_use]
    pub fn value(self) -> f64 {
        self.0
    }
}

impl core::fmt::Display for Sol {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_precision_loss)]
impl From<Sol> for Lamports {
    fn from(sol: Sol) -> Self {
        Self((sol.0 * Lamports::PER_SOL as f64) as u64)
    }
}

#[allow(clippy::cast_precision_loss)]
impl From<Lamports> for Sol {
    fn from(lamports: Lamports) -> Self {
        Self(lamports.0 as f64 / Lamports::PER_SOL as f64)
    }
}

/// Token amount in the smallest unit (before decimal adjustment).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenAmount(u64);

impl TokenAmount {
    /// Create from a raw token amount.
    #[must_use]
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    /// The raw value.
    #[must_use]
    pub fn value(self) -> u64 {
        self.0
    }
}

impl core::fmt::Display for TokenAmount {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Token decimal places.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TokenDecimals(u8);

impl TokenDecimals {
    /// Create from a decimal count.
    #[must_use]
    pub fn new(value: u8) -> Self {
        Self(value)
    }

    /// The decimal count.
    #[must_use]
    pub fn value(self) -> u8 {
        self.0
    }
}

// ---------------------------------------------------------------------------
// Transaction
// ---------------------------------------------------------------------------

/// A transaction signature.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Signature(solana_sdk::signature::Signature);

impl Signature {
    /// Wrap a `solana_sdk::signature::Signature`.
    #[must_use]
    pub fn new(inner: solana_sdk::signature::Signature) -> Self {
        Self(inner)
    }

    /// Access the inner signature.
    #[must_use]
    pub fn as_inner(&self) -> &solana_sdk::signature::Signature {
        &self.0
    }

    /// Base58 representation.
    #[must_use]
    pub fn to_base58(&self) -> String {
        self.0.to_string()
    }
}

impl core::fmt::Display for Signature {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ---------------------------------------------------------------------------
// Network
// ---------------------------------------------------------------------------

/// A Solana RPC endpoint URL.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RpcUrl(String);

impl RpcUrl {
    /// Create from a URL string.
    #[must_use]
    pub fn new(url: String) -> Self {
        Self(url)
    }

    /// The URL as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl core::fmt::Display for RpcUrl {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Solana transaction commitment level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommitmentLevel {
    /// Transaction has been processed by the leader.
    Processed,
    /// Transaction has been confirmed by the cluster.
    Confirmed,
    /// Transaction has been finalized.
    Finalized,
}

impl From<CommitmentLevel> for solana_sdk::commitment_config::CommitmentConfig {
    fn from(level: CommitmentLevel) -> Self {
        match level {
            CommitmentLevel::Processed => Self::processed(),
            CommitmentLevel::Confirmed => Self::confirmed(),
            CommitmentLevel::Finalized => Self::finalized(),
        }
    }
}

// ---------------------------------------------------------------------------
// Trading
// ---------------------------------------------------------------------------

/// Slippage tolerance in basis points (1 bp = 0.01%).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SlippageBps(u16);

impl SlippageBps {
    /// Create from basis points.
    #[must_use]
    pub fn new(bps: u16) -> Self {
        Self(bps)
    }

    /// The basis point value.
    #[must_use]
    pub fn value(self) -> u16 {
        self.0
    }
}

/// A USD price.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct UsdPrice(f64);

impl UsdPrice {
    /// Create from a floating-point price.
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(value)
    }

    /// The floating-point price.
    #[must_use]
    pub fn value(self) -> f64 {
        self.0
    }
}

impl core::fmt::Display for UsdPrice {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ---------------------------------------------------------------------------
// Token metadata
// ---------------------------------------------------------------------------

/// A token symbol (e.g. "SOL", "USDC").
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TokenSymbol(String);

impl TokenSymbol {
    /// Create from a string.
    #[must_use]
    pub fn new(symbol: String) -> Self {
        Self(symbol)
    }

    /// The symbol as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl core::fmt::Display for TokenSymbol {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A token name (e.g. "Solana", "USD Coin").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenName(String);

impl TokenName {
    /// Create from a string.
    #[must_use]
    pub fn new(name: String) -> Self {
        Self(name)
    }

    /// The name as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl core::fmt::Display for TokenName {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A metadata URI for an NFT or token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadataUri(String);

impl MetadataUri {
    /// Create from a URI string.
    #[must_use]
    pub fn new(uri: String) -> Self {
        Self(uri)
    }

    /// The URI as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl core::fmt::Display for MetadataUri {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ---------------------------------------------------------------------------
// API keys
// ---------------------------------------------------------------------------

/// Birdeye API key.
#[derive(Clone)]
pub struct BirdeyeApiKey(String);

impl BirdeyeApiKey {
    /// Create from a string.
    #[must_use]
    pub fn new(key: String) -> Self {
        Self(key)
    }

    /// The key as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl core::fmt::Debug for BirdeyeApiKey {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("BirdeyeApiKey").field(&"***").finish()
    }
}

/// Helius API key.
#[derive(Clone)]
pub struct HeliusApiKey(String);

impl HeliusApiKey {
    /// Create from a string.
    #[must_use]
    pub fn new(key: String) -> Self {
        Self(key)
    }

    /// The key as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl core::fmt::Debug for HeliusApiKey {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("HeliusApiKey").field(&"***").finish()
    }
}

/// `GoPlus` API key.
#[derive(Clone)]
pub struct GoPlusApiKey(String);

impl GoPlusApiKey {
    /// Create from a string.
    #[must_use]
    pub fn new(key: String) -> Self {
        Self(key)
    }

    /// The key as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl core::fmt::Debug for GoPlusApiKey {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("GoPlusApiKey").field(&"***").finish()
    }
}

// ---------------------------------------------------------------------------
// Price feeds
// ---------------------------------------------------------------------------

/// A Pyth price feed ID.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PythFeedId(String);

impl PythFeedId {
    /// Create from a string.
    #[must_use]
    pub fn new(id: String) -> Self {
        Self(id)
    }

    /// The feed ID as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl core::fmt::Display for PythFeedId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}
