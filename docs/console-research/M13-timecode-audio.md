# 时间码、时间线、音频与节拍

时间码给出“现在走到哪里”，时间线安排“在这个时刻执行什么”，音频提供声音内容或节拍。三者应独立建模，再通过明确的同步关系组合。

## 功能对照

| 编号 | 能力 | grandMA3 | Avolites Titan | 依据 |
| --- | --- | --- | --- | --- |
| M13-01 | 时间节目 | Timecode Show：Track Group、Track、时间段和事件 | Timeline：轨道、Playback 触发与编辑点；Cue List 也可绑定时间码 | [MA Timecode](https://help.malighting.com/grandMA3/2.5/HTML/timecode.html)、[Titan Timeline](https://manual.avolites.com/docs/timelines/creating-a-timeline) |
| M13-02 | 内部与外部时钟 | 内部计时、Timecode Slot、外部信号及生成器 | 内部、系统时间、MIDI／SMPTE、Winamp 等，依硬件配置 | [MA Slots](https://help.malighting.com/grandMA3/2.5/HTML/timecode_slots.html)、[Titan Cue Timing](https://manual.avolites.com/docs/cue-lists/cue-list-timing) |
| M13-03 | 外部格式 | SMPTE/LTC、MIDI Timecode、ArtTimeCode；部分可发送 | 外部时间码与接口支持需按机型核对 | [MA Connections](https://help.malighting.com/grandMA3/2.5/HTML/timecode_external_connections.html)、[Titan 接口表](https://manual.avolites.com/docs/running-the-show/midi-dmx-or-audio-triggering) |
| M13-04 | 现场录制 | 可录操作事件／推杆；选择手动或全部事件及远程事件 | Live Record 可叠加录制和自动简化推杆点 | [MA Settings](https://help.malighting.com/grandMA3/2.5/HTML/timecode_settings.html)、[Titan 创建](https://manual.avolites.com/docs/timelines/creating-a-timeline) |
| M13-05 | 手工编辑 | 事件、轨道、时间段和标记编辑 | 图形／表格编辑、吸附、轮控和手工添加触发 | [MA Events](https://help.malighting.com/grandMA3/2.5/HTML/timecode_events.html)、[Titan 编辑](https://manual.avolites.com/docs/timelines/running-and-editing-timelines) |
| M13-06 | 标记导入 | Marker 对象辅助时间节目编排 | 可导入音频编辑器导出的时间标记；格式应为时间而非小节 | [MA Markers](https://help.malighting.com/grandMA3/2.5/HTML/timecode_markers.html)、[Titan 创建](https://manual.avolites.com/docs/timelines/creating-a-timeline) |
| M13-07 | 偏移和范围 | Offset、Duration、Auto Start／Stop | Offset、Start／Duration、Activate In Range／Kill Out Of Range | [MA Settings](https://help.malighting.com/grandMA3/2.5/HTML/timecode_settings.html)、[Titan Options](https://manual.avolites.com/docs/timelines/timeline-options) |
| M13-08 | 循环和独立排练 | 内部时间循环／暂停；外部来源另按设置处理 | 断开外部关联后本地排练；内部源可循环 | 同上 |
| M13-09 | 跳转时恢复状态 | Assert Previous Events、Go 或 Goto 状态录制 | 中途进入可能受未执行历史 LTP 影响，Release 设置帮助控制 | [MA Settings](https://help.malighting.com/grandMA3/2.5/HTML/timecode_settings.html)、[Titan 编辑](https://manual.avolites.com/docs/timelines/running-and-editing-timelines) |
| M13-10 | 停止后的播放 | 可关闭由时间节目启动的播放，或保留 | Timeline Release 可覆盖所触发播放的释放设置 | [MA Settings](https://help.malighting.com/grandMA3/2.5/HTML/timecode_settings.html)、[Titan Options](https://manual.avolites.com/docs/timelines/timeline-options) |
| M13-11 | 音频素材播放 | Sounds Pool，可作为时间码轨道目标 | 手册描述 Winamp 时间源；不据此推定等同完整多轨音频工作站 | [MA Sounds](https://help.malighting.com/grandMA3/2.5/HTML/sound_pool.html)、[Titan Cue Timing](https://manual.avolites.com/docs/cue-lists/cue-list-timing) |
| M13-12 | 声控与节拍 | Sound Viewer 波形／频带／节拍，BPM Master 可跟随 | Audio Trigger 和 BPM／Rate 主控，支持范围按硬件 | [MA Sound Viewer](https://help.malighting.com/grandMA3/2.5/HTML/sound_viewer.html)、[Titan External](https://manual.avolites.com/docs/running-the-show/midi-dmx-or-audio-triggering) |
| M13-13 | 录制来源控制 | 可选择本用户／全部用户及远程事件 | Timeline 有自己的释放配置，不能假定完全继承当前操作用户 | [MA Settings](https://help.malighting.com/grandMA3/2.5/HTML/timecode_settings.html)、[Titan 创建](https://manual.avolites.com/docs/timelines/creating-a-timeline) |
| M13-14 | 时间码槽及配置归属 | 专题页描述 16 个固定 Slot；槽配置属于站点，不随 Show 传递 | 时间源与 Show／硬件的关系须按 Cue List／Timeline 配置分别核查 | [MA Slots](https://help.malighting.com/grandMA3/2.5/HTML/timecode_slots.html)、[Titan Timing](https://manual.avolites.com/docs/cue-lists/cue-list-timing) |
| M13-15 | 信号锁定与丢失 | Slot 的 Pre Roll 等待有效信号，After Roll 定义丢失后的继续计时范围 | 本次不承诺同样的锁定／自由运行参数，需按目标接口测试 | [MA Slots](https://help.malighting.com/grandMA3/2.5/HTML/timecode_slots.html) |

## 录制不是最终输出快照

Titan Timeline Live Record 不录 Palette 调用、Masters 状态及 Scene Master 触发行为。录制后的重放会受当时 Master 等状态影响。应先把需要重放的内容保存为 Playback，再验证录制结果。[Titan Creating a Timeline](https://manual.avolites.com/docs/timelines/creating-a-timeline)。

MA 的“录所有事件”与“只录手动事件”会改变后续 Cue Command 修改是否影响重放。显示帧率只是读数格式，不等于输入信号真的被转换成该帧率。[MA Timecode Settings](https://help.malighting.com/grandMA3/2.5/HTML/timecode_settings.html)。

## 工作流程与 StageMaster 建议

自拟流程：从音乐开头录制，随后从中间开始、倒退、暂时丢失时间码、恢复、循环并切换内部时钟；逐项比对 Cue、推杆、声音与效果相位。要核对“同一时刻重建的状态”和“事件顺序累计得到的状态”是否一致。

以下属于设计建议。Rust 的执行时钟、时间线求值、音频播放同步接口分别独立；TypeScript 编辑同一事件模型。保留精确时间基准和来源帧率，避免用 UI 刷新周期调度输出。首版需要 Cue 触发、推杆包络、标记、偏移、内部播放和基本音频配合；外部时钟增加锁定、丢失、跳转和恢复状态。

发布包应声明时钟策略、媒体版本及起始状态。云端分发时间线不意味着由云端逐帧控制 DMX，现场设备仍需独立执行。
