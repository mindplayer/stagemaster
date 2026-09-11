# 播放器、优先级、合成与总控

播放对象、屏幕／硬件上的操作柄、播放实例，以及最终参与 DMX 输出的贡献需要分开比较。两套系统都支持多路同时播放，但优先级、LTP 保留和关闭后的恢复规则并不相同。

## 功能对照

| 编号 | 能力 | grandMA3 | Avolites Titan | 依据 |
| --- | --- | --- | --- | --- |
| M09-01 | 内容与操作柄分离 | Sequence 不必分配 Executor 也能播放 | Playback 可分配至按钮／推杆；Handle 设置控制操作方式 | [MA Playback](https://help.malighting.com/grandMA3/2.5/HTML/cue_playback.html)、[Titan Options](https://manual.avolites.com/docs/cues/playback-options) |
| M09-02 | 按键和推杆配置 | Executor Configurations、Assign、自定义功能 | Key Profiles、Cue Fader Modes | [MA Executor](https://help.malighting.com/grandMA3/2.5/HTML/executor_configurations.html)、[Titan Options](https://manual.avolites.com/docs/cues/playback-options) |
| M09-03 | 多级优先级 | 多级 LTP、HTP、Swap、Super；Super 可高于编程器 | Low／Normal／High／Programmer／Very High | [MA Playback](https://help.malighting.com/grandMA3/2.5/HTML/cue_playback.html)、[Titan Options](https://manual.avolites.com/docs/cues/playback-options) |
| M09-04 | 强度合成 | HTP 对强度取高，其他属性依 LTP；优先级同时参与 | 常规 HTP／LTP；Cross Fade HTP 可改变常规取高行为 | 同上 |
| M09-05 | 推杆接管 | Soft LTP 决定同级序列强度随推杆接管的过渡方式 | Mode 2 可让 HTP／LTP 随推杆，Mode 3 是整体交叉淡化行为 | 同上 |
| M09-06 | 启停条件 | Auto Start／Stop、Restart、Off When Overridden | Kill Point、Cue List Kill At 0／Kill With Off、Fire First Cue | [MA Settings](https://help.malighting.com/grandMA3/2.5/HTML/cue_sequence_settings.html)、[Titan List Options](https://manual.avolites.com/docs/cue-lists/cue-list-options) |
| M09-07 | 闪光与临时压制 | Flash、Temp、Swap、Kill 及保护设置 | Flash、Swop、Timed Flash 与优先级配合 | [MA Settings](https://help.malighting.com/grandMA3/2.5/HTML/cue_sequence_settings.html)、[Titan Options](https://manual.avolites.com/docs/cues/playback-options) |
| M09-08 | 释放策略 | 关闭、Release、优先级和跟踪需分别处理 | Release Mask／Time 决定哪些属性恢复；Kill 不等于所有 LTP 自动恢复 | [MA Tracking](https://help.malighting.com/grandMA3/2.5/HTML/cue_tracking.html)、[Titan Options](https://manual.avolites.com/docs/cues/playback-options) |
| M09-09 | 手动交叉淡化 | XFade 单／双推杆，Split 与 AB 模式 | Cue List Manual Crossfader，可接管正在执行的淡化 | [MA Settings](https://help.malighting.com/grandMA3/2.5/HTML/cue_sequence_settings.html)、[Titan List Options](https://manual.avolites.com/docs/cue-lists/cue-list-options) |
| M09-10 | 速度与速率 | Speed 与 Rate 可独立或关联，全局 Master 可统一调节 | Speed／Rate／BPM Masters，效果与 Chase 配置对应来源 | [MA Speed](https://help.malighting.com/grandMA3/2.5/HTML/masters_speed.html)、[Titan Playback Controls](https://manual.avolites.com/docs/running-the-show/playback-controls) |
| M09-11 | 效果幅度 | Phaser 参数及 Master／执行操作共同控制 | Size Master、Size On Fader、Speed On Fader、倍率 | [MA Phaser](https://help.malighting.com/grandMA3/2.5/HTML/phaser.html)、[Titan Options](https://manual.avolites.com/docs/cues/playback-options) |
| M09-12 | 现场时间覆盖 | Exec Time 在触发时取值；Rate 能改变正在运行的时间进程 | 播放／闪光时间、速度主控和现场临时时间 | [MA Timing](https://help.malighting.com/grandMA3/2.5/HTML/cue_timing.html)、[Titan Playback Controls](https://manual.avolites.com/docs/running-the-show/playback-controls) |
| M09-13 | 分组与播放总控 | Grand、Group、World、Playback 等 Masters | Grand、Playback、Group 等 Masters，型号及分配方式不同 | [MA Masters](https://help.malighting.com/grandMA3/2.5/HTML/masters.html)、[Titan Playback Controls](https://manual.avolites.com/docs/running-the-show/playback-controls) |
| M09-14 | 换页保护 | Executor 页、Auto Fix 与操作分配 | Handle Locked／Transparent Lock／Unlocked | [MA Settings](https://help.malighting.com/grandMA3/2.5/HTML/cue_sequence_settings.html)、[Titan Options](https://manual.avolites.com/docs/cues/playback-options) |
| M09-15 | 活动播放查看 | Running Playbacks、Sequence 状态、来源层 | 活动 Playback、显示窗口及释放操作 | [MA Running](https://help.malighting.com/grandMA3/2.5/HTML/executor_running_playbacks.html)、[Titan Playback Controls](https://manual.avolites.com/docs/running-the-show/playback-controls) |
| M09-16 | 过滤与隔离 | Sequence 输入／输出 Filter 或 World；同一序列的所有 Executor 共用输出过滤 | Playback Blind 只送可视化；Release Mask 是另一种用途 | [MA Settings](https://help.malighting.com/grandMA3/2.5/HTML/cue_sequence_settings.html)、[Titan Options](https://manual.avolites.com/docs/cues/playback-options) |

## 工作流程：底色、主唱追光、频闪叠加

自拟比较流程：一个 Playback 负责全场底色，一个高优先级 Playback 接管主唱灯，一个临时按钮触发频闪。依次按下／释放频闪、降低主唱推杆、切换页、重新启动底色。每一步查看颜色、位置、强度和快门分别来自谁。

Titan 的 Release Mask 尤其值得单独验证：关闭一个播放并不意味着所有 LTP 属性都会回到之前的来源。MA 的 Soft LTP 也有条件：同属性、同 LTP 优先级和活动状态都影响是否形成期望的接管淡化。[Titan Release](https://manual.avolites.com/docs/cues/playback-options)、[MA Soft LTP](https://help.malighting.com/grandMA3/2.5/HTML/cue_playback.html)。

## 对 StageMaster 的吸收建议

以下属于设计建议。定义统一的播放贡献结构：来源、属性、绝对／相对值、优先级、起始时间、权重、释放策略和时间包络。合成器不能依赖界面按钮最后绘制的顺序；应能解释某个属性为什么输出当前值。

基础版本先明确一套一致的优先级和恢复规则，再通过可配置的操作配置适应 MA／Titan 用户习惯。UI 可以简化，HTP、LTP、快门释放、黑场和总控之间的规则必须完整。自拟验收应覆盖同时启停、相同时间戳、连续换页、推杆归零、暂停恢复和节目切换，使用确定的输入事件序列核对每次输出结果。
