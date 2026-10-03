# 桌面设备运行接口

DEVICE-003 第八增量，依据 [ADR-134](../development/decisions/PRODUCT-ADR-134-desktop-device-runtime.md)。桌面 `ApplicationHost.deviceRuntime` 调用 Tauri `device_runtime_request`，后者复用 `stagemaster-device-host::Service::runtime_request`；蓝牙与保活仍由原 Service／Ble 唯一持有，节目仍由设备原 Runtime／ManagedWorker 推进。

## 职责与身份

- JSON 请求定义在 `crates/stagemaster-device-host/src/runtime_ui/command.rs`，回复在 `view.rs`；前端对应 `apps/ui-prototype/src/device-runtime-types.ts`。JSON 仅为本机应用接口，不替代 [SMRT](device-runtime-wire.md) 或增加一套网络协议。
- `epoch` 是原连接服务的 u32 代次。全部 u64 请求号、修订、实例、租约与时间采用十进制字符串，不能转成 JavaScript Number；输入拒绝前导零、符号、空白及溢出。16 字节标识使用 32 位十六进制，摘要为 64 位十六进制。设备发现标识 `id` 仍是平台原始字符串，不能将其当成密码或授权。
- 运行连接的期望权限从已加载的可信本机配置生成 `ExpectedAccess`，不反序列化前端权限位。无运行范围的旧配置不能打开运行连接。期望范围不是权限授予，实际安全会话和设备授权继续逐次检查。
- `peer` 表示当前运行连接事实；`lastResponse` 是带 `connectionEpoch` 的历史；`reply` 是此次操作确认；`pending` 表示仍有未确认意图。历史回复不能证明当前在线，也不能证明物理发送。

## 调用入口

| 请求 kind | 输入 | 行为 |
| --- | --- | --- |
| `connect` | epoch、设备发现 id | 请求明确运行连接，立即返回新代次快照；连接仍可能进行中，须观察原设备状态和运行 peer。无自动取权、载入或执行。 |
| `snapshot` | epoch | 本机只读，不发送设备命令；断线后可读取仍保留的历史／未确认状态。 |
| `refresh` | epoch | 原生发送状态读取，返回当前设备确认状态。 |
| `catalog`／`step` | epoch、revision、index | 按确认修订逐项读取节目或已载入节目的步骤；目录变化时拒绝。 |
| `apply` | epoch、revision、action | 按下表转成原运行操作；不新增播放器或控制序号所有者。 |

| action.kind | 附加字段／效果 |
| --- | --- |
| `acquire` | `takeover` 必填；普通取得与接管明确分开。 |
| `renew`／`release` | 续期或归还原租约，前端不传期限。 |
| `select` | `program: {kind: "scene" 或 "sequence", id}`；选择不等于载入。 |
| `load` | 载入设备所选节目，不开始执行。 |
| `start` | `step` 是明确起始步骤标识。 |
| `pause`／`resume`／`next`／`stop` | 调用原暂停、继续、下一步、停止语义。 |
| `beginMaintenance`／`cancelMaintenance`／`finishMaintenance` | 进入静默准备、取消准备或结束维护；确认静默属于设备内部，不接受界面伪造。 |

请求和动作均拒绝未知字段。没有附加参数的动作采用严格的空结构体变体，避免带标签的 Serde 单元变体忽略额外字段。确认业务失败以状态回复中的 `error` 返回，不当成通信中断；传输／本地准入失败返回原 `Problem`。

## 界面协调

`DeviceCenter` 持有连接模式与 `useDeviceRuntime`，`DeviceDiscovery` 负责发现选择，`DeviceRuntime` 展示节目与操作。面板隐藏不卸载草稿；切换连接或设备会清除旧运行选择。浏览器宿主没有运行端口，不模拟蓝牙能力。

状态、目录和操作串行协调；完整分页成功后发布目录，取消会等待当前在途请求结束并丢弃其结果。读取失败、未读完整与真实空目录分别提示，保留明确的刷新入口。节目筛选不悄悄更换选择，隐藏选择有提示；已绑定包、设备所选、已载入、当前步骤、起始步骤、运行实例及物理发送分别展示。

本次显式取得的租约与设备状态仍匹配时，界面按设备时间至少间隔 20 秒续期；原生期限不超过剩余固定准入期限减 1 秒传输余量，也不超过 60 秒。设备是最终裁决者。此为运行控制租约及开发准入，并非商业限时策略；不因续期延长底层配置期限。收起面板继续协调，整个界面销毁后不再续期，节目仍自主运行。重连和观察不自动抢占或恢复旧租约。

发送或状态读取失败后立即禁止新操作；上次状态明确标记为历史读取，重新取得有效状态后才恢复。传输中断不声称节目停止，不自动重放不确定命令。现场执行不提供虚假撤销；工程编辑历史完全独立。物理输出仅在诊断明确禁止时显示禁止，否则显示尚未取得发送回执。

## 验证范围

原生 JSON 集成经过真实安全会话、原安装包、Service 和设备工作器，覆盖目录／载入／执行、维护、修订、严格解析、全宽 u64 字符串、可信配置与旧配置拒绝。前端纯函数覆盖迟到回复、控制权与续期；隔离页面使用正式组件验证搜索、上下文、接管取消、读取故障、取消与断线，不连接真实设备。实际计数、日志与打包记录见 [DEVICE-003](../development/tasks/DEVICE-003-remote-runtime.md#第八增量桌面设备节目操作)。

正式原生入口已编译并打包。本增量没有刷入新固件或完成实板 GATT／UART DMX；软件状态不可当成灯具已经收到信号。后续沿同一接口实机验收，不创建另一个执行引擎。
