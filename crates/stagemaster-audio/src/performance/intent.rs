use std::sync::atomic::{AtomicU64, Ordering};

/// Host intent, distinct from the request actually applied at an output-frame boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlaybackRequest {
    pub revision: u64,
    pub playing: bool,
}
impl PlaybackRequest {
    pub(super) const fn decode(word: u64) -> Self {
        Self {
            revision: word >> 1,
            playing: word & 1 != 0,
        }
    }
}
pub(super) struct Intent(AtomicU64);
impl Intent {
    pub fn new() -> Self {
        // A raw PerformanceSource plays when consumed; Transport explicitly prepares a paused voice.
        Self(AtomicU64::new(1))
    }
    pub fn requested(&self) -> PlaybackRequest {
        PlaybackRequest::decode(self.0.load(Ordering::Acquire))
    }
    pub fn request(&self, playing: bool) -> Result<PlaybackRequest, &'static str> {
        let update = |word: u64| {
            if (word & 1 != 0) == playing {
                Some(word)
            } else {
                (word >> 1)
                    .checked_add(1)?
                    .checked_mul(2)?
                    .checked_add(u64::from(playing))
            }
        };
        let previous = self
            .0
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, update)
            .map_err(|_| "音频播放请求代次已耗尽")?;
        Ok(PlaybackRequest::decode(
            update(previous).ok_or("音频播放请求代次已耗尽")?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn same_intent_is_idempotent_and_exhaustion_cannot_reuse_a_request() {
        let intent = Intent::new();
        assert_eq!(intent.request(true).unwrap(), intent.requested());
        let paused = intent.request(false).unwrap();
        assert_eq!(
            paused,
            PlaybackRequest {
                revision: 1,
                playing: false
            }
        );
        assert_eq!(intent.request(false).unwrap(), paused);
        assert_eq!(intent.request(true).unwrap().revision, 2);
        let exhausted = Intent(AtomicU64::new(u64::MAX));
        assert!(exhausted.request(false).is_err());
        assert_eq!(exhausted.request(true).unwrap().revision, u64::MAX >> 1);
        assert!(exhausted.requested().playing);
    }
}
