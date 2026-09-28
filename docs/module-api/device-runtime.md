# 设备运行应用层

PLAYER-003E，基线 `4b3ffe9`，依据 [ADR-033](../development/decisions/PRODUCT-ADR-033-device-runtime.md)。`stagemaster-runtime` 已实现 no_std＋alloc 状态机；复用 Installed、Archive、Player 和 Output，不依赖文件、NOR、无线、UI 或物理输出。

## 对象和实际调用

`Runtime<R, P>` 持有一个已验证安装快照、一个待选节目和至多一个载入计划。R 是稳定读源，P 是可信宿主提供的 `PlaybackPolicy`。Installer 的最新安装候选独立于 Runtime 绑定内容；安装不会自动切换或执行。

```rust,ignore
let mut runtime = Runtime::new(fresh_boot_id, now_ms, loader_budget, playback_policy)?;
// 初始不生成帧。输出宿主实际清理发送队列、关闭输出后才确认。
let request = runtime.quiescence_request().ok_or(Code::Mode)?;
let permit = runtime.confirm_quiescent(request, now_ms)?;
runtime.finish_maintenance(permit, now_ms, || installer.snapshot().map(Some))?;

// 此 grant 来自宿主验证过的面板或远程连接权限，不从任意网络字段构造。
let lease = runtime.acquire(Grant {
    principal, origin: Origin::Panel, duration_ms: 30_000,
}, false, now_ms)?;
let state = runtime.state();
let receipt = runtime.submit(Request {
    lease, serial: 1, expected_revision: state.revision,
    action: Action::Select(program_key),
}, now_ms)?;
// 后续以新序号和回执修订提交 Load，再显式 Start { step: stable_step_id }。
// 相同请求重试使用原 lease／serial／revision／action。

runtime.tick(monotonic_ms)?;
if let Some(info) = runtime.render(&mut slots)? {
    // 这是逻辑完整帧；物理宿主另行校验模式／启动身份／修订／采样时间并发送。
}
```

完整可运行例子见 [replay.rs](../../crates/stagemaster-runtime/examples/replay.rs)。`catalog()` 借用当前包的节目目录，`steps()` 借用实际载入节目的步骤；界面不要把数组下标持久化为控制身份。`ProgramKey` 为 `(Kind, UUID)`；同一 UUID 在场景和列表中的键不同。

## 状态与操作细节

State 包含启动身份、业务修订、已观察单调时间、模式、绑定 Commit、待选／载入键、原 Player 状态、实例 ID、步骤 ID、步骤经过时间和控制者。选择／载入／实例明确分开：选择下一节目不干扰当前节目；待选与载入不一致时执行会拒绝，避免悄悄执行旧内容。

| 操作 | 条件和结果 |
|---|---|
| Select | 当前包目录存在此键；只更新待选，错误保留原选择 |
| Load | 无播放器或播放器 Idle；校验预算后释放旧计划再载入，成功为 Idle，不执行 |
| Start { step } | 待选与载入一致、步骤 ID 存在、许可允许；创建新实例，包括主动重执行 |
| Pause | 当前实例 Running；暂停保留实例和当前值 |
| Resume | 当前实例 Paused、许可允许；继续相同实例，不重启节目 |
| Next | 当前实例 Running、有后续步骤、许可允许；保持同一实例，暂停状态不暗中恢复 |
| Stop | Operation 下始终允许，恢复原档案默认值并清除实例；不受播放许可到期阻止 |
| BeginMaintenance | 已停止、无实例；进入 Quiescing，等待输出宿主确认 |
| CancelMaintenance | 只在 Quiescing 允许；恢复原内容／Operation，使迟到确认失效 |

Finished 保留末帧和已结束实例，载入／维护须先 Stop。Stop 不能把所有通道笼统归零；既有灯具默认值仍是权威来源。核心不声称默认位置等于任何机构或物理装置的安全状态。

载入前的目录、状态和预算拒绝保留旧计划。真正开始载入后旧计划已经释放；读取、摘要或分配失败则保持当前待选项、无载入计划，可重试。此时 `render()` 返回 None，并保持传入缓冲原值，不能把旧缓冲当新帧发送；宿主须按无帧状态处理输出。失败状态有新修订和明确错误回执，未声称原子回滚旧计划。

`tick()` 使用可信宿主单调时间，正常调度无包读源访问／逐帧分配。时间倒退在任何状态变更前拒绝。所有带时间的控制和维护方法也先推进；无效业务请求不会冻结正常播放时间。`state()` 返回最后观察状态，不自行读取系统时间。`FrameInfo` 含启动身份、修订、采样时间、输出线路、节目键和实例；只表示软件生成，实际接纳／发送完成仍由输出适配报告。

## 控制租约与回执

`acquire(grant, takeover, now)` 只供可信宿主调用。主体非零，期限 1–60,000 ms，核心发放启动内递增租约；已有控制权时必须显式 takeover。`renew` 不能恢复已过期租约。`release`、超时和接管只撤销输入权，不停止节目；旧连接的断开事件不能撤销新控制者。

本地面板与远程使用相同 Request；Origin 只记录来源，不自行授予权限。网络适配必须先验证真实权限，再把输入映射到此入口；诊断 GATT 握手不够。当前只有一个控制者；本地停止可由宿主显式接管后提交 Stop，不开放任意远程免鉴权停止。

每个租约序号从 1 连续递增，最多一个在途。精确重发最后请求返回相同历史 Receipt，不重复启动、推进或调用许可策略；播放时间仍持续推进。同号不同内容、旧序号或跳号撤销该控制者。不同租约的恶意请求直接拒绝，不撤销合法持有者。

Receipt 包含原 Request、业务结果和当时 State。业务失败同样消费序号；修复后使用新请求。期望修订错误不应用动作，也不会绕过旧上下文。其他进入业务处理的动作无论成功失败都推进修订，覆盖“载入失败但旧计划已释放”。结构性租约／时钟／序号错误没有业务回执。

回执是历史结果，不是实时设备快照；query 应读新的 State。租约失效／重连后不能将旧请求换个租约自动重发；先读取状态，再接受新的明确操作意图。重启使用新的 boot、冷启动不执行，没有承诺跨断电的恰好一次。

## 安装维护门

`confirm_quiescent` 需要精确的当前 Quiescence 请求，宿主必须实际完成输出关闭及队列清理；网络不能自行提交“已静默”。确认后才释放旧 Player、目录和读源租约，进入 Maintenance。

`with_maintenance(permit, work)` 仅在该窗口仍有效时执行同步闭包；此方法只控制调用资格，自己不持有 NOR 写接口。宿主必须把每次实际安装操作置于此门内，不能将闭包返回值或未开始执行的 Future 当成永久写许可。结束窗口后旧凭据不能再调用闭包；运行／载入和安装不并行。

`finish_maintenance(permit, now, loader)` 在旧运行内容已经释放后调用只读快照恢复闭包；加载失败仍停留维护，可重试。成功绑定但不选择／载入／播放，原凭据失效；明确返回 None 可退出到空设备。安装事务本身的取消和待确认提交仍由 Installer 处理，不能用退出运行维护来伪造安装成功。

## 预算与许可边界

loader_budget≤64 KiB，按已验证目录的 loader_peak_bytes 在释放旧计划前判断；绑定目录也检查其常驻预算。主机必须为快照的完整验证另行预留既有 Archive 准入额度，不能以小节目预算限制已经在闭包中发生的验证内存。此预算是宿主声明，仍需实际预留；它不是堆剩余量测量。

新增 `Player::try_new` 为设备运行层可恢复地分配当前值／渐变起点缓冲；旧 `Player::new` 桌面调用保持兼容。大计划切换主动放弃保留两份计划的原子回退；没有输入源码、旧计划和新计划任意并存的承诺。

PlaybackPolicy 在 Start／Resume／Next 应用前看到包摘要／长度、节目键、实例、操作和单调时间；默认构造必须显式提供策略。Stopped 和维护操作不因许可失效被禁止。`policy_mut()` 只供可信宿主更新验证状态，不能从网络写一个 allowed 布尔量。

这只是许可调用边界，**尚未实现自动到期检测、有限收尾、签名或防回退现实时间**。ADR-004 的 24 小时临时授权仍由 AUTH-001 完成。继续旧实例与新启动已经分开，后续授权层可据此实现规则；调度单调毫秒不是安全授权时钟。

## 目标板准备与验证

`runtime-readiness` 独立镜像位于 `target/esp32-runtime-check/`，复用官方 NOR 区域与同一 Runtime。只读恢复已安装目录；实际策略始终拒绝播放，不建立控制连接，也不调用执行／输出。保留真实 submit／载入／Player／render 路径进行目标代码生成和链接，避免仅通用库类型检查通过。

实际 Xtensa 类型：Runtime 880 B（含一个缓存回执和内联控制／载入元数据，另有计划／目录堆内容），Request 64 B，Receipt 312 B，State 240 B。不能把缓存回执再次加进 Runtime 大小；宿主额外的请求／回执副本另算。运行提交入口栈帧 1072 B，主任务入口 7232 B；入口不是调用链峰值，仍需与 NOR、包解码及无线任务嵌套测量。沿用 003D 的 128 KiB 堆、64 KiB 装载预算及旧计划释放要求，未使用 PSRAM 或宣称实际峰值已测。

完整本地镜像 574,992 B，占 3 MiB 应用区 18.28%；默认诊断镜像、存储检查镜像分别保留。复现：

```sh
CARGO_HOME="$PWD/tmp/cargo-home" TMPDIR="$PWD/tmp" cargo test -p stagemaster-runtime --locked --offline
CARGO_HOME="$PWD/tmp/cargo-home" TMPDIR="$PWD/tmp" cargo run -p stagemaster-runtime --example replay --locked --offline -- data/PLAYER-003A/final.smpkg
bash tools/hardware/firmware.sh runtime-build
bash tools/hardware/firmware.sh runtime-check
bash tools/hardware/firmware.sh runtime-report
```

第二条读取指定真实包，在项目 tmp 下用文件安装参考完成选择、载入、执行、暂停、继续、断开和停止，并与独立 Player 逐帧比较；没有物理输出驱动。既有桌面导出的三个节目各 400 帧通过。测试另覆盖场景／列表、循环、Finished、错误控制、恢复和维护；具体证据见[工单](../development/tasks/PLAYER-003E-device-runtime.md)。未刷机、无线控制、实板持续运行、真实 DMX或生产授权；这些是后续独立出口。

## 安装任务承接

DEVICE-002C 已由 [ManagedWorker](maintained-install-worker.md) 组合运行层和串行存储工作器，ESP32 的每条内部安装命令进入实际维护令牌闭包。结束维护需终结安装事务并重新验证最新目录；不会自动选择或播放。正式远程维护命令、物理输出静默确认和商业许可仍未实现。
