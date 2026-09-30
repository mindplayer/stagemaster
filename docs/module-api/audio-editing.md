# 音乐波形与灯光卡点

实现任务 AUDIO-001／AUDIO-002；架构决定 ADR-053／054。核心不绑定 UI、文件系统或 ESP32，所有音频功能位于主机。

PREVIS-002／ADR-055 将音频会话提升为工作台唯一所有者，音频编辑和公共三维进度共享声音、波形及原生快照。三维下方提供音乐播放／暂停／停止和 10 ms 步长定位；拖动期间只显示候选位置，释放发送一次 Seek，Esc／失焦／指针取消不提交。播放头定位不写工程历史。该进度针对音乐裁切范围，不为等待手动推进的场景列表伪造总时间。

切换工作区不卸载会话、不重复加载仍有效的原生声源；重新进入音频页先检查声音所有权，被其他预览释放后才重载。三维仅打开时不加载音乐，明确播放音乐才准备。资源／范围／工程变化清理旧显示并拒绝迟到加载，排队操作绑定发起时工程代次；正常卡点编辑不重置声音。音频时间和灯光求值仍以 Rust 为准。

## 模块接口

- `stagemaster-project::Document::audio_timeline()` 返回持久音乐与卡点；`EditCommand::Audio` 提供 SetAsset／Trim／PutMarker／RemoveMarker／Clear。编辑是全工程原子事务，复用撤销／恢复点／保存；被卡点引用的场景不能直接删除。
- `AudioTimeline::scene_at(time_ms)` 找到当前生效的最近已绑定卡点；仅作节奏标记的点不会打断前一个场景。开始前用灯具默认值，场景采用隔离硬切、动态效果以该点为起点；不累积追赶已跳过的时长。
- `stagemaster-audio::analyze(path,cancelled)` 流式解码峰值；`Resources` 处理摘要、归档、缺失／损坏和受控重定位；`Transport` 维护单一音频输出声源、软件消费位置、试听音量和输出故障。解码／音频设备适配使用锁定 rodio 0.22.2、CPAL 0.17.3、Symphonia 0.5.5。启用 WAV PCM、MP3、FLAC，未启用其他格式以控制依赖。
- 桌面 `audio_prepare(generation, import|load|locate)` 准备资源／波形，迟到工程代次拒绝。Import 只返回已准备资源，不直接编辑；界面随后通过同一工程命令添加，故可以撤销。Load／Locate 载入暂停的试听，不自动发声。
- `audio_request(generation, command)`：Snapshot／Play／Pause／Stop／Seek `{positionMs}`／Volume `{percent}`；用户控制串行发出，轮询没有写权限。工程切换／音频范围改变会释放旧声源，普通卡点编辑保留音乐位置，撤销会暂停。正常场景列表载入清除音频试听，二者共用已有 UE 播放预览来源。

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
