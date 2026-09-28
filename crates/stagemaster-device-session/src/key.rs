use crate::{Entropy, Error};
use snow::{
    params::DHChoice,
    resolvers::{CryptoResolver, DefaultResolver},
};
use zeroize::Zeroize;

/// Secret owned by the application credential store. Never Debug/Serialize/Clone.
pub struct SecretKey([u8; 32]);
impl SecretKey {
    /// Import provisioned key material; this does not establish its trust or ownership.
    /// # Errors
    /// Reject an empty key. Callers must supply securely generated, unique material.
    pub fn import(mut bytes: [u8; 32]) -> Result<Self, Error> {
        let result = if bytes == [0; 32] {
            Err(Error::Invalid)
        } else {
            Ok(Self(bytes))
        };
        bytes.zeroize();
        result
    }
    /// Generate key material using the platform's cryptographic random source.
    /// # Errors
    /// Propagate entropy failure; no fallback or deterministic key is used.
    pub fn generate(entropy: Entropy) -> Result<Self, Error> {
        let mut bytes = [0; 32];
        let result = entropy(&mut bytes).and_then(|()| Self::import(bytes));
        bytes.zeroize();
        result
    }
    #[must_use]
    /// # Panics
    /// Only if the fixed compiled Curve25519 provider violates its key-size contract.
    pub fn public(&self) -> [u8; 32] {
        let mut dh = DefaultResolver
            .resolve_dh(&DHChoice::Curve25519)
            .expect("fixed compiled suite");
        dh.set(&self.0);
        dh.pubkey().try_into().expect("fixed Curve25519 key size")
    }
    pub(crate) fn bytes(&self) -> &[u8; 32] {
        &self.0
    }
}
impl Drop for SecretKey {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}
