//! GATT adapter for the diagnostic protocol. It owns no playback commands.
#[cfg(feature = "installation-gatt")]
mod installation;
mod session;
use embassy_futures::join::join;
#[cfg(feature = "binding-readiness")]
use embassy_time::{Duration, Timer, with_timeout};
use esp_println::println;
use trouble_host::prelude::*;

#[cfg(not(feature = "security-readiness"))]
#[gatt_server]
struct Server {
    link: LinkService,
}

#[cfg(all(feature = "security-readiness", not(feature = "installation-gatt")))]
#[gatt_server]
struct Server {
    link: LinkService,
    security_probe: SecurityProbe,
}

#[cfg(feature = "installation-gatt")]
#[gatt_server]
struct Server {
    link: LinkService,
    security_probe: SecurityProbe,
    installation: InstallationService,
}

#[cfg(feature = "installation-gatt")]
#[gatt_service(uuid = "f889ed80-0100-4e83-968e-799ab99558fa")]
struct InstallationService {
    #[characteristic(uuid = "f889ed81-0100-4e83-968e-799ab99558fa", read, value = [0; 80], permissions(read = authenticated))]
    receipt: [u8; 80],
    #[characteristic(uuid = "f889ed82-0100-4e83-968e-799ab99558fa", write, value = [0; 244], permissions(write = authenticated))]
    request: [u8; 244],
    #[characteristic(uuid = "f889ed83-0100-4e83-968e-799ab99558fa", notify, value = [0; 244], permissions(cccd = authenticated))]
    response: [u8; 244],
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
    #[cfg(feature = "binding-readiness")]
    let mut binding = crate::bindings::link::Link::start().await;
    let mut random = [0; 6];
    esp_hal::rng::Rng::new().read(&mut random);
    random[5] |= 0xc0; // A boot-scoped static random BLE address, not device authentication.
    #[cfg(feature = "binding-readiness")]
    if let Some(local) = binding.local() {
        random = local.address().bytes();
    }
    let mut resources: HostResources<DefaultPacketPool, 1, 2, 1, 4> = HostResources::new();
    let builder =
        trouble_host::new(controller, &mut resources).set_random_address(Address::random(random));
    #[cfg(feature = "security-readiness")]
    let builder = builder.set_io_capabilities(IoCapabilities::DisplayOnly);
    #[cfg(feature = "binding-readiness")]
    let builder = match binding.local() {
        Some(local) => builder.enable_privacy(
            IdentityResolvingKey::new(u128::from_le_bytes(*local.irk().bytes())).unwrap(),
        ),
        None => builder,
    };
    let stack = builder.build();
    #[cfg(feature = "binding-readiness")]
    assert!(binding.reload(&stack));
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
            #[cfg(feature = "binding-readiness")]
            binding.open_local_test_window();
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
                #[cfg(not(feature = "installation-gatt"))]
                let raw = advertiser.accept().await.unwrap();
                #[cfg(feature = "installation-gatt")]
                let raw = {
                    use embassy_futures::select::{Either, select};
                    // An old worker may finish after disconnection. Always drain it,
                    // including while no peer is connected, so it cannot block recovery.
                    match select(advertiser.accept(), async {
                        loop {
                            crate::installation::COMPLETIONS.receive().await;
                        }
                    })
                    .await
                    {
                        Either::First(result) => result.unwrap(),
                        Either::Second(()) => unreachable!(),
                    }
                };
                #[cfg(feature = "security-readiness")]
                raw.set_bondable(true).unwrap();
                #[cfg(feature = "binding-readiness")]
                binding.connect();
                let conn = raw.with_attribute_server(&server).unwrap();
                let mut nonce = [0; 8];
                esp_hal::rng::Rng::new().read(&mut nonce);
                let session_id = u64::from_le_bytes(nonce).max(1);
                conn.set(&server.link.response, &[0; 20]).unwrap();
                // Fixed for this connection, including every offset of ATT long reads.
                #[cfg(not(feature = "installation-gatt"))]
                let enabled = false;
                #[cfg(feature = "installation-gatt")]
                let enabled = binding.local().is_some()
                    && crate::installation::READY.load(core::sync::atomic::Ordering::Acquire) == 1;
                let description = identity.describe(session_id, enabled);
                conn.set(&server.link.description, &description).unwrap();
                #[cfg(feature = "installation-gatt")]
                conn.set(&server.installation.receipt, &[0; 80]).unwrap();
                #[cfg(feature = "security-readiness")]
                {
                    #[cfg(not(feature = "binding-readiness"))]
                    let mut proof = [0; 16];
                    #[cfg(not(feature = "binding-readiness"))]
                    {
                        proof[..4].copy_from_slice(b"SMTP");
                        proof[4] = 1;
                        proof[8..].copy_from_slice(&session_id.to_le_bytes());
                    }
                    #[cfg(feature = "binding-readiness")]
                    let proof = [0; 16];
                    conn.set(&server.security_probe.proof, &proof).unwrap();
                }
                println!("GATT connected");
                session::serve(
                    &server,
                    &conn,
                    &description,
                    snapshot,
                    &mut connected,
                    #[cfg(feature = "binding-readiness")]
                    &mut binding,
                )
                .await;
                connected(false);
                #[cfg(feature = "binding-readiness")]
                {
                    binding.close();
                    conn.raw().disconnect();
                    // Allow the runner to finish physical disconnection before changing bonds.
                    assert!(
                        with_timeout(Duration::from_secs(5), async {
                            while conn.raw().is_connected() {
                                Timer::after_millis(20).await;
                            }
                        })
                        .await
                        .is_ok(),
                        "physical disconnect deadline"
                    );
                    drop(conn);
                    binding.persist().await;
                    assert!(binding.reload(&stack));
                    Timer::after_millis(100).await;
                }
            }
        },
    )
    .await;
    panic!("BLE runner stopped");
}
