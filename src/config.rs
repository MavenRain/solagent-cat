//! Agent configuration.
//!
//! [`Config`] holds optional API keys and default settings.
//! Built via chained `with_*` methods.

use crate::types::{BirdeyeApiKey, CommitmentLevel, GoPlusApiKey, HeliusApiKey, SlippageBps};

/// Configuration for the Solana agent.
///
/// All API keys are optional.  Tools that require a missing key
/// will return [`Error::Config`](crate::error::Error::Config) at call time.
#[derive(Debug)]
pub struct Config {
    birdeye_api_key: Option<BirdeyeApiKey>,
    helius_api_key: Option<HeliusApiKey>,
    goplus_api_key: Option<GoPlusApiKey>,
    default_slippage: SlippageBps,
    commitment: CommitmentLevel,
}

impl Config {
    /// Create a new config with required defaults.
    #[must_use]
    pub fn new(default_slippage: SlippageBps, commitment: CommitmentLevel) -> Self {
        Self {
            birdeye_api_key: None,
            helius_api_key: None,
            goplus_api_key: None,
            default_slippage,
            commitment,
        }
    }

    /// Set the Birdeye API key.
    #[must_use]
    pub fn with_birdeye_api_key(self, key: BirdeyeApiKey) -> Self {
        Self {
            birdeye_api_key: Some(key),
            ..self
        }
    }

    /// Set the Helius API key.
    #[must_use]
    pub fn with_helius_api_key(self, key: HeliusApiKey) -> Self {
        Self {
            helius_api_key: Some(key),
            ..self
        }
    }

    /// Set the `GoPlus` API key.
    #[must_use]
    pub fn with_goplus_api_key(self, key: GoPlusApiKey) -> Self {
        Self {
            goplus_api_key: Some(key),
            ..self
        }
    }

    /// The Birdeye API key, if set.
    #[must_use]
    pub fn birdeye_api_key(&self) -> Option<&BirdeyeApiKey> {
        self.birdeye_api_key.as_ref()
    }

    /// The Helius API key, if set.
    #[must_use]
    pub fn helius_api_key(&self) -> Option<&HeliusApiKey> {
        self.helius_api_key.as_ref()
    }

    /// The `GoPlus` API key, if set.
    #[must_use]
    pub fn goplus_api_key(&self) -> Option<&GoPlusApiKey> {
        self.goplus_api_key.as_ref()
    }

    /// The default slippage tolerance.
    #[must_use]
    pub fn default_slippage(&self) -> SlippageBps {
        self.default_slippage
    }

    /// The transaction commitment level.
    #[must_use]
    pub fn commitment(&self) -> CommitmentLevel {
        self.commitment
    }

    /// Require the Birdeye API key, returning
    /// [`Error::Config`](crate::error::Error::Config) if missing.
    ///
    /// # Errors
    ///
    /// Returns `Error::Config` when the key has not been set.
    pub fn require_birdeye_api_key(&self) -> Result<&BirdeyeApiKey, crate::error::Error> {
        self.birdeye_api_key
            .as_ref()
            .ok_or_else(|| crate::error::Error::Config {
                field: "birdeye_api_key".into(),
            })
    }

    /// Require the Helius API key, returning
    /// [`Error::Config`](crate::error::Error::Config) if missing.
    ///
    /// # Errors
    ///
    /// Returns `Error::Config` when the key has not been set.
    pub fn require_helius_api_key(&self) -> Result<&HeliusApiKey, crate::error::Error> {
        self.helius_api_key
            .as_ref()
            .ok_or_else(|| crate::error::Error::Config {
                field: "helius_api_key".into(),
            })
    }

    /// Require the `GoPlus` API key, returning
    /// [`Error::Config`](crate::error::Error::Config) if missing.
    ///
    /// # Errors
    ///
    /// Returns `Error::Config` when the key has not been set.
    pub fn require_goplus_api_key(&self) -> Result<&GoPlusApiKey, crate::error::Error> {
        self.goplus_api_key
            .as_ref()
            .ok_or_else(|| crate::error::Error::Config {
                field: "goplus_api_key".into(),
            })
    }
}
