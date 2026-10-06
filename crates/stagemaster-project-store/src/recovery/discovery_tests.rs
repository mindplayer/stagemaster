use super::*;
use uuid::Uuid;

#[test]
fn every_insertion_keeps_the_candidate_heap_bounded_and_preserves_the_old_selection() {
    for descending in [false, true] {
        let mut candidates = Candidates::new();
        for value in 0..4096 {
            let number = if descending { 4095 - value } else { value };
            candidates
                .push(&Uuid::from_u128(number).to_string())
                .unwrap();
            assert!(candidates.largest.len() <= MAX_RECOVERY_RECORDS);
            assert!(candidates.largest.capacity() <= MAX_RECOVERY_RECORDS);
        }
        let discovery = candidates.finish();
        assert_eq!(discovery.omitted, 4096 - MAX_RECOVERY_RECORDS);
        let original: Vec<_> = (0..64).map(|n| Uuid::from_u128(n).to_string()).collect();
        assert_eq!(discovery.ids, original);
    }
}

#[test]
fn empty_and_underfull_discovery_preserve_all_candidates_and_count_overflow_is_explicit() {
    let empty = Candidates::new().finish();
    assert!(empty.ids.is_empty());
    assert_eq!(empty.omitted, 0);
    let mut candidates = Candidates::new();
    for number in [7, 1, 12] {
        candidates
            .push(&Uuid::from_u128(number).to_string())
            .unwrap();
    }
    let underfull = candidates.finish();
    let expected: Vec<_> = [1, 7, 12]
        .into_iter()
        .map(|n| Uuid::from_u128(n).to_string())
        .collect();
    assert_eq!(underfull.ids, expected);
    assert_eq!(underfull.omitted, 0);
    let mut overflow = Candidates::new();
    overflow.count = usize::MAX;
    assert!(overflow.push(&Uuid::nil().to_string()).is_err());
    assert!(overflow.largest.is_empty());
    assert_eq!(overflow.count, usize::MAX);
}

#[test]
fn unreadable_directory_remains_an_error_instead_of_a_successful_empty_result() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tmp");
    let directory = tempfile::tempdir_in(root).unwrap();
    assert!(scan(&directory.path().join("absent")).is_err());
    let file = directory.path().join("not-directory");
    fs::write(&file, b"keep").unwrap();
    assert!(scan(&file).is_err());
    assert_eq!(fs::read(file).unwrap(), b"keep");
}
