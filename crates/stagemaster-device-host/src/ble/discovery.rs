use super::*;

impl Ble {
    pub(super) async fn scan(&mut self) -> Result<(), Problem> {
        if self.adapter.is_none() {
            let initialization = self.initializing.get_or_insert_with(|| {
                Box::pin(async {
                    let manager = Manager::new().await.map_err(error)?;
                    manager
                        .adapters()
                        .await
                        .map_err(error)?
                        .into_iter()
                        .next()
                        .ok_or_else(|| Problem::new(C::Adapter))
                })
            });
            let result = initialization.await;
            self.initializing = None;
            self.adapter = Some(result?);
        }
        let adapter = self
            .adapter
            .as_ref()
            .ok_or_else(|| Problem::new(C::Adapter))?;
        match adapter.adapter_state().await.map_err(error)? {
            CentralState::PoweredOn => (),
            CentralState::PoweredOff => return Err(Problem::new(C::PoweredOff)),
            CentralState::Unknown => return Err(Problem::new(C::Adapter)),
        }
        self.found.clear();
        self.events = Some(adapter.events().await.map_err(error)?);
        self.scanning = true; // cleanup also runs after a cancelled start_scan future
        adapter
            .start_scan(ScanFilter {
                services: vec![SERVICE],
            })
            .await
            .map_err(error)
    }
    pub(super) async fn next_candidate(&mut self) -> Result<Candidate, Problem> {
        loop {
            let event = self
                .events
                .as_mut()
                .ok_or_else(|| Problem::new(C::Adapter))?
                .next()
                .await
                .ok_or_else(|| Problem::new(C::Adapter))?;
            let id = match event {
                CentralEvent::DeviceDiscovered(id)
                | CentralEvent::DeviceUpdated(id)
                | CentralEvent::ServicesAdvertisement { id, .. }
                | CentralEvent::RssiUpdate { id, .. } => id,
                CentralEvent::StateUpdate(CentralState::PoweredOff) => {
                    return Err(Problem::new(C::PoweredOff));
                }
                _ => continue,
            };
            let peripheral = self
                .adapter
                .as_ref()
                .ok_or_else(|| Problem::new(C::Adapter))?
                .peripheral(&id)
                .await
                .map_err(error)?;
            let Some(properties) = peripheral.properties().await.map_err(error)? else {
                continue;
            };
            if !properties.services.contains(&SERVICE) {
                continue;
            }
            let id = id.to_string();
            if self.found.len() < MAX_CANDIDATES || self.found.contains_key(&id) {
                self.found.insert(id.clone(), peripheral);
            }
            // Still report the extra candidate so the service can report truncation.
            return Ok(Candidate {
                id,
                name: properties
                    .local_name
                    .unwrap_or_else(|| "未命名设备".into())
                    .chars()
                    .filter(|c| !c.is_control())
                    .take(80)
                    .collect(),
                rssi: properties.rssi,
            });
        }
    }
    pub(super) async fn stop(&mut self) -> Result<(), Problem> {
        if self.scanning {
            self.adapter
                .as_ref()
                .ok_or_else(|| Problem::new(C::Adapter))?
                .stop_scan()
                .await
                .map_err(error)?;
            self.scanning = false;
        }
        self.events = None;
        Ok(())
    }
}
