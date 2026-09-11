# 遥控、外部触发与 API

遥控界面、外部按钮／推杆映射、时间码输入、追踪位置输入和网络 API 是几类不同接口。它们最终可以调用共同的节目命令，但输入特性和权限范围不同。

## 功能对照

| 编号 | 能力 | grandMA3 | Avolites Titan | 依据 |
| --- | --- | --- | --- | --- |
| M17-01 | 浏览器遥控 | Web Remote 显示主机的用户屏幕配置和操作能力 | 可用 WebAPI 构建专用 Web 控制页面，不视为同等内置完整控台镜像 | [MA Web Remote](https://help.malighting.com/grandMA3/2.5/HTML/network_webremote.html)、[Titan API](https://api.avolites.com/19.2/) |
| M17-02 | 手机／平板 App | 浏览器即可连接对应站点 | Titan Remote：Keypad、Fixture、Group、Palette、Cue 等 | [MA Web Remote](https://help.malighting.com/grandMA3/2.5/HTML/network_webremote.html)、[Titan Remote](https://manual.avolites.com/docs/remote-control/operating-the-remote) |
| M17-03 | 遥控编程 | 根据登录用户权限操作主机功能 | 遥控可录 Group／Palette／Cue，属性轮和简单 Fan | 同上 |
| M17-04 | 遥控自身状态 | 可不同用户登录；同 Profile 的命令行可关联 | 遥控有自己的编程器，离开后改动不会因关 App 自动释放 | 同上 |
| M17-05 | 连接和发现 | IP／二维码、分辨率／连接数配置 | 局域网发现及手动 IP，显示响应时间和连接状态 | [MA Web Remote](https://help.malighting.com/grandMA3/2.5/HTML/network_webremote.html)、[Titan Setup](https://manual.avolites.com/docs/remote-control/setting-up-the-remote)、[Titan Remote](https://manual.avolites.com/docs/remote-control/operating-the-remote) |
| M17-06 | MIDI 映射 | Note／Attack／Decay／CC 等触发模式 | MIDI Learn，映射硬件动作或节目对象 | [MA MIDI](https://help.malighting.com/grandMA3/2.5/HTML/remote_inputs_midi.html)、[Titan External](https://manual.avolites.com/docs/running-the-show/midi-dmx-or-audio-triggering) |
| M17-07 | MSC | 收发、设备／组、Executor 映射 | MSC 预设映射与官方列出的命令子集 | [MA MSC](https://help.malighting.com/grandMA3/2.5/HTML/remote_inputs_msc.html)、[Titan External](https://manual.avolites.com/docs/running-the-show/midi-dmx-or-audio-triggering) |
| M17-08 | DMX／sACN 触发 | DMX Remotes，可设置分辨率及会话变化触发条件 | DMX／sACN 输入映射，区别于把输入直接合并到输出 | [MA DMX Remote](https://help.malighting.com/grandMA3/2.5/HTML/remote_inputs_dmx.html)、[Titan External](https://manual.avolites.com/docs/running-the-show/midi-dmx-or-audio-triggering) |
| M17-09 | 接点触发 | DC Remote 按设备信号起点映射 | GPIO 接点，机型接口不同 | [MA DC](https://help.malighting.com/grandMA3/2.5/HTML/remote_inputs_dc.html)、[Titan External](https://manual.avolites.com/docs/running-the-show/midi-dmx-or-audio-triggering) |
| M17-10 | OSC | UDP／TCP，推杆／按键／对象及命令行收发；不支持 OSC Bundle | 本次未核实原生同等 OSC；不能把 WebAPI 当作 OSC | [MA OSC](https://help.malighting.com/grandMA3/2.5/HTML/remote_inputs_osc.html) |
| M17-11 | HTTP API | 本次未核实与 Titan 同构的官方 HTTP 对象 API | HTTP 4430、JSON、Get／Set、脚本方法、Handle 查询 | [Titan API 19.2](https://api.avolites.com/19.2/) |
| M17-12 | 外部追踪 | PSN 映射追踪点到 MArker，支持轴映射与反向 | 本次未核实同构原生 PSN／XYZ 追踪链路 | [MA PSN](https://help.malighting.com/grandMA3/2.5/HTML/remote_inputs_psn.html) |
| M17-13 | 场馆简化面板 | 可基于用户权限、布局和 Web Remote 配置 | D3 Touch 的分区场景／宏按钮及锁定 | [MA Web Remote](https://help.malighting.com/grandMA3/2.5/HTML/network_webremote.html)、[Titan Touch](https://manual.avolites.com/docs/remote-control/programming-touch-panels) |
| M17-14 | 外部控制接管 | 按协议映射和执行对象处理 | Level Match、Set／Fire／Re-Fire At Level 等区别接管与重发 LTP | [Titan External](https://manual.avolites.com/docs/running-the-show/midi-dmx-or-audio-triggering) |
| M17-15 | 音频／DJ 来源 | Sound 输入和 BPM，另可通过协议集成 | Audio Trigger、Pioneer Pro DJ Link Bridge 与 BPM Master | [MA Sound](https://help.malighting.com/grandMA3/2.5/HTML/sound.html)、[Titan Pioneer](https://manual.avolites.com/docs/running-the-show/linking-pioneerdj-system-to-titan) |
| M17-16 | API 运行与删除分离 | 不在本次比较中推断同构 HTTP 方法 | `KillPlayback` 停止播放，不是删除；以方法页语义纠正简介中不严谨的描述 | [Titan KillPlayback](https://api.avolites.com/19.2/api/Playbacks.KillPlayback.html) |
| M17-17 | API 强制设置电平 | 以已核实的 OSC／执行器入口为准，不推断 HTTP 等价 | `FirePlaybackAtLevel` 不做 Level Match，未加载时会加载，`alwaysRefire` 可先停止再触发 | [Titan FirePlaybackAtLevel](https://api.avolites.com/19.2/api/Playbacks.FirePlaybackAtLevel.html) |

## 移动端的实际边界

MA Web Remote 文档分别给 onPC 最高五个连接、控台／replay unit 最高两个连接，并限制可登录用户。Titan Remote 的 Cue 屏操作 Playbacks 窗口中的对象，不能直接触发所有物理推杆／Executor 上的 Cue；需要相应对象副本或另一接口。[MA Web Remote](https://help.malighting.com/grandMA3/2.5/HTML/network_webremote.html)、[Titan Operating the Remote](https://manual.avolites.com/docs/remote-control/operating-the-remote)。

Titan API 索引已经提供 19.2 版本，但简介仍有旧版本示例和个别文字不严谨处。实际接口应以相应方法页、目标版本及实机结果核对，不能把“能通过 API 自动化”理解为不受授权、型号和上下文限制。

## 工作流程与 StageMaster 建议

自拟流程：平板选灯并改位置，桌面查看修改来源；平板断网后重连，检查临时值的释放策略。再将一个 MIDI 推杆和 Web 推杆映射到同一播放，验证接管、匹配、重复消息和归零行为。

以下属于设计建议。桌面、移动和 Web 共用 TypeScript 命令模型／React 控件，通过本地连接或远程网关访问同一 Rust 执行实例。手机可提供真实的简化编排能力，不必仅复制桌面屏幕。

未来云端控制增加身份、设备绑定、权限和命令确认，控制端断网不影响现场节目。不要把局域网 API 直接等同公网控制服务，也不要把浏览器直接发送 RS485 作为跨平台基础假设。
