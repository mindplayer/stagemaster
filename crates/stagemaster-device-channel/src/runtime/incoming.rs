use super::RuntimeClient;
use crate::{Error, RecordIo, wire};
use stagemaster_runtime_protocol::Response;

impl<R: RecordIo> RuntimeClient<R> {
    /// Consume at most one message. A late exact duplicate of the last reply changes nothing.
    /// Business failures are returned as replies, not transport failures or implicit retries.
    /// # Errors
    /// Wrong request/session/boot, unsolicited data and malformed replies close the channel.
    pub fn receive(&mut self) -> Result<Option<Response>, Error> {
        let result = (|| {
            self.check()?;
            let Some(bytes) = self.channel.receive()? else {
                return Ok(None);
            };
            let response = Response::decode(&bytes).map_err(wire)?;
            if self.last.as_ref() == Some(&response) {
                return Ok(None);
            }
            let request = self
                .pending()
                .ok_or_else(|| Error::Protocol("收到未请求的设备运行回复".into()))?;
            response
                .correlate(request, self.peer().ok_or(Error::Closed)?.peer.boot)
                .map_err(wire)?;
            self.pending = None;
            self.last = Some(response);
            Ok(Some(response))
        })();
        self.checked(result)
    }
}
