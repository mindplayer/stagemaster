//! One physical diagnostic connection; authenticated reads have an independent gate.
use super::Server;
use embassy_futures::select::{Either, select};
use embassy_time::{Duration, Instant, Timer, with_timeout};
use esp_println::println;
use stagemaster_device_link::Session;
use trouble_host::prelude::*;

pub(super) async fn serve<P: PacketPool>(
    server: &Server<'_>,
    conn: &GattConnection<'_, '_, P>,
    description: &[u8; 96],
    snapshot: fn() -> [u8; 20],
    connected: &mut impl FnMut(bool),
    #[cfg(feature = "application-gatt")]
    credentials: &stagemaster_device_auth::application::Configuration,
    #[cfg(feature = "application-gatt")] worker_epoch: stagemaster_install_worker::Epoch,
    #[cfg(feature = "secure-gatt-test")] secure_key: &stagemaster_device_session::SecretKey,
    #[cfg(feature = "binding-readiness")] binding: &mut crate::bindings::link::Link,
) {
    let description = stagemaster_device_info::Description::decode(description).unwrap();
    let session_id = description.session;
    #[cfg(feature = "application-gatt")]
    let mut application = super::application::Channel::new(description, worker_epoch);
    #[cfg(feature = "secure-gatt-test")]
    let mut secure = super::secure_probe::Probe::new(description, conn.raw().att_mtu());
    #[cfg(feature = "installation-gatt")]
    let mut installation = super::installation::Channel::new(description);
    let mut session = Session::new(session_id, Instant::now().as_millis()).unwrap();
    // Experimental pairing must precede HELLO: CoreBluetooth can serialize ATT
    // behind its native passkey sheet. Established diagnostic leases stay 6 s.
    #[cfg(feature = "security-readiness")]
    let admission_deadline = Instant::now() + Duration::from_secs(90);
    #[cfg(feature = "security-readiness")]
    let mut awaiting_hello = true;
    loop {
        #[cfg(feature = "security-readiness")]
        let expired = if awaiting_hello {
            Instant::now() >= admission_deadline
        } else {
            session.poll(Instant::now().as_millis()).unwrap()
        };
        #[cfg(not(feature = "security-readiness"))]
        let expired = session.poll(Instant::now().as_millis()).unwrap();
        #[cfg(feature = "secure-gatt-test")]
        let expired = expired && !secure.started();
        #[cfg(feature = "binding-readiness")]
        let expired = expired || binding.expired();
        #[cfg(feature = "application-gatt")]
        let expired = expired && !application.started();
        if expired {
            println!("GATT application heartbeat expired; local playback continues");
            conn.raw().disconnect();
            break;
        }
        #[cfg(feature = "installation-gatt")]
        if !installation.tick(&server.installation, conn, binding).await {
            conn.raw().disconnect();
            break;
        }
        #[cfg(feature = "application-gatt")]
        if !application.tick(server, conn).await {
            conn.raw().disconnect();
            break;
        }
        let delay = 100;
        #[cfg(feature = "application-gatt")]
        let delay = if application.sending() { 1 } else { delay };
        #[cfg(feature = "secure-gatt-test")]
        if !secure.tick(&server.secure_probe, conn).await {
            conn.raw().disconnect();
            break;
        }
        #[cfg(feature = "secure-gatt-test")]
        let delay = if secure.sending() { 1 } else { delay };
        #[cfg(feature = "installation-gatt")]
        let delay = if installation.sending() { 1 } else { delay };
        let event = match select(conn.next(), Timer::after_millis(delay)).await {
            Either::First(event) => event,
            Either::Second(()) => continue,
        };
        match event {
            #[cfg(feature = "security-readiness")]
            GattConnectionEvent::PassKeyDisplay(key) => {
                #[cfg(feature = "binding-readiness")]
                if !binding.admit() {
                    println!("绑定准入未开启或次数已用完");
                    conn.raw().disconnect();
                    break;
                }
                // Development USB display only. Never log the LTK or bond structure.
                println!("DEVELOPMENT PAIRING CODE: {}", key);
            }
            #[cfg(feature = "security-readiness")]
            GattConnectionEvent::PairingComplete {
                security_level,
                bond,
            } => {
                println!("PAIRING {:?}; bonded={}", security_level, bond.is_some());
                #[cfg(feature = "binding-readiness")]
                {
                    if conn.raw().security_level() == Ok(security_level) {
                        binding.paired(bond.as_ref(), security_level);
                    }
                    conn.raw().disconnect();
                    break;
                }
            }
            #[cfg(feature = "security-readiness")]
            GattConnectionEvent::Encrypted {
                security_level,
                bond,
            } => {
                println!("ENCRYPTED {:?}; resumed={}", security_level, bond.is_some());
                #[cfg(feature = "binding-readiness")]
                if let Some(bond) = &bond {
                    if conn.raw().security_level() != Ok(security_level)
                        || !binding.resumed(bond, security_level)
                    {
                        conn.raw().disconnect();
                        break;
                    }
                    let mut proof = [0; 16];
                    proof[..4].copy_from_slice(b"SMTP");
                    proof[4] = 1;
                    proof[8..].copy_from_slice(&session_id.to_le_bytes());
                    conn.set(&server.security_probe.proof, &proof).unwrap();
                }
            }
            #[cfg(feature = "security-readiness")]
            GattConnectionEvent::PairingFailed(error) => {
                println!("PAIRING FAILED {:?}", error);
                #[cfg(feature = "binding-readiness")]
                {
                    conn.raw().disconnect();
                    break;
                }
            }
            #[cfg(feature = "binding-readiness")]
            GattConnectionEvent::BondLost => {
                conn.raw().disconnect();
                break;
            }
            GattConnectionEvent::Disconnected { reason } => {
                println!("GATT disconnected {:?}", reason);
                break;
            }
            GattConnectionEvent::Gatt { event } => {
                let mut response = None;
                let reply = match event {
                    #[cfg(feature = "application-gatt")]
                    GattEvent::Write(event)
                        if super::application::request_mode(server, event.handle()).is_some() =>
                    {
                        let mode =
                            super::application::request_mode(server, event.handle()).unwrap();
                        let valid = super::application::response(server, mode)
                            .is_some_and(|response| response.should_notify(conn))
                            && event.with_data(|offset, bytes| {
                                offset == 0
                                    && application.receive(
                                        mode,
                                        bytes,
                                        credentials,
                                        conn.raw().att_mtu(),
                                    )
                            });
                        if valid {
                            event.accept_unprocessed()
                        } else {
                            conn.raw().disconnect();
                            event.reject(AttErrorCode::INSUFFICIENT_AUTHORISATION)
                        }
                    }
                    #[cfg(feature = "secure-gatt-test")]
                    GattEvent::Write(event)
                        if event.handle() == server.secure_probe.request.handle =>
                    {
                        let valid = server.secure_probe.response.should_notify(conn)
                            && event.with_data(|offset, bytes| {
                                offset == 0
                                    && secure.receive(bytes, secure_key, conn.raw().att_mtu())
                            });
                        if valid {
                            event.accept_unprocessed()
                        } else {
                            conn.raw().disconnect();
                            event.reject(AttErrorCode::UNLIKELY_ERROR)
                        }
                    }
                    #[cfg(feature = "installation-gatt")]
                    GattEvent::Read(event)
                        if event.handle() == server.installation.receipt.handle =>
                    {
                        if installation.authorize(conn, binding) {
                            event.accept()
                        } else {
                            conn.raw().disconnect();
                            event.reject(AttErrorCode::INSUFFICIENT_AUTHORISATION)
                        }
                    }
                    #[cfg(feature = "installation-gatt")]
                    GattEvent::Write(event)
                        if event.handle() == server.installation.request.handle =>
                    {
                        let valid = event.with_data(|offset, bytes| {
                            offset == 0
                                && installation.receive(bytes, &server.installation, conn, binding)
                        });
                        if valid {
                            event.accept_unprocessed()
                        } else {
                            conn.raw().disconnect();
                            event.reject(AttErrorCode::INSUFFICIENT_AUTHORISATION)
                        }
                    }
                    #[cfg(feature = "binding-readiness")]
                    GattEvent::Read(event)
                        if event.handle() == server.security_probe.proof.handle =>
                    {
                        if conn.raw().security_level() != Ok(SecurityLevel::EncryptedAuthenticated)
                        {
                            event.reject(AttErrorCode::INSUFFICIENT_AUTHENTICATION)
                        } else if binding.proven(conn.raw().security_level()) {
                            event.accept()
                        } else {
                            event.reject(AttErrorCode::INSUFFICIENT_AUTHORISATION)
                        }
                    }
                    GattEvent::Read(event) if event.handle() == server.link.info.handle => {
                        // Use the attribute table's bounded read/offset handling.
                        conn.set(&server.link.info, &snapshot()).unwrap();
                        event.accept()
                    }
                    GattEvent::Write(event) if event.handle() == server.link.request.handle => {
                        match event.with_data(|offset, data| offset == 0 && data.len() == 20) {
                            true => {
                                response = Some(event.with_data(|_offset, data| {
                                    let now = Instant::now().as_millis();
                                    #[cfg(feature = "security-readiness")]
                                    if awaiting_hello {
                                        let mut candidate = Session::new(session_id, now).unwrap();
                                        let reply = candidate.receive(data, now);
                                        if reply.code == 0 {
                                            session = candidate;
                                            awaiting_hello = false;
                                        }
                                        return reply.encode();
                                    }
                                    session.receive(data, now).encode()
                                }));
                                event.accept_unprocessed()
                            }
                            false => event.reject(AttErrorCode::INVALID_ATTRIBUTE_VALUE_LENGTH),
                        }
                    }
                    _ => event.accept(),
                };
                #[cfg(feature = "binding-readiness")]
                if response.is_some_and(|b| b[2] == 0)
                    && !binding.heartbeat(conn.raw().security_level())
                {
                    conn.raw().disconnect();
                    break;
                }
                if let Some(bytes) = response {
                    conn.set(&server.link.response, &bytes).unwrap();
                }
                if let Ok(reply) = reply
                    && with_timeout(Duration::from_millis(250), reply.send())
                        .await
                        .is_err()
                {
                    println!("GATT ATT reply deadline exceeded");
                    conn.raw().disconnect();
                    break;
                }
                if let Some(bytes) = response {
                    match with_timeout(
                        Duration::from_millis(250),
                        server.link.response.notify(conn, &bytes, false),
                    )
                    .await
                    {
                        Ok(Ok(())) => {
                            if bytes[1] == stagemaster_device_link::HELLO | 0x80 && bytes[2] == 0 {
                                connected(true);
                            }
                        }
                        Ok(Err(error)) => {
                            println!("GATT notification failed {:?}", error);
                            conn.raw().disconnect();
                            break;
                        }
                        Err(_) => {
                            println!("GATT notification deadline exceeded");
                            conn.raw().disconnect();
                            break;
                        }
                    }
                }
            }
            _ => {}
        }
    }
    #[cfg(feature = "installation-gatt")]
    installation.close();
}
