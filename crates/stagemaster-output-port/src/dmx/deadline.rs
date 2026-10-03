use super::{Clock, Error};
use core::{
    future::{Future, poll_fn},
    pin::pin,
    task::Poll,
};

pub(super) fn observe<C: Clock, E>(clock: &C, last: &mut u64) -> Result<u64, Error<E>> {
    let now = clock.now_us();
    if now < *last {
        return Err(Error::Clock);
    }
    *last = now;
    Ok(now)
}

pub(super) async fn bounded<C, F, T, E>(
    clock: &C,
    last: &mut u64,
    deadline: u64,
    operation: F,
) -> Result<T, Error<E>>
where
    C: Clock,
    F: Future<Output = Result<T, E>>,
{
    let mut operation = pin!(operation);
    let mut timer = pin!(clock.wait_until_us(deadline));
    let result = poll_fn(|context| {
        let now = match observe(clock, last) {
            Ok(now) => now,
            Err(error) => return Poll::Ready(Err(error)),
        };
        if now >= deadline {
            return Poll::Ready(Err(Error::Deadline));
        }
        if timer.as_mut().poll(context).is_ready() {
            // A timer firing early violates the clock contract; never spin on it.
            return Poll::Ready(Err(if clock.now_us() < deadline {
                Error::Clock
            } else {
                Error::Deadline
            }));
        }
        operation
            .as_mut()
            .poll(context)
            .map(|result| result.map_err(Error::Line))
    })
    .await;
    // Also check operations that synchronously consumed time before becoming ready.
    if observe(clock, last)? >= deadline {
        return Err(Error::Deadline);
    }
    result
}
