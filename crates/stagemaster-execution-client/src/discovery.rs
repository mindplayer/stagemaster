use serde::Deserialize;
use std::{fs, io::Read, path::Path};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Discovery {
    pub protocol: u8,
    pub host_id: String,
    pub url: String,
    pub read_token: String,
    pub control_token: String,
}
impl Discovery {
    pub fn read(path: &Path) -> Result<Self, String> {
        private(path.parent().ok_or("缺少后台运行目录")?, true)?;
        private(path, false)?;
        let mut bytes = Vec::new();
        fs::File::open(path)
            .map_err(|_| "无法读取后台连接信息")?
            .take(4097)
            .read_to_end(&mut bytes)
            .map_err(|_| "无法读取后台连接信息")?;
        if bytes.len() > 4096 {
            return Err("后台连接信息超限".into());
        }
        let value: Self = serde_json::from_slice(&bytes).map_err(|_| "后台连接信息格式错误")?;
        value.validate()?;
        Ok(value)
    }
    fn validate(&self) -> Result<(), String> {
        let url = reqwest::Url::parse(&self.url).map_err(|_| "后台地址无效")?;
        let id = uuid::Uuid::parse_str(&self.host_id).map_err(|_| "后台身份无效")?;
        if self.protocol != 2
            || id.is_nil()
            || id.to_string() != self.host_id
            || url.scheme() != "http"
            || url.host_str() != Some("127.0.0.1")
            || !matches!(url.port(), Some(1..=65535))
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
            || url.path() != format!("/v2/{id}")
            || url.as_str() != self.url
            || [&self.read_token, &self.control_token].iter().any(|t| {
                t.len() != 64
                    || !t
                        .bytes()
                        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            })
        {
            return Err("后台连接信息不符合本机协议".into());
        }
        Ok(())
    }
}
pub(crate) fn private(path: &Path, directory: bool) -> Result<(), String> {
    let info = fs::symlink_metadata(path).map_err(|_| "后台运行记录不存在")?;
    if info.file_type().is_symlink()
        || (directory && !info.is_dir())
        || (!directory && !info.is_file())
    {
        return Err("后台运行记录类型错误".into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if info.permissions().mode() & 0o077 != 0 {
            return Err("后台运行记录必须仅当前用户可访问".into());
        }
    }
    #[cfg(not(unix))]
    return Err("当前系统的后台连接权限尚未验证".into());
    Ok(())
}
