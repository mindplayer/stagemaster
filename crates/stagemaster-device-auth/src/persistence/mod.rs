//! Optional persistence adapter. Caller owns the sole storage executor and maintenance policy.
mod format;
mod nor;
use crate::{Code, LocalIdentity, RECORD_BYTES, Vault};
use ekv::{Config, Database};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embedded_storage::nor_flash::NorFlash;
use nor::DatabaseFlash;
pub use nor::IoError;
use zeroize::Zeroizing;

pub const PAGE_BYTES: usize = 4096;
pub const DATABASE_PAGES: usize = 32;
pub const STORAGE_BYTES: usize = (DATABASE_PAGES + 1) * PAGE_BYTES;
const KEY: &[u8] = b"binding-vault-v1";

#[derive(Debug)]
pub enum Error<E> {
    Code(Code),
    Io(IoError<E>),
    Mount(ekv::MountError<IoError<E>>),
    Format(ekv::FormatError<IoError<E>>),
    Read(ekv::ReadError<IoError<E>>),
    Write(ekv::WriteError<IoError<E>>),
    Commit(ekv::CommitError<IoError<E>>),
}
impl<E> From<Code> for Error<E> {
    fn from(value: Code) -> Self {
        Self::Code(value)
    }
}

/// No implicit formatting, no fallback to cached authority after an uncertain write.
pub struct VaultStore<F: NorFlash> {
    database: Database<DatabaseFlash<F>, NoopRawMutex>,
    current: Option<Vault>,
}
impl<F: NorFlash> VaultStore<F> {
    /// # Errors
    /// Require a precisely bounded dedicated partition and the frozen flash geometry.
    pub fn new(flash: F, random_seed: u32) -> Result<Self, Code> {
        let flash = DatabaseFlash::new(flash)?;
        let mut config = Config::default();
        config.random_seed = random_seed;
        Ok(Self {
            database: Database::new(flash, config),
            current: None,
        })
    }
    /// Only a verified durable snapshot. This is not live connection authentication.
    #[must_use]
    pub const fn current(&self) -> Option<&Vault> {
        self.current.as_ref()
    }

    /// # Errors
    /// Reads never format storage. Any error clears previous in-memory availability.
    pub async fn recover(&mut self) -> Result<&Vault, Error<F::Error>> {
        self.current = None;
        {
            let mut flash = self.database.lock_flash().await;
            flash.faulted = false;
            format::check(&mut flash.inner).map_err(Error::Io)?;
        }
        self.database.mount().await.map_err(Error::Mount)?;
        let verified = self.read().await?;
        Ok(self.current.insert(verified))
    }

    /// Trusted local provisioning only, while physical output is quiescent.
    /// # Errors
    /// Refuses ANY existing bytes, including partial initialization and unknown formats.
    /// A dropped/failed future never produces a usable cached identity.
    pub async fn initialize(&mut self, local: LocalIdentity) -> Result<&Vault, Error<F::Error>> {
        self.current = None;
        {
            let mut flash = self.database.lock_flash().await;
            format::require_blank(&mut flash.inner).map_err(Error::Io)?;
            flash.faulted = false;
        }
        self.database.format().await.map_err(Error::Format)?;
        let initial = Vault::new(local);
        self.write(&initial).await?;
        if self.read().await? != initial {
            return Err(Code::Integrity.into());
        }
        {
            let mut flash = self.database.lock_flash().await;
            format::write(&mut flash.inner).map_err(Error::Io)?;
        }
        self.recover().await
    }

    /// Persist one proposed successor before granting it any business meaning.
    /// # Errors
    /// Reject stale generations/local identity replacement without touching storage.
    /// A mutation, cancellation, write/read error or uncertain commit clears availability
    /// until explicit recovery verifies the durable outcome.
    pub async fn commit(&mut self, proposal: &Vault) -> Result<&Vault, Error<F::Error>> {
        let current = self.current.as_ref().ok_or(Code::Unavailable)?;
        if current.generation().checked_add(1) != Some(proposal.generation())
            || current.local() != proposal.local()
        {
            return Err(Code::Conflict.into());
        }
        let previous = self.current.take().ok_or(Code::Unavailable)?;
        if self.read().await? != previous {
            return Err(Code::Conflict.into());
        }
        self.write(proposal).await?;
        let verified = self.read().await?;
        if &verified != proposal {
            return Err(Code::Integrity.into());
        }
        Ok(self.current.insert(verified))
    }
    async fn read(&self) -> Result<Vault, Error<F::Error>> {
        let transaction = self.database.read_transaction().await;
        let mut bytes = Zeroizing::new([0; RECORD_BYTES]);
        let len = transaction
            .read(KEY, &mut bytes[..])
            .await
            .map_err(Error::Read)?;
        drop(transaction);
        // EKV probes all possible metadata pages and can ignore a failed read.
        // An unreadable candidate might be newer: never expose authority from a
        // snapshot recovered by skipping a physical I/O error.
        if self.database.lock_flash().await.faulted {
            return Err(Code::Unavailable.into());
        }
        Vault::decode(&bytes[..len]).map_err(Error::Code)
    }
    async fn write(&self, vault: &Vault) -> Result<(), Error<F::Error>> {
        let bytes = vault.encode();
        let mut transaction = self.database.write_transaction().await;
        transaction
            .write(KEY, &bytes[..])
            .await
            .map_err(Error::Write)?;
        transaction.commit().await.map_err(Error::Commit)
    }
}
