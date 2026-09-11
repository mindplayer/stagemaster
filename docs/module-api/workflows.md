# 调用链、异常和生命周期

本文件规定伪 API 0.3 的模块协作。参数类型见 [contracts.ts](contracts.ts)，调用样例见 [examples.ts](examples.ts)，内部构造与 trait 见 [rust-modules.md](rust-modules.md)。外部控制／监看及硬件接口另见本目录索引，均为待实现设计。

## 1. 本机与跨设备连接

```text
React 组件／控制面
  → StageClient 的 ProjectApi／ControlApi 等代理
  → RpcTransport：本机 IPC 或远程连接
  → 服务端认证、协议协商、schema 与大小校验
  → 经过授权的应用服务方法
  → DTO 结果／任务回执／订阅事件
```

`connectStage` 完成握手才返回 StageClient。连接信息包含服务实例、协议、支持操作和请求限制；不支持的调用在接纳前返回明确错误。客户端可以适配布局或隐藏不可用入口，服务端仍必须验证权限和能力。

本机和远程使用同一请求含义，不保证相同延迟。业务代码既不持有服务端 Project 对象，也不直接调用跨机的 Rust 引用。代理中的 Promise 和迭代器留在客户端；线上只有 DTO。读取 input-resource 或 read-resource 时验证 token 所属节点及会话，外部节点不能仅凭 ID 访问本机路径。

鉴权身份由接入层建立。客户端提交的品牌 ID、run、租约 DTO、能力概览和快照说明均要与服务端权威记录比对；TypeScript 类型不会阻止恶意请求或陈旧数据。

## 2. 从编程到现场输出

| 顺序 | 调用 | 返回与后续责任 |
| --- | --- | --- |
| 1 | `fixtures.importProfile`／`assets.ingest` | 后台任务；完成后得到不可变档案／素材引用 |
| 2 | `projects.create`、`projects.edit` | 原子工程修订，保存状态独立；同批内先解析创建对象再验证引用 |
| 3 | `sessions.openBlind`、`sessions.edit` | 独立会话修订；选择／Preset 调用不自动改工程或真实输出 |
| 4 | `sessions.snapshot` | 固定活动值、引用、选择上下文与项目来源的 token |
| 5 | `projects.edit(record-cue)` | 校验快照与基准工程兼容后记录；成功才产生新工程修订 |
| 6 | `bindings.create/edit` | 映射灯具连接段和外部控制目标；监看布局由独立配置保存 |
| 7 | `projects.save` → `jobs.wait` | 得到明确 saved 修订；保存失败仍显示未保存，不改变既有现场计划 |
| 8 | `targets.domains/inspect`、`builds.compile` | 指定 domainId，服务端锁定该域执行能力与引用；任务成功才得到 BuildArtifact |
| 9 | `deployment.prepare` | 检查运行游标、资源和端点，完成预读、内存预算与资源预留 |
| 10 | `deployment.activate` → `jobs.wait` | 核对目标／domainId／expectedRun 后切换该域，成功得到 ActiveRun；外部执行另查 |
| 11 | `control.acquire`、`control.submit(go)` | 范围租约＋有序命令，先返回 ControlTicket |
| 12 | `control.outcome` | 返回 applied 才表示运行核心已应用；实际设备反馈另查询 |
| 13 | 宿主 `RuntimeKernel.tick` → 编码 → `OutputPort.try_submit` | UI 不参与 tick；端口异步发送并报告接纳／写入／反馈层级 |

Prepare 资源过期或相关 executionCapabilityRevision 改变时，需要重新准备，不复用旧 token。监看／面板能力的独立变化不应让执行产物失效。Activate 的 expectedRun 已变化则拒绝，不抢占其他新计划。一个准备句柄只能按契约激活一次；相同 commandId 的重试返回同一结果。跨节点部署逐域协调，不将本接口当作全场原子提交。

示例先保存再激活用于闭环演示；用户现场推杆和 Programmer 操作不必等待保存。结构性计划切换与运行时参数控制是不同通路。应用服务返回错误时保留原因和作用范围，不能把整条链折叠成一个含义不明的 `success`。

## 3. 快速现场控制与手动接管

`control.acquire` 取得某个 Playback／Programmer／同步组的控制租约。`submit` 带租约、单调序号、必要版本前置条件和类型化动作。网关检查范围与类型后转换为紧凑命令，运行核心在应用边界再次核查有效代次和过期状态。

连续电平可按同一控制量合并为最新待应用值；离散 Go／Release 按顺序接纳。超量应明确返回 QUEUE_FULL，不无限积压。长操作按服务给出的有效期 renew；到期不会由后台网络连接自动续租。

用户操作权和输出端口占有权分别管理。`relinquish` 不自动 Release 已运行 Playback；不同控制模式在到期或断连时采用各自政策。计划管理器才可按授权流程获取／交接物理输出资源。

Session 的 Live 绑定由 `control.submit(bind-programmer)` 创建，不通过改 Session DTO 字段开启真实输出。关闭 Live 会话时必须按策略解除贡献，并报告未确认的解除；不能简单删除内存后假定现场已经清除。

## 4. 背景视频定位与独立预演

先取得 TransportCursor 和对应租约，再提交本地组 `seek`。协调器检查外部定位能力、准备本地状态并按策略投递外部动作；新的 TransportGeneration 只影响目标组。本地 applied 不代表外部定位成功，逐目标读取 external.outcome/state；不支持的定位不能偷偷模拟成 Stop＋Start。

只取消该同步组旧位置的任务。背景视频 Seek 不改变独立音乐组、工程修订、其他执行域计划或所有端口租约。消息到达时游标已过期则返回 STALE_CONTEXT，不猜测用户是否仍想定位。

`preview.open/seek` 使用独立 PreviewRef；它没有 ActiveRun、ControlLease 或 OutputLease。预演复用语义和参考素材，但不能成为真实输出对象。现场状态来自 ObserveApi 投影，实时声音／画面另经 MonitorApi 接收；监听默认静音，不自动重复播放主扩音频。

## 5. 请求重试、结果查询与订阅

### 请求结果未知

每个新逻辑操作使用新 commandId；相同操作重试必须使用原 ID 与原负载。网络 requestId 只是某次通信尝试，不替代 commandId。服务端按主体和命令作用域记录负载摘要及结果。

遇到超时：

1. 保留原请求和 commandId，不立即创建新命令。
2. `requests.inspect(commandId)` 查询该服务实例的接纳记录。
3. `processing` 继续观察；`job` 按原操作的结果 schema 还原 JobRef 并查询；`control` 查询 ControlTicket；`completed` 经对应解码器校验后使用。
4. `not-seen` 只反映查询时尚未看到，原请求仍可能在途中；若策略允许，只能在同一幂等作用域内重发原命令，不能换 ID。
5. `retention-expired`、服务重启或外部动作反馈未知时，先对账当前状态；没有证据不能声称未执行或跨故障恰好一次。

0.2 新增 `external` 恢复结果，携带 ExternalTicket；通过 external.outcome 查询发送／对端确认／状态报告。本地 commandId 去重不保证外部接收端恰好执行一次，peer-acknowledged 也不能当作实物输出证据。

JobRef 的泛型只是本地编译期提示。通过结果查询找回 jobId 时必须检查 method 和 resultSchema，不能强转为任意结果类型。取消请求与观察超时都不意味着任务已取消；激活、上传等任务需报告实际终态。

### 订阅恢复

`observe.snapshot` 给出完整状态和游标，随后用该游标 `subscribe`。服务端需要保留两次调用之间的有限增量；已超出窗口则返回 resync-required，不能默默漏事件。

每个 delta 带 base／next 游标。客户端只在 base 匹配时应用；否则重新获取完整快照。服务端队列有上限，慢客户端被要求重同步；不能让一个断线平板积压无界状态。订阅的关闭／结束只清理订阅资源。

当前草案 delta 展示 Playback 投影；以后补其他投影时使用独立 schema、过滤与大小限制。不可将任意巨型内部状态对象序列化作为通用“变化事件”。

## 6. 云端上传和设备分发

```text
本机 projects.save → exports.deployment
  → LocalPackage（清单＋本机读取能力）
云端 openUpload(manifest)
  → UploadSession（所属云服务＋受限传输授权）
本机 transfers.upload(resource, uploadSession)
  → 实际传输文件／素材，不经过实时核心或 UI JSON 大帧
云端 verifyUpload → publish
  → 不可变 PackageId
云端 assign(publishedDeployment, deviceId)
  → desired
设备下载与校验 → 本机 PlanManager.prepare → 按激活政策 activate
  → 设备报告 downloaded / prepared / active
```

服务端验证清单哈希、全部必需文件、权限与目标兼容性。manifestHash 对规范化清单有效载荷计算，排除该哈希字段本身及传输临时信息。输出包固定工程、构建、绑定与资源，不携带凭据。

只交换可编辑工程时使用 exports.project，不需要 BuildArtifact 或现场设备；可复用上传／校验流程，但不进入 assign。publish 返回带包类型的结果，客户端检查 deployment 后才请求设备分配；服务端仍从权威清单验证。工程分支同步、场地／库资源交付另有工作流，不能用设备执行包替代。

本地必需资源与外部系统依赖分别记录。控制包下发完成不代表外部播放器／媒体服务器已经收到或载入素材；外部就绪独立检查，不用参考代理代替正式资源。

上传 grant 由已配置可信云服务签发并绑定范围、大小、时效和主体；本机传输模块按 owner 找到授权连接，不接受任意陌生 URL 或凭一个 ID 访问别人的存储。上传完成仍不等于云端验证成功。

数据库事务不能覆盖对象存储和设备激活，步骤需要独立状态、幂等和清理。`assign` 只记录期望版本；远程设备离线时显示最后上报状态，不能先把 active 改成目标版本。包清理不得删除活跃／待激活资源。

## 7. 关闭、失败与资源回收

| 资源／操作 | 结束规则 | 不应隐含发生的操作 |
| --- | --- | --- |
| 客户端连接 | `disconnect` 终止本客户端请求／订阅；控制租约按规则到期 | 停止引擎或释放全场 |
| 会话 | `sessions.close`；已签发快照按期限或持有关系回收 | 修改已存 Cue；忽略仍绑定的 Live 贡献 |
| 准备计划 | `discard` 或到期，工作层释放预留资源；激活后转运行所有者 | 丢弃当前有效运行计划 |
| 控制租约 | relinquish／到期／显式接管 | 把它当成物理端口所有权 |
| 媒体帧 | 消费完成／栅栏确认后归还；失败返回原所有权 | 在实时线程析构大缓冲或释放尚在播放的资源 |
| 运行计划 | 新计划切换并收到回收确认后由工作层释放 | 内存句柄刚替换就回收全部旧资源 |
| 任务／请求记录 | 保留到声明期限；终态清理与活跃资源分开 | 过期后假装仍可可靠去重 |

清理失败作为独立诊断保留，不覆盖原操作失败原因。示例中的 awaitJob 有等待上限，产品还应提供用户取消观察和恢复入口；等待上限不取消任务。真实实现需要验证 lease、引用计数、驱动缓冲与故障重启的一致性。

## 8. 接口验收与开发交接

外部控制按“读取目录／上下文 → 只读预检 → 获得范围租约 → 提交类型化动作 → 查询回执／状态”执行。租约需在真正发出动作前再核查，失联重连时上下文代次变化，旧动作不自动补发。

手动租约绑定 OperatorSessionRef，节目驱动租约绑定 TransportCursor；Seek 后旧自动动作失效，但不取消其他同步组或独立手动操作的合法会话。已发送动作可能无法撤回，不能以本地队列清理伪报撤销成功。硬件 attach 只依赖操作会话，启用运行映射时才解析 ActiveRun；空节点可返回 idle 快照。

监看按“查询源及取点 → 协商 viewer／质量 → open → 本地 attach（默认静音）→ 收到并呈现数据 → 本地 close＋服务端 close”执行。长会话显式续租；viewer 和监听设备只能属于当前客户端。关闭监看不发 Stop，API 在线但媒体断流分别呈现。硬件控制面按 attach／基准同步／映射屏障／有序输入／权威反馈／detach 协作，连接本身不赋予业务控制权。

保存的监看布局只引用 MonitorSourceKey，重连时 resolve 到当前 MonitorSourceRef 后再 open；缺源时保留布局并显示不可用，不改写灯光执行绑定。更多启动与交换样例见 [evolution-examples.ts](evolution-examples.ts)。

每个模块交付必须附：调用方、构造依赖、状态所有者、输入输出、错误码、执行上下文、权限／作用域、幂等／版本、资源生命周期与验收场景。

首批联调要验证：同一用例经本机和远程代理结果一致；陈旧修订被拒绝；在声明的本地去重范围内重试不重复 Go，外部结果未知时不自动重发；视频 Seek 不影响别组；预演不能调用真实输出；保存／编译乱序不覆盖新状态；帧／任务队列有界；未知高级内容不被简化端破坏。

类型检查只覆盖静态误用。服务端仍需契约测试、真实网络故障注入和虚拟时钟测试；实际硬件最后按档位验证。当前没有这些运行结果，不能把伪 API 文件数量当作模块化已经完成。
