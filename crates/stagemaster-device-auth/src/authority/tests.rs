use super::*;
use crate::{Address, Secret};

#[test]
fn epoch_exhaustion_cannot_wrap_into_an_old_worker_epoch() {
    let local = LocalIdentity::new(
        Address::new(true, [1, 2, 3, 4, 5, 0xc6]).unwrap(),
        Secret::new([1; 16]).unwrap(),
    )
    .unwrap();
    let mut auth = Authority::new(Vault::new(local), 0);
    auth.next_epoch = u32::MAX - 1;
    let last = auth.connect([1; 16], 0).unwrap();
    assert_eq!(last.epoch().get(), u32::MAX);
    auth.disconnect(last);
    assert_eq!(auth.connect([2; 16], 1), Err(Error::Exhausted));
    assert_eq!(auth.connect([3; 16], 2), Err(Error::Exhausted));
    assert!(auth.active.is_none());
}
