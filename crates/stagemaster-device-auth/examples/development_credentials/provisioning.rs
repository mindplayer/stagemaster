//! Controlled local provisioning. No radio or cloud enrollment; never prints secrets.
use stagemaster_device_auth::application::{CONFIGURATION_BYTES, Configuration, Role};
use stagemaster_device_session::SecretKey;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    os::unix::fs::{DirBuilderExt, OpenOptionsExt},
    path::Path,
};
use zeroize::Zeroizing;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn random<const N: usize>() -> Result<Zeroizing<[u8; N]>> {
    let mut bytes = Zeroizing::new([0; N]);
    getrandom::fill(bytes.as_mut()).map_err(|_| "系统安全随机源不可用")?;
    Ok(bytes)
}
fn encode(
    role: Role,
    device: [u8; 16],
    principal: [u8; 16],
    secret: &[u8; 32],
    peer: [u8; 32],
) -> Result<Zeroizing<[u8; CONFIGURATION_BYTES]>> {
    let mut bytes = Zeroizing::new([0; CONFIGURATION_BYTES]);
    bytes[..8].copy_from_slice(b"SMDV\x01\0\xa0\0");
    bytes[5] = if role == Role::Device { 1 } else { 2 };
    bytes[8..24].copy_from_slice(&device);
    bytes[24..40].copy_from_slice(&principal);
    bytes[40..48].copy_from_slice(&1_u64.to_le_bytes());
    bytes[48..52].copy_from_slice(&600_000_u32.to_le_bytes());
    bytes[56..88].copy_from_slice(secret);
    bytes[88..120].copy_from_slice(&peer);
    bytes[120..152].copy_from_slice(&SecretKey::import(*secret)?.public());
    Configuration::import(bytes.as_ref(), role)?;
    Ok(bytes)
}
fn write(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}
pub(super) fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 2 || args[0].len() != 32 || !args[0].is_ascii() {
        return Err("用法：development_credentials 32位设备编号 项目data内的新目录".into());
    }
    let mut device = [0; 16];
    for (byte, text) in device.iter_mut().zip(args[0].as_bytes().chunks_exact(2)) {
        *byte = u8::from_str_radix(std::str::from_utf8(text)?, 16)?;
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../data")
        .canonicalize()?;
    let destination = Path::new(&args[1]);
    let parent = destination.parent().ok_or("须指定新目录")?.canonicalize()?;
    if !parent.starts_with(&root) || destination.exists() {
        return Err("配置必须新建于项目data目录，禁止覆盖".into());
    }
    let destination = parent.join(destination.file_name().ok_or("目录无效")?);
    let device_secret = random::<32>()?;
    let controller_secret = random::<32>()?;
    let principal = *random::<16>()?;
    let device_bytes = encode(
        Role::Device,
        device,
        principal,
        &device_secret,
        SecretKey::import(*controller_secret)?.public(),
    )?;
    let controller_bytes = encode(
        Role::Controller,
        device,
        principal,
        &controller_secret,
        SecretKey::import(*device_secret)?.public(),
    )?;
    fs::DirBuilder::new().mode(0o700).create(&destination)?;
    write(&destination.join("device.smddev"), device_bytes.as_ref())?;
    write(
        &destination.join("controller.smddev"),
        controller_bytes.as_ref(),
    )?;
    println!(
        "已生成设备专用开发配置：{}；两个文件权限0600，未输出密钥",
        destination.display()
    );
    Ok(())
}
