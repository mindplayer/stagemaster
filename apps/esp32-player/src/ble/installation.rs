//! Physical-connection adapter; authority, reassembly and storage stay separate.
use super::InstallationService;
use crate::{
    bindings::link::Link,
    installation::{COMPLETIONS, LIVE_EPOCH, REQUESTS},
};
use core::sync::atomic::Ordering;
use embassy_time::{Duration, Instant, with_timeout};
use stagemaster_device_auth::authority::Grant;
use stagemaster_device_info::{Description, capability};
use stagemaster_device_link::management::{Receipt, Serial};
use stagemaster_install_worker::{Endpoint, Epoch, Phase};
use stagemaster_transfer::AuthorizedLink;
use trouble_host::prelude::*;

pub(super) struct Channel {
    description: Description,
    endpoint: Option<Endpoint>,
    grant: Option<Grant>,
    receipt: Option<Receipt>,
    published: bool,
    rx: Serial,
    tx: Serial,
}
fn now() -> u64 {
    Instant::now().as_millis()
}
impl Channel {
    pub fn new(description: Description) -> Self {
        Self {
            description,
            endpoint: None,
            grant: None,
            receipt: None,
            published: false,
            rx: Serial::default(),
            tx: Serial::default(),
        }
    }
    pub fn close(&mut self) {
        LIVE_EPOCH.store(0, Ordering::Release);
        if let Some(endpoint) = &mut self.endpoint {
            endpoint.close();
        }
        self.published = false;
    }
    fn valid<P: PacketPool>(&self, conn: &GattConnection<'_, '_, P>, binding: &mut Link) -> bool {
        self.grant.is_some() && self.grant == binding.grant(conn.raw().security_level())
    }
    pub fn authorize<P: PacketPool>(
        &mut self,
        conn: &GattConnection<'_, '_, P>,
        binding: &mut Link,
    ) -> bool {
        if self.endpoint.is_some() {
            return self.valid(conn, binding);
        }
        if !self.description.declares(capability::INSTALLATION) {
            return false;
        }
        let Some(grant) = binding.grant(conn.raw().security_level()) else {
            return false;
        };
        let fragment_bytes = conn
            .raw()
            .att_mtu()
            .saturating_sub(3)
            .min(244)
            .saturating_sub(4);
        let receipt = Receipt {
            device: self.description.device,
            boot: self.description.boot,
            diagnostic: self.description.session,
            session: grant.session(),
            fragment_bytes,
        };
        if receipt.validate().is_err() {
            return false;
        }
        let epoch = Epoch::new(grant.connection().epoch().get()).unwrap();
        let link = AuthorizedLink {
            principal: grant.principal(),
            session: grant.session(),
        };
        let Ok((endpoint, command)) =
            Endpoint::open(epoch, link, usize::from(fragment_bytes), now())
        else {
            return false;
        };
        self.endpoint = Some(endpoint);
        self.grant = Some(grant);
        self.receipt = Some(receipt);
        LIVE_EPOCH.store(epoch.get(), Ordering::Release);
        if REQUESTS.try_send(command).is_err() {
            self.close();
            return false;
        }
        true
    }
    pub fn sending(&self) -> bool {
        self.endpoint
            .as_ref()
            .is_some_and(|e| e.phase() == Phase::Sending)
    }
    pub fn receive<P: PacketPool>(
        &mut self,
        bytes: &[u8],
        service: &InstallationService,
        conn: &GattConnection<'_, '_, P>,
        binding: &mut Link,
    ) -> bool {
        if !self.published || !self.valid(conn, binding) || !service.response.should_notify(conn) {
            self.close();
            return false;
        }
        let result = (|| {
            let bytes = self.rx.receive(bytes).ok()?;
            let command = self.endpoint.as_mut()?.receive(bytes, now()).ok()?;
            if let Some(command) = command {
                REQUESTS.try_send(command).ok()?;
            }
            Some(())
        })();
        if result.is_none() {
            self.close();
        }
        result.is_some()
    }
    pub async fn tick<P: PacketPool>(
        &mut self,
        service: &InstallationService,
        conn: &GattConnection<'_, '_, P>,
        binding: &mut Link,
    ) -> bool {
        if self.endpoint.is_some() && !self.valid(conn, binding) {
            self.close();
            return false;
        }
        let result = self.tick_inner(service, conn).await;
        if !result || (self.endpoint.is_some() && !self.valid(conn, binding)) {
            self.close();
            return false;
        }
        true
    }
    async fn tick_inner<P: PacketPool>(
        &mut self,
        service: &InstallationService,
        conn: &GattConnection<'_, '_, P>,
    ) -> bool {
        // Obsolete completions are consumed even before authentication.
        while let Ok(completion) = COMPLETIONS.try_receive() {
            if let Some(endpoint) = &mut self.endpoint
                && let Err(error) = endpoint.complete(completion, now())
            {
                esp_println::println!("安装工作回执失败：{}", error);
                return false;
            }
        }
        let Some(endpoint) = &mut self.endpoint else {
            return true;
        };
        if endpoint.poll(now()).is_err() {
            return false;
        }
        if !self.published && endpoint.phase() == Phase::Receiving {
            let bytes = self.receipt.unwrap().encode().unwrap();
            if conn.set(&service.receipt, &bytes).is_err() {
                return false;
            }
            self.published = true;
            esp_println::println!(
                "安装业务会话已就绪；片段={} 字节",
                self.receipt.unwrap().fragment_bytes
            );
        }
        let bytes = match endpoint.fragment(now()) {
            Ok(Some(bytes)) => bytes,
            Ok(None) => return true,
            Err(_) => return false,
        };
        if !service.response.should_notify(conn) {
            return false;
        }
        let Ok(packet) = self.tx.encode(bytes) else {
            return false;
        };
        if !matches!(
            with_timeout(
                Duration::from_millis(250),
                service.response.notify_raw(conn, packet.bytes(), false)
            )
            .await,
            Ok(Ok(()))
        ) {
            return false;
        }
        self.tx.advance().is_ok() && endpoint.sent(now()).is_ok()
    }
}
impl Drop for Channel {
    fn drop(&mut self) {
        self.close();
    }
}
