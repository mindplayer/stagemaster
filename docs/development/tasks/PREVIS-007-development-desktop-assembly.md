# PREVIS-007：受限 Development 桌面组装与真实连接

状态：done，2026-10-05，有限内部组装与两轮原生 GPU／保存重开及完整软件出口通过，自审通过。基线 main `20d2476c4318527fe1e532961f221ee9f30019e7`，限定计划／ADR 先提交 `0fc97db779b0564e1de9f7f35ab59415d4a82bce`，实现结果 `cfda9371fde8556ea6e8aa9bbc8145442051a756`；主工作区单写者，用户 output/ 保持。属于 H5 的独立预演准备，不是客户发行或 H5 完整出口。

## 一个可用增量

复用 PREVIS-004 原独立 Game、PREVIS-005 包内 Node／官方信令及 PREVIS-006 正式桌面，生成一个全新的项目内内部验收包。按 [ADR-167](../decisions/PRODUCT-ADR-167-development-desktop-assembly.md)，不安装到原应用、不改原组件、不执行 Xcode／UAT，不用编辑器或假视频替代独立 Game。

- 工具显式接受三个项目内来源，先检查平台／路径／必要上下文，再创建唯一实例。复制保留相对链接，验证来源保持与资源等价；只给复制 Game 重组原四项资格加五个准确实例目录，嵌套库不重签。给复制桌面生成完整资源封套、独立验收身份与实际组合最低系统 14.0；不改变产品全局发行配置。
- Game 进程的平台用户目录归自己 user/platform-user，清理继承的动态加载覆盖；编辑器分支保持。只改子进程环境，不改 HOME／系统偏好、格式、时钟、控制权、依赖或信令认证。
- 工具不自动打开应用、设备或输出。保存来源清单、静态依赖、签名、限定资格及失败记录；正式桌面用所属验收实例实际开启、关闭、重开唯一三维连接，检查真实 GPU 画面与工程保持。

## 验收与失败边界

路径逃逸／链接、重复目标、扩大权限、签名或依赖失败须明确拒绝，不能转编辑器、补系统 Node、关闭沙盒或将退出 0 当画面通过。先执行工具保护及相关 Rust；宿主改动做 fmt／全目标严格 Clippy／当前全量／正式构建。工具执行 Node 全套、语法／格式、文档引用／JSON／差异检查。原生实际结果单独记录：真实进程路径、签名、GPU／视频／查看动作、禁用后所属进程和端口退出、工程恢复点与来源哈希。

若 GPU 或动态加载未通过，保留失败并定位，仅暂停需要 UE 重编译／新权限／真实设备的部分；不虚报客户可用。客户通用目录、签名团队／公证、许可、性能／专业光学、听音、厂家资料、物理差分／长期及第三方代表任务仍未完成。

成熟机制沿用本机 codesign／PlistBuddy、现有严格 Mach-O 检查、原官方信令与 PREVIS-004 已实际验证的限定目录资格。新增工具按路径规划、文件验证与组装分文件；入口不承载第二播放器或 UI。

## 实际实现与证据

2026-10-05：限定实施四个 Rust 文件及五个工具文件，未改 UI／UE／核心／执行宿主／锁文件、时钟或协议。入口只组装，规划／来源验证／测试分别组织，最长新增文件 244 行。

- 子进程环境由正式 Rust 启动器明确设置 `CFFIXED_USER_HOME`，不继承编辑器 SDK 快捷变量或动态加载覆盖；`platform-user` 链接与其余目录一起在任何 mkdir 前检查。编辑器环境与 UE UserDir／DDC 机制保持，不修改父进程 HOME。
- 组装工具显式检查 ARM64／系统依赖／最低系统、原上下文／锁／许可、路径、复制等价及签名；只生成新实例，不构建／打开／安装。Game 保留原四项与准确五目录，原官方信令无观测薄层。原桌面 debug 的封套失败原样保存，不重签它来隐藏事实。
- 最终实例 `previs-desktop-ggmASF`，包为 `data/PREVIS-007/previs-desktop-ggmASF/舞台大师 内部验收.app`。组装记录与原生后复核分别为 `assembly-record.json`、`data/PREVIS-007/artifact-checks.json`；原 Game 32 文件、Node 2,273 文件、桌面来源四文件不变。原／复制 Game／完整内部桌面三份严格签名通过，副本 Game 只变主程序签名、封套和官方模板，嵌套库／Pak／sidecar／图标保持。

实际组装命令（来源显式，不自动启动，生成新的唯一实例）：

~~~sh
TMPDIR="$PWD/tmp" NODE_DISABLE_COMPILE_CACHE=1 node tools/previs/assemble-development-desktop.mjs \
  'tmp/framework-001-light-target/debug/bundle/macos/舞台大师.app' \
  'data/PREVIS-004/previs-package-dGdEOa/Mac/StageMasterPreview.app/Contents/MacOS/StageMasterPreview' \
  'data/PREVIS-005/previs-signalling-sR8DnA/previs'
~~~

正式桌面两轮实际验收，仅操作本轮独立工程：

1. 真实 Node／Game 位于组装包，NoSound／离屏 GPU，无 .uproject／编辑器／NullRHI；实际平台用户／缓存／临时／日志、Node cwd 属于实例。UE 初始化后自行切到复制的 cooked 资源 cwd，不能误报为运行目录失效或扩大权限。原生父应用未预设 Game 的 CFFIXED_USER_HOME 来掩盖启动器责任。
2. 未摆放未知灯具的初始工程没有几何，黑画面不是灯光／渲染缺陷证明。通过正式界面创建并保存 8×6×5 米空间和一项墙体／地板，实际 GPU 画面、透视／俯视、场景／空间页同一公共视窗与连接保持。没有模型／真实灯效／运行共享时钟或性能通过声明。
3. 明确停止预演后 Game／Node 退出，两实际端口拒绝连接；正常退出，再从同实例最近工程重开，保存字节一致，真实 GPU 再次显示空间。第二轮运行中退出桌面，同样自动回收进程和两端口。受保护工程／默认最近目录／历史 PREVIS-004～006 证据哈希保持，用户 output/ 未操作；没有音乐、执行后台、设备或 DMX 输出。

AX／JPEG 与进程／目录／关闭记录为 `data/PREVIS-007/native-{first,reopen}-*.json`、`native-{perspective,top,cross-page,reopen}.ax.txt/.jpg`，运行日志在 `logs/PREVIS-007/previs-desktop-ggmASF/`。没有公开记录信令角色凭据。

## 当前检查与失败记录

- 最终工具 **135 Node（119 原保护＋16 新组装）** 通过；原工具首轮两项 ENOTDIR 错误提示失败已修正，旧断言不改，`logs/previs-007-tools-first.log` 保留。五个工具的语法／Prettier 通过。
- 最终相关 Rust **36 项**包含在桌面 **142 通过、2 个既有子进程入口由父测试实际调用**中；全目标工作区严格 Clippy、fmt 与正式 `.app` 构建通过。日志 `logs/previs-007-desktop-tests-final.log`、`previs-007-clippy-cached.log`、`previs-007-fmt-final.log`、`previs-007-desktop-final.log`。没有 UI／UE 源码修改，不累加历史 UI／编辑器测试作本次通过。
- 完整工作区同一原命令 `cargo test --workspace --locked --offline`已完成，工具会话 `29845`实际返回退出 0，Cargo PID `18610`退出；完整汇总 **1300 Rust＋2文档，3个既有子进程入口由父测试调用**。记录 `logs/previs-007-workspace-delivery.log`、`data/PREVIS-007/workspace-result.json`与最终 `verification.json`；旧 pending检查点保留，不用局部计数冒充全量。前两轮在测试入口前等待，采样 `_dyld_start` 和同 PID 的系统加载记录只作诊断，根因未证实。不同路径的字节／签名等价副本能启动；同路径缓存字节等价更新仍未解决。最终成功不等于历史等待根因修复，原测试、签名与断言未改，没有系统保护关闭或越权修复。
- 全新项目内目标的原命令曾继续编译并跑到桌面用例，但汇总前取消，不能计作全量通过；取消后发现的本轮孤立测试宿主按确切路径／PID 回收，非用户后台。实际记录在 `data/PREVIS-007/{interrupted-workspace,startup-diagnostic,raw-startup-diagnostic,test-cache-refresh,cache-refresh-not-resolved,clean-build-interrupted,orphan-test-host-cleanup}.json`，原失败／取消日志保持。
- 原生核对首次误把 UE 的实际 cooked 资源 cwd 当成启动 cwd；最终脚本按官方实际行为核对，未拓宽写入目录。交付检查脚本误读工程的构件字段已按现行 `stage.constructions`／`lighting.fixtures` 改正并加零摆放断言，首轮日志保持。格式工具错误路径已改用原项目内 npm 缓存的已安装工具，无版本／依赖升级；引用／严格 JSON／差异通过。以上不是产品缺陷修复。原日志仍有 `MaxKeyFrameInterval=-1`／VT -12900 警告，画面通过不等于该警告根因已解决。

## 自审范围与下一入口

已完成限定组装／权限、原生连接与完整软件出口自审，最终记录 `data/PREVIS-007/verification.json`固定实现版本 `cfda937`，计划后继不影响来源归属。此有限增量结项；原生静音音乐循环／后台只读GPU组合接 [PREVIS-008](PREVIS-008-native-audio-loop-observation.md)，已完成的局部与原生不重复。之后接 H5 客户可移动运行目录、签名／Shipping／最低系统与许可，先限定方案再实施，不能拿本机绝对路径 ad-hoc 副本当客户发行。新的 Xcode／UAT 仍等系统临时例外答复；厂家参数、听音、专业光学、物理差分／完整最坏组合／8 小时及未参与开发者三任务保持未完成。完整 goal active，不扩 H6、不恢复旧工作器。
