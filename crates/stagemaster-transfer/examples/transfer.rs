//! Local software reference only. A bounded source snapshot crosses the actual encoded protocol.
use sha2::{Digest, Sha256};
use stagemaster_install::Installer;
use stagemaster_install_store::FileStore;
use stagemaster_package::{Archive, MAX_PACKAGE_BYTES};
use stagemaster_playback::Player;
use stagemaster_transfer::{Assembler, AuthorizedLink, Frame, Outcome, Service, Upload};
use std::{fs::File, io::Read, path::Path};

#[derive(Default)]
struct Counts {
    requests: usize,
    bytes: usize,
    fragments: usize,
    reconnects: usize,
}
fn transmit(
    frame: &Frame,
    payload: usize,
    counts: &mut Counts,
) -> Result<Frame, Box<dyn std::error::Error>> {
    let mut assembler = Assembler::new();
    for chunk in frame.bytes().chunks(payload) {
        counts.bytes += chunk.len();
        counts.fragments += 1;
        assembler.push(chunk)?;
    }
    assembler
        .take()
        .ok_or_else(|| "软件传输未收到完整消息".into())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if !(3..=4).contains(&args.len()) {
        return Err(
            "用法：transfer 专用软件验收目录 播放包.smpkg 每片有效字节 [丢弃第几个应用回执]".into(),
        );
    }
    let payload: usize = args[2].to_str().ok_or("分片字节无效")?.parse()?;
    if !(20..=512).contains(&payload) {
        return Err("软件参考分片范围为 20–512 字节".into());
    }
    let drop_reply: Option<usize> = args
        .get(3)
        .map(|v| {
            v.to_str()
                .ok_or("回执编号无效")?
                .parse()
                .map_err(|_| "回执编号无效")
        })
        .transpose()?;
    let mut source = Vec::new();
    File::open(&args[1])?
        .take(MAX_PACKAGE_BYTES as u64 + 1)
        .read_to_end(&mut source)?;
    if source.len() > MAX_PACKAGE_BYTES {
        return Err("播放包超过参考限额".into());
    }
    // The reference tool deliberately owns a stable bounded host snapshot, not a mutable file.
    let mut upload = Upload::new(source.as_slice())?;
    let boot = *uuid::Uuid::new_v4().as_bytes();
    let (installer, report) = Installer::open(FileStore::open(Path::new(&args[0]))?, boot)?;
    let mut service = Service::new(installer)?;
    let principal = *uuid::Uuid::new_v4().as_bytes();
    let session = *uuid::Uuid::new_v4().as_bytes();
    service.attach(AuthorizedLink { principal, session })?;
    upload.connect(session)?;
    println!("软件传输参考：内存链路＋文件存储，未连接蓝牙，不发送 DMX");
    println!(
        "源包 {} 字节；每片 {payload} 字节；启动候选代数 {:?}",
        source.len(),
        report.selected.map(|c| c.generation)
    );
    let mut counts = Counts::default();
    while let Some(frame) = upload.outbound()?.cloned() {
        counts.requests += 1;
        let request = transmit(&frame, payload, &mut counts)?;
        let reply = service.process(request.bytes())?;
        let reply = transmit(&reply, payload, &mut counts)?;
        if drop_reply == Some(counts.requests) {
            service.detach();
            upload.disconnect();
            let session = *uuid::Uuid::new_v4().as_bytes();
            service.attach(AuthorizedLink { principal, session })?;
            upload.connect(session)?;
            counts.reconnects += 1;
        } else {
            upload.accept(reply.bytes())?;
        }
        if counts.requests > 10_000 {
            return Err("软件传输未收敛".into());
        }
    }
    let Some(Outcome::Installed(commit)) = upload.outcome() else {
        return Err("未得到安装成功回执".into());
    };
    println!(
        "已安装候选代数 {}，槽 {}；未启动节目",
        commit.generation,
        commit.slot.index()
    );
    println!(
        "协议统计：{} 次请求，双向 {} 字节，{} 片，{} 次重连；不是蓝牙速率",
        counts.requests, counts.bytes, counts.fragments, counts.reconnects
    );
    let snapshot = service.snapshot()?;
    let expected = Archive::open(source.as_slice())?;
    assert_eq!(snapshot.archive().entries(), expected.entries());
    for (index, entry) in expected.entries().iter().enumerate() {
        let a = snapshot.load(index)?;
        let b = expected.load(source.as_slice(), index)?;
        let mut actual = Player::new(a.plan, 0);
        let mut original = Player::new(b.plan, 0);
        actual.execute(0, 0)?;
        original.execute(0, 0)?;
        let mut hash = Sha256::new();
        for time in (0..10_000).step_by(25) {
            actual.advance(time)?;
            original.advance(time)?;
            let mut left = [0; 512];
            let mut right = [0; 512];
            a.output.render(actual.values(), &mut left)?;
            b.output.render(original.values(), &mut right)?;
            if left != right {
                return Err("软件重放与源包不一致".into());
            }
            hash.update(left);
        }
        println!("{}：400 帧一致；帧摘要 {:x}", entry.name, hash.finalize());
    }
    Ok(())
}
