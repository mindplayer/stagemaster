use super::{Result, Trust, discovery, notifications::Incoming};
use btleplug::{
    api::{CharPropFlags, Characteristic, Peripheral as _, WriteType},
    platform::{Adapter, Peripheral},
};
use stagemaster_device_info::Description;
use stagemaster_device_link::{
    client::{Client, Diagnostics},
    secure::{Record, Sender},
};
use std::time::Duration;
use tokio::time::Instant;
use uuid::Uuid;

#[derive(Clone, Copy)]
pub enum Fault {
    None,
    Gap,
    Duplicate,
    FirstOnly,
}
pub struct Link {
    pub peripheral: Peripheral,
    pub description: Description,
    pub origin: Instant,
    incoming: Option<Incoming>,
    sender: Sender,
    request: Characteristic,
    diagnostic: Client,
    write_type: WriteType,
}
impl Link {
    pub async fn connect(adapter: &Adapter, trust: &Trust, cap: usize, fast: bool) -> Result<Self> {
        let peripheral = discovery::discover(adapter).await?;
        let result = tokio::time::timeout(
            Duration::from_secs(10),
            Self::prepare(&peripheral, trust, cap, fast),
        )
        .await
        .unwrap_or_else(|_| Err("设备连接准备超时".into()));
        if result.is_err() {
            let _ = tokio::time::timeout(Duration::from_secs(5), peripheral.disconnect()).await;
        }
        result
    }
    async fn prepare(
        peripheral: &Peripheral,
        trust: &Trust,
        cap: usize,
        fast: bool,
    ) -> Result<Self> {
        peripheral.connect().await?;
        peripheral.discover_services().await?;
        let origin = Instant::now();
        let mut diagnostic = Client::default();
        heartbeat(peripheral, &mut diagnostic).await?;
        let bytes = peripheral
            .read(&find(
                peripheral,
                discovery::DIAGNOSTIC,
                discovery::diagnostic(4),
                CharPropFlags::READ,
            )?)
            .await?;
        let description = Description::decode(&bytes)?;
        description.check_session(diagnostic.session_id().ok_or("诊断会话缺失")?)?;
        if description.device != trust.device
            || description.boot != trust.boot
            || description.authentication != 0
            || description.capabilities != 1
        {
            return Err("设备不匹配本次 USB 信任或只读能力".into());
        }
        let budget = usize::from(peripheral.mtu().saturating_sub(3).min(244));
        let request = find(
            peripheral,
            discovery::SERVICE,
            discovery::WRITE,
            if fast {
                CharPropFlags::WRITE_WITHOUT_RESPONSE
            } else {
                CharPropFlags::WRITE
            },
        )?;
        let response = find(
            peripheral,
            discovery::SERVICE,
            discovery::NOTIFY,
            CharPropFlags::NOTIFY,
        )?;
        let incoming = Incoming::spawn(peripheral.notifications().await?, origin, budget);
        peripheral.subscribe(&response).await?;
        println!(
            "CONNECTED locator={} mtu={} tx_budget={}",
            peripheral.id(),
            peripheral.mtu(),
            budget.min(cap)
        );
        Ok(Self {
            peripheral: peripheral.clone(),
            description,
            origin,
            incoming: Some(incoming),
            sender: Sender::new(budget.min(cap), 0)?,
            request,
            diagnostic,
            write_type: if fast {
                WriteType::WithoutResponse
            } else {
                WriteType::WithResponse
            },
        })
    }
    pub fn now(&self) -> u64 {
        u64::try_from(self.origin.elapsed().as_millis()).unwrap()
    }
    pub async fn send(&mut self, bytes: &[u8], fault: Fault) -> Result<()> {
        self.sender.queue(bytes, self.now())?;
        let mut index = 0;
        while let Some(packet) = self.sender.fragment(self.now())? {
            index += 1;
            if !matches!((fault, index), (Fault::Gap, 2)) {
                self.peripheral
                    .write(&self.request, packet.bytes(), self.write_type)
                    .await?;
            }
            if matches!((fault, index), (Fault::Duplicate, 1)) {
                self.peripheral
                    .write(&self.request, packet.bytes(), self.write_type)
                    .await?;
            }
            self.sender.sent(self.now())?;
            if matches!(fault, Fault::FirstOnly) {
                return Ok(());
            }
        }
        Ok(())
    }
    pub async fn receive(&mut self) -> Result<Record> {
        self.incoming.as_mut().ok_or("已关闭")?.receive().await
    }
    pub async fn diagnostic(&mut self) -> Result<()> {
        heartbeat(&self.peripheral, &mut self.diagnostic).await
    }
    pub async fn snapshot(&self) -> Result<Diagnostics> {
        let bytes = self
            .peripheral
            .read(&find(
                &self.peripheral,
                discovery::DIAGNOSTIC,
                discovery::diagnostic(3),
                CharPropFlags::READ,
            )?)
            .await?;
        let snapshot = Diagnostics::decode(&bytes).map_err(|e| format!("诊断失败：{e:?}"))?;
        if !snapshot.output_disabled || !snapshot.self_test {
            return Err("测试边界不满足".into());
        }
        Ok(snapshot)
    }
    pub async fn disconnected(&self, within: Duration) -> Result<()> {
        tokio::time::timeout(within, async {
            while self.peripheral.is_connected().await? {
                tokio::time::sleep(Duration::from_millis(25)).await;
            }
            Ok::<_, btleplug::Error>(())
        })
        .await??;
        Ok(())
    }
    pub async fn close(&mut self) -> Result<()> {
        self.incoming = None;
        self.sender.close();
        match tokio::time::timeout(Duration::from_secs(5), self.peripheral.disconnect()).await? {
            Ok(()) | Err(btleplug::Error::NotConnected) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}
fn find(
    peripheral: &Peripheral,
    service: Uuid,
    uuid: Uuid,
    property: CharPropFlags,
) -> Result<Characteristic> {
    peripheral
        .characteristics()
        .into_iter()
        .find(|c| c.service_uuid == service && c.uuid == uuid && c.properties.contains(property))
        .ok_or_else(|| format!("缺少实验特征 {uuid}").into())
}
async fn heartbeat(peripheral: &Peripheral, client: &mut Client) -> Result<()> {
    let request = client.request().map_err(|e| format!("诊断请求：{e:?}"))?;
    peripheral
        .write(
            &find(
                peripheral,
                discovery::DIAGNOSTIC,
                discovery::diagnostic(1),
                CharPropFlags::WRITE,
            )?,
            &request,
            WriteType::WithResponse,
        )
        .await?;
    let response = find(
        peripheral,
        discovery::DIAGNOSTIC,
        discovery::diagnostic(2),
        CharPropFlags::READ,
    )?;
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            let bytes = peripheral.read(&response).await?;
            if client.accept(&bytes).is_ok() {
                return Ok::<_, btleplug::Error>(());
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await??;
    Ok(())
}
