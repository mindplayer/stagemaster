use crate::{DevelopmentConfiguration, Problem, ProblemCode, Service, Snapshot, Transport};
use stagemaster_device_auth::application::Scope;
use stagemaster_runtime_protocol::Access;

/// Native configuration expectation, not a grant and never deserialized from the frontend.
#[derive(Default)]
pub struct ExpectedAccess(Option<Access>);
impl ExpectedAccess {
    #[must_use]
    pub fn from_configuration(config: &DevelopmentConfiguration) -> Self {
        let p = config.permissions();
        Self(p.contains(Scope::Observe).then_some(Access {
            observe: true,
            control: p.contains(Scope::Control),
            installation: p.contains(Scope::Installation),
        }))
    }
    pub(super) fn connect<B: Transport>(
        &self,
        host: &Service<B>,
        epoch: u32,
        id: String,
    ) -> Result<Snapshot, Problem> {
        host.connect_runtime(
            epoch,
            id,
            self.0.ok_or_else(|| {
                Problem::new(ProblemCode::Runtime).detail("本机尚未配置此设备的运行访问凭据".into())
            })?,
        )
    }
}
