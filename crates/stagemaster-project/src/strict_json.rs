use crate::MAX_BYTES;
use serde::{
    Deserializer,
    de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor},
};
use serde_json::{Map, Value};
use std::fmt;

pub(super) fn decode(bytes: &[u8]) -> Result<Value, String> {
    if bytes.len() > MAX_BYTES {
        return Err("工程超过 8 MiB 限制".into());
    }
    let input = std::str::from_utf8(bytes).map_err(|_| "工程必须使用 UTF-8 编码")?;
    let mut decoder = serde_json::Deserializer::from_str(input);
    let value = StrictValue(0)
        .deserialize(&mut decoder)
        .map_err(|error| format!("工程 JSON 格式错误：{error}"))?;
    decoder
        .end()
        .map_err(|error| format!("工程 JSON 结尾错误：{error}"))?;
    Ok(value)
}

// Reject duplicate keys before constructing a Value; serde_json's ordinary Value
// reader keeps only the final duplicate, which is unsuitable for project inspection.
struct StrictValue(u8);
impl<'de> DeserializeSeed<'de> for StrictValue {
    type Value = Value;
    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Value, D::Error> {
        if self.0 > 64 {
            return Err(de::Error::custom("JSON 嵌套超过 64 层"));
        }
        deserializer.deserialize_any(self)
    }
}
impl<'de> Visitor<'de> for StrictValue {
    type Value = Value;
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("严格 JSON 值")
    }
    fn visit_bool<E: de::Error>(self, value: bool) -> Result<Value, E> {
        Ok(Value::Bool(value))
    }
    fn visit_i64<E: de::Error>(self, value: i64) -> Result<Value, E> {
        if value.unsigned_abs() > 9_007_199_254_740_991 {
            return Err(E::custom("大整数必须使用字符串"));
        }
        Ok(value.into())
    }
    fn visit_u64<E: de::Error>(self, value: u64) -> Result<Value, E> {
        if value > 9_007_199_254_740_991 {
            return Err(E::custom("大整数必须使用字符串"));
        }
        Ok(value.into())
    }
    fn visit_f64<E: de::Error>(self, value: f64) -> Result<Value, E> {
        if !value.is_finite() || (value.fract() == 0.0 && value.abs() > 9_007_199_254_740_991.0) {
            return Err(E::custom("JSON 数值超出精确范围"));
        }
        // JSON Schema treats 4.0 and 4e0 as integers. Normalize exact safe
        // integers before domain accessors so they cannot become missing values.
        if value.fract() == 0.0 {
            return format!("{value:.0}")
                .parse::<i64>()
                .map(Value::from)
                .map_err(E::custom);
        }
        serde_json::Number::from_f64(value)
            .map(Value::Number)
            .ok_or_else(|| E::custom("JSON 数值不是有限数"))
    }
    fn visit_str<E: de::Error>(self, value: &str) -> Result<Value, E> {
        Ok(Value::String(value.into()))
    }
    fn visit_none<E: de::Error>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }
    fn visit_unit<E: de::Error>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut sequence: A) -> Result<Value, A::Error> {
        let mut values = Vec::new();
        while let Some(value) = sequence.next_element_seed(StrictValue(self.0 + 1))? {
            values.push(value);
        }
        Ok(Value::Array(values))
    }
    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Value, A::Error> {
        let mut values = Map::new();
        while let Some(key) = map.next_key::<String>()? {
            if values.contains_key(&key) {
                return Err(de::Error::custom(format!("JSON 存在重复字段：{key}")));
            }
            values.insert(key, map.next_value_seed(StrictValue(self.0 + 1))?);
        }
        Ok(Value::Object(values))
    }
}
