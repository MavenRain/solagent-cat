//! Solana wallet: keypair management and signing.
//!
//! [`Wallet`] wraps a `solana_sdk::signer::keypair::Keypair`
//! and provides domain-safe access to the public key and signer.
//! The private key is never exposed directly.

use solana_sdk::signer::Signer;

use crate::error::Error;
use crate::types::Pubkey;

/// A Solana wallet backed by an Ed25519 keypair.
///
/// Provides signing capability without exposing the private key.
pub struct Wallet {
    keypair: solana_sdk::signer::keypair::Keypair,
}

impl Wallet {
    /// Create a wallet from a `solana_sdk` keypair.
    #[must_use]
    pub fn new(keypair: solana_sdk::signer::keypair::Keypair) -> Self {
        Self { keypair }
    }

    /// Create a wallet from raw keypair bytes (64 bytes).
    ///
    /// # Errors
    ///
    /// Returns `Error::Signing` if the bytes are not a valid keypair.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        solana_sdk::signer::keypair::Keypair::try_from(bytes)
            .map(Self::new)
            .map_err(|e| Error::Signing {
                message: e.to_string(),
            })
    }

    /// Create a wallet from a base58-encoded private key.
    ///
    /// # Errors
    ///
    /// Returns `Error::Base58` or `Error::Signing` on invalid input.
    pub fn from_base58(s: &str) -> Result<Self, Error> {
        let bytes = bs58::decode(s).into_vec()?;
        Self::from_bytes(&bytes)
    }

    /// The wallet's public key.
    #[must_use]
    pub fn pubkey(&self) -> Pubkey {
        Pubkey::new(self.keypair.pubkey())
    }

    /// Access the inner signer for transaction signing.
    ///
    /// This returns a reference to the keypair as a `Signer` trait impl.
    /// The private key bytes are not directly accessible.
    #[must_use]
    pub fn signer(&self) -> &solana_sdk::signer::keypair::Keypair {
        &self.keypair
    }
}

impl core::fmt::Debug for Wallet {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Wallet")
            .field("pubkey", &self.pubkey())
            .finish_non_exhaustive()
    }
}
