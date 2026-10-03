#![allow(dead_code)]
use stagemaster_runtime::{Mode, State};
use stagemaster_runtime_protocol::{
    Access, Body, Observation, Operation, Peer, Ready, Request, Response,
};

pub fn request(operation: Operation) -> Request {
    Request {
        session: [3; 16],
        id: 1,
        expected_revision: 0,
        operation,
    }
}
pub fn state() -> State {
    State {
        boot: [2; 16],
        revision: 0,
        observed_ms: 100,
        mode: Mode::Operation,
        bound_package: None,
        selected: None,
        loaded: None,
        status: None,
        instance: None,
        step: None,
        elapsed_ms: 0,
        owner: None,
    }
}
pub fn response() -> Response {
    let state = state();
    Response {
        request: request(Operation::Status),
        observed: Observation {
            boot: state.boot,
            revision: state.revision,
            observed_ms: state.observed_ms,
            program_count: 0,
            step_count: 0,
        },
        body: Body::State {
            state,
            result: Ok(()),
        },
    }
}
pub fn ready() -> Ready {
    Ready {
        peer: Peer {
            device: [1; 16],
            boot: [2; 16],
            connection: 1,
            session: [3; 16],
            principal: [4; 16],
            permission_revision: 1,
        },
        version: 1,
        message_bytes: 1280,
        access: Access {
            installation: false,
            observe: true,
            control: true,
        },
        remaining_ms: 1000,
    }
}
pub fn decode_all_truncations<T>(
    bytes: &[u8],
    decode: impl Fn(&[u8]) -> Result<T, stagemaster_runtime_protocol::Error>,
) {
    for n in 0..bytes.len() {
        assert!(decode(&bytes[..n]).is_err(), "accepted prefix {n}");
    }
    let mut extra = bytes.to_vec();
    extra.push(0);
    assert!(decode(&extra).is_err());
    for index in [0, 4, 5, 6, 7] {
        let mut changed = bytes.to_vec();
        changed[index] ^= 0x80;
        assert!(decode(&changed).is_err());
    }
}
