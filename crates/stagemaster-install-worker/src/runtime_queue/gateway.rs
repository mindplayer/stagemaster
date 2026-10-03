use super::{Command, Completion, EXCHANGE_MS, Error, Live, WORK_MS, Work, after};
use crate::Epoch;
use stagemaster_device_auth::application::{Scope, Session};
use stagemaster_device_session::{CIPHERTEXT_BYTES, Kind, PLAINTEXT_BYTES};
use stagemaster_runtime_protocol::{Frame, Offer, Request, Response};

pub(super) struct Pending {
    pub ticket: u64,
    pub work: Work,
    pub deadline: u64,
    pub frame: Option<Frame>,
}
#[derive(Clone, Copy)]
pub(super) enum Sending {
    Reply,
    Heartbeat,
}
/// Own on the communication task. No storage I/O or playback happens here.
/// Adapter teardown MUST clear the independent Live slot, including on Drop.
pub struct Gateway {
    pub(super) access: Session,
    epoch: Epoch,
    ticket: u64,
    admitted_ms: u64,
    last_ms: u64,
    opening_until: Option<u64>,
    pub(super) ready: bool,
    pub(super) pending: Option<Pending>,
    pub(super) heartbeat_until: Option<u64>,
    pub(super) sending: Option<(Sending, u64)>,
    pub(super) out: [u8; CIPHERTEXT_BYTES],
    pub(super) length: usize,
    closed: bool,
}
impl Gateway {
    /// Does not open maintenance or take input ownership. Await the client's SMRT offer.
    /// # Errors
    /// Admission must be current and include observation. Installation-only is refused.
    pub fn new(mut access: Session, epoch: Epoch, now: u64) -> Result<Self, Error> {
        access.require(Scope::Observe, now)?;
        Ok(Self {
            access,
            epoch,
            ticket: 0,
            admitted_ms: now,
            last_ms: now,
            opening_until: Some(after(now, EXCHANGE_MS)?),
            ready: false,
            pending: None,
            heartbeat_until: None,
            sending: None,
            out: [0; CIPHERTEXT_BYTES],
            length: 0,
            closed: false,
        })
    }
    pub fn close(&mut self) {
        self.access.revoke();
        self.closed = true;
        self.ready = false;
        self.pending = None;
        self.heartbeat_until = None;
        self.sending = None;
        self.out.fill(0);
        self.length = 0;
    }
    /// Fresh exact validity for the adapter's independent synchronized slot.
    /// Always replace the slot with this result, including None, after every operation.
    pub fn live(&mut self, now: u64) -> Option<Live> {
        self.poll(now).ok()
    }
    /// # Errors
    /// Fixed work, reply and heartbeat deadlines cannot be extended by polling or retries.
    pub fn poll(&mut self, now: u64) -> Result<Live, Error> {
        let result = (|| {
            if self.closed {
                return Err(Error::Closed);
            }
            if now < self.last_ms {
                return Err(Error::Clock);
            }
            self.last_ms = now;
            let grant = self.access.require(Scope::Observe, now)?;
            let mut until = self.access.valid_until(now)?;
            for deadline in [
                self.opening_until,
                self.pending.as_ref().map(|p| p.deadline),
                self.heartbeat_until,
                self.sending.map(|(_, deadline)| deadline),
            ]
            .into_iter()
            .flatten()
            {
                until = until.min(deadline);
            }
            if now >= until {
                return Err(Error::Expired);
            }
            Ok(Live {
                epoch: self.epoch,
                grant,
                admitted_ms: self.admitted_ms,
                until,
            })
        })();
        self.checked(result)
    }
    /// A precise retry of pending work does not create another queued command.
    /// # Errors
    /// Malformed, unsolicited or changed requests, foreign sessions and excess heartbeats close.
    pub fn receive(&mut self, cipher: &[u8], now: u64) -> Result<Option<Command>, Error> {
        let live = self.poll(now)?;
        let mut plain = [0; PLAINTEXT_BYTES];
        let result = (|| {
            let record = self.access.open(cipher, &mut plain, now)?;
            if record.kind == Kind::Heartbeat && self.heartbeat_until.is_none() {
                self.heartbeat_until = Some(after(now, EXCHANGE_MS)?);
                return Ok(None);
            }
            if record.kind != Kind::Message {
                return Err(Error::Order);
            }
            let work = if self.ready {
                let request = Request::decode(record.payload)?;
                if request.session != live.grant.session() {
                    return Err(Error::Protocol(
                        stagemaster_runtime_protocol::Error::Identity,
                    ));
                }
                Work::Request(request)
            } else {
                let offer = Offer::decode(record.payload)?;
                offer.select()?;
                Work::Open(offer)
            };
            if let Some(pending) = &self.pending {
                return if pending.work == work {
                    Ok(None)
                } else {
                    Err(Error::Order)
                };
            }
            self.ticket = self.ticket.checked_add(1).ok_or(Error::Order)?;
            let deadline = if matches!(work, Work::Open(_)) {
                self.opening_until.ok_or(Error::Order)?
            } else {
                after(now, WORK_MS)?
            };
            self.pending = Some(Pending {
                ticket: self.ticket,
                work,
                deadline,
                frame: None,
            });
            Ok(Some(Command {
                epoch: self.epoch,
                ticket: self.ticket,
                deadline,
                work,
            }))
        })();
        plain.fill(0);
        self.checked(result)
    }
    /// # Errors
    /// Only the exact pending ticket can complete; a late old generation is ignored.
    /// Old work cannot restore revoked access or replace an already prepared reply.
    pub fn complete(&mut self, completion: Completion, now: u64) -> Result<bool, Error> {
        let live = self.poll(now)?;
        let result = (|| {
            if completion.epoch != self.epoch || completion.ticket < self.ticket {
                return Ok(false);
            }
            let pending = self.pending.as_mut().ok_or(Error::Order)?;
            if completion.ticket != pending.ticket || pending.frame.is_some() {
                return Err(Error::Order);
            }
            let mut frame = completion.result.map_err(Error::Worker)?;
            match pending.work {
                Work::Open(offer) => {
                    frame = super::outgoing::check_ready(
                        &frame,
                        offer,
                        live.grant,
                        self.opening_until.ok_or(Error::Order)? - EXCHANGE_MS,
                        now,
                    )?;
                }
                Work::Request(request) => {
                    Response::decode(frame.bytes())?
                        .correlate(request, live.grant.context().boot)?;
                }
            }
            pending.frame = Some(frame);
            pending.deadline = after(now, EXCHANGE_MS)?;
            Ok(true)
        })();
        self.checked(result)
    }
    pub(super) fn delivered(&mut self) -> Result<(), Error> {
        let pending = self.pending.take().ok_or(Error::Order)?;
        if matches!(pending.work, Work::Open(_)) {
            self.ready = true;
            self.opening_until = None;
        }
        Ok(())
    }
    pub(super) fn checked<T>(&mut self, result: Result<T, Error>) -> Result<T, Error> {
        if result.is_err() {
            self.close();
        }
        result
    }
}
impl Drop for Gateway {
    fn drop(&mut self) {
        self.close();
    }
}
