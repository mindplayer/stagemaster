# 独立执行宿主

HOST-001／[ADR-098](../development/decisions/PRODUCT-ADR-098-independent-runtime-host.md)；`stagemaster-runtime-host` 组合既有 `stagemaster-runtime`，不依赖 Tauri、文件驱动、BLE、UE 或云端。本 crate 是标准系统的可嵌入宿主库；HOST-002 在外层增加[本机独立执行进程](local-execution-process.md)，尚未替换桌面。

## 所有权与调用

可信应用先在运行线程之外验证／安装／载入节目，得到 Operation、Idle、选中与载入一致、无运行实例和控制者的 Runtime。`Host::start(runtime, configuration)` 接管所有权，使用独立单调时钟推进；Host 不暴露读文件、装载、维护或逐帧调用。首个档位是一份已准备单节目与一路逻辑帧，不限制后续专业执行器设计。

| 对象／方法 | 状态归属与结果 |
| --- | --- |
| `Host::connect(grant, takeover, ttl)` | 只供可信组装方；应用会话已验证后才传 Grant，返回 Connection，不等于已取得控制权 |
| `Connection::wait(timeout)` | 得到实际取得租约的 Client，或明确拒绝；超时不取消，可以继续等同一回执 |
| `Client::acquired_state()` | 取得控制权时的历史权威状态，提供首条命令的基准修订；不是当前实时观察 |
| `Client::submit(serial, revision, action, ttl)` | 内部固定该 Client 的租约，只开放开始／暂停／恢复／下一步／停止，返回 Ticket |
| `Ticket::wait(timeout)` | 外层是等待结果，内层为应用调用结果；Submit 的 Receipt 还包含业务执行结果和历史状态 |
| `Client::renew`／`release` | 仅能续期／释放自己的输入权；过期或被替换的连接不能影响新控制者 |
| `Observer::read()` | 只读复制快照；繁忙明确重试，不把锁或 Runtime 交给客户端 |
| `Host::shutdown(timeout)` | 可信宿主结束运行，走独立关闭信号；等待超时保留同一线程句柄，可再等待 |

可编译调用示例在 crate 文档测试中。上层必须检查 Receipt.result，不能把获得 Ticket／Receipt 都显示为执行成功。原租约内精确重试最后业务请求保持原 serial／revision／payload；已超时但尚无回执时先查询原 Ticket，不能自动发新序号启动第二次节目。连接回执丢失时控制权可能已经取得，须按租约和接管政策核对。

## 时间与资源预算

刷新周期允许 1～100 ms，默认 25 ms；时间始终来自宿主 Instant，基点为传入 Runtime 的 observed_ms。延迟后按实际时刻求值，刷新相位保持、跳过过期发送时隙，不突发补帧。快照报告 cycles、采样毫秒、发布跳过次数、最大迟到／超周期毫秒与 missed_periods；这不是物理 DMX 刷新率或硬实时保证。

命令队列固定 32 项，每周期最多 8 项；TTL 大于零且不超过 5 秒。过期未开始处理的请求返回 Deadline，不消耗内核业务序号。队列满返回未接纳。每回执槽容量为一，观察只有一份固定大小的最新快照；慢消费者不会积累运行端历史，也不阻塞帧求值。调用端持有的 Client／Ticket 数量仍由调用端承担，后续网络适配必须另限会话数和在途请求数，不能由这些局部上限推断整个应用内存无上限问题。

加载后不再读包；许可策略只允许有界、无 I/O 的可信实现。恶意／错误策略阻塞线程时无法由普通 Rust 线程强制终止；关闭等待如实返回超时，不产生第二个替代线程。异常 unwind 和运行错误变为 Faulted 并撤销有效观察；进程 abort／崩溃仍需进程隔离与输出端失联策略。

## 生命周期与证据边界

Client／Observer 全部消失也不结束 Host；Client Drop 不自动 Release，输入权最终依显式释放或租约到期回收，节目继续。Host Drop 只发出关闭请求，不证明已经同步退出；需要证据的调用必须 shutdown 并等待终态。

关闭或故障时观察不再提供可用快照／逻辑帧；Fault 区分核心错误码与意外 panic，保留首个终态原因。请求已进入执行后可能在关闭期间完成；关闭不是物理撤销。Snapshot 的 Frame 仅是 Rust 生成的 512 通道逻辑值，不是端口接纳、UART 完成或灯具反馈。当前不能把 Observer 直接当可靠物理输出端口，实际驱动、端口租约和确认静默需独立接入。

此 Rust API 是同进程应用边界；不直接序列化内部 Lease／Grant 给不可信客户端。HOST-002 的本机 DTO 是一个外层适配，并非全产品 SDK；后续跨端适配须绑定认证主体、范围、连接代次及请求预算。当前仅在 Mac 验证；手机／平板的后台生命周期仍需平台验证，不能从标准线程推断后台永远运行。真实桌面迁移、跨节点同步、生产授权和专业多执行器仍另验收。
