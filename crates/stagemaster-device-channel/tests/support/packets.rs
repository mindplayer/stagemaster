//! Test-only ordered GATT-sized packet carrier, using the real secure fragment codecs.
use super::now;
use stagemaster_device_channel::{Error, RecordIo};
use stagemaster_device_link::secure::{Receiver, Sender};
use tokio::{sync::mpsc, time::Instant};

pub struct Packets {
    tx: Option<mpsc::Sender<Vec<u8>>>,
    rx: mpsc::Receiver<Vec<u8>>,
    sender: Sender,
    receiver: Receiver,
    origin: Instant,
    sending: bool,
}
pub fn pair() -> (Packets, Packets) {
    let (at, br) = mpsc::channel(128);
    let (bt, ar) = mpsc::channel(128);
    let new = |tx, rx| Packets {
        tx: Some(tx),
        rx,
        sender: Sender::new(20, 0).unwrap(),
        receiver: Receiver::new(20, 0).unwrap(),
        origin: Instant::now(),
        sending: false,
    };
    (new(at, ar), new(bt, br))
}
impl RecordIo for Packets {
    async fn send(&mut self, bytes: &[u8]) -> Result<(), Error> {
        if !self.healthy() {
            return Err(Error::Closed);
        }
        self.sending = true;
        self.sender.queue(bytes, now(self.origin)).map_err(wire)?;
        while let Some(packet) = self.sender.fragment(now(self.origin)).map_err(wire)? {
            self.tx
                .as_ref()
                .ok_or(Error::Closed)?
                .send(packet.bytes().to_vec())
                .await
                .map_err(wire)?;
            self.sender.sent(now(self.origin)).map_err(wire)?;
        }
        self.sending = false;
        Ok(())
    }
    fn try_receive(&mut self) -> Result<Option<Vec<u8>>, Error> {
        self.receiver.poll(now(self.origin)).map_err(wire)?;
        while let Ok(packet) = self.rx.try_recv() {
            if let Some(record) = self
                .receiver
                .push(&packet, now(self.origin))
                .map_err(wire)?
            {
                return Ok(Some(record.bytes().to_vec()));
            }
        }
        Ok(None)
    }
    fn healthy(&self) -> bool {
        !self.sending && self.tx.as_ref().is_some_and(|s| !s.is_closed())
    }
    fn close(&mut self) {
        self.tx = None;
        self.rx.close();
        self.receiver.close();
    }
}
impl Drop for Packets {
    fn drop(&mut self) {
        self.close();
    }
}
fn wire(error: impl std::fmt::Display) -> Error {
    Error::Protocol(error.to_string())
}
