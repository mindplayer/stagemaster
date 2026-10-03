# 节目包交付入口

DELIVERY-001／[ADR-125](../development/decisions/PRODUCT-ADR-125-package-delivery-ingress.md)。`stagemaster-delivery` 是主机侧采集与完整验证模块，核心不绑定界面、账号、云供应商或设备。当前验证状态见工单，不将接口存在等同云端上线。

## 模块与调用

```rust,ignore
let package = stagemaster_delivery::from_file(&chosen_path, expected_identity, &cancel)?;
// 只有需要网络的产品组装启用 stagemaster-delivery/http。
let package = stagemaster_delivery::Http::new()?
    .download(&chosen_https_url, release_identity, cancel_future).await?;
let prepared = stagemaster_device_upload::Prepared::from_package(package)?;
let task = existing_upload_service.start(prepared, connection_epoch, &device_id)?;
// start 仍检查当前真实连接的安装权限、设备身份、执行语义和预算。
// installed 回执不表示正在播放；Runtime 仍独立校验许可与控制权。
```

文件读取／包验证在非实时工作队列执行。网络适配异步收集，完整包校验通过 `spawn_blocking` 执行；界面不自行判断包格式或写安装槽。网络取消 future 完成或调用 future 被丢弃时，丢弃尚未返回的收集结果，不产生安装／激活副作用。文件的 AtomicBool 在有界读取之间及完整验证前后检查，不承诺中断单次阻塞操作系统读取。

`Incoming::new(expected)`、`push(chunk)`、`finish()` 为不同传输的共用收集入口；实际字节超过预算后永久拒绝此收集器，即使调用者忽略错误也不能完成。只有 finish 通过全包版本／结构／语义／摘要／预算校验才返回 `Package`。`Package` 拥有不可变字节和验证后的目录，实现既有 ReadAt；克隆共享原内容，来源文件或网络消失不影响它。`archive()` 只暴露只读目录；没有授权／自动播放方法。

## 身份、信任与边界

身份复用 Identity `{ bytes, digest }`；digest 使用 Archive 摘要算法，不是文件名、下载 URL、ETag 或整文件直接 SHA。HTTPS 必须传独立的预期身份；本地文件可不传，先做结构和自洽完整性验证再交给应用审阅。调用者提供的身份本身不证明可信，后续云发布描述／许可须由可信应用校验；本模块不实现签名、生产身份或 TUF。

HTTPS 只接受无内嵌用户名密码、无片段的 HTTPS 地址；固定连接超时 10 秒、请求及响应体总超时 30 秒。默认校验证书；不自动重定向、无通用账号凭据或持久 Cookie、不自动解压。响应须 200，Content-Encoding 缺失或 identity，声明长度若存在须匹配；实际字节持续受预期长度约束，并要求精确完整。错误消息不带 URL，避免泄露签名查询参数。云供应商地址解析、登录与受验发布描述属于调用宿主；将来对象存储重定向必须先解析到明确目标，不能在此自动携带凭据跨域。

当前档位最多 2 MiB，内存收集与 Arc 冻结转换的载荷峰值约 4 MiB，加已有包验证、网络/TLS 和运行时预算；不是全进程内存上界。新模块默认不启用网络，现有设备上传仅依赖内容部分；固件不依赖此主机模块。大型媒体资源、断点下载、永久缓存和多资源依赖闭合按后续独立档位实现，不能把音视频强塞入这一受限灯光包。

## 安装和运行

交付不改持久安装状态。接收失败／取消不会产生可用 Package；成功后沿[原安装事务](package-installation.md)或[原安装任务](host-installation-task.md)执行。相同身份重复安装返回原提交；新版本只成为安装候选，当前固定快照继续运行。切换仍需既有维护／绑定和显式播放命令；[设备运行层](device-runtime.md)在开始／恢复／推进时调用原 PlaybackPolicy。

本轮不因云端或 U 盘入口实现新的商业到期策略，按 ADR-101 后议。软件验证中的允许／拒绝策略不能当生产许可。
