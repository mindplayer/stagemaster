use super::*;
use crate::Command;
use stagemaster_install::{Progress, Slot};

#[test]
fn final_request_success_is_reported_before_sequence_exhaustion() {
    let identity = Identity {
        bytes: 64,
        digest: [3; 32],
    };
    let transaction = Transaction {
        boot: [2; 16],
        counter: 1,
    };
    let commit = Commit {
        slot: Slot::A,
        generation: 1,
        identity,
    };
    // Seed the finite-state boundary directly: iterating 2^64 requests cannot test exhaustion.
    // This path must not read the source; it consumes only the matching final commit receipt.
    let mut upload = Upload {
        source: &[][..],
        identity,
        link: Some([1; 16]),
        last_link: Some([1; 16]),
        next_id: Some(u64::MAX),
        pending: Some(
            Request {
                link: [1; 16],
                id: u64::MAX,
                action: Action::Commit(transaction),
            }
            .encode()
            .unwrap(),
        ),
        latest: None,
        outcome: None,
        cancel: false,
        halted: None,
        query: false,
    };
    let response = Response {
        link: [1; 16],
        id: u64::MAX,
        command: Command::Commit,
        result: Ok(()),
        state: State {
            boot: transaction.boot,
            head: Some(commit),
            progress: Some(Progress {
                transaction,
                identity,
                received: 64,
                phase: Phase::Committed,
                commit,
            }),
            owned: true,
            max_chunk: 1024,
            max_package: 2 * 1024 * 1024,
        },
    };
    upload.accept(response.encode().unwrap().bytes()).unwrap();
    assert_eq!(upload.next_id, None);
    assert!(upload.outbound().unwrap().is_none());
    assert_eq!(upload.outcome(), Some(Outcome::Installed(commit)));
}
