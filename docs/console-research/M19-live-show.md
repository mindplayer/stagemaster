# 现场演出、Set List 与临时改动

现场能力不只是“能播放已编好的 Cue”。快速切歌、临时重排、叠加效果、撤销临时覆盖、查看异常来源和恢复熟悉的操作布局，决定了系统能否作为专业单机使用。

## 功能对照

| 编号 | 能力 | grandMA3 | Avolites Titan | 依据 |
| --- | --- | --- | --- | --- |
| M19-01 | 多播放即时演出 | Executor、Sequence、Preset、Master 与 Layout 协作 | Playbacks、Quick Palettes、Masters、Shapes 等 | [MA Playback](https://help.malighting.com/grandMA3/2.5/HTML/cue_playback.html)、[Titan Controls](https://manual.avolites.com/docs/running-the-show/playback-controls) |
| M19-02 | 歌单组织 | 可组合 Pages／Data Pools／Macros；本次未确认同名原生 Set List 对象 | Set List 将曲目与播放页、备注、Workspace 和宏关联 | [MA Data Pools](https://help.malighting.com/grandMA3/2.5/HTML/datapool.html)、[Titan Set List](https://manual.avolites.com/docs/running-the-show/set-list-window) |
| M19-03 | 曲目重排与跳过 | 基于已有对象和宏构建，行为由编排决定 | Track 复制／移动／删除、Park Track、多歌单 | [Titan Set List](https://manual.avolites.com/docs/running-the-show/set-list-window) |
| M19-04 | 每首歌恢复入口 | 页面、视图、宏可联动 | 全歌单宏每次切曲执行，单曲宏仅对指定 Track 执行 | [MA Macros](https://help.malighting.com/grandMA3/2.5/HTML/macros.html)、[Titan Set List](https://manual.avolites.com/docs/running-the-show/set-list-window) |
| M19-05 | 互斥播放集合 | Tags 的 Kill Instant／Kill Delayed 分别在新播放启动时或完成淡入后关闭同 Tag 的其他播放，另有引用级 Protect | Playback Groups 可设置互斥及何时关闭其他成员 | [MA Tags](https://help.malighting.com/grandMA3/2.5/HTML/tags.html)、[Titan Controls](https://manual.avolites.com/docs/running-the-show/playback-controls) |
| M19-06 | 现场素材过渡 | Programmer／Executor Time、预设和时间图层 | Palette Fade／Overlap、Master Time／Overlap | [MA Time Control](https://help.malighting.com/grandMA3/2.5/HTML/masters_grand_time.html)、[Titan Palette Time](https://manual.avolites.com/docs/palettes/timing-with-palettes) |
| M19-07 | 预备下一组合 | Preview／Blind 与播放控制 | Scene Master 预备、提交、自动反向／手动提交 | [MA Preview](https://help.malighting.com/grandMA3/2.5/HTML/preview.html)、[Titan Controls](https://manual.avolites.com/docs/running-the-show/playback-controls) |
| M19-08 | 临时冻结／静态覆盖 | Freeze、Stomp、Pause 是不同操作 | Shape Mask、Playback 优先级及暂停等 | [MA Programmer](https://help.malighting.com/grandMA3/2.5/HTML/operate_programmer.html)、[MA Phaser](https://help.malighting.com/grandMA3/2.5/HTML/phaser.html)、[Titan Shape](https://manual.avolites.com/docs/effects/shape-generator) |
| M19-09 | 分组强度控制 | Group Master 的多种模式 | Scale／HTP／Limit／Take Over／Disabled | [MA Groups](https://help.malighting.com/grandMA3/2.5/HTML/group.html)、[Titan Controls](https://manual.avolites.com/docs/running-the-show/playback-controls) |
| M19-10 | 节拍和速度控制 | Speed／Rate／BPM 及声音来源 | Tap Tempo、BPM／Rate／Size，Pioneer 自动 BPM | [MA Speed](https://help.malighting.com/grandMA3/2.5/HTML/masters_speed.html)、[Titan Pioneer](https://manual.avolites.com/docs/running-the-show/linking-pioneerdj-system-to-titan) |
| M19-11 | 换页时继续操作 | Executor 页与 Auto Fix | Handle Lock、Transparent Lock 和换页保持 | [MA Settings](https://help.malighting.com/grandMA3/2.5/HTML/cue_sequence_settings.html)、[Titan Options](https://manual.avolites.com/docs/cues/playback-options) |
| M19-12 | 快速定位异常 | 来源层、活动播放、Masters Window、对象 Locate | Active Playbacks、各 Master 状态、其他编程器标记 | [MA Masters](https://help.malighting.com/grandMA3/2.5/HTML/masters.html)、[Titan Controls](https://manual.avolites.com/docs/running-the-show/playback-controls)、[Titan Users](https://manual.avolites.com/docs/titan-basics/multi-user-operation) |
| M19-13 | 临时节目修改 | Update、Cue Only、Preset 引用和 Preview | Include、Update、Palette Quick Merge 与 Cue List 编辑 | [MA Update](https://help.malighting.com/grandMA3/2.5/HTML/cue_update.html)、[Titan Edit Cues](https://manual.avolites.com/docs/cues/editing-cues) |
| M19-14 | 场馆日常操作 | 受限用户、布局、Agenda 及运行对象 | Venue Mode、Startup Show／Playback、D3 Touch | [MA Users](https://help.malighting.com/grandMA3/2.5/HTML/user.html)、[MA Agenda](https://help.malighting.com/grandMA3/2.5/HTML/agenda.html)、[Titan User Settings](https://manual.avolites.com/docs/system-settings/user-settings)、[Titan Touch](https://manual.avolites.com/docs/remote-control/programming-touch-panels) |

## 现场恢复需要分层

释放临时频闪、清某用户的编程器、复位 Master、关闭全部播放和系统恢复是不同动作。MA 官方明确禁止在现场演出中使用 Panic Macro，它会重设大量运行与协议状态；不能把它当成常规黑场按钮。[MA Panic Macro](https://help.malighting.com/grandMA3/2.5/HTML/ts_panic_macro.html)。

Titan Group Master 被设为 Disabled 或移到无推杆 Handle 后可能保持当前电平，这说明“禁用控件”不一定等于“恢复中性值”。这类行为应进入验收案例。[Titan Playback Controls](https://manual.avolites.com/docs/running-the-show/playback-controls)。

## 工作流程与 StageMaster 建议

自拟演练：主唱临时加一首歌，调整歌单顺序，切到新播放页，叠加一次性扫光，再释放频闪并保持底色；随后用平板修正一台灯位置，桌面完成 Cue Only 更新，检查下一首歌不受污染。

以下属于设计建议。首版提供可保存演出页面、可见的活动播放、效果速度／幅度、临时覆盖及明确释放。歌单对象应引用节目和页面、备注及初始化动作，避免直接依赖推杆物理编号。

把“恢复基础演出状态”设计为可查看内容的动作集合，限定影响范围。日常工作人员的手机／墙面控制仅展示分区场景，专业灯光师仍可进入完整编程模式；两者使用同一节目与命令模型。
