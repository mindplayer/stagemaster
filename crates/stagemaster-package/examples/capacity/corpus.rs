use super::programs::{Shape, program};
use stagemaster_package::{Archive, Builder, Error, Kind, Program, Source};

pub const PROJECT: [u8; 16] = [0x72; 16];
pub const TITLE: &str = "ESP32 容量验收";
pub fn archive(programs: &[Program], count: usize) -> Result<(Vec<u8>, Archive), Error> {
    let mut builder = Builder::new(Source {
        project_id: PROJECT,
        revision_id: [0x73; 16],
        snapshot_digest: [0x74; 32],
        project_name: TITLE.into(),
    });
    for i in 0..count {
        builder.add(
            Kind::Sequence,
            (i as u128 + 1).to_be_bytes(),
            &format!("容量验收 {i:02}"),
            &programs[i % programs.len()],
        )?;
    }
    builder.finish()
}
/// Find the last accepted parameter against the actual requested catalogue.
/// The next parameter must fail the original admission checks, not a test limit.
pub fn frontier(shape: Shape, count: usize) -> (usize, Program) {
    let mut low = shape.min_scale();
    let mut high = shape.max_scale();
    assert!(archive(&[program(shape, low)], count).is_ok());
    while low < high {
        let mid = low + (high - low).div_ceil(2);
        if archive(&[program(shape, mid)], count).is_ok() {
            low = mid;
        } else {
            high = mid - 1;
        }
    }
    assert!(
        low < shape.max_scale(),
        "expected a resource boundary for {shape:?}"
    );
    assert!(archive(&[program(shape, low + 1)], count).is_err());
    (low, program(shape, low))
}
