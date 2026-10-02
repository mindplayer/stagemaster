# 演出循环区段

依据 [ADR-096](../development/decisions/PRODUCT-ADR-096-performance-loop-sections.md) 与 [AUDIO-020](../development/tasks/AUDIO-020-performance-loop-sections.md)。当前实现到工程数据／编辑命令与独立调度核心；原生多区段音频、运行控制和可见编辑尚未接通，不能据此宣称正式循环可播放。

## 工程归属与兼容

`AudioTimeline.loopRegions` 为裁切后音乐时间线上的区段，省略等价空数组。每段保存 UUID、中文名称、整数毫秒半开范围 `startMs..endMs`、`plays`、`enabled` 与 `locked`。总计最多 128 段，按开始时间排序；停用段仍占据区间，不允许重叠或嵌套。播放次数为 `{kind:"count",count:1..4294967295}`，包含第一遍；持续循环为 `{kind:"untilExit"}`，不能用零次表示。

非空区段要求 `media.audio-loop-regions@1`，删除所有区段可保留声明，移除音乐清除声明。旧工程不自动增加字段。严格 schema 和 Rust 检查共同拒绝未知字段、重复身份、缺少能力、无效范围和超容量；使用既有 8 MiB 工程上限。区段不包含音频副本、场景副本或 DMX 帧。

## 编辑命令

全部通过 `EditCommand::Audio { command: AudioEdit::LoopRegions { command } }`，复用工程事务、历史和保存。

| `AudioLoopEdit` | 行为 |
| --- | --- |
| `Add {name,startMs,endMs,plays}` | 核心分配身份，默认启用且未锁定，按时间排序 |
| `Put {region}` | 只修改已存在未锁定区段，须保持原启停和锁定字段 |
| `Edit {ids,action}` | 1–128 个唯一有效目标，整组成功或整组拒绝 |

组操作 `Move／Copy {destinationMs}` 以最早所选的新起点为基准，保持原相对间隔、长度、名称、次数和启停；复制分配独立身份并解锁。`Remove {}` 留下原音乐；`Enabled {enabled}` 只切换是否参与编排；`Locked {locked}` 显式锁定／解锁；`Plays {plays}` 统一播放方式。锁定项允许复制和显式解锁，其余组修改含锁定对象即整组拒绝。相同值不产生多余历史；失败保持工程、版本与重做分支。

音乐裁切保持局部毫秒坐标，导致任一循环越界则拒绝，不自动删段、挤压或移动。旧音乐替换继续要求先移除，可以撤销；定位原文件保持内容摘要匹配。

## 独立调度核心

`stagemaster-playback::LoopSchedule::new(duration, regions)` 接受一致的调用方整数刻度，`LoopRegion {start,end,plays}` 与 JSON、音频、UI、云端及设备无关。`LoopPlayback::new(schedule,tick)` 拥有不可变编排和运行游标。

- `position()` 返回源位置、当前区段索引、总播放次数中的当前遍数、边界退出状态和结束状态。空隙／结束处无区段与遍数。
- `advance(consumedTicks)` 只按调用方已消费刻度推进；固定次数结束进入后段，持续段一直回环。大增量用整数商／余数跳过重复，工作量随区段数有界；成功路径不分配，不读取系统时钟，数值失败不改变原游标。
- `set_exit_at_end(regionIndex,requested)` 申请或取消在本圈末尾继续；重复请求幂等，目标不是当前区段即拒绝。宿主还必须绑定播放实例，不能让旧会话控制新播放。
- `seek(tick)` 显式开始新的局部进入，当前遍数置 1 并清除退出请求，之前区段不重放。越界保持原状态。暂停期间不推进；停止由宿主定位到 0。

`AudioTimeline::compile_loops(ticksPerSecond)` 生成 `CompiledAudioLoops {region_ids,schedule}`，只纳入启用段；身份数组与调度索引一一对应，编译结果独立于后续工程编辑。统一用整数 `ms * rate / 1000` 换算端点，拒绝零时基或量化消失的范围。此接口不运行解码器、不申请音频设备。

## 音频适配出口仍待完成

原生播放必须从真实样本消费获得同一游标，暂停保存圈数与退出意图，定位重置、停止归零，边界退出／取消具有播放实例保护。循环 PCM 和流式队列须有总预算；多区段不等于整场 PCM 常驻，回环不得每圈重新解码。准备、取消、版本改变、内存不足和欠载均须验证后才替换已有可用状态。

在该适配接通前，桌面明确拒绝载入启用正式循环的音乐；正在使用旧线性播放器时，提交启用区段会释放旧播放，避免静默忽略次数。此保护属于 AUDIO-020 的中间状态，接入真实播放后由运行快照／准备状态替代。临时局部试听仍是 AUDIO-005 的独立功能，不能冒充本契约。

未增加设备音乐时间线导出，显式选择单场景／单列表生成设备包仍按原契约；这不包含音乐与循环区段。真实输出、媒体混音和机构动作不在本项实现范围内。
