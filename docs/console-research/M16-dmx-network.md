# 网络输出、RDM 与协议接入

灯具逻辑值、DMX Universe 帧、网络协议包和 RS485 物理发送是不同层次。MA-Net3／TitanNet 的协作与分布计算，也不是 Art-Net／sACN 输出的同义词。

## 功能对照

| 编号 | 能力 | grandMA3 | Avolites Titan | 依据 |
| --- | --- | --- | --- | --- |
| M16-01 | 物理 DMX 输出 | 端口分配 Universe，Off／Out／RDM／In | 物理输出节点关联逻辑 Line | [MA Ports](https://help.malighting.com/grandMA3/2.5/HTML/dmx_port_config.html)、[Titan DMX](https://manual.avolites.com/docs/system-settings/dmx-output-mapping) |
| M16-02 | Art-Net 输出 | Art-Net 4，广播／单播／自动模式，节点发现与配置 | Art-Net 节点、广播／单播及输出配置 | [MA Art-Net](https://help.malighting.com/grandMA3/2.5/HTML/dmx_ethernet_artnet.html)、[Titan DMX](https://manual.avolites.com/docs/system-settings/dmx-output-mapping) |
| M16-03 | sACN 输出 | 单播／组播、Universe、优先级、Preview 等 | sACN 输出、优先级、多网卡及同步地址 | [MA sACN](https://help.malighting.com/grandMA3/2.5/HTML/dmx_ethernet_sacn.html)、[Titan DMX](https://manual.avolites.com/docs/system-settings/dmx-output-mapping) |
| M16-04 | 内外地址映射 | Local Universe 映射 Art-Net／sACN 编号 | 逻辑 Line 可同时连多个节点，产生相同内容的多个输出 | 同上 |
| M16-05 | 输出启用层级 | 协议总开关、行启用、IdleMaster 输出等 | 模块总开关、节点／Line 分配、自动分配 | [MA Art-Net](https://help.malighting.com/grandMA3/2.5/HTML/dmx_ethernet_artnet.html)、[Titan DMX](https://manual.avolites.com/docs/system-settings/dmx-output-mapping) |
| M16-06 | 网络接口选择 | 各协议可选择网卡／Preferred IP | 模块选择输出网卡；多接口需正确地址规划 | 同上；[Titan Network](https://manual.avolites.com/docs/networking/controlling-fixtures-over-a-network) |
| M16-07 | 输入及合并 | DMX、Art-Net、sACN 输入；Prio／HTP／LowTP／Off 等合并策略 | 外部 sACN 合并到输出节点，并设置本机输出相对优先级 | [MA Ports](https://help.malighting.com/grandMA3/2.5/HTML/dmx_port_config.html)、[Titan DMX](https://manual.avolites.com/docs/system-settings/dmx-output-mapping) |
| M16-08 | 输入用于编程 | 可从 DMX 来源录制，合并与记录分别处理 | 19.0 新增 sACN 输入录制 Cue／Palette 的工作流 | [MA Store](https://help.malighting.com/grandMA3/2.5/HTML/cue_store_settings_preferences.html)、[Titan DMX](https://manual.avolites.com/docs/system-settings/dmx-output-mapping) |
| M16-09 | 数据失效／缓存 | 端口 Failure Mode 可保留或超时停止 | 输入状态区分接收／保持／无数据，可清输入缓存 | [MA Ports](https://help.malighting.com/grandMA3/2.5/HTML/dmx_port_config.html)、[Titan DMX](https://manual.avolites.com/docs/system-settings/dmx-output-mapping) |
| M16-10 | 发送节奏 | 物理 Out 文档列 30 Hz，RDM 模式节奏不同；网络另按规则 | System Render Rate 可设，物理 Break／MAB 等可调 | [MA Ports](https://help.malighting.com/grandMA3/2.5/HTML/dmx_port_config.html)、[Titan User Settings](https://manual.avolites.com/docs/system-settings/user-settings)、[Titan DMX](https://manual.avolites.com/docs/system-settings/dmx-output-mapping) |
| M16-11 | 慢设备兼容 | Art-Net 包间延时和输出延时 | Continuous／Overrun／Legacy 等 Art-Net 配置 | [MA Art-Net](https://help.malighting.com/grandMA3/2.5/HTML/dmx_ethernet_artnet.html)、[Titan DMX](https://manual.avolites.com/docs/system-settings/dmx-output-mapping) |
| M16-12 | RDM 管理 | 全局和端口启用，发现、参数读写；网络 RDM 依相应协议设置 | 配适发现与各模块 Block RDM 等设置 | [MA RDM](https://help.malighting.com/grandMA3/2.5/HTML/rdm.html)、[MA Art-Net](https://help.malighting.com/grandMA3/2.5/HTML/dmx_ethernet_artnet.html)、[Titan DMX](https://manual.avolites.com/docs/system-settings/dmx-output-mapping) |
| M16-13 | 分布输出诊断 | 设备列表、参数授权、缺席设备和 DMX Sheet | DMX Overview 显示 TNP 分配、处理槽和负载 | [MA Ports](https://help.malighting.com/grandMA3/2.5/HTML/dmx_port_config.html)、[Titan DMX](https://manual.avolites.com/docs/system-settings/dmx-output-mapping) |
| M16-14 | 配置保存与迁移 | 整体或单设备输出配置可导入导出 | Show 加载时选择是否沿用 DMX 设置，缺失节点可重分配 | [MA Ports](https://help.malighting.com/grandMA3/2.5/HTML/dmx_port_config.html)、[Titan Save](https://manual.avolites.com/docs/titan-basics/loading-and-saving-shows) |

## 不能混为一谈的规则

外部输入优先级、控台内 Playback 优先级、HTP／LTP 合成和总控衰减各有作用。MA 的 DMX Tester、Park、Highlight 等也影响最终结果。协议层必须与灯具属性层合成分开分析。[MA DMX Priorities](https://help.malighting.com/grandMA3/2.5/HTML/dmx_priorities.html)。

官方手册里关于某协议的描述只代表该产品所实现的部分。例如 Titan 文档“sACN 节点不会自动发现”应理解为它的节点配置行为，不能推广成整个协议家族没有任何发现消息。Art-Net 地址范围也应以当前实现和协议版本核对，避免沿用旧教程的 256 Universe 说法。

## 工作流程与 StageMaster 建议

自拟流程：同一 Universe 同时发物理 DMX 和网络节点；改变网络映射但不改灯具地址；接入第二来源，测试优先级与失效恢复；拔掉输出设备再接回，观察是否有重复发送或错误保留。

以下属于设计建议。Rust 核心生成逻辑输出快照，协议适配器负责编码、路由、调度及状态反馈。RS485 由明确支持 DMX 时序的设备／驱动发送，不能假定普通 USB 串口及桌面 UI 定时器足以稳定产生全部时序。

首版完成一个可靠物理输出路径和 Art-Net／sACN 的基础发送，设定实测容量、断连策略和观察面板。RDM、输入合并、同步、多节点和分布计算逐项扩展。具体 DMX 时序数值需在实现阶段依据当前标准与选定硬件验证，本资料不替代协议规范。
