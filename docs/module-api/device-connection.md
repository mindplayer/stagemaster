# 主机设备连接接口

依据 [ADR-034](../development/decisions/PRODUCT-ADR-034-device-connection-workspace.md)；实现范围和验收状态见 [DEVICE-001](../development/tasks/DEVICE-001-connection-workspace.md)。macOS 原生 QA 已通过真实发现、搜索取消、连接、保活、主动重连、主机进程暂停后的过期恢复，以及最新构建退出重开后的新会话。不把组件测试、进程暂停或跨平台库的支持范围当整机睡眠／跨平台实测证据。

## 所有权和调用

`stagemaster-device-link::client`：无 I/O、无堆分配的主机请求／回执状态与诊断解码，和固件共用既有协议。`stagemaster-device-host`：应用级连接服务，现有 `Transport` 仍组合扫描／诊断／安装，`Ble` 为原生适配；它不是所有硬件必须实现的统一接口。TRANSPORT-001 已将[应用记录会话](application-record-carriers.md)抽为不依赖蓝牙的模块。Tauri `device_request` 仅转发；`ApplicationHost.device` 是组件唯一宿主入口。[主机安装消息接口](host-installation-io.md)与保活共用同一连接任务，不拥有工程／播放状态，不执行存储或 DMX I/O。

```rust,ignore
// 应用管理一个实例，在 Tokio 运行时内调用。创建／查看不会初始化蓝牙。
let devices = Service::new(Ble::default());
let initial = devices.request(Request::Status)?;
let preparing = devices.request(Request::Scan { epoch: initial.epoch })?;
// 搜索异步进行，最多 8 秒；可以 Cancel 提前结束，清理完成后再连接。
let ready = devices.request(Request::Status)?;
let connecting = devices.request(Request::Connect {
    epoch: ready.epoch, id: chosen_discovery_handle,
})?;
// 页面关闭不调用 disconnect；应用退出负责有界清理。
devices.shutdown().await?;
```

服务析构会请求取消，避免遗留后台心跳；正式应用退出仍应等待 `shutdown` 以获得清理结果。

现有前端主机命令仅 `status`／`scan`／`connect`／`cancel`。`cancel` 用于取消准备／搜索／连接及主动断开；新操作必须带当前 `epoch`。每个扫描／连接意图产生新的代号，过期命令被拒绝；清理结束前禁止复用适配器。修订 `revision` 供界面拒绝迟到状态，重复查询不创建蓝牙会话。查询可能发现连接已过期并启动释放，不会给设备续期。

DEVICE-003 第五增量新增[原生运行入口](native-device-runtime-service.md) `connect_runtime`／`exchange_runtime`／`runtime_snapshot`，复用同一个 Service 和清理任务；安装、运行分别握手，暂未接入前端命令。运行模式使用独立 GATT 服务，不能误入安装维护；固件端实现与实板验收仍待完成。

## 状态与资源

`idle → preparing → scanning → stopping → idle`；选择本次结果后 `connecting → connected`。取消、通信失败或过期进入 `stopping`，清理确认后 `idle` 或 `fault`。清理超时／失败、驱动异常进入 `blocked`，要求退出应用检查系统状态，不能呈现为已断开并立即复用。

- 一个原生适配器，一次扫描或一个待连接／已连接设备。最多 32 个应用候选，超出明确提示；原生系统／第三方库的内部缓存不是本模块可保证的硬上限。
- 初始化与连接分别最多等待 15 秒；扫描从系统确认启动后计 8 秒；一组写入／回执／诊断读取最多 2.5 秒，资源释放最多 5 秒。
- macOS 初始化等待权限时保留同一个 Future，取消仅停止当前等待，重试不会再创建中央管理器；系统权限弹窗不是应用可以撤销的操作。未获授权时不会后台启动扫描。
- 原生适配保留待连接句柄，取消不能只丢弃 Rust Future；必须调用系统取消连接。CoreBluetooth 断开后会移除内部外设，因此连接前通过库的标识检索重新取得原生句柄。仅当平台明确不支持检索时使用发现句柄；其他错误不静默忽略。
- 服务 UUID 筛选并再次校验，连接后严格发现请求写入、回执读取、诊断读取特征。广播名称／系统标识不构成设备身份；不持久化地址、不自动抢连。

## 活性与诊断

协议沿用 [20 字节诊断接口](device-link-probe.md)，当前 GATT 用带响应写入和读取回执。ATT 写入成功不足以判定应用握手成功，必须同时核对版本／类型／会话／序号／结果与心跳周期，再成功读取诊断。缓存回执不匹配时仅重新读取，绝不重复写握手或心跳；错误结果立即失败。

有效握手后每 2 秒发送下一序号心跳。有效回执距今达到 4.5 秒即不再显示在线／继续发送，早于固件 6 秒边界；长调度停顿、休眠或时钟跳变触发释放，恢复必须显式重新连接。单调时钟用于调度，系统时间仅用于保守识别休眠／时钟跳变，不用于授权／节目时间，也不会延长会话。

诊断字段为固件实际回复的自检、禁止输出标志、回绕的启动毫秒与推进计数、128 KiB 配置内部堆占用；失败立即清除实时值。自检失败／未禁止输出如实呈现，不从“已连接”推断正常。时间／计数不是持久节目状态，RSSI 是发现时观测而非持续测量。

## 界面与后续接入

顶部“设备”在无工程时也可用。设备面板独立组件，收起／切工作区保留查询和选择，不提交编辑草稿；Esc 收起并还原入口焦点。状态查询每 0.7 秒，原生保活独立运行；IPC 5 秒无回应即显示状态不可用，避免保留过期在线标志。迟到响应通过修订拒绝。

网页宿主明确不可原生连接。早期诊断固件不提供安装；后续 [DEVICE-002](../development/tasks/DEVICE-002-direct-installation-acceptance.md) 已通过专用免配对 GATT 镜像与真实节目安装验收，无开发配置时仍只诊断。诊断会话不授予节目安装或播放权限。不在此模块重造播放器或云端授权，也不把平台 BLE 库的支持列表视为跨平台验收完成。
