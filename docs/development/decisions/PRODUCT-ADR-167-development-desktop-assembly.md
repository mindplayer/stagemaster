# PRODUCT-ADR-167：内部 Development 桌面组件组装

状态：2026-10-05 已实施，内部 Development 副本的真实 Game／Node／桌面连接和 GPU 画面通过；不是客户发行决定。关联 [PREVIS-007](../tasks/PREVIS-007-development-desktop-assembly.md)、[ADR-165](PRODUCT-ADR-165-packaged-renderer-file-access.md)、[ADR-166](PRODUCT-ADR-166-component-launch-isolation.md)。

## 事实与决定

现有独立 Game 与包内 Node 已分别通过有限资格，但正式 debug 桌面只有链接器签名，无完整资源封套；PREVIS-006 只证明拒绝和进程回收，没有实际初始化 Game／GPU。不能用编辑器画面或重新签原包改写这些记录。

1. 新工具只生成项目内、唯一实例的内部 Development 副本，显式指定来源。来源在写入前做规范路径／上下文检查，复制相对链接保持，来源清单前后相同；不自动运行构建、安装、下载或发布。
2. 沿用 ADR-166 的验收实例布局。复制 Game 保留 Sandbox／网络 client／server／get-task-allow 四项，只增加 user、cache、tmp/previs、logs/previs、tmp/previs/report 五个专用目录的绝对读写资格。报告目录是临时目录的子目录，保留独立证据目的；不授予实例根、恢复／工程／导航数据、完整项目或系统目录。不增加 inherit、不关闭沙盒、不重签嵌套库。
3. 复制桌面使用独立的内部验收 Bundle ID，防止 LaunchServices 将原应用与验收副本混同；副本最低系统设为真实组合的 macOS 14.0。Game 主程序签名及桌面外层资源封套均严格验证，不使用 --deep 签名或把 ad-hoc 验证叫商业签名。全局产品 ID／12.0 基础桌面配置不改变。
4. 独立 Game 的 Foundation 用户目录显式归自己 user/platform-user，复用既有文件资格验收的进程局部机制；动态加载覆盖在 Game 子进程清理。编辑器原行为保持；不用父应用环境默默承担这项产品启动要求，不修改全局 HOME。该机制须经真实独立 Game 验证，不只以环境变量断言宣布通过。
5. 接入原 Rust 桥、原官方信令、唯一观看者与既有视频视窗；NoSound／离屏渲染保持，时间与输出仍归 Rust。故障保持工程／播放，不连接灯具。启动／GPU画面／关闭回收分别记录。

本方案只为当前 Mac ARM64 内部资格，不是客户任意目录方案。客户签名团队、App Group／继承或其他正式权限、Shipping 的调试资格移除、公证及开源许可单列后续决定；不提前假定用户拥有签名证书或 App Store 发行要求。Xcode 系统临时例外未获答复，新的编译／烘焙仍暂停，已有包的局部组装不需要该例外。

## 实际边界验证

`data/PREVIS-007/previs-desktop-ggmASF/assembly-record.json` 保存组装时的来源、资格、组合最低系统与签名，`data/PREVIS-007/artifact-checks.json` 保存两轮正式桌面之后的复核。原 Game 32 文件、Node 2,273 文件、桌面来源四文件不变；副本 Game 只变主程序签名／资源封套／官方模板。原 debug 桌面严格封套拒绝仍是事实，复制 Game、原 Game及内部桌面严格验证通过；没有重新签原包改写结果。

独立 Game 的实际 Foundation／UE 用户与缓存／日志／临时目录归本实例，未依赖父应用的 CFFIXED_USER_HOME 或加载覆盖。UE 初始化后自行将 cwd 改到复制包的 cooked 资源目录，这是只读资源基址，不是新的写入资格；原 Node cwd 与临时目录保持实例布局。没有 --NullRHI、.uproject、编辑器或另一个视窗，原官方信令入口逐字保留。

在独立验收工程中，经正式界面保存一个 8×6×5 米空间及墙体／地板，实际 GPU 显示、透视／俯视和跨场景／空间页的同一连接通过；未知灯具未摆放，不能由此宣称真实光学、音乐共享时间或性能验收。明确停止后回收 Game／Node／端口；正常退出再从最近工程重开字节相同，第二轮直接退出应用也自动回收。未连接设备、播放音乐或操作真实输出，客户可移动权限与商业门槛保持开放。
