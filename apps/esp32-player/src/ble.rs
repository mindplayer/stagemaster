//! GATT adapter for the diagnostic protocol. It owns no playback commands.
use embassy_futures::{
    join::join,
    select::{Either, select},
};
use embassy_time::{Duration, Instant, Timer, with_timeout};
use esp_println::println;
use stagemaster_device_link::Session;
use trouble_host::prelude::*;

#[cfg(not(feature = "security-readiness"))]
#[gatt_server]
struct Server {
    link: LinkService,
}

#[cfg(feature = "security-readiness")]
#[gatt_server]
struct Server {
    link: LinkService,
    security_probe: SecurityProbe,
}

/// Experimental read-only proof. No install command or production wire contract.
#[cfg(feature = "security-readiness")]
#[gatt_service(uuid = "f889ed70-0100-4e83-968e-799ab99558fa")]
struct SecurityProbe {
    #[characteristic(uuid = "f889ed71-0100-4e83-968e-799ab99558fa", read, permissions(read = authenticated))]
    proof: [u8; 16],
}

#[gatt_service(uuid = "f889ed60-0100-4e83-968e-799ab99558fa")]
struct LinkService {
    #[characteristic(uuid = "f889ed61-0100-4e83-968e-799ab99558fa", write)]
    request: [u8; 20],
    #[characteristic(uuid = "f889ed62-0100-4e83-968e-799ab99558fa", read, notify)]
    response: [u8; 20],
    #[characteristic(uuid = "f889ed63-0100-4e83-968e-799ab99558fa", read)]
    info: [u8; 20],
    #[characteristic(uuid = "f889ed64-0100-4e83-968e-799ab99558fa", read, value = [0; 96])]
    description: [u8; 96],
}

pub async fn run<C: Controller>(
    controller: C,
    identity: crate::identity::Identity,
    snapshot: fn() -> [u8; 20],
    mut connected: impl FnMut(bool),
) -> ! {
    let mut random = [0; 6];
    esp_hal::rng::Rng::new().read(&mut random);
    random[5] |= 0xc0; // A boot-scoped static random BLE address, not device authentication.
    let mut resources: HostResources<DefaultPacketPool, 1, 2> = HostResources::new();
    let builder =
        trouble_host::new(controller, &mut resources).set_random_address(Address::random(random));
    #[cfg(feature = "security-readiness")]
    let builder = builder.set_io_capabilities(IoCapabilities::DisplayOnly);
    let stack = builder.build();
    let mut runner = stack.runner();
    let mut peripheral = stack.peripheral();
    let server = Server::new_with_config(GapConfig::Peripheral(PeripheralConfig {
        name: "StageMaster",
        appearance: &appearance::power_device::GENERIC_POWER_DEVICE,
    }))
    .unwrap();
    let mut adv = [0; 31];
    // The 128-bit service is little endian on air. Put the full name in the scan response.
    let uuid = [
        0xfa, 0x58, 0x95, 0xb9, 0x9a, 0x79, 0x8e, 0x96, 0x83, 0x4e, 0, 1, 0x60, 0xed, 0x89, 0xf8,
    ];
    let len = AdStructure::encode_slice(
        &[
            AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
            AdStructure::CompleteServiceUuids128(&[uuid]),
        ],
        &mut adv,
    )
    .unwrap();
    let mut scan = [0; 31];
    let scan_len =
        AdStructure::encode_slice(&[AdStructure::CompleteLocalName(b"StageMaster")], &mut scan)
            .unwrap();
    join(
        async {
            runner.run().await.unwrap();
        },
        async {
            loop {
                println!(
                    "GATT advertising; heap used={} free={}",
                    esp_alloc::HEAP.used(),
                    esp_alloc::HEAP.free()
                );
                let advertiser = peripheral
                    .advertise(
                        &Default::default(),
                        Advertisement::ConnectableScannableUndirected {
                            adv_data: &adv[..len],
                            scan_data: &scan[..scan_len],
                        },
                    )
                    .await
                    .unwrap();
                let raw = advertiser.accept().await.unwrap();
                #[cfg(feature = "security-readiness")]
                raw.set_bondable(true).unwrap();
                let conn = raw.with_attribute_server(&server).unwrap();
                let mut nonce = [0; 8];
                esp_hal::rng::Rng::new().read(&mut nonce);
                let session_id = u64::from_le_bytes(nonce).max(1);
                conn.set(&server.link.response, &[0; 20]).unwrap();
                // Fixed for this connection, including every offset of ATT long reads.
                conn.set(&server.link.description, &identity.describe(session_id))
                    .unwrap();
                #[cfg(feature = "security-readiness")]
                {
                    let mut proof = [0; 16];
                    proof[..4].copy_from_slice(b"SMTP");
                    proof[4] = 1;
                    proof[8..].copy_from_slice(&session_id.to_le_bytes());
                    conn.set(&server.security_probe.proof, &proof).unwrap();
                }
                println!("GATT connected");
                serve(&server, &conn, session_id, snapshot, &mut connected).await;
                connected(false);
            }
        },
    )
    .await;
    panic!("BLE runner stopped");
}

async fn serve<P: PacketPool>(
    server: &Server<'_>,
    conn: &GattConnection<'_, '_, P>,
    session_id: u64,
    snapshot: fn() -> [u8; 20],
    connected: &mut impl FnMut(bool),
) {
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
        if expired {
            println!("GATT application heartbeat expired; local playback continues");
            conn.raw().disconnect();
            break;
        }
        let event = match select(conn.next(), Timer::after_millis(100)).await {
            Either::First(event) => event,
            Either::Second(()) => continue,
        };
        match event {
            #[cfg(feature = "security-readiness")]
            GattConnectionEvent::PassKeyDisplay(key) => {
                // Development USB display only. Never log the LTK or bond structure.
                println!("DEVELOPMENT PAIRING CODE: {}", key);
            }
            #[cfg(feature = "security-readiness")]
            GattConnectionEvent::PairingComplete {
                security_level,
                bond,
            } => {
                println!("PAIRING {:?}; bonded={}", security_level, bond.is_some());
            }
            #[cfg(feature = "security-readiness")]
            GattConnectionEvent::Encrypted {
                security_level,
                bond,
            } => {
                println!("ENCRYPTED {:?}; resumed={}", security_level, bond.is_some());
            }
            #[cfg(feature = "security-readiness")]
            GattConnectionEvent::PairingFailed(error) => {
                println!("PAIRING FAILED {:?}", error);
            }
            GattConnectionEvent::Disconnected { reason } => {
                println!("GATT disconnected {:?}", reason);
                break;
            }
            GattConnectionEvent::Gatt { event } => {
                let mut response = None;
                let reply = match event {
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
}
