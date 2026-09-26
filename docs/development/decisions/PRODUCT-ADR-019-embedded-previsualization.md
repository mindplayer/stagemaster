# PRODUCT-ADR-019：应用内三维预演视窗

2026-09-27；PREVIS-001；基线 `dd8a8ce`。用户明确要求三维画面放在舞台大师内部。当前会话评估通过；Mac 原生内部视窗及查看操作已验收，完整预演闭环继续。

产品只有一个工作台：在舞台工作区切换平面布置／三维预演，沿用场地对象、属性与工程历史。取消打开另一个 UE 窗口的产品交互。单独进程只作为可替换渲染适配器的内部机制，核心工程与播放时钟仍归 Rust。用户不需要操作 UE 编辑器。

优先采用 Epic 官方 Pixel Streaming 2 和对应 UE 5.8 前端／信令库，将本机离屏渲染画面传入 Tauri 视窗。官方支持 Mac／VideoToolbox；本项目已实际验证 Apple Silicon 编码、WKWebView 解码和失联提示／隔离；连续操作延迟仍须量化。现有中立几何与灯值 HTTP 桥保留，与视频连接分离。禁止以截图轮询或外部窗口摆在旁边冒充嵌入。

选择该路线是为了复用官方 WebRTC、输入与跨平台实现。暂不自研 macOS 跨进程 Metal 纹理共享，也不把 UE 原生窗口强行重挂为 WKWebView 子窗口。后续若实测延迟／色彩精度不满足操作要求，再在同一视窗接口下引入本机纹理传输；不影响工程格式和灯光语义。

首期仅本机：信令服务绑定 127.0.0.1 动态端口，渲染器与前端使用不同的随机会话令牌；浏览器请求限制来源、单个观看端、载荷大小，无公共网页／STUN／TURN／外网发布。令牌只存在当前进程内存，进入官方日志之前去除连接路径；前端只能得到观看连接，不能得到 Rust 工程编辑凭据。命令消息不开放任意 UE 控制台或文件访问。后台进程随工作台关闭回收。

复用官方 TypeScript 信令库会增加一个后台运行时依赖；开发期使用本机 Node，分发前必须将固定运行时或独立可执行服务纳入安装包。它只承载预演连接，不加入 Rust 核心或 ESP32 固件。尚未完成独立 UE 打包时如实记录为开发验收环境，不宣称客户可免安装使用。

灯位修改仍须遵守 ADR-018 的草稿保护、同版本提交与一次撤销，不能因为画面位于同一窗口就跳过。进入可拖动状态前处理草稿；播放／场景灯值始终来自 Rust。

依据：[Epic Pixel Streaming 参考](https://dev.epicgames.com/documentation/unreal-engine/unreal-engine-pixel-streaming-reference)、[Pixel Streaming 2](https://dev.epicgames.com/documentation/unreal-engine/pixel-streaming-2-overview-in-unreal-engine)、[官方基础库](https://github.com/EpicGames/PixelStreamingInfrastructure)，并核对本机 UE 5.8.3 插件平台名单与公开头文件。前端锁定 `@epicgames-ps/lib-pixelstreamingfrontend-ue5.8@0.1.2`，信令库锁定 `@epicgames-ps/lib-pixelstreamingsignalling-ue5.8@0.2.0`。

本轮审查：同一 Tauri 窗口显示实时 UE 视频，原生中文工具栏经受限数据通道控制透视／俯视／全场／聚焦和工作照明，点击画面可拾取真实灯具。正常退出与渲染故障回收、平面往返重新挂载通过。信令 `ws` 锁定 8.22.0，审计及实际握手测试通过；工程凭据不进入 WebView，日志抽查未含连接令牌。当前不开放三维工程修改入口，客户独立组件及性能验收继续。
