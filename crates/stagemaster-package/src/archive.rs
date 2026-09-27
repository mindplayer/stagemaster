use crate::{
    COMPILER, Error, Id, Kind, MAX_CATALOG_BYTES, MAX_LOADER_BYTES, MAX_PACKAGE_BYTES,
    MAX_PROGRAM_BYTES, MAX_PROGRAMS, Program, Usage,
    codec::{array, digest, encoder, end, id, read_text, text},
    decode_program, encode_program, own,
    program::scan,
    reserve,
};
use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use minicbor::Decoder;
use sha2::{Digest, Sha256};
const HEADER: usize = 64;
const MAGIC: &[u8; 8] = b"STMPLAY\0";

/// A stable random-access snapshot. Adapters own files, flash partitions and error translation.
pub trait ReadAt {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// # Errors
    /// Returns an error on any short or failed read; never silently fills missing bytes.
    fn read_exact(&self, offset: usize, target: &mut [u8]) -> Result<(), Error>;
}
impl<T: ReadAt + ?Sized> ReadAt for &T {
    fn len(&self) -> usize {
        T::len(self)
    }
    fn read_exact(&self, offset: usize, target: &mut [u8]) -> Result<(), Error> {
        T::read_exact(self, offset, target)
    }
}

impl ReadAt for [u8] {
    fn len(&self) -> usize {
        <[u8]>::len(self)
    }
    fn read_exact(&self, offset: usize, target: &mut [u8]) -> Result<(), Error> {
        let end = offset.checked_add(target.len()).ok_or(Error::Read)?;
        target.copy_from_slice(self.get(offset..end).ok_or(Error::Read)?);
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Source {
    pub project_id: Id,
    pub revision_id: Id,
    pub snapshot_digest: [u8; 32],
    pub project_name: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub kind: Kind,
    pub id: Id,
    pub name: String,
    pub usage: Usage,
    offset: usize,
    length: usize,
    digest: [u8; 32],
}
/// All blocks have passed structural, semantic, integrity and budget validation.
/// Source bytes are not retained. A single loaded plan has its own lifetime.
#[derive(Debug)]
pub struct Archive {
    source: Source,
    entries: Vec<Entry>,
    total_bytes: usize,
    data_offset: usize,
    catalog_resident_bytes: usize,
    digest: [u8; 32],
    universe: u16,
}
impl Archive {
    /// Verify the complete package, then scan each program one at a time before accepting it.
    /// # Errors
    /// Rejects unsupported versions, corruption, malformed data, overlaps and resource excesses.
    pub fn open<R: ReadAt + ?Sized>(reader: &R) -> Result<Self, Error> {
        let (header, catalog_length, program_count) = read_header(reader)?;
        let digest: [u8; 32] = header[32..64].try_into().map_err(|_| Error::Read)?;
        if hash_reader(reader, &header)? != digest {
            return Err(Error::Integrity);
        }
        let bytes = read_block(reader, HEADER, catalog_length)?;
        let mut archive = parse_catalog(&bytes, reader.len(), program_count, digest)?;
        drop(bytes);
        let mut universe = None;
        for index in 0..archive.entries.len() {
            let entry = &archive.entries[index];
            let bytes = archive.read_program(reader, entry)?;
            let checked =
                scan(&bytes, archive.catalog_resident_bytes).map_err(|e| Error::AtProgram {
                    index,
                    message: e.to_string(),
                })?;
            if universe.is_some_and(|value| value != checked.universe) {
                return Err(Error::Invalid("各节目须使用同一输出线路"));
            }
            universe = Some(checked.universe);
            if entry.kind == Kind::Scene && !checked.held_scene {
                return Err(Error::Invalid("单场景节目必须为静态等待的一步"));
            }
            archive.entries[index].usage = checked.usage;
        }
        archive.universe = universe.ok_or(Error::Invalid("播放包没有节目"))?;
        Ok(archive)
    }
    #[must_use]
    pub const fn source(&self) -> &Source {
        &self.source
    }
    #[must_use]
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }
    #[must_use]
    pub const fn total_bytes(&self) -> usize {
        self.total_bytes
    }
    #[must_use]
    pub const fn catalog_resident_bytes(&self) -> usize {
        self.catalog_resident_bytes
    }
    #[must_use]
    pub const fn digest(&self) -> &[u8; 32] {
        &self.digest
    }
    #[must_use]
    pub const fn universe(&self) -> u16 {
        self.universe
    }
    /// Recheck the block digest and decode only the selected program.
    /// # Errors
    /// Rejects invalid indices, changed storage, malformed data or failed allocations.
    pub fn load<R: ReadAt + ?Sized>(&self, reader: &R, index: usize) -> Result<Program, Error> {
        let entry = self
            .entries
            .get(index)
            .ok_or(Error::Invalid("节目索引越界"))?;
        let bytes = self.read_program(reader, entry)?;
        let (program, _) = decode_program(&bytes, self.catalog_resident_bytes)?;
        Ok(program)
    }
    fn read_program<R: ReadAt + ?Sized>(
        &self,
        reader: &R,
        entry: &Entry,
    ) -> Result<Vec<u8>, Error> {
        if reader.len() != self.total_bytes {
            return Err(Error::Integrity);
        }
        let bytes = read_block(reader, self.data_offset + entry.offset, entry.length)?;
        if hash(&bytes) != entry.digest {
            return Err(Error::Integrity);
        }
        Ok(bytes)
    }
}
fn read_header<R: ReadAt + ?Sized>(reader: &R) -> Result<([u8; HEADER], usize, usize), Error> {
    if !(HEADER..=MAX_PACKAGE_BYTES).contains(&reader.len()) {
        return Err(Error::Limit("包大小须在 64 字节至 2 MiB 内"));
    }
    let mut h = [0; HEADER];
    reader.read_exact(0, &mut h)?;
    if &h[..8] != MAGIC {
        return Err(Error::Invalid("文件标识不符"));
    }
    if h[8..16] != [1, 0, 64, 0, 1, 0, 1, 0] {
        return Err(Error::Version);
    }
    if h[26..32] != [0; 6] {
        return Err(Error::Invalid("保留字段必须为零"));
    }
    let catalog = usize::try_from(u32::from_le_bytes(
        h[16..20].try_into().map_err(|_| Error::Read)?,
    ))
    .map_err(|_| Error::Limit("目录长度"))?;
    let total = usize::try_from(u32::from_le_bytes(
        h[20..24].try_into().map_err(|_| Error::Read)?,
    ))
    .map_err(|_| Error::Limit("包长度"))?;
    let count = usize::from(u16::from_le_bytes([h[24], h[25]]));
    if !(1..=MAX_CATALOG_BYTES).contains(&catalog) || !(1..=MAX_PROGRAMS).contains(&count) {
        return Err(Error::Limit("目录大小或节目数量"));
    }
    if total != reader.len() || HEADER + catalog >= total {
        return Err(Error::Invalid("文件长度与目录不符"));
    }
    Ok((h, catalog, count))
}
fn hash_reader<R: ReadAt + ?Sized>(reader: &R, h: &[u8; HEADER]) -> Result<[u8; 32], Error> {
    let mut hasher = Sha256::new();
    hasher.update(&h[..32]);
    let mut buffer = [0; 1024];
    let mut offset = HEADER;
    while offset < reader.len() {
        let size = buffer.len().min(reader.len() - offset);
        reader.read_exact(offset, &mut buffer[..size])?;
        hasher.update(&buffer[..size]);
        offset += size;
    }
    Ok(hasher.finalize().into())
}
fn read_block<R: ReadAt + ?Sized>(
    reader: &R,
    offset: usize,
    length: usize,
) -> Result<Vec<u8>, Error> {
    let mut bytes = reserve(length)?;
    bytes.resize(length, 0);
    reader.read_exact(offset, &mut bytes)?;
    Ok(bytes)
}
fn parse_catalog(
    bytes: &[u8],
    total: usize,
    expected: usize,
    package_digest: [u8; 32],
) -> Result<Archive, Error> {
    let mut d = Decoder::new(bytes);
    array(&mut d, 6)?;
    if read_text(&mut d)? != COMPILER {
        return Err(Error::Version);
    }
    let project_id = id(&mut d)?;
    let revision_id = id(&mut d)?;
    let snapshot_digest = digest(&mut d)?;
    let project_name = read_text(&mut d)?;
    array(&mut d, expected)?;
    let mut entries = reserve(expected)?;
    let mut resident = 512 + project_name.len() + expected * 224;
    let mut offset = 0;
    let mut previous = None;
    for _ in 0..expected {
        array(&mut d, 6)?;
        let kind = Kind::read(d.u8()?)?;
        let id = id(&mut d)?;
        let key = (kind, id);
        if previous.is_some_and(|p| p >= key) {
            return Err(Error::Invalid("节目须按类型和标识排序且不能重复"));
        }
        previous = Some(key);
        let name = read_text(&mut d)?;
        resident += name.len() + 32;
        let start = usize::try_from(d.u32()?).map_err(|_| Error::Limit("节目偏移"))?;
        let length = usize::try_from(d.u32()?).map_err(|_| Error::Limit("节目大小"))?;
        if !(1..=MAX_PROGRAM_BYTES).contains(&length) {
            return Err(Error::Limit("单节目块 32 KiB"));
        }
        if start != offset {
            return Err(Error::Invalid("节目块偏移不连续或重叠"));
        }
        offset += length;
        let digest = digest(&mut d)?;
        if resident + bytes.len() + 4096 > MAX_LOADER_BYTES {
            return Err(Error::Limit("目录装载内存"));
        }
        entries.push(Entry {
            kind,
            id,
            name: own(name)?,
            usage: Usage::default(),
            offset: start,
            length,
            digest,
        });
    }
    end(&d, bytes)?;
    let data_offset = HEADER + bytes.len();
    if data_offset + offset != total {
        return Err(Error::Invalid("节目长度总和不符"));
    }
    Ok(Archive {
        source: Source {
            project_id,
            revision_id,
            snapshot_digest,
            project_name: own(project_name)?,
        },
        entries,
        total_bytes: total,
        data_offset,
        catalog_resident_bytes: resident,
        digest: package_digest,
        universe: 0,
    })
}
fn hash(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}
struct Encoded {
    kind: Kind,
    id: Id,
    name: String,
    bytes: Vec<u8>,
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
        let mut e = encoder(MAX_CATALOG_BYTES);
        e.array(6)?
            .str(COMPILER)?
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
