use crate::{
    Action, Command, Error, Frame, GROUP, Id, RemoteError, Request, Response, State, VERSION,
    framing::{HEADER, header},
};
use minicbor::{Decoder, Encoder, data::Type, encode::write::Cursor};
use stagemaster_install::{Commit, Identity, MAX_CHUNK_BYTES, Phase, Progress, Slot, Transaction};
use stagemaster_package::MAX_PACKAGE_BYTES;

type Encode<'a> = Encoder<Cursor<&'a mut [u8]>>;
fn key<'a, 'b>(e: &'a mut Encode<'b>, name: &str) -> Result<&'a mut Encode<'b>, Error> {
    Ok(e.str(name)?)
}
fn id(d: &mut Decoder<'_>) -> Result<Id, Error> {
    d.bytes()?.try_into().map_err(|_| Error::Malformed)
}
fn digest(d: &mut Decoder<'_>) -> Result<[u8; 32], Error> {
    d.bytes()?.try_into().map_err(|_| Error::Malformed)
}
fn array(d: &mut Decoder<'_>, n: u64) -> Result<(), Error> {
    if d.array()? != Some(n) {
        return Err(Error::Malformed);
    }
    Ok(())
}
fn identity(bytes: usize, digest: [u8; 32]) -> Result<Identity, Error> {
    if !(64..=MAX_PACKAGE_BYTES).contains(&bytes) {
        return Err(Error::Bounds);
    }
    Ok(Identity { bytes, digest })
}
fn transaction(d: &mut Decoder<'_>) -> Result<Transaction, Error> {
    let boot = id(d)?;
    let counter = d.u64()?;
    if boot == [0; 16] || counter == 0 {
        return Err(Error::Malformed);
    }
    Ok(Transaction { boot, counter })
}
fn write_transaction(e: &mut Encode<'_>, t: Transaction) -> Result<(), Error> {
    if t.boot == [0; 16] || t.counter == 0 {
        return Err(Error::Malformed);
    }
    e.bytes(&t.boot)?.u64(t.counter)?;
    Ok(())
}
fn slot(d: &mut Decoder<'_>) -> Result<Slot, Error> {
    match d.u8()? {
        0 => Ok(Slot::A),
        1 => Ok(Slot::B),
        _ => Err(Error::Malformed),
    }
}
fn phase(d: &mut Decoder<'_>) -> Result<Phase, Error> {
    match d.u8()? {
        0 => Ok(Phase::Receiving),
        1 => Ok(Phase::Verified),
        2 => Ok(Phase::Committed),
        3 => Ok(Phase::Cancelled),
        4 => Ok(Phase::Failed),
        5 => Ok(Phase::Uncertain),
        _ => Err(Error::Version),
    }
}
fn phase_code(p: Phase) -> u8 {
    match p {
        Phase::Receiving => 0,
        Phase::Verified => 1,
        Phase::Committed => 2,
        Phase::Cancelled => 3,
        Phase::Failed => 4,
        Phase::Uncertain => 5,
    }
}
fn envelope(e: &mut Encode<'_>, n: u64, link: Id, request_id: u64) -> Result<(), Error> {
    if link == [0; 16] || request_id == 0 {
        return Err(Error::Malformed);
    }
    e.map(n)?;
    key(e, "v")?.u8(VERSION)?;
    key(e, "link")?.bytes(&link)?;
    key(e, "id")?.u64(request_id)?;
    Ok(())
}
fn finish(
    mut frame: Frame,
    length: usize,
    command: Command,
    response: bool,
    request_id: u64,
) -> Result<Frame, Error> {
    frame.length = HEADER + length;
    frame.bytes[0] = 8 | if command == Command::Status { 0 } else { 2 } | u8::from(response);
    frame.bytes[2..4].copy_from_slice(
        &u16::try_from(length)
            .map_err(|_| Error::Bounds)?
            .to_be_bytes(),
    );
    frame.bytes[4..6].copy_from_slice(&GROUP.to_be_bytes());
    frame.bytes[6] = request_id.to_be_bytes()[7];
    frame.bytes[7] = command as u8;
    Ok(frame)
}
pub(crate) fn encode_request(request: Request<'_>) -> Result<Frame, Error> {
    let mut frame = Frame::empty();
    let mut e = Encoder::new(Cursor::new(&mut frame.bytes[HEADER..]));
    envelope(&mut e, 4, request.link, request.id)?;
    key(&mut e, "body")?;
    match request.action {
        Action::Status => {
            e.array(0)?;
        }
        Action::Begin {
            transaction,
            identity: wanted,
        } => {
            identity(wanted.bytes, wanted.digest)?;
            e.array(4)?;
            write_transaction(&mut e, transaction)?;
            e.u32(u32::try_from(wanted.bytes).map_err(|_| Error::Bounds)?)?
                .bytes(&wanted.digest)?;
        }
        Action::Write {
            transaction,
            offset,
            bytes,
        } => {
            check_chunk(offset, bytes)?;
            e.array(4)?;
            write_transaction(&mut e, transaction)?;
            e.u32(u32::try_from(offset).map_err(|_| Error::Bounds)?)?
                .bytes(bytes)?;
        }
        action => {
            e.array(2)?;
            write_transaction(&mut e, action.transaction().ok_or(Error::State)?)?;
        }
    }
    let length = e.writer().position();
    finish(frame, length, request.action.command(), false, request.id)
}
fn check_chunk(offset: usize, bytes: &[u8]) -> Result<(), Error> {
    if bytes.is_empty()
        || bytes.len() > MAX_CHUNK_BYTES
        || offset
            .checked_add(bytes.len())
            .is_none_or(|n| n > MAX_PACKAGE_BYTES)
    {
        return Err(Error::Bounds);
    }
    Ok(())
}
fn read_action<'a>(d: &mut Decoder<'a>, command: Command) -> Result<Action<'a>, Error> {
    if command == Command::Status {
        array(d, 0)?;
        return Ok(Action::Status);
    }
    array(
        d,
        if matches!(command, Command::Begin | Command::Write) {
            4
        } else {
            2
        },
    )?;
    let transaction = transaction(d)?;
    Ok(match command {
        Command::Begin => {
            let bytes = d.u32()? as usize;
            Action::Begin {
                transaction,
                identity: identity(bytes, digest(d)?)?,
            }
        }
        Command::Write => {
            let offset = d.u32()? as usize;
            let bytes = d.bytes()?;
            check_chunk(offset, bytes)?;
            Action::Write {
                transaction,
                offset,
                bytes,
            }
        }
        Command::Verify => Action::Verify(transaction),
        Command::Commit => Action::Commit(transaction),
        Command::Cancel => Action::Cancel(transaction),
        Command::Reconcile => Action::Reconcile(transaction),
        Command::Status => return Err(Error::Malformed),
    })
}
fn decode_header(bytes: &[u8], response: bool) -> Result<Command, Error> {
    let (length, command, actual_response) = header(bytes)?;
    if length != bytes.len() || response != actual_response {
        return Err(Error::Malformed);
    }
    Ok(command)
}
fn check_envelope(
    link: Option<Id>,
    request_id: Option<u64>,
    version: bool,
    bytes: &[u8],
    end: usize,
) -> Result<(Id, u64), Error> {
    let link = link.ok_or(Error::Malformed)?;
    let request_id = request_id.ok_or(Error::Malformed)?;
    if !version
        || link == [0; 16]
        || request_id == 0
        || request_id.to_be_bytes()[7] != bytes[6]
        || end != bytes.len() - HEADER
    {
        return Err(Error::Malformed);
    }
    Ok((link, request_id))
}
fn read_version(d: &mut Decoder<'_>, seen: &mut bool) -> Result<(), Error> {
    if *seen {
        return Err(Error::Malformed);
    }
    if d.u8()? != VERSION {
        return Err(Error::Version);
    }
    *seen = true;
    Ok(())
}
pub(crate) fn decode_request(bytes: &[u8]) -> Result<Request<'_>, Error> {
    let command = decode_header(bytes, false)?;
    let mut d = Decoder::new(&bytes[HEADER..]);
    if d.map()? != Some(4) {
        return Err(Error::Malformed);
    }
    let (mut link, mut request_id, mut action, mut version) = (None, None, None, false);
    for _ in 0..4 {
        match d.str()? {
            "v" => read_version(&mut d, &mut version)?,
            "link" if link.is_none() => link = Some(id(&mut d)?),
            "id" if request_id.is_none() => request_id = Some(d.u64()?),
            "body" if action.is_none() => action = Some(read_action(&mut d, command)?),
            _ => return Err(Error::Malformed),
        }
    }
    let (link, request_id) = check_envelope(link, request_id, version, bytes, d.position())?;
    Ok(Request {
        link,
        id: request_id,
        action: action.ok_or(Error::Malformed)?,
    })
}
fn write_commit(e: &mut Encode<'_>, c: Commit) -> Result<(), Error> {
    e.array(4)?
        .u8(u8::from(c.slot != Slot::A))?
        .u64(c.generation)?
        .u32(u32::try_from(c.identity.bytes).map_err(|_| Error::Bounds)?)?
        .bytes(&c.identity.digest)?;
    Ok(())
}
fn read_commit(d: &mut Decoder<'_>) -> Result<Commit, Error> {
    array(d, 4)?;
    let slot = slot(d)?;
    let generation = d.u64()?;
    let bytes = d.u32()? as usize;
    if generation == 0 {
        return Err(Error::Malformed);
    }
    Ok(Commit {
        slot,
        generation,
        identity: identity(bytes, digest(d)?)?,
    })
}
fn validate_state(s: State) -> Result<(), Error> {
    if s.boot == [0; 16]
        || s.max_chunk == 0
        || s.max_chunk > MAX_CHUNK_BYTES
        || !(64..=MAX_PACKAGE_BYTES).contains(&s.max_package)
    {
        return Err(Error::Bounds);
    }
    if let Some(c) = s.head {
        c.encode().map_err(|_| Error::Malformed)?;
    }
    if let Some(p) = s.progress {
        p.commit.encode().map_err(|_| Error::Malformed)?;
        if p.transaction.boot != s.boot
            || p.transaction.counter == 0
            || p.identity != p.commit.identity
            || p.received > p.identity.bytes
            || (matches!(
                p.phase,
                Phase::Verified | Phase::Committed | Phase::Uncertain
            ) && p.received != p.identity.bytes)
            || (p.phase == Phase::Committed && s.head != Some(p.commit))
        {
            return Err(Error::State);
        }
    } else if s.owned {
        return Err(Error::State);
    }
    Ok(())
}
fn write_state(e: &mut Encode<'_>, s: State) -> Result<(), Error> {
    validate_state(s)?;
    e.array(6)?.bytes(&s.boot)?;
    if let Some(c) = s.head {
        write_commit(e, c)?;
    } else {
        e.null()?;
    }
    if let Some(p) = s.progress {
        e.array(4)?
            .u64(p.transaction.counter)?
            .u32(u32::try_from(p.received).map_err(|_| Error::Bounds)?)?
            .u8(phase_code(p.phase))?;
        write_commit(e, p.commit)?;
    } else {
        e.null()?;
    }
    e.bool(s.owned)?
        .u16(u16::try_from(s.max_chunk).map_err(|_| Error::Bounds)?)?
        .u32(u32::try_from(s.max_package).map_err(|_| Error::Bounds)?)?;
    Ok(())
}
fn read_state(d: &mut Decoder<'_>) -> Result<State, Error> {
    array(d, 6)?;
    let boot = id(d)?;
    let head = if d.datatype()? == Type::Null {
        d.null()?;
        None
    } else {
        Some(read_commit(d)?)
    };
    let progress = if d.datatype()? == Type::Null {
        d.null()?;
        None
    } else {
        array(d, 4)?;
        let counter = d.u64()?;
        let received = d.u32()? as usize;
        let phase = phase(d)?;
        let commit = read_commit(d)?;
        Some(Progress {
            transaction: Transaction { boot, counter },
            identity: commit.identity,
            received,
            phase,
            commit,
        })
    };
    let owned = d.bool()?;
    let s = State {
        boot,
        head,
        progress,
        owned,
        max_chunk: usize::from(d.u16()?),
        max_package: d.u32()? as usize,
    };
    validate_state(s)?;
    Ok(s)
}
pub(crate) fn encode_response(response: &Response) -> Result<Frame, Error> {
    let mut frame = Frame::empty();
    let mut e = Encoder::new(Cursor::new(&mut frame.bytes[HEADER..]));
    envelope(
        &mut e,
        if response.result.is_ok() { 4 } else { 5 },
        response.link,
        response.id,
    )?;
    key(&mut e, "state")?;
    write_state(&mut e, response.state)?;
    if let Err(error) = response.result {
        key(&mut e, "err")?
            .map(2)?
            .str("group")?
            .u16(GROUP)?
            .str("rc")?
            .u8(error as u8)?;
    }
    let length = e.writer().position();
    finish(frame, length, response.command, true, response.id)
}
fn read_error(d: &mut Decoder<'_>) -> Result<RemoteError, Error> {
    if d.map()? != Some(2) {
        return Err(Error::Malformed);
    }
    let (mut group, mut code) = (None, None);
    for _ in 0..2 {
        match d.str()? {
            "group" if group.is_none() => group = Some(d.u16()?),
            "rc" if code.is_none() => code = Some(RemoteError::read(d.u8()?)?),
            _ => return Err(Error::Malformed),
        }
    }
    if group != Some(GROUP) {
        return Err(Error::Version);
    }
    code.ok_or(Error::Malformed)
}
pub(crate) fn decode_response(bytes: &[u8]) -> Result<Response, Error> {
    let command = decode_header(bytes, true)?;
    let mut d = Decoder::new(&bytes[HEADER..]);
    let count = d.map()?.ok_or(Error::Malformed)?;
    if !(4..=5).contains(&count) {
        return Err(Error::Malformed);
    }
    let (mut link, mut request_id, mut state, mut error, mut version) =
        (None, None, None, None, false);
    for _ in 0..count {
        match d.str()? {
            "v" => read_version(&mut d, &mut version)?,
            "link" if link.is_none() => link = Some(id(&mut d)?),
            "id" if request_id.is_none() => request_id = Some(d.u64()?),
            "state" if state.is_none() => state = Some(read_state(&mut d)?),
            "err" if error.is_none() => error = Some(read_error(&mut d)?),
            _ => return Err(Error::Malformed),
        }
    }
    let (link, request_id) = check_envelope(link, request_id, version, bytes, d.position())?;
    Ok(Response {
        link,
        id: request_id,
        command,
        state: state.ok_or(Error::Malformed)?,
        result: error.map_or(Ok(()), Err),
    })
}
