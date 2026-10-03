use super::{
    Fault, Receiver,
    receiver::classify,
    state::{Kind, Life, Work},
};
use crate::dmx::{Clock, Line, Transmitter, deadline::observe};
use core::{
    future::{Future, poll_fn},
    pin::pin,
    task::Poll,
};
use embassy_sync::blocking_mutex::raw::RawMutex;

impl<M: RawMutex> Receiver<'_, M> {
    pub(super) async fn interruptible<F: Future>(
        &self,
        kind: Kind,
        future: F,
    ) -> Option<F::Output> {
        let mut future = pin!(future);
        poll_fn(|cx| {
            let interrupted = self.queue.with(|s| {
                s.wake.register(cx.waker());
                s.interrupted(kind)
            });
            if interrupted {
                Poll::Ready(None)
            } else {
                future.as_mut().poll(cx).map(Some)
            }
        })
        .await
    }

    pub(super) async fn next<L: Line, C: Clock>(
        &self,
        tx: &mut Transmitter<L, C>,
        idle_until: Option<u64>,
    ) -> Option<Work> {
        // Keep one independently waking deadline alive while idle. A completed
        // packet has drained already; expiring here needs no synthetic stop ticket.
        let mut timer = pin!(async {
            if let Some(until) = idle_until {
                tx.clock.wait_until_us(until).await;
            } else {
                core::future::pending::<()>().await;
            }
        });
        let mut expired = false;
        poll_fn(|cx| {
            if !expired {
                let clock = observe::<_, ()>(&tx.clock, &mut tx.last_us);
                let fault = match clock {
                    Err(e) => Some(classify(&e)),
                    Ok(now) => match idle_until {
                        Some(until) if now >= until => {
                            tx.line.disable();
                            expired = true;
                            None
                        }
                        Some(until) if timer.as_mut().poll(cx).is_ready() => {
                            tx.line.disable();
                            expired = true;
                            // Time can cross the deadline while polling the
                            // timer. Re-read instead of blaming a stale sample.
                            match observe::<_, ()>(&tx.clock, &mut tx.last_us) {
                                Ok(after) if after >= until => None,
                                _ => Some(Fault::Clock),
                            }
                        }
                        _ => None,
                    },
                };
                if let Some(fault) = fault {
                    tx.line.disable();
                    expired = true;
                    self.queue.with(|s| s.fail(fault));
                }
            }
            self.queue.with(|s| {
                if s.life != Life::Open {
                    return Poll::Ready(None);
                }
                if let Some(work) = s.take() {
                    return Poll::Ready(Some(work));
                }
                s.wake.register(cx.waker());
                Poll::Pending
            })
        })
        .await
    }
}
