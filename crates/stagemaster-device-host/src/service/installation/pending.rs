use super::Call;
use crate::{InstallationPeer, Problem, ProblemCode as C, installation_peer::wire_error};
use stagemaster_transfer::{Assembler, Command, Frame, Request, Response};
use std::time::Duration;
use tokio::time::Instant;

const FRAME_TIME: Duration = Duration::from_secs(5);
const WORK_TIME: Duration = Duration::from_secs(30);

/// Per-message state only. Transaction ownership, progress and retry are Upload's job.
pub(in crate::service) struct Pending {
    call: Call,
    peer: InstallationPeer,
    id: u64,
    command: Command,
    sent: usize,
    received: usize,
    response: Assembler,
    deadline: Instant,
}

impl Pending {
    pub fn new(call: Call, peer: InstallationPeer) -> Result<Self, Problem> {
        peer.request(&call.frame)?;
        let request = Request::decode(call.frame.bytes()).map_err(wire_error)?;
        let (id, command) = (request.id, request.action.command());
        Ok(Self {
            call,
            peer,
            id,
            command,
            sent: 0,
            received: 0,
            response: Assembler::new(),
            deadline: Instant::now() + FRAME_TIME,
        })
    }
    pub fn abandoned(&self) -> bool {
        self.call.reply.is_closed()
    }
    pub fn started(&self) -> bool {
        self.sent != 0
    }
    pub fn remaining(&self) -> Result<Duration, Problem> {
        self.deadline
            .checked_duration_since(Instant::now())
            .filter(|d| !d.is_zero())
            .ok_or_else(|| Problem::new(C::Timeout))
    }
    pub fn fragment(&self) -> Option<&[u8]> {
        let bytes = self.call.frame.bytes();
        (self.sent < bytes.len()).then(|| {
            &bytes[self.sent
                ..bytes
                    .len()
                    .min(self.sent + usize::from(self.peer.fragment_bytes))]
        })
    }
    pub fn sent(&mut self) {
        self.sent += self.fragment().map_or(0, <[u8]>::len);
        if self.sent == self.call.frame.bytes().len() {
            self.deadline = Instant::now() + WORK_TIME;
        }
    }
    pub fn receive(&mut self, bytes: &[u8]) -> Result<Option<Frame>, Problem> {
        self.remaining()?;
        if self.fragment().is_some() || bytes.len() > usize::from(self.peer.fragment_bytes) {
            return Err(Problem::new(C::Protocol));
        }
        if self.received == 0 {
            self.deadline = self.deadline.min(Instant::now() + FRAME_TIME);
        }
        self.received += bytes.len();
        if self.received > usize::from(self.peer.message_bytes) {
            return Err(Problem::new(C::Protocol));
        }
        if !self.response.push(bytes).map_err(wire_error)? {
            return Ok(None);
        }
        let frame = self
            .response
            .take()
            .ok_or_else(|| Problem::new(C::Protocol))?;
        let response = Response::decode(frame.bytes()).map_err(wire_error)?;
        if response.link != self.peer.session
            || response.id != self.id
            || response.command != self.command
            || response.state.boot != self.peer.boot
        {
            return Err(Problem::new(C::Protocol));
        }
        Ok(Some(frame))
    }
    pub fn complete(self, result: Result<Frame, Problem>) {
        self.call.complete(result);
    }
}
