use super::{Action, Error, REPLIES, REQUESTS, Reply, Request};
use crate::shared_flash::PartitionNor;
use embassy_futures::{
    block_on,
    select::{Either3, select3},
};
use embassy_time::Timer;
use embedded_storage::nor_flash::ReadNorFlash;
use stagemaster_device_auth::persistence::{STORAGE_BYTES, VaultStore};
use stagemaster_install::Storage;
use stagemaster_install_worker::{Command, ManagedWorker};
use stagemaster_runtime::PlaybackPolicy;

pub struct Store {
    inner: VaultStore<PartitionNor>,
}
impl Store {
    pub fn new(partition: PartitionNor) -> Option<Self> {
        if partition.offset() != 0x00a1_2000 || partition.capacity() != STORAGE_BYTES {
            return None;
        }
        Some(Self {
            inner: VaultStore::new(partition, esp_hal::rng::Rng::new().random()).ok()?,
        })
    }
    /// One executor serializes both credential work and package work.
    pub async fn next<S: Storage, P: PlaybackPolicy>(
        &mut self,
        worker: &mut ManagedWorker<S, P>,
    ) -> Option<Command> {
        match select3(
            crate::installation::REQUESTS.receive(),
            REQUESTS.receive(),
            Timer::after_millis(100),
        )
        .await
        {
            Either3::First(command) => Some(command),
            Either3::Second(request) => {
                let reply = self.process(worker, request);
                REPLIES.send(reply).await;
                None
            }
            Either3::Third(()) => None,
        }
    }
    fn process<S: Storage, P: PlaybackPolicy>(
        &mut self,
        worker: &mut ManagedWorker<S, P>,
        request: Request,
    ) -> Reply {
        crate::installation::sample_stack();
        let start = esp_hal::time::Instant::now();
        let result = match request.action {
            Action::Recover => block_on(self.inner.recover()).cloned().map_err(|error| {
                esp_println::println!("绑定存储恢复未就绪：{:?}", error);
                Error::Storage
            }),
            action => worker
                .with_storage_maintenance(embassy_time::Instant::now().as_millis(), || {
                    let outcome = match action {
                        #[cfg(feature = "binding-local-test")]
                        Action::Initialize(local) => block_on(self.inner.initialize(local)),
                        Action::Commit(proposal) => block_on(self.inner.commit(&proposal)),
                        Action::Recover => unreachable!(),
                    };
                    outcome.cloned().map_err(|error| {
                        esp_println::println!("绑定存储提交未确认：{:?}", error);
                        Error::Storage
                    })
                })
                .map_err(|_| Error::Maintenance)
                .and_then(core::convert::identity),
        };
        esp_println::println!(
            "绑定存储操作完成 id={} ok={} elapsed_us={} store_bytes={} heap={}/{}",
            request.id,
            result.is_ok(),
            start.elapsed().as_micros(),
            core::mem::size_of::<Self>(),
            esp_alloc::HEAP.used(),
            esp_alloc::HEAP.free()
        );
        crate::installation::report();
        Reply {
            id: request.id,
            result,
        }
    }
}
