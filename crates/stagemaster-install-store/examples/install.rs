//! Software-only install/reopen/replay reference, with no transport or device output.
use sha2::{Digest, Sha256};
use stagemaster_install::{Identity, Installer, MAX_CHUNK_BYTES, Phase, SlotHealth};
use stagemaster_install_store::FileStore;
use stagemaster_package::{Archive, MAX_PACKAGE_BYTES, ReadAt};
use stagemaster_playback::Player;
use std::{
    cell::RefCell,
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::Path,
};

struct Source {
    file: RefCell<File>,
    bytes: usize,
}
impl Source {
    fn open(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let bytes = usize::try_from(file.metadata()?.len())?;
        if bytes > MAX_PACKAGE_BYTES {
            return Err("播放包超过参考容量".into());
        }
        Ok(Self {
            file: RefCell::new(file),
            bytes,
        })
    }
}
impl ReadAt for Source {
    fn len(&self) -> usize {
        self.bytes
    }
    fn read_exact(
        &self,
        offset: usize,
        target: &mut [u8],
    ) -> Result<(), stagemaster_package::Error> {
        let mut file = self.file.borrow_mut();
        file.seek(SeekFrom::Start(offset as u64))
            .and_then(|_| file.read_exact(target))
            .map_err(|_| stagemaster_package::Error::Read)
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if !(1..=2).contains(&args.len()) {
        return Err(
            "用法：install 专用软件验收目录 [待安装.smpkg]；省略文件时只检查已有安装".into(),
        );
    }
    let source = args
        .get(1)
        .map(|p| Source::open(Path::new(p)))
        .transpose()?;
    let identity = source
        .as_ref()
        .map(|r| Archive::open(r).map(|a| Identity::from_archive(&a)))
        .transpose()?;
    let (mut installer, report) = Installer::open(
        FileStore::open(Path::new(&args[0]))?,
        *uuid::Uuid::new_v4().as_bytes(),
    )?;
    println!("软件安装参考：未连接设备，不发送 DMX");
    for (index, slot) in report.slots.iter().enumerate() {
        match slot {
            SlotHealth::Empty => println!("槽 {index}：没有提交记录"),
            SlotHealth::InvalidRecord => println!("槽 {index}：提交记录损坏，已排除"),
            SlotHealth::InvalidPackage(error) => println!("槽 {index}：载荷无效，已排除：{error}"),
            SlotHealth::Ready(commit) => println!(
                "槽 {index}：有效代数 {}，{} 字节",
                commit.generation, commit.identity.bytes
            ),
        }
    }
    if let (Some(source), Some(identity)) = (source, identity) {
        let transaction = installer.transaction(1);
        let begin = installer.begin(transaction, identity)?;
        if begin.phase != Phase::Committed {
            let mut buffer = [0; MAX_CHUNK_BYTES];
            let mut offset = 0;
            while offset < source.len() {
                let count = buffer.len().min(source.len() - offset);
                source.read_exact(offset, &mut buffer[..count])?;
                installer.write(transaction, offset, &buffer[..count])?;
                offset += count;
            }
            installer.verify(transaction)?;
        }
        let committed = match installer.commit(transaction) {
            Ok(commit) => commit,
            Err(stagemaster_install::Error::CommitUncertain(error)) => {
                eprintln!("提交回执不确定，正在核对：{error}");
                installer.reconcile()?;
                installer.commit(transaction)?
            }
            Err(error) => return Err(error.into()),
        };
        println!(
            "已安装候选：代数 {}，槽 {}；未启动节目",
            committed.generation,
            committed.slot.index()
        );
    }
    if installer.head().is_none() {
        println!("没有有效的已安装候选");
        return Ok(());
    }
    let snapshot = installer.snapshot()?;
    println!(
        "只用已安装快照重放：{}，{} 个节目",
        snapshot.archive().source().project_name,
        snapshot.archive().entries().len()
    );
    for (index, entry) in snapshot.archive().entries().iter().enumerate() {
        let program = snapshot.load(index)?;
        let mut player = Player::new(program.plan, 0);
        player.execute(0, 0)?;
        let mut frames = 0;
        let mut digest = Sha256::new();
        let mut last = [0; 512];
        for time in (0..10_000).step_by(25) {
            player.advance(time)?;
            program.output.render(player.values(), &mut last)?;
            digest.update(last);
            frames += 1;
        }
        println!(
            "{}：{frames} 帧；末帧前四槽 {:?}；帧摘要 {:x}",
            entry.name,
            &last[..4],
            digest.finalize()
        );
    }
    Ok(())
}
