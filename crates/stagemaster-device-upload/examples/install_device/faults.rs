//! Controlled fault injection before encryption / after a real NOR commit.
//! Does not alter the immutable source file or the device authorization path.
use super::{Ble, Duration, Instant, Link, Request, Result, connect, settled};
use stagemaster_install::{Commit, Phase};
use stagemaster_transfer::{Action, Command, Frame, Outcome, RemoteError, Response, Upload};

fn damage(frame: &Frame) -> Result<Option<Frame>> {
    let request = stagemaster_transfer::Request::decode(frame.bytes())?;
    let Action::Write {
        transaction,
        offset,
        bytes,
    } = request.action
    else {
        return Ok(None);
    };
    if offset < 1024 {
        return Ok(None);
    }
    let mut damaged = bytes.to_vec();
    damaged[0] ^= 1;
    Ok(Some(
        stagemaster_transfer::Request {
            action: Action::Write {
                transaction,
                offset,
                bytes: &damaged,
            },
            ..request
        }
        .encode()?,
    ))
}
async fn reconnect(link: &Link<Ble>, epoch: u32, locator: &str) -> Result<u32> {
    link.request(Request::Cancel { epoch })?;
    let idle = settled(link).await?;
    Ok(connect(link, idle.epoch, locator).await?.epoch)
}
pub(super) async fn exercise(
    link: &Link<Ble>,
    mut epoch: u32,
    bytes: &[u8],
    mode: &str,
    locator: &str,
) -> Result<()> {
    let mut upload = Upload::new(bytes)?;
    upload.connect(
        link.installation_peer(epoch)?
            .ok_or("缺少安装权限")?
            .session,
    )?;
    let deadline = Instant::now() + Duration::from_mins(2);
    let mut before: Option<Commit> = None;
    let mut committed = None;
    let mut damaged = false;
    let mut rejected = false;
    while let Some(mut frame) = upload.outbound()?.cloned() {
        if Instant::now() >= deadline {
            return Err("故障验收超过固定期限".into());
        }
        if mode == "corrupt"
            && !damaged
            && let Some(changed) = damage(&frame)?
        {
            frame = changed;
            damaged = true;
        }
        let result = link.exchange_installation(epoch, frame).await?;
        let response = Response::decode(result.bytes())?;
        if before.is_none() {
            before = response.state.head;
            assert!(before.is_some(), "必须已有可保护的节目");
            assert_ne!(
                before.unwrap().identity,
                upload.identity(),
                "须使用另一份包触发实际写入"
            );
            println!("故障前有效节目 {:?}", before.unwrap());
        }
        if response.command == Command::Verify && mode == "corrupt" {
            assert!(damaged);
            assert_eq!(response.result, Err(RemoteError::Package));
            assert_eq!(response.state.head, before);
            assert_eq!(
                response.state.progress.ok_or("缺少失败状态")?.phase,
                Phase::Failed
            );
            assert!(upload.accept(result.bytes()).is_err());
            upload.request_cancel();
            rejected = true;
            continue;
        }
        if response.command == Command::Commit && mode == "lost-commit" {
            assert!(committed.is_none(), "恢复不得再次提交");
            response.result?;
            committed = response.state.head;
            // Intentionally withhold the authentic device response from Upload.
            // Reconnection must query durable state rather than assume failure.
            epoch = reconnect(link, epoch, locator).await?;
            upload.disconnect();
            upload.connect(link.installation_peer(epoch)?.ok_or("重连无权限")?.session)?;
            continue;
        }
        upload.accept(result.bytes())?;
    }
    if mode == "corrupt" {
        assert!(damaged && rejected);
        assert_eq!(upload.outcome(), Some(Outcome::Cancelled));
        assert_eq!(upload.state().ok_or("缺少结果")?.head, before);
        println!(
            "PASS: 篡改包被实板拒绝，失败事务已取消，原节目保持 {:?}",
            before.unwrap()
        );
    } else {
        let committed = committed.ok_or("未实际触发提交回执丢失")?;
        assert_eq!(upload.outcome(), Some(Outcome::Installed(committed)));
        assert_eq!(committed.generation, before.unwrap().generation + 1);
        println!("PASS: 丢提交回执后重新认证，对账恢复为 {committed:?}，没有重复提交");
    }
    Ok(())
}
