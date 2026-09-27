//! Compile-only runtime integration. Output stays disabled and the policy denies playback.
use stagemaster_install::Storage;
use stagemaster_runtime::{
    Code, Denial, Grant, Lease, Permission, PlaybackPolicy, Receipt, Request, Runtime,
};
use stagemaster_transfer::Service;

type PolicyCheck = fn(Permission) -> Result<(), Denial>;
struct Policy(PolicyCheck);
impl PlaybackPolicy for Policy {
    fn authorize(&mut self, permission: Permission) -> Result<(), Denial> {
        (self.0)(permission)
    }
}
fn deny(_permission: Permission) -> Result<(), Denial> {
    Err(Denial::Missing)
}
type Driver = esp_bootloader_esp_idf::partitions::NorFlashRegion<'static, 'static, 'static>;
type Reader = stagemaster_nor_store::Snapshot<Driver>;
type Device = Runtime<Reader, Policy>;
pub static RUNTIME_RESOURCE_BYTES: [usize; 4] = [
    core::mem::size_of::<Device>(),
    core::mem::size_of::<Request>(),
    core::mem::size_of::<Receipt>(),
    core::mem::size_of::<stagemaster_runtime::State>(),
];

pub fn inspect<S: Storage>(service: &Service<S>, boot: [u8; 16])
where
    S::Error: core::fmt::Debug,
{
    // The function pointer prevents compile-time permission folding, so allowed as well
    // as denied command paths must compile. The actual selected policy always denies.
    let mut runtime = Runtime::new(
        boot,
        0,
        64 * 1024,
        Policy(core::hint::black_box(deny as PolicyCheck)),
    )
    .unwrap();
    // main owns OutputDisabled, no output task or transmit queue exists in this image.
    let request = runtime.quiescence_request().unwrap();
    let permit = runtime.confirm_quiescent(request, 0).unwrap();
    let recovered = runtime.finish_maintenance(permit, 0, || match service.snapshot() {
        Ok(snapshot) => Ok(Some(snapshot)),
        Err(stagemaster_install::Error::Code(stagemaster_install::Code::Empty)) => Ok(None),
        Err(error) => Err(error),
    });
    match recovered {
        Ok(state) => esp_println::println!("设备运行只读恢复：{:?}", state),
        Err(error) => esp_println::println!("设备运行恢复失败：{:?}", error),
    }
    retain(&mut runtime);
    esp_println::println!(
        "运行对象／请求／回执／状态大小：{:?}",
        RUNTIME_RESOURCE_BYTES
    );
}

fn retain<R: stagemaster_package::ReadAt>(runtime: &mut Runtime<R, Policy>) {
    type Device<R> = Runtime<R, Policy>;
    let submit: fn(&mut Device<R>, Request, u64) -> Result<Receipt, Code> = Device::<R>::submit;
    let acquire: fn(&mut Device<R>, Grant, bool, u64) -> Result<Lease, Code> = Device::<R>::acquire;
    let renew: fn(&mut Device<R>, Lease, u64, u64) -> Result<(), Code> = Device::<R>::renew;
    let release: fn(&mut Device<R>, Lease, u64) -> Result<(), Code> = Device::<R>::release;
    type Render<R> =
        fn(&Device<R>, &mut [u8; 512]) -> Result<Option<stagemaster_runtime::FrameInfo>, Code>;
    let render: Render<R> = Device::<R>::render;
    core::hint::black_box((submit, acquire, renew, release, render, runtime));
}
