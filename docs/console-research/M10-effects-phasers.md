# 动态效果、Phaser、Shape 与关键帧

MA3 以属性多步 Phaser 为中心，并提供 Shape、Recipe、Random 等生成方式。Titan 同时提供预制 Shape Generator、Key Frame Shapes 和 Pixel Mapper。名称接近不代表数据结构相同。

## 功能对照

| 编号 | 能力 | grandMA3 | Avolites Titan | 依据 |
| --- | --- | --- | --- | --- |
| M10-01 | 属性循环 | Phaser 用一个或多个属性的多步值形成循环 | Shape Generator 提供按属性分类的预制效果 | [MA Phaser](https://help.malighting.com/grandMA3/2.5/HTML/phaser.html)、[Titan Shape](https://manual.avolites.com/docs/effects/shape-generator) |
| M10-02 | 自定义步骤 | 编辑 Step，可通过预设构建多步 | Key Frame 可手动录制，或用 Palette／Cue 快速构建 | [MA Phaser](https://help.malighting.com/grandMA3/2.5/HTML/phaser.html)、[Titan Key Frame](https://manual.avolites.com/docs/effects/key-frame-shapes) |
| M10-03 | 绝对与相对 | Phaser 可使用绝对／相对层 | 预制位置 Shape 围绕当前基值运动；Key Frame 定义各帧目标 | 同上 |
| M10-04 | 步间过渡 | Width、Transition、Acceleration、Deceleration | 帧时间、宽度、中点、曲线及目标幅度 | 同上 |
| M10-05 | 空间分布 | Phase 与 Selection Grid／MAtricks 结合 | Spread／Phase／Offset、选灯顺序及二维方向 | [MA MAtricks](https://help.malighting.com/grandMA3/2.5/HTML/matricks.html)、[Titan Shape](https://manual.avolites.com/docs/effects/shape-generator) |
| M10-06 | 节拍尺度 | Speed 与 Measure 共同定义周期 | Beats Per Cycle 分频，可匹配帧数或 Spread | [MA Phaser](https://help.malighting.com/grandMA3/2.5/HTML/phaser.html)、[Titan Key Frame](https://manual.avolites.com/docs/effects/key-frame-shapes) |
| M10-07 | 有限次运行 | NShot、方向、结束保持等 Recipe 参数 | Cycles 可限定次数；Key Frame 可按层设置 | [MA Recipe Editor](https://help.malighting.com/grandMA3/2.5/HTML/recipe-sheet.html)、[Titan Key Frame](https://manual.avolites.com/docs/effects/key-frame-shapes) |
| M10-08 | 多属性和多效果 | 属性层与多步、Shape 和 Recipe 配合 | 同灯可运行多个 Shape；Key Frame 有多层效果 | [MA Phaser](https://help.malighting.com/grandMA3/2.5/HTML/phaser.html)、[Titan Shape](https://manual.avolites.com/docs/effects/shape-generator)、[Titan Key Frame](https://manual.avolites.com/docs/effects/key-frame-shapes) |
| M10-09 | 子灯／像素运行 | 选择网格、子灯选择和 Recipe Selection Mode | Super Fixture、Sub Fixture Linear／Group；Key Frame 也可跨 cell | [MA Recipe Editor](https://help.malighting.com/grandMA3/2.5/HTML/recipe-sheet.html)、[Titan Shape](https://manual.avolites.com/docs/effects/shape-generator) |
| M10-10 | 同步与重触发 | Sync 控制可预期的起始相位 | Restart Shapes，可结合相位和节拍设置 | [MA Phaser](https://help.malighting.com/grandMA3/2.5/HTML/phaser.html)、[Titan Shape](https://manual.avolites.com/docs/effects/shape-generator) |
| M10-11 | 停止或屏蔽效果 | Stomp 将相关属性收敛为静态步骤 | Mask FX 屏蔽指定属性／灯具的 Shape、Key Frame 或 Pixel Map | [MA Phaser](https://help.malighting.com/grandMA3/2.5/HTML/phaser.html)、[Titan Shape](https://manual.avolites.com/docs/effects/shape-generator)、[Titan Pixel](https://manual.avolites.com/docs/effects/pixel-mapper) |
| M10-12 | 与其他播放的关系 | 合成、静态调用及序列设置共同决定行为 | Overlay／LTP 控制效果是否被后续属性变化覆盖 | [MA Sequence Settings](https://help.malighting.com/grandMA3/2.5/HTML/cue_sequence_settings.html)、[Titan Options](https://manual.avolites.com/docs/cues/playback-options) |
| M10-13 | 形状素材 | Shapes 对象含曲线和可选参数，可用于 Phaser Recipe | Shape 文件定义预制效果；Key Frame 是另一编辑方式 | [MA Shapes](https://help.malighting.com/grandMA3/2.5/HTML/shapes.html)、[Titan Shape](https://manual.avolites.com/docs/effects/shape-generator) |
| M10-14 | 随机变化 | Random Generator 控制上下界、速度、相位等变化，仅作用于绝对层 | Pixel Mapper 有 Random 动画及多种随机参数 | [MA Generator](https://help.malighting.com/grandMA3/2.5/HTML/generator.html)、[Titan Pixel](https://manual.avolites.com/docs/effects/pixel-mapper) |
| M10-15 | 现场幅度和速度 | Phaser 参数与 Speed Master 等配合 | Size／Speed Master、推杆幅度／速度及效果淡入淡出 | [MA Phaser](https://help.malighting.com/grandMA3/2.5/HTML/phaser.html)、[Titan Advanced](https://manual.avolites.com/docs/effects/advanced-options) |

## 工作流程与边界

自拟流程：同一批灯分别制作围绕位置素材的圆形运动、两个颜色素材之间的循环、一次性从左到右的亮度波。改变基准位置、灯具数量、顺序和 BPM；停止后检查最终状态，再重新启动确认相位。

应特别区分“幅度为零”“播放暂停”“效果释放”“静态覆盖”。幅度为零可能仍保留运行中的相位。有限次数运行也必须明确一次循环如何计数，往返方向是否算一个循环。MA Recipe 的 Alternate 与 NShot 有明确组合规则；Titan 不应直接沿用 MA 的参数解释。[MA Recipe Editor](https://help.malighting.com/grandMA3/2.5/HTML/recipe-sheet.html)、[Titan Shape Generator](https://manual.avolites.com/docs/effects/shape-generator)。

## 对 StageMaster 的吸收建议

以下属于设计建议。底层用时间函数、步骤曲线、空间相位、基值来源和终止规则描述效果；上层可以同时提供预制波形、关键帧和配方入口。效果计算共享 Rust 核心，Web／桌面只是编辑同一模型。

首版优先亮度、位置、颜色，具备速度、幅度、相位、顺序、曲线、同步和有限次播放。对随机效果保存种子及算法版本，便于预览和离线运行一致；这属于我们的设计要求，不是对两家内部实现的推断。
