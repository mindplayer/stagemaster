# PRODUCT-ADR-135：DMX 单帧发送与真实完成

2026-10-03；接受；基线 f229263，当前会话单写者；[OUTPUT-002 工单](../tasks/OUTPUT-002-dmx-transmission.md)。补充 ADR-103，不改变工程、设备包、运行消息或端口所有权。

## 依据

既有 Port 已处理来源、新鲜度、票据和维护，实际固件仍关闭发送。把完整数组写进 UART FIFO 不能报告 Sent；取消发送 Future 也不等于 FIFO 已清空。先实现可测试的单帧事务，再由有界队列把它接到原 Port，避免把 22 ms 串行等待放入原工作器的同步 Driver 调用。

参考 [esp_dmx](https://github.com/someweisguy/esp_dmx) 的写缓冲／发送／等待完成分工；该实现依赖 ESP-IDF，当前锁定的 Rust esp-hal 环境不直接嵌入它。实际复用 esp-hal 1.2.2 的 UART、可取消 break 和发送完成等待，不手写 UART 寄存器。已核对本地官方源码与[公开源码](https://docs.rs/esp-hal/1.2.2/src/esp_hal/uart/mod.rs.html)：write_async 可部分写入；flush_async 等待 FIFO 后仍等待最后移位字节；send_break_async 在取消时恢复极性。选用 250 kbit/s、8N2、120 µs Break、16 µs MAB、零起始码和完整 512 槽；时序余量是本项目候选配置，须波形验证。ESTA E1.11-2024 官方 PDF 本次仍返回 403，不能称作完成最新版合规审查。

## 决定

- 在既有 output-port 增加独立 dmx 模块，复用不可伪造的 Ticket 和 Event。UART Line、单调 Clock 与完整发送事务分离；无堆、无 UI、无节目解析或新的播放状态机。
- 构造即关闭发送使能。事务先确认旧 FIFO／移位器排空，再检查帧实际开始期限，然后 Break → MAB → 起始码与槽数据 → 真实 drain 完成，才返回 Sent。部分写入循环有界，零进展／越界写入视为故障。
- 每次操作有独立总期限，超时／时钟倒退／I/O 错误或 Future 取消时同步关闭方向使能。取消不能假装清空 UART；后续发送必须先在关闭状态排空，静默操作须关闭并确认排空才返回 Quiet。
- 正常完成保留 UART 空闲标记电平；下一帧仍由上层显式提交，不在底层自动无限重发。过期未开始只返回 Expired；已开始但失败不能返回 Expired 或 Sent。构造／销毁和所有错误路径保持独占引脚。
- 总期限依赖独立时间唤醒和驱动合作。它不能代替整核停顿／中断屏蔽情况下的硬件看门狗。具体板级使能和 UART 适配要单独编译、测量；不允许将软件 Line 用例当作物理证据。

本增量不提升固件能力位、不自动开始物理发送，也不把 GPIO17 连接灯与 UART 共用。生产接线下一步需要单帧／停止优先的有界队列、独立执行与故障反馈、原 Runtime 静默门、显式输出激活和实际波形。保持现行固件和用户窗口。
