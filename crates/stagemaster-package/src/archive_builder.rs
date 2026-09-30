//! Host-only incremental encoding. Archive owns the independent complete-file verifier.
use crate::archive::{HEADER, MAGIC, hash, hash_reader};
use crate::{
    Archive, COMPILER, Error, Id, Kind, MAX_CATALOG_BYTES, MAX_PACKAGE_BYTES, MAX_PROGRAMS,
    Program, SNAP_COMPILER, Source,
    codec::{encoder, text},
    encode_program, own, reserve,
};
use alloc::{string::String, vec::Vec};

struct Encoded {
    kind: Kind,
    id: Id,
    name: String,
    bytes: Vec<u8>,
    semantics: u16,
}
/// Host-side incremental builder. It retains encoded blocks, never all decoded plans.
pub struct Builder {
    source: Source,
    programs: Vec<Encoded>,
}
impl Builder {
    #[must_use]
    pub const fn new(source: Source) -> Self {
        Self {
            source,
            programs: Vec::new(),
        }
    }
    /// Add a compiled program. Ordering is canonicalized when finishing.
    /// # Errors
    /// Rejects duplicate identities, excessive counts and invalid program data.
    pub fn add(&mut self, kind: Kind, id: Id, name: &str, program: &Program) -> Result<(), Error> {
        if self.programs.len() >= MAX_PROGRAMS {
            return Err(Error::Limit("最多 64 个节目"));
        }
        if id == [0; 16] || self.programs.iter().any(|p| (p.kind, p.id) == (kind, id)) {
            return Err(Error::Invalid("节目标识为空或重复"));
        }
        text(name)?;
        let bytes = encode_program(program)?;
        self.programs
            .try_reserve(1)
            .map_err(|_| Error::Allocation)?;
        self.programs.push(Encoded {
            kind,
            id,
            name: own(name)?,
            bytes,
            semantics: if program.plan.snap_attributes().is_empty() {
                1
            } else {
                2
            },
        });
        Ok(())
    }
    /// Finish deterministic bytes, then independently verify the entire archive.
    /// # Errors
    /// Rejects invalid source metadata, empty packages, total size, catalogues or loader budgets.
    pub fn finish(mut self) -> Result<(Vec<u8>, Archive), Error> {
        if self.programs.is_empty() {
            return Err(Error::Invalid("请至少选择一个节目"));
        }
        self.programs.sort_unstable_by_key(|p| (p.kind, p.id));
        let semantics = self.programs.iter().map(|p| p.semantics).max().unwrap_or(1);
        let mut e = encoder(MAX_CATALOG_BYTES);
        e.array(6)?
            .str(if semantics == 1 {
                COMPILER
            } else {
                SNAP_COMPILER
            })?
            .bytes(&self.source.project_id)?
            .bytes(&self.source.revision_id)?
            .bytes(&self.source.snapshot_digest)?
            .str(text(&self.source.project_name)?)?
            .array(self.programs.len() as u64)?;
        let mut offset = 0_u32;
        for p in &self.programs {
            let length = u32::try_from(p.bytes.len()).map_err(|_| Error::Limit("节目大小"))?;
            e.array(6)?
                .u8(p.kind.code())?
                .bytes(&p.id)?
                .str(&p.name)?
                .u32(offset)?
                .u32(length)?
                .bytes(&hash(&p.bytes))?;
            offset += length;
        }
        let catalog = e.into_writer().bytes;
        let total = HEADER + catalog.len() + offset as usize;
        if total > MAX_PACKAGE_BYTES {
            return Err(Error::Limit("完整包 2 MiB"));
        }
        let mut bytes = reserve(total)?;
        bytes.resize(HEADER, 0);
        bytes[..8].copy_from_slice(MAGIC);
        bytes[8..16].copy_from_slice(&[1, 0, 64, 0, 1, 0, 1, 0]);
        bytes[12..14].copy_from_slice(&semantics.to_le_bytes());
        bytes[16..20].copy_from_slice(
            &u32::try_from(catalog.len())
                .map_err(|_| Error::Limit("目录大小"))?
                .to_le_bytes(),
        );
        bytes[20..24].copy_from_slice(
            &u32::try_from(total)
                .map_err(|_| Error::Limit("包大小"))?
                .to_le_bytes(),
        );
        bytes[24..26].copy_from_slice(
            &u16::try_from(self.programs.len())
                .map_err(|_| Error::Limit("节目数量"))?
                .to_le_bytes(),
        );
        bytes.extend_from_slice(&catalog);
        for p in self.programs {
            bytes.extend_from_slice(&p.bytes);
        }
        let h = bytes[..HEADER].try_into().map_err(|_| Error::Read)?;
        let digest = hash_reader(bytes.as_slice(), h)?;
        bytes[32..64].copy_from_slice(&digest);
        let archive = Archive::open(bytes.as_slice())?;
        Ok((bytes, archive))
    }
}
