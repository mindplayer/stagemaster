# 桌面后台执行

依据 [ADR-109](../development/decisions/PRODUCT-ADR-109-desktop-background-execution.md)，实施记录 [HOST-005](../development/tasks/HOST-005-desktop-background-execution.md)。此入口接实际 [v2 多来源进程](multi-source-process.md)，仍为单域软件输出，不会自动接物理设备；[后台预演](background-previsualization.md) 通过显式只读来源接 UE。

## 职责与接口

`stagemaster-execution-client::Client` 不依赖 Tauri，负责私有本机发现、能力和身份检查、命令序号、回执与观察。`apps/desktop/src/execution` 管理不可变工程快照、后台进程和运行记录；界面只调用 `ApplicationHost.execution(request)`。工程、编辑预演、后台节目各自拥有生命周期。

```text
Client::open(discovery_path) -> Client          // 仅观察，不创建控制会话
client.refresh() -> View                       // 核对原回执，再读取状态
client.acquire(takeover) -> View               // 首次创建会话；接管须明确
client.apply(host_id, revision, source, action) // 执行、暂停、继续、下一步、停止、电平、语义手动批量
client.output(host_id, revision, action)        // 组级总亮度／熄灯；同一控制权与回执
client.release() -> View                       // 归还输入权，不停演
client.maintain() -> View                      // 观察、必要时续约；不驱动播放器
client.shutdown()                             // 明确结束整个后台
```

这些方法均为异步调用；`View.pending` 表示结果仍待确认，成功返回 HTTP 不等于节目动作已生效。界面请求：`snapshot`、`prepare { generation, selection }`、`reconnect`、`acquire { takeover }`、`release`、`apply { hostId, revision, source, action }`、`output { hostId, revision, action }`、`shutdown { hostId }`。

准备选择 1–63 个真实场景／列表，另加一个较高优先级的手动层，仍受来源组累计预算限制。生成固定快照再启动同一后台二进制；载入不自动播放。手动层已按 EXEC-007 接入选灯、连续属性／已定义功能的原子应用、指定属性释放和整层确认释放，保留原电平控制。

## 当前步骤与阶段进度

[EXEC-005](../development/tasks/EXEC-005-source-execution-progress.md)／[ADR-149](../development/decisions/PRODUCT-ADR-149-source-execution-progress.md)：原 `Player::progress()` 只读给出延时／渐变／自动等待／人工保持、步内时间和后续步骤，来源组与后台快照／回执沿原链投影。暂停冻结，结束保留末步；媒体来源继续使用原音频同步状态。

节目卡片就地显示当前／下一步、阶段和剩余秒数，有确定时长才有进度条。起始步骤和电平草稿不随观察变化；末步无后续时禁用下一步，循环末步标出回到首步。无控制权仍可观察；读取失败明确标出最后已知步骤、停止显示有效进度。旧主机缺少进度时明确不可用，TS 不自行计时或触发命令。

## 节目浏览与常用顺序

[EXEC-006](../development/tasks/EXEC-006-execution-board-navigation.md)／[ADR-150](../development/decisions/PRODUCT-ADR-150-execution-board-navigation.md)：搜索、类型／状态／常用筛选和显示顺序仅作用于 UI，不提交运行命令或改变混合优先级。筛选保留原卡片与输入，隐藏时禁用执行并取消旧重新执行确认；筛选外运行／暂停及未应用输入有显式入口。切到编辑预演／步骤编排后后台组件保留、停止观察轮询，重新显示先刷新状态；新 hostId／layout 丢弃旧草稿。

常用偏好为本机 `stagemaster.executionPins.v1`，最多 20 工程、各 16 个语义来源键；按后台 projectId 与原场景／列表身份保存，不按随机 sourceId。尚未载入的固定项保留并可显式清理；写入失败仅影响本机持久化，不影响节目运行。状态错误显示最后已知概览，不伪造全部已停止。

[EXEC-018](../development/tasks/EXEC-018-observation-binding-lifecycle.md)：窗口内观察冻结适配连接与显示代次，切换连接、隐藏重显或卸载后的旧成功／错误不能更新新绑定。每次绑定独立观察，同一端口仍共享单在途串行入口，A→B→A 不复活第一次 A。显式动作等待原读取后再次检查绑定才发送；已发到后台的动作不因此取消、回滚或重发。重新显示只有取得新观察才启用执行／输入权按钮；刷新、重连与明确关闭保留原故障恢复语义。Rust 控制权和时钟不变，此协调不是新的宿主接口。

## 手动编程与实际持有

[EXEC-007](../development/tasks/EXEC-007-live-manual-programmer.md)／[ADR-151](../development/decisions/PRODUCT-ADR-151-live-manual-programmer.md)：后台固定目录提供灯具、属性与功能定义，客户端验证能力、唯一身份、定义区间与容量。`Action::Patch { changes: Vec<ManualEdit> }` 使用 `Normalized`／`Function`／`Release` 三种明确值；发送前校验目标与完整请求体，超限不接纳序号，也不自动分批。

新的可选 `manualOwnership` 能力要求每个手动来源提供 `held: [{fixtureId, attribute}]`，包括空数组。真实持有来自原混合器的固定 512 位掩码，再由固定配适映射投影；零值和零电平仍可持有。EXEC-008 的可选 manualValues 进一步报告电平前设定值，最终合成数值仍另属输出观察；输入明确为“待设置”，不能把空白当作实际为零。

按共同属性交集批量编辑；完整功能定义不同的定制轮盘不能直接同选。搜索保留筛选外选灯；有未应用输入时锁定目标和属性，取消不提交运行命令，失败保留草稿。确认回执只清除对应提交的输入。整层释放单独确认，隐藏视图取消确认。运行修改不写工程、不进入工程撤销；共同亮度手势、来源电平推子与手动值录入新场景见下节，其余属性连续手势及释放渐变另行实施。旧宿主可继续普通节目操作，未声明能力时不提供新的手动编辑。

## 身份、回执与失败

- 发现文件及父目录只允许本机用户访问，不接受符号链接、远程地址、代理或重定向；只连接精确的 `127.0.0.1` v2 启动地址。连接信息不返回 TypeScript。请求 8 KiB、响应 8 MiB、连接／请求期限有界。
- 请求发出前保留待确认序号，取消或响应丢失不能释放它；只 GET 原回执，不自动重发。没有可确认回执时阻止后续修改；重新连接建立新的观察客户端，原操作不重放，控制权重新明确取得。
- 确认回执的运行状态优先于较旧的已发布状态；循环计数仍是观察采样的真实计数，不冒充新调度周期。新启动／布局不匹配则拒绝，旧界面的来源操作携带启动身份和预期修订。
- `View.record` 保留最近显式操作的回执，自动续约不覆盖它；协议当前回执与序号仍由原客户端独立维护。`pending` 包含任何未确认事务，后台观察始终来自最新权威状态。重新连接清空旧记录，不重放操作。
- 控制租约 60 秒，桌面服务每 10 秒检查，剩余不足 30 秒时按相同回执路径续约。其含义是输入权限，与商业限时无关；退出应用或到期不会停演。
- 管理服务使用跨进程文件锁保护当前运行记录。后台从创建运行目录开始持有 `lifetime.lock`，启动及播放期间均保持。只接受 Child 终态或实际取得原锁作为结束证据；HTTP 失联不能触发清空和重启。
- 明确关闭先发送原 shutdown，再等同一进程终态；未结束时保留关闭状态及原记录。后台断线、应用重启、编辑切页和工程修改均不自动停演。

## 本机存储与构建

开发数据在项目 `data/execution/`；正式应用使用已有应用数据根目录。`current` 仅保存运行 UUID，快照、来源清单、日志和后台私有文件在该 UUID 子目录。当前保留已结束运行资料用于诊断，自动保留数量策略后续另设；不是用户工程备份策略。

桌面入口 `tools/desktop/run.mjs` 串行构建后台，并通过 Tauri `externalBin` 打包。直接 Cargo 编译桌面前需执行 `node tools/desktop/prepare-host.mjs`；不在 Rust 构建脚本内递归启动 Cargo。运行只启动随应用交付的固定文件，不向前端开放 shell 权限。

仅调试构建支持 `STAGEMASTER_ACCEPTANCE_INSTANCE` 的字母数字／连字符实例名，将所有桌面数据定向到项目 `tmp/desktop-<name>/`，供独立原生验收；生产路径不使用此选项。

## 本次边界

入口位于“执行步骤 → 执行视图 → 后台执行”，可检索／多选载入，读取固定来源和步骤，明确控制、独立操作并关闭后台。编辑预演保留旧音乐／草稿语义；两者切换不让旧键盘操作控制后台。后台卡片显示权威步骤和阶段时间；编辑预演总控仍不作为后台总控。

后台只读预演已按 PREVIS-003 接入；远程设备、物理输出、多时钟、完整光学、云端分发和生产安全仍按框架主线实施。若应用在创建启动记录与操作系统启动之间异常退出，且后台尚未留下生命周期证据，当前保留记录、禁止自动重启；专门的异常启动记录恢复工具后续补充，不能把连接超时视为进程死亡。

## 手动设定值监看

[EXEC-008](../development/tasks/EXEC-008-manual-value-observation.md)／[ADR-152](../development/decisions/PRODUCT-ADR-152-manual-value-observation.md)：`manualValues` 依赖 `manualOwnership`；手动来源的 `heldValues: u16[]` 与 `held` 完全同序、同长度。值来自原 Session 的手动贡献，经过原功能编码量化、尚未乘来源电平，0 与释放不同。普通来源无该字段，空手动层是两个空数组。客户端验证目标、长度、8 位扩展／16 位原值、功能区间及固定槽代表值，损坏状态整体拒绝，旧宿主缺能力时明确数值不可用。

界面汇总所选共同属性的一致原始值／不同值／部分未持有，轮盘按各灯定义显示功能与实际区间位置。明细搜索分页并显示精确原始值；零电平保持设定、只读可查看、观察失败标记最后已知，输入草稿不随观察更新。设定值不是最终获胜来源或灯具物理反馈。

## 手动值录入场景

[EXEC-009](../development/tasks/EXEC-009-manual-scene-recording.md)／[ADR-153](../development/decisions/PRODUCT-ADR-153-manual-scene-recording.md)：手动层可按全部已持有属性或所选灯具采集并录入新场景。使用新鲜权威观察中的电平前值，未持有的属性不补默认值；零值保留，功能按原定义反解并精确重编码。采集不要求控制权，超时／失联／待确认操作时拒绝新采集。未应用的界面输入也须先应用或取消。

```text
ApplicationHost.manualCapture({
  kind: "capture", generation, hostId, source, selected: null | fixtureIds
}) -> { generation, token, sourceName, revision, readings, fixtures }
ApplicationHost.manualCapture({ kind: "cancel", token }) -> null
ApplicationHost.project({ kind: "recordManualScene", generation, token, name })
```

捕获器读取后台固定工程，核对布局摘要；共享 Rust `Document::capture_manual_scene` 生成不可变 `ManualSceneCapture`，最大 512 项。记录绑定工程 ID、所涉灯具／档案语义／校准／配适，允许不影响这些内容的其他场景或名称变化。功能原始值不能精确反解时拒绝，不猜测。

宿主只存一个 5 分钟的随机票据；前端仅审阅冻结值，确认不提交自构造数值。捕获后现场继续变化不改变记录；取消、隐藏、换工程与旧异步回复清理各自票据。失败可改名重试，成功消费票据，经原工程队列和历史一次创建稀疏场景，可撤销重做与保存重开。录入不清除手动层、不停止后台，不新增设备或工程文件格式。完整输出录入仍未实现；既有场景合并见下节。

## 手动值合并既有场景

[EXEC-011](../development/tasks/EXEC-011-manual-scene-merge.md)／[ADR-155](../development/decisions/PRODUCT-ADR-155-manual-scene-merge.md)：采集前明确选择目标，新建与合并使用互不通用的票据。

```text
ApplicationHost.manualCapture({
  kind: "capture", generation, hostId, source, selected, sceneId
}) -> { generation, token, readings, fixtures, merge }
ApplicationHost.project({ kind: "mergeManualScene", generation, token })
```

Rust `Document::prepare_manual_scene_merge` 冻结目标及新增／替换／相同、原值／预设、保留效果审阅；`merge_manual_scene` 仅替换对应灯具属性，未提及项、场景身份、效果及使用位置保持。被替换的预设引用变为独立值，不改共享预设；启用的动态效果继续存在，审阅须说明它仍可能控制输出。UI 的使用位置导航复用现有工程引用读取。

提交重新核对工程语义、目标场景和审阅所依赖的预设内容，失败原子保留票据以便处理，目标变化须重新采集。成功消费票据，一次撤销／重做；完全相同不产生历史或修订。已有后台继续运行载入时的不可变版本，合并不隐式重新载入或改变手动层。取消返回采集按钮焦点、保留选择，搜索不丢失已选目标。

## 连续亮度电平

[EXEC-010](../development/tasks/EXEC-010-live-source-faders.md)／[ADR-154](../development/decisions/PRODUCT-ADR-154-live-source-faders.md)：场景／列表／手动来源的原 Level 操作共用有界手势调度；音频不复用灯光电平。`begin(source) → change(source, u16) → finish(source)`；`cancel()` 丢弃尚未发送目标，已应用值不回滚。

手势冻结后台／布局／会话／来源，只保留一个目标和一个在途请求，两条控制命令至少间隔 50 ms。每次先取当前快照和修订；匹配自己的新序号、applied 回执与实际电平后才继续发送，未知／拒绝／5 秒确认期限退出不重发。原宿主继续管理单次请求期限和续约。当前推子在最终确认期间可继续合并输入，其他操作互斥。

UI 分别显示目标和权威实际值；精确输入草稿优先，有草稿时禁用该来源推子。取消、隐藏、失联、窗口失焦和身份／控制权变更停止后续发送，旧在途请求不能操作新对象。原生 range 保留键盘／无障碍及内部滑块拖动，窗口捕获松手覆盖轨道外释放，取消标记持续到本次输入结束。归零不停止节目、不清除手动属性，不写工程或编辑撤销；观察不是物理灯具反馈。

## 所选灯组共同亮度手势

[EXEC-012](../development/tasks/EXEC-012-manual-brightness-gestures.md)在同一个调度器中增加 UI 内部冻结目标策略，复用原 `Patch`／`Normalized`，不新增宿主命令或时钟。只有全组选灯共同的无功能档位 `dimmer` 可线性控制；保留原精确输入和明确释放。来源电平与共同属性手势共享单在途／最新目标／回执保护，全局互斥。

额外冻结工程、完整所选灯组、属性和容量；选择、定义、布局、会话或控制状态变化取消后续发送。整组数量／请求字节预算开工前核对，超限整组拒绝，不悄悄拆批。每次 Patch 后须自己的 applied 回执且整组权威持有值完全一致，才能确认或发送下一个目标；零值仍持有。

不同／部分或全部未持有时明确显示实际状态，滑块暂置中点不是实际值或默认提交。按下松手但不移动不发命令；首次移动才以绝对亮度设置整组。精确草稿优先，手势期间锁定灯组和属性；取消不回滚已应用值、不释放、不修改工程历史。搜索外已选灯仍包含在原子目标中。

## 后台输出总控

[EXEC-013](../development/tasks/EXEC-013-background-output-master.md)／[ADR-156](../development/decisions/PRODUCT-ADR-156-background-output-master.md)：`outputMaster` 能力配合目录 `output.uncontrolledFixtures` 与权威状态 `output { percent, blackout }`。未声明能力的旧后台仍可观察原节目，但不能操作总控；声明却缺字段、数值越界或数量超过固定灯具目录则整条观察拒绝。

`Client::output`／`ApplicationHost.execution({ kind:"output", hostId, revision, action })` 的 `action` 分为 `level { percent }`（整数 0–100）和 `blackout { enabled }`。通过相同控制租约、修订、序号、截止时间和 applied 回执，不伪造来源。不同时发送两字段，避免旧亮度快照覆盖当前熄灯锁存。

每组 Rust Session 在语义合成后、通道编码前，复用 OutputMaster 仅衰减已识别连续强度一次；有调光不再缩放 RGB，无调光只对完整连续 RGB 回退，功能／控制通道不动。手动贡献、赢家、节目进度和音频不变；软件帧和语义诊断为总控后值，UE 观察最终帧，不再乘编辑预演总控。

总控不存工程或撤销历史。0%／熄灯／归还控制权／退出编辑器不释放或停止节目，重新连接／接管保留当前值；明确关闭并重新准备恢复 100%、非熄灯。解除熄灯显示此刻合成，不重放黑场前的帧。未识别亮度灯具明确提示，熄灯不是机械急停、完整安全措施或物理完成反馈。

组级推子复用同一个连续调度器，无来源目标仍冻结后台／布局／会话／工程／能力；所有来源与手动连续手势用 key 全局互斥。整数精确草稿优先；取消、隐藏、失联／抢占停止后续发送、不回滚在途已应用值；读取失败标最后已知。总控常驻后台区，不被节目搜索／筛选隐藏。

## 普通节目批量控制

[EXEC-014](../development/tasks/EXEC-014-source-batch-control.md)／[ADR-157](../development/decisions/PRODUCT-ADR-157-source-batch-control.md)：v2 可选 `sourceBatch`，`Client::batch(host_id, revision, sources, BatchAction)`／`ApplicationHost.execution({kind:"batch",hostId,revision,sources,action})`，action 仅 `pause/resume/stop`。1–64 唯一普通场景／列表，整组校验、原请求预算／控制权／版本／序号、一次回执。无能力禁用，错误不拆批／重发。音乐与手动层仍走各自原入口。

后台操作台有序多选，跨搜索保留且显示筛选外数量；全选当前结果只加当前普通节目，清空选择不改变运行。审阅显示完整冻结名单、状态及停止归还影响；过滤／选择、隐藏／失焦／Escape、身份／控制或修订变更取消旧审阅。确认提交原冻结修订而非偷偷采用新值；结果待确认只查原回执，不用 HTTP 成功当全部生效。连续手势与其他修改互斥，旧迟到回执不复活新对象；选择／草稿仅本窗口，不写工程、历史或优先级，重开只读需重新明确选择与控制。

暂停保留贡献，继续不重启待执行／已结束；停止只归还所选普通来源，其他来源、手动／音乐、总控／熄灯保持。不是熄灯、急停或同时发送的物理保证，内部执行失败按原故障边界撤回有效完整帧。
