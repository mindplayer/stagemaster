use super::{
    Encode, array, failure, id, read_option, request, state, text, valid_id, write_option,
};
use crate::{Body, Error, Observation, Program, Response, Step};
use minicbor::Decoder;

fn program(e: &mut Encode<'_>, p: &Program) -> Result<(), Error> {
    e.array(3)?;
    request::key(e, p.key)?;
    e.str(p.name.as_str())?.u32(p.loader_bytes)?;
    Ok(())
}
fn read_program(d: &mut Decoder<'_>) -> Result<Program, Error> {
    array(d, 3)?;
    Ok(Program {
        key: request::read_key(d)?,
        name: text(d)?,
        loader_bytes: d.u32()?,
    })
}
fn step(e: &mut Encode<'_>, s: &Step) -> Result<(), Error> {
    valid_id(s.id)?;
    e.array(3)?
        .bytes(&s.id)?
        .str(s.name.as_str())?
        .str(s.number.as_str())?;
    Ok(())
}
fn read_step(d: &mut Decoder<'_>) -> Result<Step, Error> {
    array(d, 3)?;
    Ok(Step {
        id: id(d)?,
        name: text(d)?,
        number: text(d)?,
    })
}
pub(crate) fn write(e: &mut Encode<'_>, r: &Response) -> Result<(), Error> {
    r.validate()?;
    e.array(3)?;
    request::write(e, r.request)?;
    let o = r.observed;
    e.array(5)?
        .bytes(&o.boot)?
        .u64(o.revision)?
        .u64(o.observed_ms)?
        .u16(o.program_count)?
        .u16(o.step_count)?;
    match &r.body {
        Body::State {
            state: value,
            result,
        } => {
            e.array(if result.is_ok() { 2 } else { 3 })?
                .u8(u8::from(result.is_err()))?;
            state::write(e, *value)?;
            if let Err(error) = result {
                e.u8(failure::code(*error))?;
            }
        }
        Body::Program(value) => {
            e.array(2)?.u8(2)?;
            write_option(e, value.as_ref(), program)?;
        }
        Body::Step(value) => {
            e.array(2)?.u8(3)?;
            write_option(e, value.as_ref(), step)?;
        }
    }
    Ok(())
}
pub(crate) fn read(d: &mut Decoder<'_>) -> Result<Response, Error> {
    array(d, 3)?;
    let request = request::read(d)?;
    array(d, 5)?;
    let observed = Observation {
        boot: id(d)?,
        revision: d.u64()?,
        observed_ms: d.u64()?,
        program_count: d.u16()?,
        step_count: d.u16()?,
    };
    let count = d.array()?.ok_or(Error::Format)?;
    let tag = d.u8()?;
    if count != (if tag == 1 { 3 } else { 2 }) {
        return Err(Error::Format);
    }
    let body = match tag {
        0 | 1 => Body::State {
            state: state::read(d, observed)?,
            result: if tag == 0 {
                Ok(())
            } else {
                Err(failure::read(d.u8()?)?)
            },
        },
        2 => Body::Program(read_option(d, read_program)?),
        3 => Body::Step(read_option(d, read_step)?),
        _ => return Err(Error::Format),
    };
    let r = Response {
        request,
        observed,
        body,
    };
    r.validate()?;
    Ok(r)
}
