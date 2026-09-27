use crate::{Error, Id, MAX_TEXT_BYTES, reserve};
use alloc::vec::Vec;
use minicbor::{Decoder, Encoder};

pub(crate) struct Writer {
    pub bytes: Vec<u8>,
    limit: usize,
}
impl Writer {
    pub fn new(limit: usize) -> Self {
        Self {
            bytes: Vec::new(),
            limit,
        }
    }
}
impl minicbor::encode::Write for Writer {
    type Error = Error;
    fn write_all(&mut self, bytes: &[u8]) -> Result<(), Error> {
        let len = self
            .bytes
            .len()
            .checked_add(bytes.len())
            .ok_or(Error::Limit("编码长度"))?;
        if len > self.limit {
            return Err(Error::Limit("编码块大小"));
        }
        self.bytes
            .try_reserve(bytes.len())
            .map_err(|_| Error::Allocation)?;
        self.bytes.extend_from_slice(bytes);
        Ok(())
    }
}
pub(crate) type Encode = Encoder<Writer>;
pub(crate) fn encoder(limit: usize) -> Encode {
    Encoder::new(Writer::new(limit))
}
pub(crate) fn text(value: &str) -> Result<&str, Error> {
    if value.is_empty() || value.len() > MAX_TEXT_BYTES {
        return Err(Error::Limit("名称／编号须为 1–512 个 UTF-8 字节"));
    }
    if value.chars().any(char::is_control) {
        return Err(Error::Invalid("名称／编号含控制字符"));
    }
    Ok(value)
}
pub(crate) fn read_text<'a>(d: &mut Decoder<'a>) -> Result<&'a str, Error> {
    text(d.str()?)
}
pub(crate) fn count(d: &mut Decoder<'_>, max: usize) -> Result<usize, Error> {
    let count = d.array()?.ok_or(Error::Invalid("不允许无限长数组"))?;
    let count = usize::try_from(count).map_err(|_| Error::Limit("数组长度"))?;
    if count > max {
        return Err(Error::Limit("数组元素数量"));
    }
    Ok(count)
}
pub(crate) fn array(d: &mut Decoder<'_>, len: usize) -> Result<(), Error> {
    if count(d, len)? != len {
        return Err(Error::Invalid("数组字段数量"));
    }
    Ok(())
}
pub(crate) fn id(d: &mut Decoder<'_>) -> Result<Id, Error> {
    let id: Id = d
        .bytes()?
        .try_into()
        .map_err(|_| Error::Invalid("标识长度"))?;
    if id == [0; 16] {
        return Err(Error::Invalid("标识不能为空"));
    }
    Ok(id)
}
pub(crate) fn digest(d: &mut Decoder<'_>) -> Result<[u8; 32], Error> {
    d.bytes()?
        .try_into()
        .map_err(|_| Error::Invalid("摘要长度"))
}
pub(crate) fn end(d: &Decoder<'_>, input: &[u8]) -> Result<(), Error> {
    if d.position() != input.len() {
        return Err(Error::Invalid("块尾有额外数据"));
    }
    Ok(())
}
pub(crate) fn values(d: &mut Decoder<'_>, n: usize, build: bool) -> Result<Vec<u16>, Error> {
    array(d, n)?;
    let mut values = reserve(if build { n } else { 0 })?;
    for _ in 0..n {
        let value = d.u16()?;
        if build {
            values.push(value);
        }
    }
    Ok(values)
}
pub(crate) fn write_values(e: &mut Encode, values: &[u16]) -> Result<(), Error> {
    e.array(values.len() as u64)?;
    for &v in values {
        e.u16(v)?;
    }
    Ok(())
}
