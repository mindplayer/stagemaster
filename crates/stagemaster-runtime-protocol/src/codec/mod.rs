mod failure;
pub(crate) mod negotiation;
pub(crate) mod request;
pub(crate) mod response;
mod state;
use crate::{Error, Frame, MAX_MESSAGE_BYTES, Text};
use minicbor::{Decoder, Encoder, data::Type, encode::write::Cursor};
pub(crate) type Encode<'a> = Encoder<Cursor<&'a mut [u8]>>;
const PREFIX: &[u8; 5] = b"SMRT\x01";

pub(crate) fn encode(
    kind: u8,
    write: impl FnOnce(&mut Encode<'_>) -> Result<(), Error>,
) -> Result<Frame, Error> {
    let mut frame = Frame {
        bytes: [0; MAX_MESSAGE_BYTES],
        length: 0,
    };
    frame.bytes[..5].copy_from_slice(PREFIX);
    frame.bytes[5] = kind;
    let mut e = Encoder::new(Cursor::new(&mut frame.bytes[8..]));
    write(&mut e)?;
    frame.length = 8 + e.writer().position();
    Ok(frame)
}
pub(crate) fn decode<T>(
    bytes: &[u8],
    kind: u8,
    read: impl FnOnce(&mut Decoder<'_>) -> Result<T, Error>,
) -> Result<T, Error> {
    if !(9..=MAX_MESSAGE_BYTES).contains(&bytes.len()) {
        return Err(Error::Bounds);
    }
    if &bytes[..4] != b"SMRT" || bytes[5] != kind || bytes[6..8] != [0; 2] {
        return Err(Error::Format);
    }
    if bytes[4] != 1 {
        return Err(Error::Version);
    }
    let mut d = Decoder::new(&bytes[8..]);
    let result = read(&mut d)?;
    if d.position() != bytes.len() - 8 {
        return Err(Error::Format);
    }
    Ok(result)
}
pub(crate) fn array(d: &mut Decoder<'_>, count: u64) -> Result<(), Error> {
    if d.array()? == Some(count) {
        Ok(())
    } else {
        Err(Error::Format)
    }
}
pub(crate) fn id(d: &mut Decoder<'_>) -> Result<[u8; 16], Error> {
    let id = d.bytes()?.try_into().map_err(|_| Error::Format)?;
    valid_id(id)?;
    Ok(id)
}
pub(crate) fn valid_id(id: [u8; 16]) -> Result<(), Error> {
    if id == [0; 16] {
        Err(Error::Identity)
    } else {
        Ok(())
    }
}
pub(crate) fn text(d: &mut Decoder<'_>) -> Result<Text, Error> {
    Text::new(d.str()?).map_err(|_| Error::Bounds)
}
pub(crate) fn read_option<T>(
    d: &mut Decoder<'_>,
    read: impl FnOnce(&mut Decoder<'_>) -> Result<T, Error>,
) -> Result<Option<T>, Error> {
    if d.datatype()? == Type::Null {
        d.null()?;
        Ok(None)
    } else {
        read(d).map(Some)
    }
}
pub(crate) fn write_option<T>(
    e: &mut Encode<'_>,
    value: Option<T>,
    write: impl FnOnce(&mut Encode<'_>, T) -> Result<(), Error>,
) -> Result<(), Error> {
    if let Some(value) = value {
        write(e, value)?;
    } else {
        e.null()?;
    }
    Ok(())
}
