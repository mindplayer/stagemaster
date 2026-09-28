//! GATT adapter for the diagnostic protocol. It owns no playback commands.
use embassy_futures::{
    join::join,
    select::{Either, select},
};
use embassy_time::{Duration, Instant, Timer, with_timeout};
use esp_println::println;
use stagemaster_device_link::Session;
use trouble_host::prelude::*;

#[gatt_server]
struct Server {
    link: LinkService,
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

pub async fn run<C: Controller>(controller: C, snapshot: fn() -> [u8; 20]) -> ! {
    // main has enabled the BLE controller before entering this boot-scoped adapter.
    let identity = crate::identity::Identity::capture();
    let mut random = [0; 6];
    esp_hal::rng::Rng::new().read(&mut random);
    random[5] |= 0xc0; // A boot-scoped static random BLE address, not device authentication.
    let mut resources: HostResources<DefaultPacketPool, 1, 2> = HostResources::new();
    let stack = trouble_host::new(controller, &mut resources)
        .set_random_address(Address::random(random))
        .build();
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
                let conn = raw.with_attribute_server(&server).unwrap();
                let mut nonce = [0; 8];
                esp_hal::rng::Rng::new().read(&mut nonce);
                let session_id = u64::from_le_bytes(nonce).max(1);
                let mut session = Session::new(session_id, Instant::now().as_millis()).unwrap();
                conn.set(&server.link.response, &[0; 20]).unwrap();
                // Fixed for this connection, including every offset of ATT long reads.
                conn.set(&server.link.description, &identity.describe(session_id))
                    .unwrap();
                println!("GATT connected");
                serve(&server, &conn, &mut session, snapshot).await;
            }
        },
    )
    .await;
    panic!("BLE runner stopped");
}

async fn serve<P: PacketPool>(
    server: &Server<'_>,
    conn: &GattConnection<'_, '_, P>,
    session: &mut Session,
    snapshot: fn() -> [u8; 20],
) {
    loop {
        if session.poll(Instant::now().as_millis()).unwrap() {
            println!("GATT application heartbeat expired; local playback continues");
            conn.raw().disconnect();
            break;
        }
        let event = match select(conn.next(), Timer::after_millis(100)).await {
            Either::First(event) => event,
            Either::Second(()) => continue,
        };
        match event {
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
                                    session.receive(data, Instant::now().as_millis()).encode()
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
                    conn.raw().disconnect();
                    break;
                }
                if let Some(bytes) = response
                    && !matches!(
                        with_timeout(
                            Duration::from_millis(250),
                            server.link.response.notify(conn, &bytes, false)
                        )
                        .await,
                        Ok(Ok(()))
                    )
                {
                    conn.raw().disconnect();
                    break;
                }
            }
            _ => {}
        }
    }
}
