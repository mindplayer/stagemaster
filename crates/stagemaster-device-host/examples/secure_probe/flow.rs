use super::{
    Result, Trust,
    link::{Fault, Link},
};
use stagemaster_device_session::{
    CIPHERTEXT_BYTES, Channel, Context, Error, HANDSHAKE_BYTES, Handshake, Kind, MAX_PAYLOAD,
    PLAINTEXT_BYTES, SecretKey,
};
use std::time::Duration;
use tokio::time::{Instant, sleep};

fn entropy(bytes: &mut [u8]) -> std::result::Result<(), Error> {
    getrandom::fill(bytes).map_err(|_| Error::Entropy)
}
async fn handshake(link: &mut Link, trust: &Trust) -> Result<Channel> {
    let key = SecretKey::generate(entropy)?;
    let desc = link.description;
    let context = Context {
        device: desc.device,
        boot: desc.boot,
        connection: desc.session,
    };
    let mut handshake = Handshake::initiate(context, &key, trust.public_key, entropy, link.now())?;
    let mut wire = [0; HANDSHAKE_BYTES];
    let start = Instant::now();
    let n = handshake.write(&mut wire, link.now())?;
    link.send(&wire[..n], Fault::None).await?;
    handshake.read(link.receive().await?.bytes(), link.now())?;
    let mut channel = handshake.finish(link.now())?;
    let mut cipher = [0; CIPHERTEXT_BYTES];
    let n = channel.confirmation(&mut cipher, link.now())?;
    link.send(&cipher[..n], Fault::None).await?;
    channel.confirm(link.receive().await?.bytes(), link.now())?;
    let proof = channel.peer(link.now())?.ok_or("没有完成相互确认")?;
    assert_eq!(proof.public_key(), &trust.public_key);
    assert_eq!(proof.context(), context);
    println!(
        "HANDSHAKE confirmed elapsed_ms={} session={:02x?}",
        start.elapsed().as_millis(),
        proof.session()
    );
    Ok(channel)
}
async fn exchange(
    link: &mut Link,
    channel: &mut Channel,
    kind: Kind,
    data: &[u8],
) -> Result<Vec<u8>> {
    let mut wire = [0; CIPHERTEXT_BYTES];
    let n = channel.seal(kind, data, &mut wire, link.now())?;
    link.send(&wire[..n], Fault::None).await?;
    let record = link.receive().await?;
    let mut plain = [0; PLAINTEXT_BYTES];
    let response = channel.open(record.bytes(), &mut plain, link.now())?;
    assert_eq!(
        response.kind,
        if kind == Kind::Heartbeat {
            Kind::HeartbeatReply
        } else {
            kind
        }
    );
    assert_eq!(response.payload, data);
    Ok(wire[..n].to_vec())
}
pub async fn run(link: &mut Link, trust: &Trust, mode: &str) -> Result<()> {
    if matches!(mode, "partial" | "cancel") {
        let start = Instant::now();
        link.send(&[0x5a; CIPHERTEXT_BYTES], Fault::FirstOnly)
            .await?;
        if mode == "cancel" {
            return link.close().await;
        }
        link.disconnected(Duration::from_secs(7)).await?;
        let elapsed = start.elapsed();
        assert!((Duration::from_millis(4800)..Duration::from_millis(6200)).contains(&elapsed));
        println!("PARTIAL disconnected_ms={}", elapsed.as_millis());
        return Ok(());
    }
    if matches!(mode, "bad-key" | "bad-context") {
        let wrong = Trust {
            public_key: SecretKey::generate(entropy)?.public(),
            ..*trust
        };
        if mode == "bad-context" {
            link.description.session = link.description.session.wrapping_add(1).max(1);
        }
        let peer = if mode == "bad-key" { &wrong } else { trust };
        let start = Instant::now();
        assert!(!matches!(
            tokio::time::timeout(Duration::from_secs(3), handshake(link, peer)).await,
            Ok(Ok(_))
        ));
        link.disconnected(Duration::from_secs(1)).await?;
        assert!(start.elapsed() < Duration::from_secs(4));
        return Ok(());
    }
    let mut channel = handshake(link, trust).await?;
    match mode {
        "normal" | "small" | "fast" | "small-fast" | "stress" => {
            let rounds = if mode == "stress" {
                128
            } else if mode.starts_with("small") {
                4
            } else {
                16
            };
            exercise(link, &mut channel, rounds).await?;
        }
        "plain-only" => {
            let start = Instant::now();
            let mut public_heartbeats = 0;
            for _ in 0..6 {
                sleep(Duration::from_millis(850)).await;
                if link.diagnostic().await.is_ok() {
                    public_heartbeats += 1;
                } else {
                    break;
                }
            }
            link.disconnected(Duration::from_secs(3)).await?;
            assert!(public_heartbeats >= 3);
            assert!(start.elapsed() < Duration::from_millis(7900));
            println!(
                "EXPIRED despite_plain_heartbeats={public_heartbeats} elapsed_ms={}",
                start.elapsed().as_millis()
            );
        }
        "tamper" | "replay" | "gap" | "duplicate" => {
            let mut wire = if mode == "replay" {
                exchange(link, &mut channel, Kind::Message, &[0x5a; MAX_PAYLOAD]).await?
            } else {
                let mut out = [0; CIPHERTEXT_BYTES];
                let n = channel.seal(Kind::Message, &[0x5a; MAX_PAYLOAD], &mut out, link.now())?;
                out[..n].to_vec()
            };
            if mode == "tamper" {
                *wire.last_mut().unwrap() ^= 1;
            }
            let fault = match mode {
                "gap" => Fault::Gap,
                "duplicate" => Fault::Duplicate,
                _ => Fault::None,
            };
            let start = Instant::now();
            let _ = link.send(&wire, fault).await;
            link.disconnected(Duration::from_secs(3)).await?;
            assert!(start.elapsed() < Duration::from_secs(4));
            println!(
                "REJECTED mode={mode} elapsed_ms={}",
                start.elapsed().as_millis()
            );
        }
        _ => return Err("未知实验模式".into()),
    }
    Ok(())
}

async fn exercise(link: &mut Link, channel: &mut Channel, rounds: usize) -> Result<()> {
    let before = link.snapshot().await?;
    let mut worst = Duration::ZERO;
    for round in 0..rounds {
        let payload: [u8; MAX_PAYLOAD] =
            core::array::from_fn(|i| u8::try_from((i + round) % 256).unwrap());
        let start = Instant::now();
        exchange(link, channel, Kind::Message, &payload).await?;
        worst = worst.max(start.elapsed());
    }
    for _ in 0..5 {
        sleep(Duration::from_millis(1500)).await;
        exchange(link, channel, Kind::Heartbeat, &[]).await?;
    }
    let after = link.snapshot().await?;
    assert!(after.ticks > before.ticks);
    assert_eq!(before.heap_used, after.heap_used);
    println!(
        "ECHO rounds={rounds} max_payload={MAX_PAYLOAD} worst_ms={} encrypted_heartbeats=5 ticks={}->{} heap={}->{}",
        worst.as_millis(),
        before.ticks,
        after.ticks,
        before.heap_used,
        after.heap_used
    );
    Ok(())
}
