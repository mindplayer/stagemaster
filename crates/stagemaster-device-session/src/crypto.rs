//! Delegate cryptography to Snow; inject platform entropy without an OS dependency.
use alloc::boxed::Box;
use snow::{
    Error,
    params::{CipherChoice, DHChoice, HashChoice},
    resolvers::{CryptoResolver, DefaultResolver},
    types::{Cipher, Dh, Hash, Random},
};
use subtle::ConstantTimeEq;

/// Fill every byte from a cryptographic source or fail. Never use a clock or MAC.
pub type Entropy = fn(&mut [u8]) -> Result<(), crate::Error>;
pub(crate) struct Resolver(pub Entropy);
struct Source(Entropy);
impl Random for Source {
    fn try_fill_bytes(&mut self, out: &mut [u8]) -> Result<(), Error> {
        (self.0)(out).map_err(|_| Error::Rng)
    }
}
impl CryptoResolver for Resolver {
    fn resolve_rng(&self) -> Option<Box<dyn Random>> {
        Some(Box::new(Source(self.0)))
    }
    fn resolve_dh(&self, choice: &DHChoice) -> Option<Box<dyn Dh>> {
        DefaultResolver
            .resolve_dh(choice)
            .map(|inner| Box::new(Contributory(inner)) as Box<dyn Dh>)
    }
    fn resolve_hash(&self, choice: &HashChoice) -> Option<Box<dyn Hash>> {
        DefaultResolver.resolve_hash(choice)
    }
    fn resolve_cipher(&self, choice: &CipherChoice) -> Option<Box<dyn Cipher>> {
        DefaultResolver.resolve_cipher(choice)
    }
}

/// A low-order public input must not establish a known all-zero shared secret.
struct Contributory(Box<dyn Dh>);
impl Dh for Contributory {
    fn name(&self) -> &'static str {
        self.0.name()
    }
    fn pub_len(&self) -> usize {
        self.0.pub_len()
    }
    fn priv_len(&self) -> usize {
        self.0.priv_len()
    }
    fn set(&mut self, key: &[u8]) {
        self.0.set(key);
    }
    fn generate(&mut self, rng: &mut dyn Random) -> Result<(), Error> {
        self.0.generate(rng)
    }
    fn pubkey(&self) -> &[u8] {
        self.0.pubkey()
    }
    fn privkey(&self) -> &[u8] {
        self.0.privkey()
    }
    fn dh(&self, key: &[u8], out: &mut [u8]) -> Result<(), Error> {
        self.0.dh(key, out)?;
        if out[..32].ct_eq(&[0; 32]).into() {
            Err(Error::Dh)
        } else {
            Ok(())
        }
    }
}

#[allow(clippy::needless_pass_by_value)] // Direct Result::map_err adapter.
pub(crate) fn error(error: Error) -> crate::Error {
    if error == Error::Rng {
        crate::Error::Entropy
    } else {
        crate::Error::Crypto
    }
}
