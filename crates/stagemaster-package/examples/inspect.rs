//! File-backed reference decoder. Does not depend on project JSON or desktop compilation.
use sha2::{Digest, Sha256};
use stagemaster_package::{Archive, Error, ReadAt};
use stagemaster_playback::Player;
use std::{
    cell::RefCell,
    fs::File,
    io::{Read, Seek, SeekFrom},
};
struct FileSource {
    file: RefCell<File>,
    length: usize,
}
impl ReadAt for FileSource {
    fn len(&self) -> usize {
        self.length
    }
    fn read_exact(&self, offset: usize, target: &mut [u8]) -> Result<(), Error> {
        let mut file = self.file.borrow_mut();
        file.seek(SeekFrom::Start(offset as u64))
            .map_err(|_| Error::Read)?;
        file.read_exact(target).map_err(|_| Error::Read)
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args_os()
        .nth(1)
        .ok_or("用法：inspect 文件.smpkg")?;
    let file = File::open(path)?;
    let length = usize::try_from(file.metadata()?.len())?;
    let reader = FileSource {
        file: RefCell::new(file),
        length,
    };
    let archive = Archive::open(&reader)?;
    println!(
        "工程：{}；{} 个节目；{} 字节；线路 {}",
        archive.source().project_name,
        archive.entries().len(),
        archive.total_bytes(),
        archive.universe()
    );
    for (index, entry) in archive.entries().iter().enumerate() {
        let program = archive.load(&reader, index)?;
        let mut player = Player::new(program.plan, 0);
        player.execute(0, 0)?;
        let mut digest = Sha256::new();
        let mut slots = [0; 512];
        for time in (0..10_000).step_by(25) {
            player.advance(time)?;
            program.output.render(player.values(), &mut slots)?;
            digest.update(slots);
        }
        println!(
            "{}：{} 步／{} 属性；块 {} 字节；参考装载峰值 {} 字节；400 帧摘要 {:x}",
            entry.name,
            entry.usage.steps,
            entry.usage.attributes,
            entry.usage.encoded_bytes,
            entry.usage.loader_peak_bytes,
            digest.finalize()
        );
    }
    Ok(())
}
