use super::{Server, State};
use crate::support;
use stagemaster_device_channel::StreamRecords;
use stagemaster_device_session::Kind;
use stagemaster_runtime_protocol::Request;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::io::DuplexStream;

pub(super) async fn serve(
    mut server: Server<StreamRecords<DuplexStream>>,
    data: Arc<Mutex<State>>,
) {
    let mut timer = tokio::time::interval(Duration::from_millis(10));
    loop {
        let incoming = tokio::select! {
            _=timer.tick()=> {
                let now=support::now(server.peer.origin);
                server.device.tick(now).unwrap();
                data.lock().unwrap().observation=server.device.state();
                if server.connection.poll(&mut server.device,now,|t| server.peer.secure.grant(t).ok()).is_err() {break;}
                continue;
            }
            r=server.peer.receive()=>r,
        };
        let Ok((kind, bytes)) = incoming else { break };
        let sent = match kind {
            Kind::Heartbeat => server.peer.send(Kind::HeartbeatReply, &[]).await,
            Kind::Message => {
                let frame = server.process(&bytes);
                let hold = {
                    let mut s = data.lock().unwrap();
                    s.commands.push(Request::decode(&bytes).unwrap());
                    s.observation = server.device.state();
                    s.hold_reply
                };
                if hold {
                    Ok(())
                } else {
                    server.peer.send(Kind::Message, frame.bytes()).await
                }
            }
            Kind::HeartbeatReply => panic!("unsolicited heartbeat reply"),
        };
        if sent.is_err() {
            break;
        }
    }
    server.connection.close(&mut server.device).unwrap();
    server
        .device
        .tick(support::now(server.peer.origin))
        .unwrap();
    let mut s = data.lock().unwrap();
    s.observation = server.device.state();
    s.device = Some(server.device);
}
