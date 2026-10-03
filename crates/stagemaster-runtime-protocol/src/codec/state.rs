use super::{
    Encode, array, id, read_option,
    request::{key, read_key},
    valid_id, write_option,
};
use crate::{Error, Observation};
use minicbor::Decoder;
use stagemaster_install::{Commit, Identity, Slot};
use stagemaster_runtime::{Instance, Lease, Mode, Origin, Owner, State, Status};

fn validate(s: State) -> Result<(), Error> {
    valid_id(s.boot)?;
    if s.loaded.is_some() != s.status.is_some()
        || (s.bound_package.is_none() && (s.selected.is_some() || s.loaded.is_some()))
        || (s.loaded.is_none() && (s.instance.is_some() || s.step.is_some() || s.elapsed_ms != 0))
    {
        return Err(Error::Format);
    }
    if let Some(i) = s.instance
        && (i.boot != s.boot || i.number == 0)
    {
        return Err(Error::Identity);
    }
    if let Some(o) = s.owner {
        valid_id(o.principal)?;
        if o.lease.boot != s.boot || o.lease.epoch == 0 || o.expires_ms <= s.observed_ms {
            return Err(Error::Identity);
        }
    }
    Ok(())
}
fn commit(e: &mut Encode<'_>, c: Commit) -> Result<(), Error> {
    c.encode().map_err(|_| Error::Format)?;
    e.array(4)?
        .u8(u8::from(c.slot == Slot::B))?
        .u64(c.generation)?
        .u32(u32::try_from(c.identity.bytes).map_err(|_| Error::Bounds)?)?
        .bytes(&c.identity.digest)?;
    Ok(())
}
fn read_commit(d: &mut Decoder<'_>) -> Result<Commit, Error> {
    array(d, 4)?;
    let slot = match d.u8()? {
        0 => Slot::A,
        1 => Slot::B,
        _ => return Err(Error::Format),
    };
    let generation = d.u64()?;
    let bytes = usize::try_from(d.u32()?).map_err(|_| Error::Bounds)?;
    let digest = d.bytes()?.try_into().map_err(|_| Error::Format)?;
    let c = Commit {
        slot,
        generation,
        identity: Identity { bytes, digest },
    };
    c.encode().map_err(|_| Error::Format)?;
    Ok(c)
}
fn owner(e: &mut Encode<'_>, o: Owner) -> Result<(), Error> {
    e.array(5)?
        .u64(o.lease.epoch)?
        .bytes(&o.principal)?
        .u8(match o.origin {
            Origin::Panel => 0,
            Origin::Remote => 1,
        })?
        .u64(o.expires_ms)?
        .u64(o.serial)?;
    Ok(())
}
fn read_owner(d: &mut Decoder<'_>, boot: [u8; 16]) -> Result<Owner, Error> {
    array(d, 5)?;
    let epoch = d.u64()?;
    let principal = id(d)?;
    let origin = match d.u8()? {
        0 => Origin::Panel,
        1 => Origin::Remote,
        _ => return Err(Error::Format),
    };
    Ok(Owner {
        lease: Lease { boot, epoch },
        principal,
        origin,
        expires_ms: d.u64()?,
        serial: d.u64()?,
    })
}
pub(crate) fn write(e: &mut Encode<'_>, s: State) -> Result<(), Error> {
    validate(s)?;
    e.array(9)?.u8(match s.mode {
        Mode::Operation => 0,
        Mode::Quiescing => 1,
        Mode::Maintenance => 2,
    })?;
    write_option(e, s.bound_package, commit)?;
    write_option(e, s.selected, key)?;
    write_option(e, s.loaded, key)?;
    write_option(e, s.status, |e, status| {
        e.u8(match status {
            Status::Idle => 0,
            Status::Running => 1,
            Status::Paused => 2,
            Status::Finished => 3,
        })?;
        Ok(())
    })?;
    write_option(e, s.instance, |e, i| {
        e.u64(i.number)?;
        Ok(())
    })?;
    write_option(e, s.step, |e, id| {
        valid_id(id)?;
        e.bytes(&id)?;
        Ok(())
    })?;
    e.u64(s.elapsed_ms)?;
    write_option(e, s.owner, owner)
}
pub(crate) fn read(d: &mut Decoder<'_>, observed: Observation) -> Result<State, Error> {
    array(d, 9)?;
    let mode = match d.u8()? {
        0 => Mode::Operation,
        1 => Mode::Quiescing,
        2 => Mode::Maintenance,
        _ => return Err(Error::Format),
    };
    let bound_package = read_option(d, read_commit)?;
    let selected = read_option(d, read_key)?;
    let loaded = read_option(d, read_key)?;
    let status = read_option(d, |d| match d.u8()? {
        0 => Ok(Status::Idle),
        1 => Ok(Status::Running),
        2 => Ok(Status::Paused),
        3 => Ok(Status::Finished),
        _ => Err(Error::Format),
    })?;
    let instance = read_option(d, |d| {
        Ok(Instance {
            boot: observed.boot,
            number: d.u64()?,
        })
    })?;
    let step = read_option(d, id)?;
    let elapsed_ms = d.u64()?;
    let owner = read_option(d, |d| read_owner(d, observed.boot))?;
    let s = State {
        boot: observed.boot,
        revision: observed.revision,
        observed_ms: observed.observed_ms,
        mode,
        bound_package,
        selected,
        loaded,
        status,
        instance,
        step,
        elapsed_ms,
        owner,
    };
    validate(s)?;
    Ok(s)
}
