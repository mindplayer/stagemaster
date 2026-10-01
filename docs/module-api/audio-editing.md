# 音乐波形与灯光卡点

实现任务 AUDIO-001 至 AUDIO-005；架构决定 ADR-053／054／061／063／064。核心不绑定 UI、文件系统或 ESP32，所有音频功能位于主机。

PREVIS-002／ADR-055 将音频会话提升为工作台唯一所有者，音频编辑和公共三维进度共享声音、波形及原生快照。三维下方提供音乐播放／暂停／停止和 10 ms 步长定位；拖动期间只显示候选位置，释放发送一次 Seek，Esc／失焦／指针取消不提交。播放头定位不写工程历史。该进度针对音乐裁切范围，不为等待手动推进的场景列表伪造总时间。

切换工作区不卸载会话、不重复加载仍有效的原生声源；重新进入音频页先检查声音所有权，被其他预览释放后才重载。三维仅打开时不加载音乐，明确播放音乐才准备。资源／范围／工程变化清理旧显示并拒绝迟到加载，排队操作绑定发起时工程代次；正常卡点编辑不重置声音。音频时间和灯光求值仍以 Rust 为准。

## 模块接口

- `stagemaster-project::Document::audio_timeline()` 返回持久音乐与卡点；`EditCommand::Audio` 提供 SetAsset／Trim／PutMarker／RemoveMarker／Clear。编辑是全工程原子事务，复用撤销／恢复点／保存；被卡点引用的场景不能直接删除。
- `AudioTimeline::scene_at(time_ms)` 找到当前生效的最近已绑定卡点；仅作节奏标记的点不会打断前一个场景。开始前用灯具默认值，场景隔离求值，默认硬切，可选进入渐变；动态效果以该点为起点，不累积追赶已跳过的时长。
- `stagemaster-audio::analyze(path,cancelled)` 流式解码峰值；`Resources` 处理摘要、归档、缺失／损坏和受控重定位；`Transport` 维护单一音频输出声源、软件消费位置、试听音量和输出故障。解码／音频设备适配使用锁定 rodio 0.22.2、CPAL 0.17.3、Symphonia 0.5.5。启用 WAV PCM、MP3、FLAC，未启用其他格式以控制依赖。
- 桌面 `audio_prepare(generation, import|load|locate)` 准备资源／波形，迟到工程代次拒绝。Import 只返回已准备资源，不直接编辑；界面随后通过同一工程命令添加，故可以撤销。Load／Locate 载入暂停的试听，不自动发声。
- `audio_request(generation, command)`：Snapshot／Play／Pause／Stop／Seek `{positionMs}`／Volume `{percent}`；用户控制串行发出，轮询没有写权限。工程切换／音频范围改变会释放旧声源，普通卡点编辑保留音乐位置，撤销会暂停。正常场景列表载入清除音频试听，二者共用已有 UE 播放预览来源。

UX-022 在同一波形视窗下投影灯光场景区间，纯节奏点不切段，最后一段延续至裁切末尾。单击选择卡点、双击仅定位；拖动共享边界更新该卡点时间，保持相邻绑定点顺序与毫秒唯一性，Esc 取消、释放一次提交，键盘 10 ms／Shift 1 s 微调。AUDIO-003 增加进入渐变范围显示及边界约束：不能挤占前段渐变或让本段渐变超出末端；精确输入同样由核心原子校验。紧凑布局默认收起全曲导航，播放头复用同一宿主样本与动画回调。

## 灯光进入渐变

[AUDIO-003 / ADR-061](../development/decisions/PRODUCT-ADR-061-audio-lighting-transitions.md) 在 AudioMarker 增加可选 `fadeMs`。省略／0 保留旧行为；非零要求 `media.audio-transitions@1`，必须绑定场景且不超过下一绑定点或裁切末尾。纯节奏标记不影响渐变。清除音乐同时清除两项音频能力。

`Document::compile_audio_marker(marker_id: Option<&str>)` 从已验证文档编译一个段落：前一场景在该边界的输出快照作为渐变起点；新场景效果从此处计时，旧效果不继续运行。复用 Player 的连续属性线性混合与功能属性直接切换。前一渐变必须已在边界前结束，因此最多编译两个单场景、缓存一个播放器，无全曲追帧或大计划。返回计划的 defaults 是本段边界快照，仅用于这次宿主音频进入；不作为灯具档案默认值或普通独立节目导出。首段用真正档案默认值。

桌面仍以原生音频游标驱动；后退、工程版本／卡点变化重建，暂停和任意定位与顺序播放一致。编辑源音频、范围与清理资源规则不变。当前播放包导出独立场景／列表，不含音乐、卡点调度或本段渐变，界面明确提示。

## 持久数据与资源

在既有草案格式添加能力 `media.audio-editing` v1。`media.systems`／`objects` 当前必须为空；可选 `media.audioEditing` 是受限的本机编排，不声称通用多轨 `timelines` 已执行。

```json
{
  "asset": {"digest":"<64 位小写 SHA-256>","fileName":"音乐.wav","extension":"wav","durationMs":32000},
  "inMs":500,
  "outMs":30500,
  "markers":[{"id":"<UUID v4>","name":"第一拍","timeMs":2000,"sceneId":null}]
}
```

Schema 要求真实 UUID／摘要，以上只是字段示意，不是可打开的工程。标记以裁切起点为零；标记时间有序、不重复且小于裁切时长；场景引用需要存在。最多 512 点，源音频最长 1 小时。UI 支持播放时按 M、点击／拖动播放头、拖动卡点、毫秒精确输入、缩放／显示全曲、吸附／关闭吸附、跟随播放、搜索、解除绑定、删除／撤销、取消草稿；拖动一次只提交一次。

完整音频不会写进 JSON，也不会进入 ESP32 包。开发资源缓存位于项目 `data/audio/`；发布构建位于应用本地数据目录。每次资源装载验证摘要，重定位只能恢复同内容文件。保存先把已验证资源归档到 `<工程文件全名>.assets/<摘要>.<扩展名>`，再提交 JSON；失败保持原工程文件。恢复副本引用本机缓存。跨机器需携带 JSON 及同名 .assets 目录；缓存清理／资源管理界面尚未实现。

## 有界资源和现实边界

文件最多 512 MiB、单声道或立体声、8–192 kHz。临时 `Waveform {durationMs,bucketMs,channels}` 每 10 ms、每声道保留真实有符号 `[最小值,最大值,…]` 包络；最多每声道 720,000 个 f32，两声道单份最多 5,760,000 B。桶边界按完整源帧计算，22.05 kHz 等非整数桶采样率不拆分左右声道；末尾空桶不输出。前端裁切包络后交 WaveSurfer，视窗最大 400 px/s；裁切边缘存在不足 10 ms 的显示近似，不是逐采样编辑。

WaveSurfer.js 7.12.12 只负责波形、时间刻度、全曲概览，库不接收音频 URL、不开启声音。宿主快照驱动播放头，最多 120 ms 的插值仅用于显示，停止／定位／卡点命令继续发给 Rust。独立标记覆盖层保留吸附／键盘／一次拖动一次事务／Esc 取消；概览移动视窗不改变音乐位置，显示幅度不改变音量。组件卸载调用 `destroy()`；活动视窗附近的画布由库按需生成和回收。许可证随 `public/licenses/wavesurfer.txt` 分发。

包络不进入工程或撤销历史。IPC JSON、JS 数组、Float32Array 和画布另有副本与开销，5.76 MB 不是应用总占用。导入最多一个任务，有取消和 120 秒解码期限；资源缓存限制 4 GiB，文件复制／摘要固定 16 KiB 缓冲，不整段解码 PCM。解码／绘图库内部缓冲仍归其实现，这不是任意恶意媒体永不耗尽资源的证明。

本轮是电脑上的音乐与灯光预演，软件消费时间不等于声卡 DAC 或蓝牙音箱的实际到声时间。没有跨设备精确同步、自动节拍检测、多轨混音、视频解码、手机后台或音频包下发。输出异常不自动选择其他声卡。ESP32 固件保持无真实 DMX 输出。

验收复现：`python3 tools/audio/make_probe.py`；显式播放测试使用 `CARGO_HOME="$PWD/tmp/cargo-home" TMPDIR="$PWD/tmp" cargo run -p stagemaster-audio --example audition --locked --offline -- data/AUDIO-001/节奏验收.wav`。此命令会低音量输出测试信号，不属于自动单元测试。组件隔离页 `apps/ui-prototype/tests/audio-panel.html` 不进产品构建，也不输出声音。

## 卡点成组编辑

[ADR-063](../development/decisions/PRODUCT-ADR-063-audio-marker-groups.md)：`AudioEdit::EditMarkers { ids, action }` 接受 1–512 个唯一有效卡点，action 为 move／copy（destinationMs）或 remove。目标时间是最早选中卡点的新位置，输入 ID 顺序不影响间隔；Rust 为复制生成 UUID。只更新选中项，最终碰撞／越界／渐变或容量冲突原子拒绝。使用既有卡点格式、场景引用及进入渐变，未新增工程能力；旧命令及播放行为不变。一次操作一个撤销节点。

批量视图保留筛选外选择并显示数量，独立目标输入／删除确认，选择和操作均不移动音乐游标。灯光段落边界仍由相邻绑定卡点决定，不宣称能够剪切波形或保留独立片段的隐藏结束状态。

## 原生局部循环试听

[ADR-064](../development/decisions/PRODUCT-ADR-064-local-audio-loop.md)：`audio_request` 增加 `SetLoop {range: {startMs,endMs}|null}`，Position 增加 `loopRange`（无范围为 null）。只有暂停时可配置，0.100–60 秒、单／双声道、64 MiB PCM 上限。配置不修改工程、撤销或播放包；当前实例清空／音频变化释放，普通灯光编辑保留。

`Transport::loop_request` 获取校验后的不透明请求，`LoopRequest::prepare` 在工程锁外预解码；宿主最多一个准备任务；`apply_loop` 再检验传输修订、资源路径和裁切起点，宿主同时检验工程代次。失败／过时返回错误，不覆盖旧范围。新旧缓存可短时并存，音频库／系统缓冲另计；不宣称整个应用只占 64 MiB。循环源无逐圈缓存增长和磁盘访问。

范围内位置保留、范围外设为起点；播放按样本帧连续取模，暂停保留位置，停止返回范围起点，定位到范围外（含右端点）会退出循环并定位。清除范围保留当前暂停位置。灯光继续依原生消费位置重算；WaveSurfer 的插值仅显示回环，不驱动音频或灯光。当前不是正式演出区段循环／圈数／边界退出／跨设备采样同步，不自动修饰切口音频。

## 独立灯光片段

[AUDIO-006 / ADR-067](../development/decisions/PRODUCT-ADR-067-audio-lighting-clips.md)：可选 `lightingClips` 与 `media.audio-clips@1` 一起存在（空数组仍为片段模式），省略时完整兼容卡点绑定。显式 `ConvertLightingClips` 从旧绑定生成独立 UUID 区间，保留原卡点但清除场景／渐变绑定，一次历史，可撤销；不自动改写旧文件。

`AudioLightingClip {id,name,sceneId,startMs,endMs,fadeMs,locked}` 为裁切后毫秒半开区间、最多 512 段、单轨非重叠。`AddLightingClip` 由核心分配 ID；`PutLightingClip` 只更新已存在且未锁的对象；`CopyLightingClip {id,startMs}` 保持时长、渐变和场景引用，分配独立 ID 并解除锁；`RemoveLightingClip`／`SetLightingClipLock` 显式删除／锁定。卡点编辑不会移动片段；越界、碰撞、缺失场景、超容量、锁定变更均原子拒绝。正式 schema 与独立格式审计同步。

`lighting_at(t)` 是统一只读调度身份；`compile_audio_lighting(id)` 按轨道模式编译。片段效果默认从零运行；有 effectOffsetMs 时从指定效果源时间继续，只有紧邻前段才使用它的边界快照进入渐变；空隙和音乐结束采用档案默认值。末端不包含于片段，默认值不承诺全黑。沿用 ADR-061 直接切换属性与最多两个场景编译，不引入 UI 时钟、第二音频源或设备调度。

界面单独片段目录／搜索与右侧精确属性；波形片段本体移动保持长度、两端分别调整，手势限制于相邻区间且支持节奏／边界吸附，精确输入可跨越其他片段搬移到合法空隙。按键 10 ms／Shift 1 s；Esc／失焦／指针取消不提交。片段与卡点共享导航／从此处预演／三维，片段选择可填入局部循环范围。AUDIO-010 增加效果源偏移与进入渐变结束后的分割；普通改变片段开始从已有源偏移重新运行，完整保相位裁切、渐变内分割、波纹和多轨混合仍后续。

## 灯光片段成组整理

[AUDIO-007 / ADR-071](../development/decisions/PRODUCT-ADR-071-lighting-clip-groups.md)：`EditLightingClips { ids, action }` 接受 1–512 个唯一有效片段。`move`／`copy` 的 `destinationMs` 是最早所选的新起点，`remove {}` 留下空隙。按原时间顺序保留间隔、长度、场景引用及渐变；复制由核心生成新 ID 并解锁。移动／删除含锁定项整组拒绝，任何越界、重叠、超容量或失效选择都不留下部分结果。半开区间允许首尾相接，采用 checked 运算防溢出，严格拒绝未知字段。一次工程事务和一次历史，不改持久格式或执行语义。

片段组与卡点组互斥，共用固定属性区；片段搜索／范围选择保留隐藏项并标明数量，删除显示隐藏项确认。组目标仅为显式操作参数，保存不会隐式执行；取消恢复原起点。单片段未应用草稿仍须先收集、校验并应用，错误保持原上下文。目录／组属性／草稿收集独立组件，音乐／三维保持原有单例宿主。

## 片段停用与恢复

[AUDIO-008 / ADR-072](../development/decisions/PRODUCT-ADR-072-lighting-clip-enable.md)：片段 `enabled` 省略／true 延续既有行为，false 需要 `media.audio-clip-state@1`。编辑首次停用自动添加能力，恢复仍保留声明，清除音乐移除。停用片段仍占时间区间、保留引用并受全部格式／锁保护；普通属性修改必须保留状态，不能绕过专用启停操作。

`EditLightingClips {ids, action:{kind:"enabled",enabled:boolean}}` 统一单／多片段原子更新；锁定任一目标即拒绝整组，同值不新增历史。单片段和组复制保持启停、解除锁定。`lighting_at` 忽略停用段，默认值空隙不延续前段；进入后段渐变仅采样紧邻且启用的前段。直接请求编译停用 ID 拒绝；正常音乐定位返回默认值编译。预演版本失效保留宿主声音／游标，TS 不负责输出求值。

目录按播放状态过滤，隐藏选择继续计数；停用有文字／虚线／纹理，不只靠变暗，依然可编辑、复制和恢复。单片段源参数草稿保护保持，临时输出总控与音乐音量不受影响。


## 保持动态效果进度的分割

[AUDIO-010 / ADR-073](../development/decisions/PRODUCT-ADR-073-audio-clip-effect-offset.md)：`effectOffsetMs` 缺省 0，非零须 `media.audio-clip-offset@1`，偏移加片段长度不得超过 3,600,000 毫秒。只偏移动效采样，不改变局部渐变或音乐游标；前段边界采样也使用其偏移。

`splitLightingClip {id,timeMs}` 只接受片段内部、进入渐变已完成的位置，左段身份不变，右段新身份／零进入渐变／累积源偏移，原启停保持，锁定及容量错误原子拒绝；一次历史、可以重开恢复。`resetLightingClipEffectOffset {id}` 显式归零；普通 put 不得改写偏移。复制／成组移动保留源偏移。界面精确输入、读取原生播放头、取消输入、错误定位与结果选择均接同一命令，不把插值显示时刻用作分割时间。完整保相位裁切与渐变内分割尚未开放。

### AUDIO-011：时间线组移动手势

目录与时间线共享选择；框选／移动所选是临时界面工具。移动仅生成几何提案，超过 3 CSS 像素才改变提案；音乐范围整体限位、首尾吸附、锁定／冲突反馈，松手只发一次已有 editLightingClips.move。最终合法性和历史仍由 Rust 处理。Esc、失焦、页面隐藏、缩放／工程／选择变更取消未提交手势。目标输入或删除确认待处理期间禁用另一条组移动入口；取消后恢复。完整工作流与边界见 [AUDIO-011](../development/tasks/AUDIO-011-timeline-group-motion.md)。
