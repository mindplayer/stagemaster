use super::{Encode, array, id};
use crate::{Access, Error, Offer, Peer, Ready};
use minicbor::Decoder;

pub(crate) fn offer(e: &mut Encode<'_>, v: Offer) -> Result<(), Error> {
    v.validate()?;
    e.array(3)?
        .u16(v.min_version)?
        .u16(v.max_version)?
        .u16(v.message_bytes)?;
    Ok(())
}
pub(crate) fn read_offer(d: &mut Decoder<'_>) -> Result<Offer, Error> {
    array(d, 3)?;
    let value = Offer {
        min_version: d.u16()?,
        max_version: d.u16()?,
        message_bytes: d.u16()?,
    };
    value.validate()?;
    Ok(value)
}
pub(crate) fn ready(e: &mut Encode<'_>, v: Ready) -> Result<(), Error> {
    v.validate()?;
    let access = u8::from(v.access.installation)
        | (u8::from(v.access.observe) << 1)
        | (u8::from(v.access.control) << 2);
    e.array(11)?
        .bytes(&v.peer.device)?
        .bytes(&v.peer.boot)?
        .u64(v.peer.connection)?
        .bytes(&v.peer.session)?
        .bytes(&v.peer.principal)?
        .u64(v.peer.permission_revision)?
        .u16(v.version)?
        .u16(v.message_bytes)?
        .u8(access)?
        .u32(v.remaining_ms)?
        .u32(6000)?;
    Ok(())
}
pub(crate) fn read_ready(d: &mut Decoder<'_>) -> Result<Ready, Error> {
    array(d, 11)?;
    let peer = Peer {
        device: id(d)?,
        boot: id(d)?,
        connection: d.u64()?,
        session: id(d)?,
        principal: id(d)?,
        permission_revision: d.u64()?,
    };
    let version = d.u16()?;
    let message_bytes = d.u16()?;
    let access = d.u8()?;
    if access & !7 != 0 {
        return Err(Error::Format);
    }
    let value = Ready {
        peer,
        version,
        message_bytes,
        access: Access {
            installation: access & 1 != 0,
            observe: access & 2 != 0,
            control: access & 4 != 0,
        },
        remaining_ms: d.u32()?,
    };
    if d.u32()? != 6000 {
        return Err(Error::Format);
    }
    value.validate()?;
    Ok(value)
}
