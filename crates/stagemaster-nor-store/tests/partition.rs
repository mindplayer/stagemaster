#[path = "../../../apps/esp32-player/src/package_layout.rs"]
mod policy;
use policy::{
    Entry, Error, FLASH_BYTES, PARTITION_BYTES, PARTITION_LABEL, PARTITION_OFFSET, SLOT_BYTES,
    validate,
};
use stagemaster_nor_store::Layout;
fn entries() -> Vec<Entry> {
    include_str!("../../../apps/esp32-player/partitions-storage.csv")
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
        .map(|line| {
            let columns: Vec<_> = line.split(',').collect();
            assert_eq!(columns.len(), 6);
            assert_eq!(columns[5], "");
            let hex = |s: &str| u32::from_str_radix(s.trim_start_matches("0x"), 16).unwrap();
            Entry {
                package_label: columns[0] == PARTITION_LABEL,
                kind: match columns[1] {
                    "app" => 0,
                    "data" => 1,
                    v => u8::try_from(hex(v)).unwrap(),
                },
                subtype: match columns[2] {
                    "nvs" => 2,
                    "ota" => 0,
                    "phy" => 1,
                    "ota_0" => 16,
                    "ota_1" => 17,
                    v => u8::try_from(hex(v)).unwrap(),
                },
                offset: hex(columns[3]),
                length: hex(columns[4]),
                flags: 0,
            }
        })
        .collect()
}
fn check(entries: &[Entry]) -> Result<usize, Error> {
    validate(FLASH_BYTES, entries.len(), |i| entries.get(i).copied())
}
#[test]
fn actual_profile_has_two_app_slots_and_a_separate_bounded_nor_partition() {
    let list = entries();
    let index = check(&list).unwrap();
    assert_eq!(list[index].offset, PARTITION_OFFSET);
    assert_eq!(list[index].length, PARTITION_BYTES);
    assert_eq!(
        PARTITION_BYTES as usize,
        Layout::new(SLOT_BYTES).unwrap().total_bytes()
    );
    assert_eq!(list.iter().filter(|e| e.kind == 0).count(), 2);
    assert_eq!(
        list.last().unwrap().offset + list.last().unwrap().length,
        0x00a1_2000
    );
}
#[test]
fn foreign_overlapping_truncated_or_flagged_tables_are_never_writable() {
    let original = entries();
    assert_eq!(
        validate(0x0080_0000, original.len(), |i| original.get(i).copied()),
        Err(Error::Capacity)
    );
    assert_eq!(
        validate(FLASH_BYTES, original.len(), |_| None),
        Err(Error::Table)
    );
    assert_eq!(check(&[]), Err(Error::Table));
    assert_eq!(check(&original[..5]), Err(Error::Missing));
    for (field, value) in [
        (0, 0x0060_0000),
        (1, 0),
        (2, u32::MAX),
        (3, 1),
        (4, 2),
        (5, 0x41),
        (6, 1),
    ] {
        let mut list = original.clone();
        let target = list.last_mut().unwrap();
        match field {
            0 | 2 => target.offset = value,
            1 => target.length = value,
            3 | 4 => target.flags = value,
            5 => target.kind = u8::try_from(value).unwrap(),
            6 => target.subtype = u8::try_from(value).unwrap(),
            _ => unreachable!(),
        }
        assert!(check(&list).is_err(), "field {field}");
    }
    let mut list = original.clone();
    list[3].offset += 4096;
    assert_eq!(check(&list), Err(Error::Range));
    let mut list = original.clone();
    list[0].offset = 0x8000;
    assert_eq!(check(&list), Err(Error::Range));
    let mut list = original.clone();
    list[0].length += 4096;
    assert_eq!(check(&list), Err(Error::Overlap));
    let mut list = original;
    let mut duplicate = *list.last().unwrap();
    duplicate.offset = 0x00b0_0000;
    duplicate.length = 4096;
    list.push(duplicate);
    assert_eq!(check(&list), Err(Error::Duplicate));
}
