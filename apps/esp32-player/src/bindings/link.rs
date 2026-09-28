use super::{Action, Client, keys};
use core::sync::atomic::Ordering;
use stagemaster_device_auth::authority::{Authority, Connection, Origin};
use stagemaster_device_auth::{LocalIdentity, MAX_BINDINGS, Vault};
use trouble_host::prelude::*;

pub struct Link {
    client: Client,
    authority: Option<Authority>,
    connection: Option<Connection>,
    granted: bool,
    pairing: bool,
    pending: Option<Vault>,
}
fn now() -> u64 {
    embassy_time::Instant::now().as_millis()
}
impl Link {
    pub async fn start() -> Self {
        let mut client = Client::new();
        let authority = super::startup::recover(&mut client)
            .await
            .map(|v| Authority::new(v, now()));
        Self {
            client,
            authority,
            connection: None,
            granted: false,
            pairing: false,
            pending: None,
        }
    }
    pub fn local(&self) -> Option<&LocalIdentity> {
        self.authority.as_ref()?.vault().map(Vault::local)
    }
    pub fn open_local_test_window(&mut self) {
        #[cfg(feature = "binding-local-test")]
        if let Some(authority) = &mut self.authority
            && authority.open_pairing(now()).is_ok()
        {
            esp_println::println!("本地绑定验收窗口已开启：90 秒，最多 3 次尝试");
        }
    }
    pub fn connect(&mut self) {
        self.granted = false;
        self.pairing = false;
        self.connection = self
            .authority
            .as_mut()
            .and_then(|a| a.connect(super::random(), now()).ok());
    }
    pub fn expired(&mut self) -> bool {
        let Some(authority) = &mut self.authority else {
            return false;
        };
        match authority.poll(now()) {
            Ok(None) => false,
            Ok(Some(_)) => self.granted || self.pairing,
            Err(_) => true,
        }
    }
    pub fn admit(&mut self) -> bool {
        self.pairing = match (&mut self.authority, self.connection) {
            (Some(a), Some(c)) => a.begin_pairing(c, now()).is_ok(),
            _ => false,
        };
        self.pairing
    }
    pub fn resumed(&mut self, bond: &BondInformation, level: SecurityLevel) -> bool {
        let Some(evidence) = keys::evidence(bond, Origin::Resumed, level) else {
            return false;
        };
        match (&mut self.authority, self.connection) {
            (Some(a), Some(c)) => match a.resumed(c, &evidence, now()) {
                Ok(grant) => {
                    self.granted = true;
                    // The GATT adapter must separately prepare the worker before publishing its epoch.
                    esp_println::println!("持久绑定已认证 epoch={}", grant.connection().epoch());
                    true
                }
                Err(_) => false,
            },
            _ => false,
        }
    }
    pub fn paired(&mut self, bond: Option<&BondInformation>, level: SecurityLevel) {
        self.pending = match (&mut self.authority, self.connection, bond) {
            (Some(a), Some(c), Some(b)) => keys::evidence(b, Origin::Pairing, level)
                .and_then(|e| a.paired(c, &e, super::random(), now()).ok()),
            _ => None,
        };
        self.granted = false;
        crate::installation::LIVE_EPOCH.store(0, Ordering::Release);
    }
    pub fn proven(&mut self, level: Result<SecurityLevel, trouble_host::Error>) -> bool {
        match (&mut self.authority, self.connection) {
            (Some(a), Some(c)) if self.granted => a.grant(c, keys::security(level), now()).is_ok(),
            _ => false,
        }
    }
    #[cfg(feature = "installation-gatt")]
    pub fn grant(
        &mut self,
        level: Result<SecurityLevel, trouble_host::Error>,
    ) -> Option<stagemaster_device_auth::authority::Grant> {
        match (&mut self.authority, self.connection) {
            (Some(a), Some(c)) if self.granted => a.grant(c, keys::security(level), now()).ok(),
            _ => None,
        }
    }
    pub fn heartbeat(&mut self, level: Result<SecurityLevel, trouble_host::Error>) -> bool {
        if !self.granted {
            return true;
        }
        match (&mut self.authority, self.connection) {
            (Some(a), Some(c)) => a.heartbeat(c, keys::security(level), now()).is_ok(),
            _ => false,
        }
    }
    pub fn close(&mut self) {
        crate::installation::LIVE_EPOCH.store(0, Ordering::Release);
        self.granted = false;
        if let (Some(a), Some(c)) = (&mut self.authority, self.connection.take()) {
            a.disconnect(c);
        }
    }
    /// Called after physical disconnection; cancellation cannot restore a connection.
    pub async fn persist(&mut self) {
        if let Some(proposal) = self.pending.take() {
            let result = match self.client.call(Action::Commit(proposal)).await {
                Ok(v) => Ok(v),
                Err(_) => self.client.call(Action::Recover).await,
            };
            if let (Some(a), Ok(v)) = (&mut self.authority, result) {
                let generation = v.generation();
                let peers = v.bindings().count();
                if a.restore(v).is_ok() {
                    esp_println::println!(
                        "绑定持久结果已核验 generation={} peers={}；须重连认证",
                        generation,
                        peers
                    );
                }
            }
        }
    }
    pub fn reload<C: Controller, P: PacketPool>(
        &self,
        stack: &trouble_host::Stack<'_, C, P>,
    ) -> bool {
        // Snapshot identities before mutating the stack's internal bond list.
        let identities: [Option<Identity>; MAX_BINDINGS] = stack.with_bond_information(|bonds| {
            core::array::from_fn(|i| bonds.get(i).map(|b| b.identity))
        });
        for identity in identities.into_iter().flatten() {
            if stack.remove_bond_information(identity).is_err() {
                return false;
            }
        }
        if let Some(vault) = self.authority.as_ref().and_then(Authority::vault) {
            // Explicit current-Mac cache-repair image only. Keep the durable vault
            // intact until replacement pairing commits; do not reload its old LTK
            // into the stack after macOS has forgotten that bond. Generation 2
            // and exactly one owner make this one-shot on the authorized test board.
            #[cfg(feature = "binding-repair-test")]
            if vault.generation() == 2 && vault.bindings().count() == 1 {
                esp_println::println!("当前 Mac 绑定修复：保留持久档案，等待一次新配对提交");
                return true;
            }
            for binding in vault.bindings() {
                if stack.add_bond_information(keys::bond(binding)).is_err() {
                    return false;
                }
            }
        }
        true
    }
}
