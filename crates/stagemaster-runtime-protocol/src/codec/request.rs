use super::{Encode, array, id, valid_id};
use crate::{Error, Operation, Request};
use minicbor::Decoder;
use stagemaster_package::Kind;
use stagemaster_runtime::{Action, ProgramKey};

pub(crate) fn key(e: &mut Encode<'_>, key: ProgramKey) -> Result<(), Error> {
    valid_id(key.id)?;
    e.array(2)?
        .u8(match key.kind {
            Kind::Scene => 0,
            Kind::Sequence => 1,
        })?
        .bytes(&key.id)?;
    Ok(())
}
pub(crate) fn read_key(d: &mut Decoder<'_>) -> Result<ProgramKey, Error> {
    array(d, 2)?;
    let kind = match d.u8()? {
        0 => Kind::Scene,
        1 => Kind::Sequence,
        _ => return Err(Error::Format),
    };
    Ok(ProgramKey { kind, id: id(d)? })
}
pub(crate) fn write(e: &mut Encode<'_>, r: Request) -> Result<(), Error> {
    valid_id(r.session)?;
    if r.id == 0 {
        return Err(Error::Identity);
    }
    e.array(4)?
        .bytes(&r.session)?
        .u64(r.id)?
        .u64(r.expected_revision)?;
    match r.operation {
        Operation::Status => {
            e.array(1)?.u8(0)?;
        }
        Operation::Catalog { index } => {
            e.array(2)?.u8(1)?.u16(index)?;
        }
        Operation::Step { index } => {
            e.array(2)?.u8(2)?.u16(index)?;
        }
        Operation::Acquire {
            duration_ms,
            takeover,
        } => {
            e.array(3)?.u8(3)?.u64(duration_ms)?.bool(takeover)?;
        }
        Operation::Renew { duration_ms } => {
            e.array(2)?.u8(4)?.u64(duration_ms)?;
        }
        Operation::Release => {
            e.array(1)?.u8(5)?;
        }
        Operation::FinishMaintenance => {
            e.array(1)?.u8(6)?;
        }
        Operation::Apply(Action::Select(value)) => {
            e.array(2)?.u8(7)?;
            key(e, value)?;
        }
        Operation::Apply(Action::Start { step }) => {
            valid_id(step)?;
            e.array(2)?.u8(9)?.bytes(&step)?;
        }
        Operation::Apply(action) => {
            e.array(1)?.u8(match action {
                Action::Load => 8,
                Action::Pause => 10,
                Action::Resume => 11,
                Action::Next => 12,
                Action::Stop => 13,
                Action::BeginMaintenance => 14,
                Action::CancelMaintenance => 15,
                Action::Select(_) | Action::Start { .. } => return Err(Error::Format),
            })?;
        }
    }
    Ok(())
}
pub(crate) fn read(d: &mut Decoder<'_>) -> Result<Request, Error> {
    array(d, 4)?;
    let session = id(d)?;
    let request_id = d.u64()?;
    if request_id == 0 {
        return Err(Error::Identity);
    }
    let expected_revision = d.u64()?;
    let count = d.array()?.ok_or(Error::Format)?;
    let tag = d.u8()?;
    let expected = match tag {
        1 | 2 | 4 | 7 | 9 => 2,
        3 => 3,
        0 | 5 | 6 | 8 | 10..=15 => 1,
        _ => return Err(Error::Format),
    };
    if count != expected {
        return Err(Error::Format);
    }
    let operation = match tag {
        0 => Operation::Status,
        1 => Operation::Catalog { index: d.u16()? },
        2 => Operation::Step { index: d.u16()? },
        3 => Operation::Acquire {
            duration_ms: d.u64()?,
            takeover: d.bool()?,
        },
        4 => Operation::Renew {
            duration_ms: d.u64()?,
        },
        5 => Operation::Release,
        6 => Operation::FinishMaintenance,
        7 => Operation::Apply(Action::Select(read_key(d)?)),
        8 => Operation::Apply(Action::Load),
        9 => Operation::Apply(Action::Start { step: id(d)? }),
        10 => Operation::Apply(Action::Pause),
        11 => Operation::Apply(Action::Resume),
        12 => Operation::Apply(Action::Next),
        13 => Operation::Apply(Action::Stop),
        14 => Operation::Apply(Action::BeginMaintenance),
        15 => Operation::Apply(Action::CancelMaintenance),
        _ => return Err(Error::Format),
    };
    Ok(Request {
        session,
        id: request_id,
        expected_revision,
        operation,
    })
}
