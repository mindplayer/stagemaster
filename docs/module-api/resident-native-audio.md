# 后台原生音频驻留

TIME-001／[ADR-118](../development/decisions/PRODUCT-ADR-118-resident-native-audio.md)。这些接口位于 `stagemaster-audio::Transport`，复用原解码、Rodio Player 和采样帧调度。与[受控媒体提供方](controlled-media-provider.md)组合，调用者须是媒体所有者；灯光主调度不打开声卡、不解码、不等待准备。

## 准备和静音挂载

- `load_performance_request(path, in_ms, out_ms, schedule)` 生成正式音频准备任务。无循环时传 `None`，内部使用空循环计划，普通音乐仍有完整消费和回调健康观测。传入循环计划时长度必须与素材范围一致。
- `AudioLoadRequest::prepare(cancel)` 在调度外读取／预读；`apply_load` 仍检查原 Transport 身份和修订。调用者须提供已校验且在播放生命周期内受其管理的不可变资源路径。
- `prime_performance()` 将已准备且暂停的音源挂到输出。实际回调持续拉取零样本，素材位置不变；`render` 在首个完整帧后才存在，`consumption` 保持 `None`。重复静音挂载不更换音源实例，正在播放、结束或故障的音源不能通过该入口偷偷切换状态。
- 新挂载会使先前准备票据过期；旧准备不能覆盖当前路径。`seek_preparation(position, false)`／`apply_seek` 可先安装暂停的新实例，再静音挂载、验证健康和准备灯光。

`prime_performance` 返回成功只表示已接入音频软件路径，不能直接报告声光启动成功。提供方从原回调取样，使用原时刻／序号和位置准备媒体组，等待实际激活后才调用 `play()`。暂停使用原音源的帧边界请求，播放控制不取得操作者租约。

## 实际确认

`performance_observation()` 增加完整 `request: PlaybackRequest`，原 `requested_playing` 保持兼容。提供方检查实例、错误、停止和结束状态，再比较 `snapshot.render.applied == request`；不能只比较布尔值，否则旧一次“已播放”可能冒充新请求完成。

暂停、定位、自然结束及取消分别处理。暂停只更新回调健康，不更新消费；新定位更换音源实例；自然结束保留最终位置和最后真实回调，不生成假心跳。`stop()` 取消当前音源，后续正式播放需重新准备。声卡缓冲、重采样和蓝牙音箱的呈现时刻不是此接口的确认范围。

## 输出绑定

默认 `Transport::default()` 继续使用系统默认设备。后台适配可以通过 `OutputBinding::new(rodio_mixer)` 和 `Transport::with_output(binding)` 明确绑定由外部宿主持有的输出设备或软件消费器。Rodio 类型仅出现在音频适配模块，不进入通用灯光宿主或核心语义。

外部输出所有者负责驱动 Mixer、维持设备生命周期，并在输出失效时调用 `binding.report_failure()`。所有克隆共享终止失败；之后不能自动改走系统默认设备，须显式重新准备输出。`clear()`、载入和停止保留绑定选择。报告失败不是硬件静音回执；提供方还须停止音源、处理所属媒体组的故障状态。没有实际拉取时不会产生新的健康观测。

## 验证与接续

真实 WAV／原 Player／Mixer 验证无循环音源的静音准备、实际发声样本、暂停健康、定位、自然结束、新实例、错误／过期／取消准备和固定输出失败。真实 Host 集成验证操作者退出后，原 Transport 仍按授权请求完成播放、暂停、定位和停止；100 ms 定位产生实际编排的红色通道值。

当前是原生音频适配与原宿主代表性组合，不等于独立可执行程序已迁移音频所有权。下一项在同一 `apps/execution-host` 内组装常驻所有者、不可变资源准备、实际命令／回执与桌面入口；统一普通试听、自然结束、循环回跳和提供方重启仍需跨边界验证。没有自动开启物理声卡、灯具或改用户工程。
