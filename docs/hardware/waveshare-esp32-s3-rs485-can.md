# 微雪 ESP32-S3-RS485-CAN：首台 DMX 样机基线

资料核对：2026-09-21；实板更新：2026-09-26。关联：[HW-001](../development/tasks/HW-001-first-player-baseline.md)、[首版架构决定](../development/decisions/PRODUCT-ADR-001-first-software-hardware-delivery.md)。用户确认持有此型号；PCB 修订与 DMX 接线仍未现场核对。PLAYER-002A 已刷入禁用 RS485 的 Rust 诊断固件。

连接更新：[PLAYER-002A](../development/tasks/PLAYER-002-esp32-probe.md) 通过 `/dev/cu.usbmodem2101` 实读 ESP32-S3 v0.2、40 MHz 晶振、16 MB Flash，安全启动／Flash 加密未启用（未改 eFuse）。共享内核及电脑 BLE GATT 连接已实测；PSRAM、物理 DMX 输出与电气接线仍未验证。

## 已核对与尚待验证

2026-09-29 内存核对：按默认 ESP32-S3R8 的[乐鑫规格](https://documentation.espressif.com/esp32_s3_datasheet_en.pdf)，内部 SRAM 为 512 KB、封装内 PSRAM 为 8 MB；PSRAM 在软件中按外部内存管理。当前固件只设 128 KiB 内部动态堆，其他内部空间还承载代码／静态数据／栈，未初始化 PSRAM。其实际容量／稳定性待实板验证，不能把 16 MB Flash 当运行内存，也不能把 128 KiB 当总 RAM。分层预算、未来宿主音频与容量档位方向见 [ADR-052](../development/decisions/PRODUCT-ADR-052-memory-and-host-audio-sync.md)。

官方资料列出隔离 RS485、隔离电源、USB Type-C 和 16 MB Flash。原理图已下载并渲染阅读，确认收发器和方向控制路径；足以将此板选为一路 DMX 验证平台，不能据此认定整机已经符合 DMX512-A。[微雪产品文档](https://docs.waveshare.net/ESP32-S3-RS485-CAN/)

| 项目 | 文档／原理图依据 | 本项目处理 |
| --- | --- | --- |
| 主控 | 默认 ESP32-S3R8，Xtensa 架构 | 首次设备会话再识别实物芯片与存储；不据板名推算可用堆大小 |
| RS485 TX | GPIO17／TXD1，经数字隔离到收发器 DI | DMX UART 发送候选引脚 |
| RS485 RX | GPIO18／RXD1，经数字隔离回主控 | 首版不启用 RDM，不以接收脚存在宣称支持 RDM |
| 方向控制 | GPIO21／RS485_EN，经隔离接 SP3485EN 的 DE 和低有效 RE | 高电平发送、低电平接收；实测确认复位、Break 和数据期间的电平 |
| RS485 指示灯 | LED2 绿／蓝共阳极；绿经 R21 接 TXD1，蓝经 R26 接 RXD1 | 绿灯没有独立 GPIO；仅在发送器关闭且 TX 不被 UART 占用时复用为诊断连接灯 |
| USB | GPIO19／D−、GPIO20／D＋ | 与 DMX UART 独立；固件验证 USB 控制和上传，保留恢复烧录入口 |
| 外部 RS485 端子 | J6 仅 A、B 两端；信号侧电源／地为隔离域 | 必须解决完整 DMX 连接器及信号公共地，见下文 |
| 端接 | 120 Ω 通过跳线选择，产品默认 NC | 按线路拓扑确认端接，不把发送板端接与线路末端端接混为一谈 |
| 本地操作 | BOOT、RESET；另有扩展 GPIO 接口 | 需要另配选择、执行与状态显示；不把开发按键当作交付面板 |

引脚和电路依据：[官方原理图](https://files.waveshare.com/wiki/ESP32-S3-RS485-CAN/ESP32-S3-RS485-CAN-Schematic.pdf)。实施前核对实物修订，板卡定义集中在固件适配层，不进入核心或工程文件。

2026-09-29 指示灯补充：用户要求连接后常亮、发送时闪烁。按原理图和[厂商说明](https://www.waveshare.com/wiki/ESP32-S3-RS485-CAN)，诊断构建可在 GPIO21 保持低电平时独占 GPIO17 控制绿灯；实现与实测见 [HW-003](../development/tasks/HW-003-link-indicator.md)。以后 UART 接管 TX 时，此灯亮灭直接跟随串行位与数据内容，不能独立设置为 DMX 帧频或人眼慢闪；不能为了灯的节奏停发、插空包或扭曲真实 DMX 时序。独立连接／活动指示由未来面板的专用灯承担。

## 接口需要补齐的一点

图中主控 GND 与隔离侧 SGND 分开，RS485 外部端子没有第三根信号公共地。DMX 适配需要对照标准处理 Data Link Common、屏蔽和连接器；**不能把 USB／主控 GND 当作隔离侧公共地直接接出**。图中的 EARTH 网络也不能仅凭名称当成一个现成外部接地点。

先确认板上可用的隔离侧引出方式，并形成可检查的转接线／小板方案；无法可靠引出时，换用有完整 DMX 接口的隔离输出适配。该问题不妨碍先做软件、固件构建和逻辑侧时序验证，但正式接灯前必须闭合。A/B 命名还须核对实际数据极性，不仅凭不同厂商的字母约定接线。

## 固件输出策略

- 首版单向 DMX：250 kbit/s、8N2、Break、Mark After Break、零 Start Code、512 个数据槽。通道数组不包含 Start Code，物理驱动负责封装；普通 RS485／Modbus 网关固件不等于 DMX 固件。[TI DMX512 物理层应用说明](https://www.ti.com/lit/an/slyt425/slyt425.pdf)
- UART／定时外设生成串行时序，CPU 准备完整帧；避免 UI、USB 传输或逐字节软件延时决定线路节拍。方向使能须覆盖完整 Break、MAB 与数据发送，启动首帧也要合法。
- SDK 的“已写入发送缓冲”与“线路已发送完”分别上报。ESP-IDF 的 `uart_write_bytes_with_break()` 在数据**之后**追加 Break，不能把一次调用误认作自动生成完整 DMX 包；Rust 驱动仍须核查实际 HAL 能力。[Espressif UART 文档](https://docs.espressif.com/projects/esp-idf/en/stable/esp32s3/api-reference/peripherals/uart.html)
- 首选验证 Rust `no_std`／`esp-hal`，S3 需要专用 Xtensa 工具链。固件目标与桌面 workspace 分开锁定工具链，纯执行算法共用；具体版本通过交叉构建再记录。[Espressif Rust 工具链](https://docs.espressif.com/projects/rust/book/getting-started/toolchain.html)
- 40 Hz 完整帧为候选工程目标，不是协议强制值或已测指标。先冻结测试条件，再测帧间隔、最坏偏差、发送故障和恢复，不只看平均帧率。

TI 应用说明是物理层参考，不能替代当前标准。ESTA 目录提供 ANSI E1.11-2024；本轮未成功取得其完整规范正文，不能声称已按该版本完成合规审查。固件时序与接口验收前补齐规范依据。[ESTA 标准目录](https://tsp.esta.org/tsp/documents/published_docs.php)

## 分阶段验收证据

| 检查 | 所需证据 | 当前状态 |
| --- | --- | --- |
| 工具链与板级构建 | 锁定版本、可重复构建、Flash／RAM 报告、USB 与 UART 外设方案 | 002A 已完成诊断固件构建／烧录与内核／GATT，UART 待实现 |
| 无灯具时序验证 | 受控测试负载；首帧及连续帧的 Break／MAB／8N2／513 槽、方向控制和极性记录 | 未开始 |
| 实际 DMX 接口 | 接线／隔离公共地方案、端接、合适仪器的差分线路测量 | 待补适配方案 |
| 灯具功能 | 指定型号／模式／地址的颜色、亮度、渐变、跳转与释放，和软件参考结果比较 | 灯具型号待实测阶段确认 |
| 独立运行 | 独立供电、USB 拔除、电脑关闭、无网络；本地选场景／执行与输出保持正常 | BLE 断开后测试内核持续；尚无独立供电／面板／用户工程播放验证 |
| 故障与压力 | 坏包、传输中断、重复命令、重启、写入阶段掉电、面板／USB 压力、帧间隔与内存记录 | 002A 完成诊断消息拒绝／心跳超时／三次重连，物理输出与持久安装故障未覆盖 |

首次连续运行以 8 小时作为工程验证门槛，再根据测试负载和故障结果扩展；这是拟定测试时长，不能等同于商业寿命认证。不得在 USB 唯一供电的情况下把拔线掉电误判为通信失联。测量设备接地也需要与隔离方案一致。

## 资料留存

- 原理图来源：[微雪官方 PDF](https://files.waveshare.com/wiki/ESP32-S3-RS485-CAN/ESP32-S3-RS485-CAN-Schematic.pdf)。
- 项目内缓存：`data/hardware/waveshare-esp32-s3-rs485-can/schematic.pdf`；渲染核对：`tmp/pdfs/waveshare-schematic.png`，均按项目规则忽略，不是固件产物。
- PDF SHA-256：`c6620c0f318166c043733febd0c30835972c9cd09dcda4395fb39526fd5ea915`。厂商今后更新同一 URL 时需重新核对。
- HW-001 原轮次仅资料核对；后续实板证据与限制见 PLAYER-002A。当前代码固定 GPIO21 为低电平，没有发送 DMX。
