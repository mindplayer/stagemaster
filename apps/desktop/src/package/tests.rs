use super::{Prepared, Service};
use std::sync::Arc;

#[test]
fn cache_replacement_cannot_change_a_previously_captured_package() {
    let service = Service::default();
    assert!(service.prepared(1, "first").is_err());
    *service.prepared.lock().unwrap() = Some(Prepared {
        generation: 1,
        token: "first".into(),
        bytes: Arc::from([1, 2, 3]),
    });
    let captured = service.prepared(1, "first").unwrap();
    assert!(service.prepared(2, "first").is_err());
    assert!(service.prepared(1, "other").is_err());
    *service.prepared.lock().unwrap() = Some(Prepared {
        generation: 1,
        token: "second".into(),
        bytes: Arc::from([4, 5]),
    });
    assert!(service.prepared(1, "first").is_err());
    assert_eq!(&*captured, &[1, 2, 3]);
    assert_eq!(&*service.prepared(1, "second").unwrap(), &[4, 5]);
}

#[test]
fn a_concurrent_package_operation_is_rejected_without_waiting() {
    let service = Arc::new(Service::default());
    let operation = service.operation().unwrap();
    let other = service.clone();
    assert!(
        std::thread::spawn(move || other.operation().is_err())
            .join()
            .unwrap()
    );
    drop(operation);
    assert!(service.operation().is_ok());
}
