# 演出循环区段

依据 [ADR-096](../development/decisions/PRODUCT-ADR-096-performance-loop-sections.md) 与 [AUDIO-020](../development/tasks/AUDIO-020-performance-loop-sections.md)。当前已实现工程数据／编辑命令、独立调度核心、有界原生音源、Transport／桌面运行控制、共享运行栏，以及正式区段目录／单段精确属性。创建／启停／锁定／删除复用原命令，原生独立工程的一次撤销／保存重开已验证；时间线／成组编排、真实音乐／UE 的完整联动仍待完成，不能据此宣称 AUDIO-020 已交付。

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

## 原生音源适配

`stagemaster-audio::PerformanceAudio::prepare(path,inMs,&scheduleMs,cancel)` 接收毫秒编排，宿主可由 `compile_loops(1000)` 提供；读取采样率后按端点转换为样本帧，返回不可变可共享缓存。宿主须提供已核对内容摘要、生命周期内不改写的归档音乐；准备不持有工程锁、不申请声卡，也不改变现有播放器。

- `cached_bytes()` 返回全部循环 PCM 载荷，`sample_rate()`／`duration_ms()` 提供时基和原时间线长度。
- `.source(positionMs,cancel)` 在后台预热普通段落后返回 `(PerformanceSource,PerformanceControl)`，不消费播放帧。每次创建从所选原位置开始局部第 1 遍，不继承另一音源的退出意图；此准备应在工程锁外执行。
- `PerformanceSource` 实现 rodio 的 `Source`；只有完整声道帧消费后才推进 LoopPlayback。重复段取固定缓存，普通段取有界非阻塞队列。准备读到哪里不影响运行游标。暂停必须保留同一个源；销毁重建会重置圈数。`check_ready()` 在宿主替换旧源前再次拒绝已知后台故障或取消，不代替工程版本检查，也不承诺后续 I/O 永不出错。
- `PerformanceControl::snapshot()` 返回源帧位置、当前区段／遍数／已应用退出、待应用退出、控制拒绝、播放故障和音源释放状态。原子发布避免混合两次更新；短暂读冲突返回可重试错误。按 [ADR-115](../development/decisions/PRODUCT-ADR-115-audio-render-health.md)，另提供完整消费与回调健康观测；主机请求、回调确认和声卡呈现分别表达，不能将队列非空当作正在播放。
- `request_exit(index,pass,desired)` 校验并绑定当前源的区段和播放遍数，同值幂等，尚未应用时以最后意图为准；实际帧边界再次核验，迟到操作报告拒绝，既不影响下一段，也不暗中退出同段的下一圈。`pending_exit` 为 `LoopExitIntent {region,pass,requested}`。`cancel()` 通知源在完整帧边界结束，丢弃源则直接释放消费端和解码线程。宿主仍须校验播放实例身份。

预算：最多 64 MiB 重复 PCM，播放次数为 1 的区段不缓存，持续重复也只存一份；预读 128 块，每块最多 1024 帧、最多两声道，样本载荷约 1 MiB，另有固定块和编解码器空间。全局最多两条流式解码线程；新旧缓存替换期可同时存在，64 MiB 不代表应用总内存。文件沿用 512 MiB 上限；准备最多两分钟，每 1024 帧检查取消。精确定位目前从源头数帧，长文件定位可能需要准备时间。

欠载、短数据／解码错误、游标异常明确终止并保留最后真实消费位置，不补零伪造继续运行；控制目标过期是独立可恢复提示。普通段预读失败不被视为整首正常结束。音源不输出 DMX，也不控制 UE；其发布游标供宿主统一采样灯光。这不是已校准的扬声器输出时间。

## Transport 与桌面运行适配

`Transport::load_ticket()` 在文件选择／摘要验证之前捕获音频所有者和修订；`.request(file,inMs,outMs,scheduleMs)` 绑定媒体，`.prepare(cancel)` 在工程锁外建立 `PreparedAudioLoad`。`apply_load()` 再检查所有者／修订和已知源故障，成功才替换旧音源。桌面 `LoadIntent` 同时固定真实工程轨道，不能把已准备的音源绑定到另一个轨道。无启用区段继续使用原线性路径；有启用区段使用本契约的多段音源，取消了原中间版本的拒绝保护。

`play_preparation()`／`seek_preparation(positionMs,playing)` 在需要新音源时返回不可变请求，`.prepare(cancel)` 复用 PCM 缓存并预热；`apply_seek()` 验证后原子替换。正式源在完整帧边界应用暂停／恢复，Player 保持拉取暂停零样本以报告真实回调健康，同一源、遍数和退出意图保持；显式定位创建新实例并从第 1 遍开始。停止释放源并归零，保留 PCM；再次播放生成新实例。自然结束停在音乐终点，再播放从 0 开始。设备打开失败或准备失败保留原可用状态。

音频返回值新增可空 `performance`：`instance`、`region`、`pass`、`exitRequested`、`pendingExit`、`controlProblem`、`ended`、`snapshotPending`、`boundaryMs`、`cachedBytes`。实例和 u64 遍数作为十进制字符串传给界面；区段索引对应启用区段。读取游标冲突时保留上一份一致快照并标记 `snapshotPending`，不将临时冲突报告为音源故障。界面视觉插值最多到 `boundaryMs`，不能自行推算回环或圈数，读冲突时停止插值和退出操作。

桌面运行命令 `{kind:"exitLoop",instance,regionId,pass,requested}` 绑定工程代次、实际加载轨道、播放实例、区段和遍数；过期请求拒绝，暂停期间可以申请／取消，下一完整样本帧边界实际应用。运行命令不改工程内容／历史。真正修改启用区段的身份、范围、次数或顺序会释放旧播放器；改名称／锁定／不影响启用计划的停用段编辑保持实例，界面采用相同的有效计划身份进行准备。

媒体加载、临时试听循环和正式定位共用准备闸门；取消版本在宿主命令进入时固定，取消／暂停／停止令此前等待或解码中的准备失效。最终替换再次检查取消、工程代次和音频修订。界面暂停／停止清掉旧排队批次并直接发送，不等待正在解码的定位；旧“定位后播放”批次不得在停止之后继续播放。准备活动结束前保持进度和取消入口。

正式循环与临时试听循环互斥，启用正式计划时隐藏临时入口。公共三维下的运行栏与独立音频工作区共用 `AudioPerformanceControls`、同一个 `useAudio` 和唯一原生会话；不会产生第二个声音或三维时钟。正式区段目录支持搜索与启停筛选，单段属性按既有工作台草稿／事务编辑；失败保持输入与工程，取消不改内容，锁定须显式解锁。真实声卡延迟、音乐／灯光／UE 联动及时间线／成组编辑仍在 AUDIO-020 后续验收范围内。

未增加设备音乐时间线导出，显式选择单场景／单列表生成设备包仍按原契约；这不包含音乐与循环区段。真实输出、媒体混音和机构动作不在本项实现范围内。
