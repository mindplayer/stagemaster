use axum::serve::Listener;
use std::{
    future::Future,
    io,
    net::SocketAddr,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::Duration,
};
use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    net::{TcpListener, TcpStream},
    sync::{OwnedSemaphorePermit, Semaphore},
    time::Sleep,
};

pub(crate) const MAX_CONNECTIONS: usize = 16;
pub(crate) const CONNECTION_LIFETIME: Duration = Duration::from_secs(5);
pub(crate) struct BoundedListener {
    inner: TcpListener,
    slots: Arc<Semaphore>,
}
impl BoundedListener {
    pub fn new(inner: TcpListener) -> Self {
        Self {
            inner,
            slots: Arc::new(Semaphore::new(MAX_CONNECTIONS)),
        }
    }
}
impl Listener for BoundedListener {
    type Io = Connection;
    type Addr = SocketAddr;
    async fn accept(&mut self) -> (Self::Io, Self::Addr) {
        loop {
            let Ok(permit) = self.slots.clone().acquire_owned().await else {
                continue;
            };
            if let Ok((stream, address)) = self.inner.accept().await {
                return (
                    Connection {
                        stream,
                        deadline: Box::pin(tokio::time::sleep(CONNECTION_LIFETIME)),
                        _permit: permit,
                    },
                    address,
                );
            }
            // Bound retries on resource exhaustion without affecting the execution thread.
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
    fn local_addr(&self) -> io::Result<SocketAddr> {
        self.inner.local_addr()
    }
}
pub(crate) struct Connection {
    stream: TcpStream,
    deadline: Pin<Box<Sleep>>,
    _permit: OwnedSemaphorePermit,
}
impl Connection {
    fn expired(&mut self, cx: &mut Context<'_>) -> bool {
        self.deadline.as_mut().poll(cx).is_ready()
    }
}
fn timeout<T>() -> Poll<io::Result<T>> {
    Poll::Ready(Err(io::Error::new(
        io::ErrorKind::TimedOut,
        "本机连接已超过存活期限，请重新连接",
    )))
}
impl AsyncRead for Connection {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        if self.expired(cx) {
            return timeout();
        }
        Pin::new(&mut self.stream).poll_read(cx, buf)
    }
}
impl AsyncWrite for Connection {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        if self.expired(cx) {
            return timeout();
        }
        Pin::new(&mut self.stream).poll_write(cx, buf)
    }
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        if self.expired(cx) {
            return timeout();
        }
        Pin::new(&mut self.stream).poll_flush(cx)
    }
    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.stream).poll_shutdown(cx)
    }
}
