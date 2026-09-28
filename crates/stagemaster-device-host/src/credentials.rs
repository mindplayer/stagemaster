//! Trusted native injection only; no frontend command returns or accepts secret bytes.
use crate::{Problem, ProblemCode};
pub use stagemaster_device_auth::application::Configuration as DevelopmentConfiguration;
use stagemaster_device_auth::application::{CONFIGURATION_BYTES, Role};
use std::{fs::File, io::Read, path::Path};
use zeroize::Zeroizing;

#[cfg(test)]
mod tests;

/// Load an explicit local development file, never a path supplied by a device.
/// # Errors
/// Reject symlinks, nonregular/oversized files, broad Unix permissions and malformed keys.
pub fn read_development_configuration(path: &Path) -> Result<DevelopmentConfiguration, Problem> {
    let problem =
        || Problem::new(ProblemCode::Installation).detail("设备开发凭据无效或不可读取".to_owned());
    let metadata = path.symlink_metadata().map_err(|_| problem())?;
    if !metadata.is_file() || metadata.len() != CONFIGURATION_BYTES as u64 {
        return Err(problem());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o777 != 0o600 {
            return Err(problem());
        }
    }
    let mut file = File::open(path).map_err(|_| problem())?;
    let mut bytes = Zeroizing::new([0; CONFIGURATION_BYTES]);
    file.read_exact(bytes.as_mut()).map_err(|_| problem())?;
    let mut extra = [0; 1];
    if file.read(&mut extra).map_err(|_| problem())? != 0 {
        return Err(problem());
    }
    DevelopmentConfiguration::import(bytes.as_ref(), Role::Controller).map_err(|_| problem())
}
