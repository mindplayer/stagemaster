use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use stagemaster_runtime_host::Action;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub(crate) struct Decimal(pub u64);
impl TryFrom<String> for Decimal {
    type Error = &'static str;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let parsed = value
            .parse::<u64>()
            .map_err(|_| "须为无符号整数的十进制字符串")?;
        if parsed.to_string() != value {
            return Err("整数不能含前导零、符号或空白");
        }
        Ok(Self(parsed))
    }
}
impl From<Decimal> for String {
    fn from(value: Decimal) -> Self {
        value.0.to_string()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Input {
    pub serial: Decimal,
    pub ttl_ms: u64,
    pub command: Command,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub(crate) enum Command {
    Acquire {
        duration_ms: u64,
        takeover: bool,
    },
    Submit {
        expected_revision: Decimal,
        action: Operation,
    },
    Renew {
        duration_ms: u64,
    },
    Release {},
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub(crate) enum Operation {
    Source {
        source: String,
        action: crate::group::wire::Operation,
    },
    Start {
        step: String,
    },
    Pause {},
    Resume {},
    Next {},
    Stop {},
}
impl Operation {
    pub fn action(&self) -> Result<Action, Failure> {
        Ok(match self {
            Self::Source { .. } => return Err(Failure::invalid()),
            Self::Start { step } => Action::Start {
                step: *Uuid::parse_str(step)
                    .map_err(|_| Failure::invalid())?
                    .as_bytes(),
            },
            Self::Pause {} => Action::Pause,
            Self::Resume {} => Action::Resume,
            Self::Next {} => Action::Next,
            Self::Stop {} => Action::Stop,
        })
    }
}
impl Input {
    pub fn validate(&self) -> Result<(), Failure> {
        if self.serial.0 == 0 || !(1..=5000).contains(&self.ttl_ms) {
            return Err(Failure::invalid());
        }
        match &self.command {
            Command::Acquire { duration_ms, .. } | Command::Renew { duration_ms }
                if !(1..=60_000).contains(duration_ms) =>
            {
                Err(Failure::invalid())
            }
            Command::Submit {
                action: Operation::Source { source, action },
                ..
            } => {
                crate::group::wire::identity(source)?;
                action.validate()
            }
            Command::Submit { action, .. } => action.action().map(|_| ()),
            _ => Ok(()),
        }
    }
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct RecordView {
    pub serial: Decimal,
    pub status: &'static str,
    pub outcome: Option<Value>,
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct Failure(pub StatusCode, pub &'static str, pub &'static str);
impl Failure {
    pub fn invalid() -> Self {
        Self(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid",
            "请求格式、参数或长度不正确",
        )
    }
    pub fn busy() -> Self {
        Self(
            StatusCode::SERVICE_UNAVAILABLE,
            "busy",
            "服务正在处理其他请求，请稍后重试",
        )
    }
    pub fn conflict(code: &'static str, message: &'static str) -> Self {
        Self(StatusCode::CONFLICT, code, message)
    }
    pub fn closed() -> Self {
        Self(StatusCode::GONE, "closed", "执行宿主正在关闭")
    }
}
impl IntoResponse for Failure {
    fn into_response(self) -> Response {
        (self.0, Json(json!({"code":self.1,"message":self.2}))).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wire_requires_exact_integers_and_rejects_unknown_authority() {
        for invalid in ["1", r#""01""#, r#""+1""#, r#""18446744073709551616""#] {
            assert!(serde_json::from_str::<Decimal>(invalid).is_err());
        }
        assert_eq!(
            serde_json::from_str::<Decimal>(r#""18446744073709551615""#)
                .unwrap()
                .0,
            u64::MAX
        );
        let mut request = json!({"serial":"1","ttlMs":100,"command":{"kind":"acquire","durationMs":1000,"takeover":false}});
        assert!(
            serde_json::from_value::<Input>(request.clone())
                .unwrap()
                .validate()
                .is_ok()
        );
        request["command"]["principal"] = "caller-controlled".into();
        assert!(serde_json::from_value::<Input>(request).is_err());
        for command in [
            json!({"kind":"release","extra":true}),
            json!({"kind":"submit","expectedRevision":"0","action":{"kind":"stop","extra":true}}),
        ] {
            assert!(
                serde_json::from_value::<Input>(
                    json!({"serial":"1","ttlMs":100,"command":command})
                )
                .is_err()
            );
        }
    }
}
