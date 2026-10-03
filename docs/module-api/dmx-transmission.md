# DMX 单帧发送事务

[OUTPUT-002](../development/tasks/OUTPUT-002-dmx-transmission.md)／[ADR-135](../development/decisions/PRODUCT-ADR-135-dmx-transmission.md)。代码在 `stagemaster-output-port::dmx`，复用原 Ticket／Event；基本事务无堆、无运行依赖。此层是异步执行者，不是另一个端口控制器。同步 `Driver::submit` 与此层已通过可选[有界队列](queued-dmx.md)完成软件连接与实际 S3 构建，原服务的[运行／输出协调](runtime-output-coordination.md)及逻辑侧固件组装也已链接；独立看门狗与真实物理发送仍待验收，不能直接同步等待整帧发送。

| 调用 | 约束与结果 |
| --- | --- |
| `Transmitter::new(line, clock, timeout_us)` | 独占线路并立即关闭使能；总操作预算 22,708～1,000,000 µs，具体部署应按测量选择 |
| `send(ticket, &slots, valid_until_ms).await` | 拥有只读 512 槽借用直至结束；先排空、实际开始前重检期限，再 Break／MAB／零起始码／完整数据／最后移位字节；完成返回原票据的 Sent |
| `quiesce(ticket).await` | 先同步关闭使能，再等待旧 FIFO 和移位器排空；只有成功才返回 Quiet |
| 取消发送／停止 Future | 已开始轮询的操作在取消时同步关闭使能；不伪造任何完成回执，不声称已排空 |
| 销毁发送器 | 同步关闭使能；并不替代上层收到 Quiet 所需的排空证据 |

`Line` 提供独占方向脚、可取消 Break、允许部分写入的 UART 写接口及真正排空。`Clock` 使用与 Port 相同的单调时间原点，定时器须独立唤醒且不能提前完成。普通 Frame 数组不含起始码；事务使用固定 513 字节缓冲，不按帧分配、自动重发或保存节目。

采用 250 kbit/s、8N2、30 bit／120 µs Break、至少 16 µs MAB。标准槽时序由 UART 外设负责；软件只协调阶段。数据从头到尾写完才等待最后移位字节；FIFO 接纳或“数据已提交”均不构成 Sent。等待旧 UART 期间过期的帧返回 Expired，且没有发出新 Break。已开始后错误或超时返回失败，不把截断帧报告为 Expired 或 Sent。

总期限覆盖准备、Break、MAB、写入和排空。时钟倒退、期限计算溢出、提前唤醒、零／越界写进度、I/O 错误均停止并关闭使能。期限优先于迟到完成；正常完成保留空闲 MARK，后续帧仍由上层独立节拍决定。取消后 UART 内可能尚有旧数据，下一次发送先在使能关闭的情况下排空，不能重新开使能后泄漏旧帧尾部。

实际 S3 适配位于 `apps/esp32-player/src/board/dmx.rs`，独占 UART1／GPIO17／GPIO21，与原连接指示灯所有者不可同时构造。复用锁定 esp-hal 的配置、Break、部分写与 flush；同原运行器使用 Embassy 单调时钟。HAL 的最后字节检查包含同步轮询：硬件卡死或整核停顿时，异步期限不能代替独立硬件看门狗。该门槛及中断优先级、实际时序仍在正式输出接线任务中开放。

`dmx-uart` 独立实验镜像仅供逻辑侧测量准备，所有调用保持 GPIO21 低；它不会调用 enable，不会返回产品级“RS485 已发送”回执，无蓝牙或节目安装。当前只完成构建，未刷入或测波形。正式 runtime-gatt 镜像、设备能力位和桌面接口均未切换为物理输出。

软件证据覆盖真实 JSON → 编译包 → Installer → Runtime → Port → 异步事务 → 可控线路，20 帧的完整字节逐一比较；取消、超时与停止门另行检查。后续有界请求／完成槽、停止优先和独立发送已按上述队列契约补验。这些是软件线路证据，不是已接灯或合规认证。原运行服务的逻辑侧组装已完成，接续独立看门狗、实板资源／时序，再以差分波形和受控负载验收。
