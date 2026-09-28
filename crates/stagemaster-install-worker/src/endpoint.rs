//! Bounded ordered byte channel in front of the worker; owns no I/O or authority.
use crate::{Command, Completion, Epoch, Reply};
use stagemaster_transfer::{Assembler, AuthorizedLink, Frame, MAX_FRAME_BYTES, Request, Response};

const CHANNEL_MS: u64 = 5_000;
const WORK_MS: u64 = 30_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Opening,
    Receiving,
    Working,
    Sending,
    Closed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChannelError {
    Closed,
    Clock,
    Timeout,
    State,
    Bounds,
    Protocol(stagemaster_transfer::Error),
    Worker(crate::Error),
}
impl core::fmt::Display for ChannelError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Closed => f.write_str("安装通道已关闭"),
            Self::Clock => f.write_str("安装通道计时异常，请重新连接"),
            Self::Timeout => f.write_str("安装通信超时，请重新连接并核对结果"),
            Self::State => f.write_str("安装通道顺序异常，请重新连接"),
            Self::Bounds => f.write_str("安装通信片段超出范围"),
            Self::Protocol(error) => error.fmt(f),
            Self::Worker(_) => f.write_str("安装工作连接已失效，请重新连接并核对结果"),
        }
    }
}
impl core::error::Error for ChannelError {}

#[derive(Clone, Copy)]
struct Expected {
    id: u64,
    command: stagemaster_transfer::Command,
}
// Fixed buffers reserve a bound instead of allocating from untrusted lengths.
#[allow(clippy::large_enum_variant)]
enum State {
    Opening(u64),
    Receiving(Option<u64>),
    Working {
        until: u64,
        expected: Expected,
    },
    Sending {
        until: u64,
        frame: Frame,
        offset: usize,
        flight: Option<usize>,
    },
    Closed,
}

/// One authenticated physical connection, one request and response at a time.
/// Every error closes this endpoint. The adapter MUST immediately publish the
/// resulting `live_epoch()` to the worker's independent revocation channel.
pub struct Endpoint {
    epoch: Epoch,
    session: [u8; 16],
    payload: usize,
    last_time: u64,
    assembler: Assembler,
    state: State,
}
impl Endpoint {
    /// The grant comes only from a trusted authentication adapter, never peer bytes.
    /// Enqueue the returned Open once. Queue failure must close the endpoint.
    /// # Errors
    /// Rejects invalid identities, zero/oversized payload limits or clock overflow.
    pub fn open(
        epoch: Epoch,
        link: AuthorizedLink,
        payload: usize,
        now: u64,
    ) -> Result<(Self, Command), ChannelError> {
        if link.principal == [0; 16] || link.session == [0; 16] {
            return Err(ChannelError::Protocol(stagemaster_transfer::Error::Denied));
        }
        if !(1..=MAX_FRAME_BYTES).contains(&payload) {
            return Err(ChannelError::Bounds);
        }
        Ok((
            Self {
                epoch,
                session: link.session,
                payload,
                last_time: now,
                assembler: Assembler::new(),
                state: State::Opening(deadline(now, CHANNEL_MS)?),
            },
            Command::Open { epoch, link },
        ))
    }

    #[must_use]
    pub const fn phase(&self) -> Phase {
        match self.state {
            State::Opening(_) => Phase::Opening,
            State::Receiving(_) => Phase::Receiving,
            State::Working { .. } => Phase::Working,
            State::Sending { .. } => Phase::Sending,
            State::Closed => Phase::Closed,
        }
    }

    #[must_use]
    pub const fn live_epoch(&self) -> Option<Epoch> {
        if matches!(self.state, State::Closed) {
            None
        } else {
            Some(self.epoch)
        }
    }

    pub fn close(&mut self) {
        self.state = State::Closed;
        self.assembler = Assembler::new();
    }

    /// Call on timer ticks even when no bytes or worker completions arrive.
    /// Idle receive is governed by the outer diagnostic/authentication lease.
    /// # Errors
    /// Closed endpoints, backwards clocks and expired fixed deadlines require reconnect.
    pub fn poll(&mut self, now: u64) -> Result<(), ChannelError> {
        let until = match self.state {
            State::Closed => return Err(ChannelError::Closed),
            State::Opening(until) | State::Working { until, .. } | State::Sending { until, .. } => {
                Some(until)
            }
            State::Receiving(until) => until,
        };
        let result = if now < self.last_time {
            Err(ChannelError::Clock)
        } else if until.is_some_and(|until| now >= until) {
            Err(ChannelError::Timeout)
        } else {
            self.last_time = now;
            Ok(())
        };
        self.checked(result)
    }

    /// Accept one ordered fragment. A complete request is moved into the queue command.
    /// # Errors
    /// Malformed input, wrong session, backpressure or time failure closes the channel.
    pub fn receive(&mut self, bytes: &[u8], now: u64) -> Result<Option<Command>, ChannelError> {
        self.poll(now)?;
        let result = self.receive_inner(bytes, now);
        self.checked(result)
    }

    fn receive_inner(&mut self, bytes: &[u8], now: u64) -> Result<Option<Command>, ChannelError> {
        let State::Receiving(until) = &mut self.state else {
            return Err(ChannelError::State);
        };
        if bytes.is_empty() || bytes.len() > self.payload {
            return Err(ChannelError::Bounds);
        }
        if until.is_none() {
            *until = Some(deadline(now, CHANNEL_MS)?);
        }
        if !self.assembler.push(bytes).map_err(ChannelError::Protocol)? {
            return Ok(None);
        }
        let frame = self.assembler.take().ok_or(ChannelError::State)?;
        let request = Request::decode(frame.bytes()).map_err(ChannelError::Protocol)?;
        if request.link != self.session {
            return Err(ChannelError::Protocol(
                stagemaster_transfer::Error::Connection,
            ));
        }
        self.state = State::Working {
            until: deadline(now, WORK_MS)?,
            expected: Expected {
                id: request.id,
                command: request.action.command(),
            },
        };
        Ok(Some(Command::Frame {
            epoch: self.epoch,
            frame,
        }))
    }

    /// Drain all completions, including obsolete ones. Returns false for another epoch.
    /// # Errors
    /// Same-epoch mismatches, worker failures or expired deadlines close the channel.
    pub fn complete(&mut self, completion: Completion, now: u64) -> Result<bool, ChannelError> {
        self.poll(now)?;
        if completion.epoch != self.epoch {
            return Ok(false);
        }
        let result = self.complete_inner(completion, now);
        self.checked(result)
    }

    fn complete_inner(&mut self, completion: Completion, now: u64) -> Result<bool, ChannelError> {
        let reply = completion.result.map_err(ChannelError::Worker)?;
        match (&self.state, reply) {
            (State::Opening(_), Reply::Opened) => self.state = State::Receiving(None),
            (State::Working { expected, .. }, Reply::Frame(frame)) => {
                let response = Response::decode(frame.bytes()).map_err(ChannelError::Protocol)?;
                if response.link != self.session
                    || response.id != expected.id
                    || response.command != expected.command
                {
                    return Err(ChannelError::Protocol(
                        stagemaster_transfer::Error::Connection,
                    ));
                }
                self.state = State::Sending {
                    until: deadline(now, CHANNEL_MS)?,
                    frame,
                    offset: 0,
                    flight: None,
                };
            }
            _ => return Err(ChannelError::State),
        }
        Ok(true)
    }

    /// Repeated reads return the same fragment until `sent` confirms transport success.
    /// # Errors
    /// Closed, expired or backwards-clock access requires reconnect.
    pub fn fragment(&mut self, now: u64) -> Result<Option<&[u8]>, ChannelError> {
        self.poll(now)?;
        let State::Sending {
            frame,
            offset,
            flight,
            ..
        } = &mut self.state
        else {
            return Ok(None);
        };
        let length = *flight.get_or_insert_with(|| self.payload.min(frame.bytes().len() - *offset));
        Ok(Some(&frame.bytes()[*offset..*offset + length]))
    }

    /// Call only after the exact current fragment was accepted by the transport.
    /// The last fragment returns true. This is NOT an installation success receipt.
    /// # Errors
    /// Missing outstanding fragment, clock or deadline failure closes the channel.
    pub fn sent(&mut self, now: u64) -> Result<bool, ChannelError> {
        self.poll(now)?;
        let result = if let State::Sending {
            frame,
            offset,
            flight,
            ..
        } = &mut self.state
        {
            if let Some(length) = flight.take() {
                *offset += length;
                let done = *offset == frame.bytes().len();
                if done {
                    self.state = State::Receiving(None);
                }
                Ok(done)
            } else {
                Err(ChannelError::State)
            }
        } else {
            Err(ChannelError::State)
        };
        self.checked(result)
    }

    fn checked<T>(&mut self, result: Result<T, ChannelError>) -> Result<T, ChannelError> {
        if result.is_err() {
            self.close();
        }
        result
    }
}

fn deadline(now: u64, duration: u64) -> Result<u64, ChannelError> {
    now.checked_add(duration).ok_or(ChannelError::Clock)
}
