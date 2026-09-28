//! Explicit development provisioning; production enrollment uses a separate provider.
#[cfg(unix)]
#[path = "development_credentials/provisioning.rs"]
mod provisioning;

#[cfg(unix)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    provisioning::run()
}

#[cfg(not(unix))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    Err("开发凭据生成当前仅支持 Unix 权限；其他平台须先实现私有存储适配".into())
}
