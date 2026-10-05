#[path = "../../../tools/package-acceptance/bounded_corpus.rs"]
mod corpus;
#[path = "../../../tools/package-acceptance/reader_metrics.rs"]
mod metrics;
use metrics::Reader;
use stagemaster_package::{Archive, Error, MAX_PACKAGE_BYTES};

#[test]
fn maximum_files_and_late_bad_programs_require_complete_bounded_validation() {
    for case in corpus::cases() {
        let reader = Reader::new(&case.bytes);
        assert_eq!(
            Archive::open(&reader).unwrap_err(),
            case.expected,
            "{}",
            case.name
        );
        reader.assert_complete_hash();
        if case.name.starts_with("max-") {
            assert_eq!(case.bytes.len(), MAX_PACKAGE_BYTES);
        }
        if case.name.starts_with("last-") {
            // Hash requests, one catalogue request, then every block up to and including #64.
            let hash_reads = (case.bytes.len() - 64).div_ceil(1024);
            assert_eq!(reader.reads.borrow().len(), 1 + hash_reads + 1 + 64);
        }
    }
}
#[test]
fn maximum_hash_middle_and_final_read_failures_never_admit_a_partial_archive() {
    let bytes = corpus::cases().remove(0).bytes;
    for offset in [64 + 512 * 1024, 64 + 2047 * 1024] {
        let mut reader = Reader::new(&bytes);
        reader.fail_at = Some(offset);
        assert_eq!(Archive::open(&reader).unwrap_err(), Error::Read);
        assert_eq!(reader.reads.borrow().last().unwrap().0, offset);
        assert_eq!(reader.reads.borrow().len(), 1 + (offset - 64) / 1024 + 1);
    }
}
#[test]
fn a_byte_above_the_file_limit_refuses_before_any_source_io() {
    let mut bytes = corpus::cases().remove(0).bytes;
    bytes.push(0);
    let reader = Reader::new(&bytes);
    assert_eq!(
        Archive::open(&reader).unwrap_err(),
        Error::Limit("包大小须在 64 字节至 2 MiB 内")
    );
    assert!(reader.reads.borrow().is_empty());
}
#[test]
fn all_64_legal_programs_are_independently_loadable() {
    let bytes = corpus::valid_64();
    let reader = Reader::new(&bytes);
    let archive = Archive::open(&reader).unwrap();
    reader.assert_complete_hash();
    assert_eq!(archive.entries().len(), 64);
    reader.reads.borrow_mut().clear();
    for index in 0..64 {
        let program = archive.load(&reader, index).unwrap();
        assert_eq!(
            stagemaster_package::encode_program(&program).unwrap(),
            corpus::valid_block()
        );
        assert_eq!(program.output.mappings[0].coarse, 17);
    }
    assert_eq!(reader.reads.borrow().len(), 64);
}
