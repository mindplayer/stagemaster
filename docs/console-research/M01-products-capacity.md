# 产品体系、输出容量与授权

研究对象是 grandMA3 软件体系和 Avolites Titan 软件体系。“老虎控台”在这里不只指 Tiger Touch 机型。较老的 Pearl／Tiger Classic、grandMA2 或 MA3 硬件的其他软件模式不自动计入本基线。

## 功能与能力边界

| 编号 | 能力 | grandMA3 | Avolites Titan | 依据 |
| --- | --- | --- | --- | --- |
| M01-01 | 独立控台 | full-size、light、compact／XT、replay unit 等 | D9、D7、D3、Quartz、Tiger Touch、Arena、Sapphire 等 | [MA 参数表](https://help.malighting.com/grandMA3/2.5/HTML/system_parameter_calculate.html)、[Titan 型号](https://manual.avolites.com/docs/about-the-consoles) |
| M01-02 | 电脑控制 | onPC 配合授权硬件、控制面和节点 | Titan Go 配合 T1／T2／T3／Mobile 等 | [MA 扩展](https://help.malighting.com/grandMA3/2.5/HTML/system_parameter_expand.html)、[Titan T1/T2](https://manual.avolites.com/docs/about-the-consoles/t1-and-t2) |
| M01-03 | 离线编排 | 不解锁实际输出也能编程及内置 3D 预览 | Simulator 需要相应硬件／AvoKey，模拟输出有周期性 spoiler | [MA Parameters](https://help.malighting.com/grandMA3/2.5/HTML/system_parameter.html)、[Titan Simulator](https://manual.avolites.com/docs/titan-basics/titan-simulator) |
| M01-04 | 容量计量 | Parameter 不等于 DMX 通道；粗／细通道可能只计一个参数 | 以 DMX Line／Universe、系统许可和本机处理能力区分 | [MA Parameters](https://help.malighting.com/grandMA3/2.5/HTML/system_parameter.html)、[Titan DMX](https://manual.avolites.com/docs/system-settings/dmx-output-mapping) |
| M01-05 | 分布计算 | Processing Unit 提供参数及计算扩展 | TNP 分担输出计算，不自行提升系统许可上限 | [MA 扩展](https://help.malighting.com/grandMA3/2.5/HTML/system_parameter_expand.html)、[Titan DMX](https://manual.avolites.com/docs/system-settings/dmx-output-mapping) |
| M01-06 | 端口扩展 | 普通 Node 和 onPC 解锁 Node 的角色不同 | 面板／Wing／网络节点增加接口或控制面，不等于增加许可 | 同上；[Titan T3](https://manual.avolites.com/docs/about-the-consoles/t3) |
| M01-07 | 无人值守形态 | replay／rack 等角色，运行能力依组合 | D3 Core、TNP Console Mode、开机节目与触发接口 | [MA 参数表](https://help.malighting.com/grandMA3/2.5/HTML/system_parameter_calculate.html)、[Titan D3](https://manual.avolites.com/docs/about-the-consoles/d3)、[Titan TNP](https://manual.avolites.com/docs/titan-net) |
| M01-08 | 平台 | 当前 onPC 文档列 Windows 和 macOS；Windows ARM 不支持 | Titan PC Suite 使用 Windows，移动端遥控另属客户端 | [MA 系统要求](https://help.malighting.com/grandMA3/2.5/HTML/onpc_system_requirements.html)、[Titan Simulator](https://manual.avolites.com/docs/titan-basics/titan-simulator) |
| M01-09 | 第三方可视化授权 | 内置 3D 与第三方可视化输出授权不同，viz-key 有专门用途 | 内置 Capture 与外部完整 Capture 的功能／版本分别核对 | [MA Parameters](https://help.malighting.com/grandMA3/2.5/HTML/system_parameter.html)、[Titan Capture](https://manual.avolites.com/docs/capture-visualiser/capture-show-files) |
| M01-10 | 接口差异 | MIDI、音频、LTC、DC、网络及 DMX 端口依设备 | 官方外部触发表按型号列 Audio／GPIO／MIDI／LTC／WebAPI 等 | [MA Device Overview](https://help.malighting.com/grandMA3/2.5/HTML/device_overview.html)、[Titan 接口表](https://manual.avolites.com/docs/running-the-show/midi-dmx-or-audio-triggering) |

## 容量快查

以下是文档基线中的容量，不是 StageMaster 性能目标。MA 的 Parameter 不能简单除以 512 与 Titan Universe 直接比较。

| MA 设备／系统 | 参数容量或作用 |
| --- | --- |
| full-size／full-size CRV | 20,480 |
| light／light CRV | 16,384 |
| compact、compact XT、replay unit | 8,192 |
| Processing Unit M／L／XL | 分别增加 4,096／8,192／16,384 |
| 控台会话上限 | 262,144；多台控台自身的参数数不相加 |
| 纯 onPC 系统上限 | 4,096；所需解锁硬件与端口数另算 |
| onPC DMX-key／starter | 4,096／1,024 |
| onPC + viz-key | 文档列 512；第三方可视化用途应单独看相应许可 |
| 普通 xPort Node／extension | 本身不提供新增参数额度 |

来源：[MA Calculate Parameters](https://help.malighting.com/grandMA3/2.5/HTML/system_parameter_calculate.html)、[MA Expand Parameters](https://help.malighting.com/grandMA3/2.5/HTML/system_parameter_expand.html)。型号名称中的 “2k” 等旧标记不能直接代替当前参数表。

| Titan 设备 | 本机处理 Line | 系统 Line 上限 |
| --- | --- | --- |
| T1／T2 | 1／2 | 1／2 |
| T3 | 16 | 基础 16；可选许可扩至 64，额外计算需要 TNP |
| D3-010／D3-110／D3 Core | 8／24／16 | 8／24／16 |
| D7／D9 | 32 | 64，超出本机范围需 TNP |
| Titan Mobile／Quartz／Tiger Touch 2／Arena／Sapphire | 16 | 64，超出本机范围需 TNP |
| Simulator | 不作演出输出承诺 | 可模拟 64，但存在 spoiler |

来源：[Titan DMX Settings](https://manual.avolites.com/docs/system-settings/dmx-output-mapping)。19.x 可用 Line 编号至 9999，但同时分配的总量仍受 64 及设备许可限制；低容量设备加载大节目保留编排，只输出其允许范围。

## 对 StageMaster 的吸收建议

以下属于设计建议。首先建立单机能力清单：可计算属性数、DMX Universe 数、物理输出口数、输入协议、媒体能力和硬件平台。它们必须分别表示，不能用一个“支持多少台灯”概括。

桌面、平板、浏览器和盒子共享节目模型，具有不同能力声明。基础版本用实际负载测试确定容量，不以商业产品的授权数推断我们的性能；也不把它们的硬件许可模式照搬为产品要求。
