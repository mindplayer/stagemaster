# OUTPUT-002：DMX 完整帧发送适配

当前结果：运行／输出协调与原服务逻辑侧固件组装增量完成，完整输出链仍进行中；基线 `df435e7`，main，结果为本次 `feat(device): coordinate runtime frames and output quiescence` 提交，当前会话单写者。上一轮有界队列已提交，分类 progress。依据 [ADR-137](../decisions/PRODUCT-ADR-137-runtime-output-coordination.md) 接原 ManagedWorker 维护门、当前快照与 Port，并在原运行服务的实验组装中验证真实 UART／诊断引脚独占；GPIO21 仍强制关闭，本轮未刷机／打开串口或蓝牙，用户窗口／工程／output/ 保持。硬件看门狗和物理时序出口继续保留。

## 运行与输出协调结果

可选 `install-worker/output` 中的 LocalOutput 只协调原 Runtime 和 Port。新播放实例请求本地来源，载入／读取不激活；开始后暂停、控制租约释放保留自主帧，停止仍使用原档案默认值。维护或显式关闭撤回输出，真实 Queue／UART 静默后才能确认当前维护请求；帧身份／修订／节目／实例／采样时间严格核对，错误锁存。见[调用契约](../../module-api/runtime-output-coordination.md)。

128 项相关测试通过，含本轮新增 8 项，0 失败／忽略，日志 `logs/output-002-runtime-output-final-regression.log`。使用原真实工程编译、安装协议、ManagedWorker 与共享队列／事务核对 20 个完整动态包及暂停／停止默认值；实际存储计数验证未排空前不放行写入、清理前不释放旧读取者。取消维护、新实例等待旧排空、超时后旧实例不重启、错帧与失败排空均覆盖。

复现并修复“停止初次播放时，尚未消费的 Quiet 会先授予输出权”：协调器现在先处理撤销意图，再轮询可能授予来源或发送待发帧的 Port。失败记录 `logs/output-002-runtime-output-stop-race-before.log` 保留，回归未放宽。新开始在等待旧来源过期清理时的意图也按单次许可丢失处理，不被重复清除。

工作区全部目标严格 Clippy（application／output／queued-dmx／development-device-access）通过，日志 `logs/output-002-runtime-output-final-clippy.log`。实际 `runtime-dmx-probe` 组装沿原认证 GATT、存储和第二核运行器，第一核独立发送任务持有 UART，通过既有队列跨核交接；GPIO17 与连接指示所有者互斥，旧不输出的 runtime-gatt 保留原行为。Sampler 继续只计算一次真实帧并交给输出协调器；逻辑 UART 状态单独发布，未冒充物理发送。

使用已有虚构身份 v2 构建夹具，实际 S3 最终严格检查／完整链接见 `logs/output-002-runtime-output-accepted-xtensa-check.log`、`logs/output-002-runtime-output-accepted-xtensa-build.log`，保留原裸机 RWX 链接告警。原正常运行固件及独立队列示例严格检查通过，见 `logs/output-002-runtime-output-original-check.log`、`logs/output-002-runtime-output-example-check.log`。实验 Line 迁入板级模块并被两种实验组装复用，GPIO21 从不使能；未刷入，不能以此报告新的实板时序、CPU 或内存峰值。

两个锁文件只增加依赖边，无包版本新增、升级或移除；没有工程／设备包／网络协议字节变化。新生产文件最多 114 行，原安装入口仍低于 300 行；未堆入大入口、未增加另一播放器。格式、脚本语法、差异及本地文档链接检查通过（491 处）。下一项独立硬件看门狗与逻辑侧实板时序／资源观测，之后才启用经过电气确认的 RS485，并补正式产品输出状态。完整 DEVICE-003／PLAYER-002／AUDIT-001／goal 继续开放，不重开已完成框架轮。

## 已提交的有界驱动增量

当前结果：有界驱动交接增量完成，完整输出链仍进行中；基线 `08b550f`，main，结果为本次 `feat(output): connect DMX transactions through a bounded priority queue` 提交，当前会话单写者。上一轮已提交，分类 progress；本轮按 [ADR-136](../decisions/PRODUCT-ADR-136-queued-dmx-driver.md) 完成原 Port 与 Transmitter 的可选队列连接、停止优先、故障／取消／空闲期限及实际 S3 构建。未刷机、打开串口／蓝牙或启用物理线路，用户 output/、窗口及已安装节目保持。

## 有界驱动交接结果

新 `dmx::queued` 按队列状态、同步 Driver、异步 Receiver、等待协调分离；固定帧／完成／停止槽，背压时仍由原 Port 保留最新完整帧。停止取消原发送事务，排空后反馈当前票据；旧完成不覆盖新停止。错误锁存、两端销毁和单次绑定已处理，独立空闲期限关闭不依赖上层轮询，也不冒充 Quiet。调用及边界见[队列接口](../../module-api/queued-dmx.md)。

45 项相关测试通过，含本轮新增 15 项，日志 `logs/output-002-queue-final-tests.log`。实际工程／Installer／Runtime／Port／真实队列／事务的 20 帧全部字节一致，延迟发送期间的后续四帧只发送在途和最新帧。独立线程仅在真实 Waker 唤醒时轮询，验证 16 帧、独立空闲到期与未读取完成时的停止。另验收各阶段取消、开始前过期、满槽、时钟范围、超时、失败停止不忙循环、停止与最后完成竞态、生产者／消费者销毁、错误后真实 Quiet 仍不能重开维护。软件布局记录：本机队列 712 字节，测试接收 Future 3,496 字节；不是实板资源峰值。

发现并修复“读取时钟后恰好到期的合法定时器被误判为提前唤醒”，先复现失败再通过回归，原失败见 `logs/output-002-queue-timer-race-before.log`。首轮测试夹具将软件线路借用跨过停止调用，引发 RefCell 清理 panic；限定断言借用范围修正，未削弱产品取消清理或断言。严格 lint 要求的借用、命名和测试职责拆分已修正。

全工作区全部目标严格 Clippy（queued-dmx／application／development-device-access）通过，日志 `logs/output-002-queue-workspace-clippy.log`。实际 S3 同步组件、共享队列与真实 UART 类型严格检查及完整链接通过，日志 `logs/output-002-queue-xtensa-final-check.log`、`logs/output-002-queue-xtensa-build.log`，保留原裸机 RWX 链接告警。独立 `dmx-queue` 镜像通过实验 Line 永不使能 GPIO21，没有蓝牙／安装或物理输出能力声明；未刷入。正常 runtime-gatt 没有组装该驱动。

默认生产依赖树仍只有 output-port 本体。可选功能使用固件已有 embassy-sync 0.8.0；主工作区新增该版本、embedded-io 0.7.1／embedded-io-async 0.7.0／heapless 0.9.3，两个锁文件均无既有包版本升级或移除，固件只新增本地依赖边。新生产文件不超过 110 行。格式、脚本语法、差异及 485 处本地文档链接检查通过；未修改界面、设备包或协议字节，未重复旧界面验收。

## 已提交的单帧发送增量

状态：单帧事务／S3 适配准备增量完成，完整输出链进行中；基线 f229263，main，结果为本次 `feat(output): add bounded DMX frame transactions and S3 UART adapter` 提交，当前会话单写者。上一轮实板测量已提交，分类 progress；用户 output/ 保留。依据 [ADR-135](../decisions/PRODUCT-ADR-135-dmx-transmission.md)。

当前增量限定原 output-port 的无堆单帧事务与真实 esp-hal UART 适配准备、独立测试和构建。不是再造 Port／播放器；异步发送与同步提交之间的有界队列、正式固件接线和物理输出仍须接续。无授权的真实灯具操作不执行，本轮不刷机，设备与用户窗口保持。

验收：实际 513 字节与 Break／MAB 顺序；部分写入；发送完成与 FIFO 接纳区分；开始前过期；错误、零写入、时钟倒退、取消和超时使能关闭；取消后下次发送先排空；Quiet 等待真实排空；原 Port 的接纳／完成及静默门组合。使用真实 Runtime 样本与软件线路对照，另编译实际 S3 适配，不能仅测试手写伪帧或类型签名。源文件按物理接口、事务及故障测试分离，保持小文件。

## 结果与审查

原 output-port 增加 dmx 的 Line／Clock、固定帧事务和期限协调三个小文件，原 Port／Driver 契约不改。新增 S3 UART1 适配和独立逻辑侧实验镜像，GPIO21 始终保持禁用；不是在正常固件里隐藏自动启动发送的开关。正常完成保留 MARK，错误／取消／销毁关闭方向；下一操作仍须排空，避免取消后泄漏旧尾帧。精确调用见[模块接口](../../module-api/dmx-transmission.md)。

30 项 output-port 测试通过，含原 19 项及新增 11 项。覆盖实际完整字节和 128 字节分批接纳、MAB 不提前、最后移位器等待、各阶段取消、排空后期限重检、每类 I/O 错误、零／超量写入、时钟倒退／溢出、预算、停止超时与销毁关闭。真实工程编译／安装后的 Runtime 在释放控制租约后连续产生动态帧，原 Port 分配各帧票据、提交并等待新异步事务完成，20 帧完整 513 字节逐字节一致；维护也经原端口 Quiet 后放行。测试线路的可控等待不是物理证明。日志 `logs/output-002-port-runtime-tests.log`。

全工作区全部目标严格 Clippy（application／development-device-access）、实际 S3 适配严格检查及逻辑探针完整链接通过；原 runtime-gatt 的独立严格检查通过。分别见 `logs/output-002-accepted-clippy.log`、`logs/output-002-xtensa-check.log`、`logs/output-002-xtensa-build.log`、`logs/output-002-existing-runtime-check.log`。保留原裸机 RWX 链接告警。首次锁文件更新命令在根目录未读取固件 build-std 配置导致 core 找不到，转回固件脚本的正确工作目录后通过；非缺失工具链，原日志保留。文档标识／测试夹具状态和命名检查错误已修正，未屏蔽相关 lint 或放宽断言。

固件只增加 output-port 本地可选依赖与独立实验目标，锁文件只新增这一条本地 crate，无第三方升级；主工作区锁文件不变。生产文件最大 95 行、最大新增测试支持文件不足 200 行。fmt、脚本语法、差异及文档本地链接检查通过。UI／工程／设备包／协议字节没有变更，未重复旧界面测试或声明已发送真实 DMX。

## 接续边界

原同步 Driver、有界异步发送与 Runtime 快照／激活／维护协调已按上文接通，原服务的逻辑侧固件组装已链接；引脚所有权明确从诊断灯交给 UART。接续独立硬件看门狗与逻辑侧实板时序／资源观测，再进行经过电气确认且获授权的差分波形测试和正式产品输出状态接线。不能把逻辑实验镜像的发送回执显示成正式 RS485 已发送。

HAL flush_async 最后移位字节使用短同步轮询，外设异常卡死时异步期限无法抢占它；整核／外设卡死的独立看门狗必须在正式物理输出前验证。完整标准正文、隔离公共地／连接器、真实 UART／RS485 时序、最大负载和 8 小时验收仍开放。此次未刷机、未打开串口／蓝牙、未发送灯具信号，用户窗口／output/ 和已安装节目保留。DEVICE-003／PLAYER-002／完整 AUDIT-001／goal 保持 active，已完成框架轮不重开。
