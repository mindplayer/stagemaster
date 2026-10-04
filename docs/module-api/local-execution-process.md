# 本机独立执行进程

HOST-004 已在同一程序提供 [v2 多来源入口](multi-source-process.md)，复用本篇的权限、会话和生命周期；以下 v1 单节目接口继续有效。

[HOST-002](../development/tasks/HOST-002-local-execution-process.md)／[ADR-099](../development/decisions/PRODUCT-ADR-099-local-execution-process.md)。`apps/execution-host` 组合工程、包、安装存储与 [HOST-001](runtime-host.md)，通过独立进程拥有运行生命周期。本接口是第一个本机适配，不是全产品冻结 SDK；没有物理输出驱动，也未替换桌面运行链。

## 准备与启动

程序参数依次为工程路径、`scene|sequence`、明确的节目 UUID、全新运行目录、`--software-output`。例如在项目根目录运行：

```sh
CARGO_HOME="$PWD/tmp/cargo-home" TMPDIR="$PWD/tmp" cargo run -p stagemaster-execution-host --locked --offline -- \
  docs/project-format/examples/lighting-basic.project.json sequence \
  00000000-0000-4000-8000-000000000040 tmp/local-execution-run --software-output
```

`tmp/local-execution-run` 必须不存在且父目录存在；再次启动须选新目录，不自动删除或复用旧安装内容。工程只读打开，经现有编译／包校验／双槽安装／载入后才发布发现文件。一次明确选择一个场景或场景列表，不自动播放、不默认选第一个节目。当前沿用 `reference-single-line-v1` 灯光档位，不代表音视频或专业多执行器已接入。

发现文件为运行目录下 `discovery.json`，包含协议版本、每次启动新生成的 `hostId`、本机入口 URL、读取凭据和控制凭据。当前只在具备 POSIX 私有目录权限的系统启用，Mac 已验；目录权限 0700、文件 0600，stdout 不输出凭据。不要把发现文件上传云端或存进工程。重启不恢复正在演出的实例、会话或控制权，残留旧凭据不能控制新进程。

## 本机应用入口

所有请求路径基于发现文件中的 URL（`http://127.0.0.1:端口/v1/hostId`），使用 `Authorization: Bearer 凭据`。读取凭据仅允许 GET；控制凭据可查询和操作，代表同一本机操作者，并未实现多账号权限。禁止 Origin 与重复认证头，不开放 CORS／外部网卡；不得把该接口直接暴露为公网或局域网遥控服务。

| 路径 | 作用 |
| --- | --- |
| `GET /source` | 获取不可变来源报告、选择的节目、步骤与明确的软件执行标识；不返回本机文件路径或秘密凭据 |
| `GET /state` | 最新宿主阶段、故障、执行状态及一路 512 槽软件采样；繁忙返回可重试错误 |
| `POST /sessions` | 新建操作会话，返回 `sessionId` 和首个序号；不自动取得控制权 |
| `POST /sessions/{sessionId}/commands` | 接纳类型化操作，立即返回 pending／complete 记录 |
| `GET /sessions/{sessionId}/receipts/{serial}` | 查询该会话最后一条已接纳操作的状态／结果 |
| `POST /shutdown` | 撤销新请求接入、停止并等待执行线程、关闭监听并清除发现文件 |

命令示例（接纳并不等于已执行）：

```json
{"serial":"1","ttlMs":5000,"command":{"kind":"acquire","durationMs":60000,"takeover":false}}
```

命令类型为 `acquire`、`submit`、`renew`、`release`。`submit` 包含 `expectedRevision` 十进制字符串及 `action`；动作类型为 `start`（必需步骤 UUID）、`pause`、`resume`、`next`、`stop`。无其他参数的类型也拒绝多余字段。不能提交 Grant、Lease、主体身份、加载、维护或逐帧时间。

`acquire` 成功结果带取得控制权当时的历史权威状态，用其 revision 发第一条动作；后续使用各动作回执中的状态，不将延迟观察当作强一致回执。状态投影不暴露内核租约；实例编号、修订、计时与计数均以十进制字符串传输，避免客户端数值精度丢失。

## 回执、控制权与生命周期

会话序号从 1 连续增长，每会话最多一个在途操作，只保留最后一条请求与回执。相同序号和完整类型化负载（含 ttlMs）返回原记录，不重新排队、延期或启动；同号不同负载拒绝。跳号拒绝；旧于保留窗口的序号明确不可追溯，不重新执行。未接纳的格式／容量拒绝不消耗会话序号；接纳后的业务拒绝仍有自己的完成记录。

内核控制序号与会话序号分别维护。收到内核 Receipt（包括业务拒绝）才推进内核序号；排队／期限／租约等未产生 Receipt 的失败不推进。成功重新取得控制权后使用新租约及其首个序号。传输结果不明先查原回执，不能自动新建会话并重新启动节目。

适配层在提交内核之前拒绝动作时，已接纳的会话操作仍以 complete／rejected 结束，保留原 code／message（例如 busy、closed、invalid），不将繁忙或关闭统称为参数非法；不虚构执行状态或媒体 request，也不消耗尚未提交的内核序号。输入控制权 Binding 保持，unknown 仍沿原规则清理。见 [ADR-162](../development/decisions/PRODUCT-ADR-162-adapter-refusal-receipts.md)。HTTP POST 后只读观测返回 503 不代表动作未接纳；只能核对原序号回执和权威状态，不能重发控制。媒体 accepted 与同一 request 的 Applied／Failed／TimedOut 仍分开。

HTTP 响应丢失不取消已接纳处理；有界后台任务持有原 Ticket，等待超时继续等同一回执。明确区分业务拒绝与结果不明；后者须核对状态／重新取得控制权。会话或控制租约失效、客户端关闭、控制客户端被杀都不自动停止已开始的节目。新客户端按策略显式取得／接管控制权，旧控制者的延迟指令不能改变新控制者的运行。

执行进程被操作系统杀死不在上述继续运行承诺内；本轮未提供自动重启、断电恢复正在播放、桌面进程托管、操作系统关机协调或输出节点失联接管。正常关闭会删除发现文件，保留安装目录用于核对；异常进程退出可能留下发现文件，后续启动仍要求新运行目录与身份。

## 预算与适用范围

- 最多 8 个操作会话；无在途工作且空闲满 60 秒回收；未知旧会话不能由调用者重建同一身份。
- 最多 8 个操作处理任务；宿主仍使用既有 32 项队列及每周期最多 8 项的预算。
- 每请求体最多 8 KiB，覆盖查询、会话创建、操作和关闭；请求处理最多等待 3 秒。
- 最多 16 个接受的 TCP 连接，每个连接最多存活 5 秒，包括不完整头／慢体；客户端按需重连，这不改变操作会话或节目实例。
- 命令期限从应用接纳计，最多 5 秒；控制租约最多 60 秒，与商业使用期限无关，遵循 [ADR-101](../development/decisions/PRODUCT-ADR-101-commercial-security-boundaries.md)。

所有 JSON、文件与网络工作位于调度线程之外。来源和软件采样是观察信息，不是真实 DMX 发送确认。当前无生产商业许可、无远程网络身份、无固件／灯具输出；不能因为软件执行策略放行而推断有权驱动物理设备。HOST-002 的进程证据与限制见工单，后续按小步增量接桌面应用入口及第二种设备承载。
