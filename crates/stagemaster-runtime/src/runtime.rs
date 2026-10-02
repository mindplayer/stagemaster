mod control_api;
use crate::{
    Action, Code, FrameInfo, Id, Instance, Maintenance, Mode, Permission, PermissionAction,
    PlaybackPolicy, ProgramKey, Quiescence, State, Status, authority::Authority, package_error,
};
use alloc::vec::Vec;
use stagemaster_install::Installed;
use stagemaster_package::{Entry, MAX_LOADER_BYTES, Output, Program, ReadAt, StepLabel};
use stagemaster_playback::Player;

struct Loaded {
    key: ProgramKey,
    player: Player,
    output: Output,
    labels: Vec<StepLabel>,
}
#[derive(Clone, Copy)]
enum Phase {
    Operation,
    Quiescing(Quiescence),
    Maintenance(Maintenance),
}
#[derive(Debug, PartialEq, Eq)]
pub enum MaintenanceError<E> {
    State(Code),
    Load(E),
}
impl<E: core::fmt::Display> core::fmt::Display for MaintenanceError<E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::State(code) => code.fmt(f),
            Self::Load(error) => write!(f, "维护恢复失败：{error}"),
        }
    }
}
impl<E: core::fmt::Debug + core::fmt::Display> core::error::Error for MaintenanceError<E> {}

pub struct Runtime<R, P> {
    boot: Id,
    revision: u64,
    last_ms: u64,
    instance_counter: u64,
    loader_budget: usize,
    phase: Phase,
    source: Option<Installed<R>>,
    selected: Option<ProgramKey>,
    loaded: Option<Loaded>,
    instance: Option<Instance>,
    control: Authority<Action, State>,
    policy: P,
}
impl<R: ReadAt, P: PlaybackPolicy> Runtime<R, P> {
    /// Boot identities must be fresh per runtime lifetime. Start with no output and no controller.
    /// # Errors
    /// Reject a zero identity or a budget exceeding the supported package profile.
    pub fn new(boot: Id, now_ms: u64, loader_budget: usize, policy: P) -> Result<Self, Code> {
        if boot == [0; 16] {
            return Err(Code::Identity);
        }
        if loader_budget > MAX_LOADER_BYTES {
            return Err(Code::Budget);
        }
        Ok(Self {
            boot,
            revision: 0,
            last_ms: now_ms,
            instance_counter: 0,
            loader_budget,
            phase: Phase::Quiescing(Quiescence { boot, revision: 0 }),
            source: None,
            selected: None,
            loaded: None,
            instance: None,
            control: Authority::new(boot, now_ms)?,
            policy,
        })
    }
    #[must_use]
    pub fn state(&self) -> State {
        State {
            boot: self.boot,
            revision: self.revision,
            observed_ms: self.last_ms,
            mode: match self.phase {
                Phase::Operation => Mode::Operation,
                Phase::Quiescing(_) => Mode::Quiescing,
                Phase::Maintenance(_) => Mode::Maintenance,
            },
            bound_package: self.source.as_ref().map(Installed::commit),
            selected: self.selected,
            loaded: self.loaded.as_ref().map(|p| p.key),
            status: self.loaded.as_ref().map(|p| p.player.status()),
            instance: self.instance,
            step: self
                .loaded
                .as_ref()
                .and_then(|p| p.player.index().map(|i| p.labels[i].id)),
            elapsed_ms: self.loaded.as_ref().map_or(0, |p| p.player.elapsed_ms()),
            owner: self.control.owner(),
        }
    }
    #[must_use]
    pub fn catalog(&self) -> &[Entry] {
        self.source.as_ref().map_or(&[], |s| s.archive().entries())
    }
    #[must_use]
    pub fn steps(&self) -> &[StepLabel] {
        self.loaded.as_ref().map_or(&[], |p| p.labels.as_slice())
    }
    /// Trusted policy management, not a peer-writable authorization flag.
    pub fn policy_mut(&mut self) -> &mut P {
        &mut self.policy
    }
    #[must_use]
    pub const fn quiescence_request(&self) -> Option<Quiescence> {
        match self.phase {
            Phase::Quiescing(request) => Some(request),
            _ => None,
        }
    }
    /// Continuous scheduling is independent of control connections. No package reads or allocation.
    /// # Errors
    /// A backwards timestamp leaves runtime, control and player state unchanged.
    pub fn tick(&mut self, now_ms: u64) -> Result<(), Code> {
        if now_ms < self.last_ms {
            return Err(Code::Clock);
        }
        if let Some(loaded) = &mut self.loaded {
            loaded.player.advance(now_ms).map_err(|_| Code::Playback)?;
        }
        self.last_ms = now_ms;
        self.control.tick(now_ms)?;
        Ok(())
    }
    fn next_revision(&self) -> Result<u64, Code> {
        self.revision.checked_add(1).ok_or(Code::Exhausted)
    }
    fn apply(&mut self, action: Action, next: u64) -> Result<(), Code> {
        if action == Action::CancelMaintenance {
            if !matches!(self.phase, Phase::Quiescing(_)) {
                return Err(Code::Mode);
            }
            self.phase = Phase::Operation;
            return Ok(());
        }
        if !matches!(self.phase, Phase::Operation) {
            return Err(Code::Mode);
        }
        match action {
            Action::Select(key) => {
                if self.source.is_none() {
                    return Err(Code::Empty);
                }
                if !self
                    .catalog()
                    .iter()
                    .any(|e| e.kind == key.kind && e.id == key.id)
                {
                    return Err(Code::Selection);
                }
                self.selected = Some(key);
                Ok(())
            }
            Action::Load => self.load(),
            Action::Start { step } => self.start(step),
            Action::Pause | Action::Resume | Action::Next | Action::Stop => {
                self.control_player(action)
            }
            Action::BeginMaintenance => {
                if self.instance.is_some()
                    || self
                        .loaded
                        .as_ref()
                        .is_some_and(|p| p.player.status() != Status::Idle)
                {
                    return Err(Code::Busy);
                }
                self.phase = Phase::Quiescing(Quiescence {
                    boot: self.boot,
                    revision: next,
                });
                Ok(())
            }
            Action::CancelMaintenance => Err(Code::Mode),
        }
    }
    fn load(&mut self) -> Result<(), Code> {
        let source = self.source.as_ref().ok_or(Code::Empty)?;
        let selected = self.selected.ok_or(Code::Selection)?;
        if self
            .loaded
            .as_ref()
            .is_some_and(|p| p.player.status() != Status::Idle)
        {
            return Err(Code::Busy);
        }
        let (index, entry) = source
            .archive()
            .entries()
            .iter()
            .enumerate()
            .find(|(_, e)| e.kind == selected.kind && e.id == selected.id)
            .ok_or(Code::Selection)?;
        if entry.usage.loader_peak_bytes > self.loader_budget {
            return Err(Code::Budget);
        }
        if self.loaded.as_ref().is_some_and(|p| p.key == selected) {
            return Ok(());
        }
        self.loaded = None;
        self.instance = None;
        let Program {
            plan,
            output,
            labels,
        } = source.load(index).map_err(|e| package_error(&e))?;
        let player = Player::try_new(plan, self.last_ms).map_err(|_| Code::Allocation)?;
        self.loaded = Some(Loaded {
            key: selected,
            player,
            output,
            labels,
        });
        Ok(())
    }
    fn start(&mut self, step: Id) -> Result<(), Code> {
        let loaded = self.loaded.as_ref().ok_or(Code::NotLoaded)?;
        if self.selected != Some(loaded.key) {
            return Err(Code::NotLoaded);
        }
        let index = loaded
            .labels
            .iter()
            .position(|s| s.id == step)
            .ok_or(Code::Step)?;
        let number = self
            .instance_counter
            .checked_add(1)
            .ok_or(Code::Exhausted)?;
        let instance = Instance {
            boot: self.boot,
            number,
        };
        self.permission(PermissionAction::Start, instance)?;
        self.loaded
            .as_mut()
            .ok_or(Code::NotLoaded)?
            .player
            .execute(index, self.last_ms)
            .map_err(|_| Code::Playback)?;
        self.instance_counter = number;
        self.instance = Some(instance);
        Ok(())
    }
    fn permission(&mut self, action: PermissionAction, instance: Instance) -> Result<(), Code> {
        let package = self.source.as_ref().ok_or(Code::Empty)?.commit().identity;
        let program = self.loaded.as_ref().ok_or(Code::NotLoaded)?.key;
        self.policy
            .authorize(Permission {
                package,
                program,
                instance,
                action,
                now_ms: self.last_ms,
            })
            .map_err(Code::Permission)
    }
    fn control_player(&mut self, action: Action) -> Result<(), Code> {
        if action == Action::Stop {
            if let Some(loaded) = &mut self.loaded {
                loaded
                    .player
                    .stop(self.last_ms)
                    .map_err(|_| Code::Playback)?;
            }
            self.instance = None;
            return Ok(());
        }
        let instance = self.instance.ok_or(Code::State)?;
        let loaded = self.loaded.as_ref().ok_or(Code::NotLoaded)?;
        match action {
            Action::Pause if loaded.player.status() == Status::Running => {}
            Action::Resume if loaded.player.status() == Status::Paused => {
                self.permission(PermissionAction::Resume, instance)?;
            }
            Action::Next if loaded.player.status() == Status::Running => {
                if !loaded.player.can_next() {
                    return Err(Code::Step);
                }
                self.permission(PermissionAction::Next, instance)?;
            }
            _ => return Err(Code::State),
        }
        let player = &mut self.loaded.as_mut().ok_or(Code::NotLoaded)?.player;
        match action {
            Action::Pause => player.pause(self.last_ms),
            Action::Resume => player.resume(self.last_ms),
            Action::Next => player.next(self.last_ms),
            _ => return Err(Code::State),
        }
        .map_err(|_| Code::Playback)
    }
    /// A generated logical frame is not an acknowledgement that a device transmitted it.
    /// # Errors
    /// Invalid core mapping is reported without clearing the caller's buffer.
    pub fn render(&self, target: &mut [u8; 512]) -> Result<Option<FrameInfo>, Code> {
        if !matches!(self.phase, Phase::Operation) {
            return Ok(None);
        }
        let Some(loaded) = &self.loaded else {
            return Ok(None);
        };
        loaded
            .output
            .render(loaded.player.values(), target)
            .map_err(|e| package_error(&e))?;
        Ok(Some(FrameInfo {
            boot: self.boot,
            revision: self.revision,
            sampled_ms: self.last_ms,
            universe: loaded.output.universe,
            program: loaded.key,
            instance: self.instance,
        }))
    }
    /// Trusted output-host acknowledgement: all physical output and queued frames are quiescent.
    /// Never route a network field directly to this method.
    /// # Errors
    /// Old/cancelled confirmations cannot grant an installation window.
    pub fn confirm_quiescent(
        &mut self,
        request: Quiescence,
        now_ms: u64,
    ) -> Result<Maintenance, Code> {
        self.tick(now_ms)?;
        if !matches!(self.phase,Phase::Quiescing(current) if current==request) {
            return Err(Code::Mode);
        }
        let revision = self.next_revision()?;
        self.loaded = None;
        self.source = None;
        self.selected = None;
        self.instance = None;
        let permit = Maintenance {
            boot: self.boot,
            revision,
        };
        self.phase = Phase::Maintenance(permit);
        self.revision = revision;
        Ok(permit)
    }
    fn maintenance(&self, permit: Maintenance) -> Result<(), Code> {
        if matches!(self.phase,Phase::Maintenance(current) if current==permit) {
            Ok(())
        } else {
            Err(Code::Mode)
        }
    }
    /// Run one serialized installation operation. The host must keep all actual writes behind this gate.
    /// # Errors
    /// Do not call the closure unless this exact maintenance window is still active.
    pub fn with_maintenance<T>(
        &mut self,
        permit: Maintenance,
        work: impl FnOnce() -> T,
    ) -> Result<T, Code> {
        self.maintenance(permit)?;
        Ok(work())
    }
    /// Reopen an installed snapshot only after old plans/catalogues/leases were released.
    /// An explicit None leaves an empty device; a loader error preserves maintenance for retry.
    /// # Errors
    /// Reject expired windows, backwards time, oversized catalogues, or propagate loader failures.
    pub fn finish_maintenance<E>(
        &mut self,
        permit: Maintenance,
        now_ms: u64,
        loader: impl FnOnce() -> Result<Option<Installed<R>>, E>,
    ) -> Result<State, MaintenanceError<E>> {
        self.tick(now_ms).map_err(MaintenanceError::State)?;
        self.maintenance(permit).map_err(MaintenanceError::State)?;
        let revision = self.next_revision().map_err(MaintenanceError::State)?;
        let source = loader().map_err(MaintenanceError::Load)?;
        if source
            .as_ref()
            .is_some_and(|s| s.archive().catalog_resident_bytes() > self.loader_budget)
        {
            return Err(MaintenanceError::State(Code::Budget));
        }
        self.source = source;
        self.phase = Phase::Operation;
        self.revision = revision;
        Ok(self.state())
    }
}
