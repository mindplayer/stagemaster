use super::*;

#[test]
fn local_reader_contention_is_temporary_unavailability_not_identity_conflict() {
    let message = "后台观察正在读取";
    let failure = Failure::background(ReadFailure::Busy(message.into()));
    assert_eq!(failure.0, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(failure.1, message);
}

#[test]
fn only_the_exact_upstream_read_503_is_temporary_unavailability() {
    let message = "后台请求未成功（503），请核对连接与原回执";
    let classified = ReadFailure::from_reader(message.into());
    assert!(matches!(&classified, ReadFailure::Busy(_)));
    let failure = Failure::background(classified);
    assert_eq!(failure.0, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(failure.1, message);
}

#[test]
fn unknown_and_lookalike_refusals_never_gain_a_frame_hold_or_retry_class() {
    for message in [
        "后台请求未成功（409），请核对连接与原回执",
        "后台请求未成功（403），请核对连接与原回执",
        "后台请求未成功（408），请核对连接与原回执",
        " 后台请求未成功（503），请核对连接与原回执",
        "后台请求未成功（503），请核对连接与原回执 ",
        "后台请求未成功（503），请核对连接与原回执（其他错误）",
        "后台观察正在读取",
        "后台观察响应已过期，请重新读取",
        "后台采样身份、版本或完整性不一致",
        "后台已停止或发生故障，三维暂停显示灯光",
        "后台观察采样没有推进",
        "后台连接未响应；已发送操作须核对原回执",
        "后台响应格式无效",
        "",
    ] {
        let classified = ReadFailure::from_reader(message.into());
        assert!(matches!(&classified, ReadFailure::Refused(_)), "{message}");
        let failure = Failure::background(classified);
        assert_eq!(failure.0, StatusCode::CONFLICT, "{message}");
        assert_eq!(failure.1, message);
    }
}
