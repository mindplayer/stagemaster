# HW-003：诊断连接状态灯

日期：2026-09-29；基线 `308e015`，结果为本次 `feat(hardware): show diagnostic connection on the RS485 green indicator` 提交；主工作区／当前会话单一写入。用户要求连接后 RS485 绿灯常亮，DMX 发送时随频率闪烁。

核对[微雪官方说明](https://www.waveshare.com/wiki/ESP32-S3-RS485-CAN)与[官方原理图](https://files.waveshare.com/wiki/ESP32-S3-RS485-CAN/ESP32-S3-RS485-CAN-Schematic.pdf)：LED2 为绿／蓝双色共阳极，绿灯经 R21 接 TXD1／GPIO17，蓝灯经 R26 接 RXD1／GPIO18。它没有独立灯控引脚。当前只读诊断固件没有 UART 输出，GPIO21 始终低电平，允许独占 GPIO17 以低电平点亮绿灯而不驱动 A/B 总线。

实现范围：上电／等待握手时绿灯灭；已有诊断协议 HELLO 成功回执发送后亮；主动断开、协议失联／回复超时后灭。BLE 适配只发布连接布尔事件，板级 OutputDisabled 独占方向和发送脚；核心不依赖 GPIO。以后 UART 接管 GPIO17 时必须移除此复用路径，灯随真实串行位变化，不能按自定人眼节拍改变 DMX 帧或线电平。独立可编程状态灯留在后续硬件面板适配。

验证计划：真实 ESP32 默认／安全／只读工作器目标构建及严格检查；保持原分区表、只刷诊断安全镜像；原生应用连接／断开和板端日志核对方向仍禁用，恢复用户可见连接。软件驱动状态不能代替肉眼确认，也不能证明 DMX 已输出。无需重复已授权的受控固件验证；本轮不启用 RS485。

## 实际验证

默认／security-readiness／worker-readiness 三种真实 Xtensa release 构建和各自严格 Clippy、固件 fmt 通过；保留原有 RWX 链接告警。改动只涉及 board、BLE 状态回调和主入口；文件分别 38／265／80 行，没有引入灯效循环或阻塞延时。

沿用 `partitions-storage.csv` 刷入只读安全镜像 591,728 B，未改变节目区或生产保护。ELF SHA-256 `2a7f7a645f47338ae363ea2bf7abe1ac5228a5fc10912bab70c93cef273e101e`。本次重启清除了上一轮仅保存在 RAM 的测试绑定，不影响已存节目；本轮未重新触发配对，也没有正式安装授权。

复用既有独立 GATT 探针：非法短写／版本／旧会话／重复序号拒绝，16 次有效保活／30 秒运行、5.912 秒失联检测及三次重连通过。四条连接的串口记录恰好为四组“绿灯亮／灭”，每条均检查 GPIO21 仍低。初末堆均 41,044 B，诊断数据继续报告输出禁用；灯只表示有效应用连接，不表示授权或 DMX 输出。

日志 `logs/hw-003-{default-build,default-check,security-build,security-check,worker-build,worker-check,flash,gatt-probe,serial}.log`。原用户 PID 44624 的圆弧剧场工程保持已保存；正式界面重新搜索发现新启动标识，选择并重新连接，已观察到 18 次保活／158 ms 最近通信，串口对应第五次点亮后保持。用户随后明确反馈“已经绿灯常亮”，实物连接状态确认通过；没有进行光度或仪器测量。UART／DMX 发送闪烁仍待输出模块，不为模拟闪烁发送假数据。
