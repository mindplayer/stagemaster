# PREVIS-004：独立预演组件的可复现打包

2026-10-06后继事实：[PREVIS-013](PREVIS-013-owned-crash-reporter-cleanup.md)发现第三轮失败烘焙 `previs-package-Kd2QO0` 的独立 CrashReportClientEditor PID90300仍持续占约100% CPU；原UAT／编辑器退出不证明报告助手退出。本次核对实际命令、加载文件、原烘焙日志与准确报告目录后已SIGTERM收尾，报告和原日志保留；不会将原failed改成通过。下文的原进程退出记录应按当时捕获对象阅读，不能扩大成所有派生报告助手已结束。

状态：**进行中，Development 副本的限定目录 JSON／HTML验收通过，客户权限／可复现完整发行仍未通过**。首轮基线 main `0885847e3de629ac0ead1eb498b4e84cc30082b7`，计划先提交 `b937b1a`，基础工具结果 `0cf4668`；静态依赖及最新文件资格接续见下方。主工作区单写者；用户未跟踪 `output/` 不触碰。属于 H5 客户无 UE 编辑器路径的准备，不关闭 H5 或整个持续 goal。原四份默认 pipeline 失败记录保持，不把最新副本结果回写为原包当时已通过。

## 本增量范围

现有 `StageMasterPreview` Game Target 与桌面资源启动分支已存在，但无可复现 Build／Cook／Stage／Archive 流程。先只生成 **Mac ARM64 Development 独立渲染组件**，检查实际包内资源与非编辑器程序启动。复用原 UE 适配器、官方 UAT 与本机已安装引擎；不重写渲染、信令、播放器或时钟。

- 源码／工具限 `apps/previs-unreal/`、`tools/previs/`，以及相关文档；无 Rust／UI／持久格式、控制权或公开协议变化。原私有打包／启动修正不改公共语义；实施中发现的文件权限边界另由下方 ADR-165 记录，具体权限尚未实施。
- 构建缓存／暂存在项目 `tmp/` 与既有 UE 工程缓存，日志在 `logs/`，独立包与证据在 `data/`。每次生成独立目录，不覆盖已有合格包；失败保留日志且不登记为成功。
- 使用 Development 便于实际运行 UE 自动化；它不是 Shipping／签名／公证／公开发行承诺。SDK 检查保留，不用运行时跳过 SDK 的措施掩盖构建条件。
- 添加一个只读的必需视觉资源自动化检查，覆盖当前实际渲染所需资源，兼容编辑器与 Game 测试；原官方库广泛加载检查仍保留，不为了让小包通过而改弱旧验收。
- 不将 Epic 原始素材加入 Git，不执行 DMX、不连接真实设备，不安装或修改全局引擎配置。

## 验收与失败行为

1. 打包命令、平台／架构／输出路径与错误处理具可执行脚本测试；非支持平台、缺失引擎／工程、子进程失败明确拒绝，不留下假成功记录。
2. 实际执行官方 UAT，输出独立 Game `.app` 及烘焙内容，保存完整命令、版本、构建结果和产物清单。
3. 运行产物中的 Game 程序而非 UnrealEditor；在项目内 UserDir／日志、无图形模式执行必需资源检查。必须核对自动化报告中的实际成功项，不只看退出码；空报告、失败报告、缺资源不能判成功。
4. 实际运行现有 UE 全部相关编辑器自动化，原信令测试及差异／文档／JSON 检查。只读资源／打包变化不重复不相关 Rust／UI 全量；构建失败先定位，不更改原核心保护。
5. 没有 GPU／真实内嵌画面就明确未验收；无 UE 编辑器的桌面整包还需要 Node 与信令运行依赖、自包含动态库、Tauri 组装及正式桌面操作，作为后续内聚增量。

## 成熟机制依据

采用 Epic 的 [BuildCookRun 分阶段流程](https://dev.epicgames.com/documentation/unreal-engine/build-operations-cooking-packaging-deploying-and-running-projects-in-unreal-engine?lang=en-US) 与 [独立打包说明](https://dev.epicgames.com/documentation/unreal-engine/packaging-your-project?lang=en-US)：预先烘焙、暂存、归档，不要求客户使用 cook server。参数同时核对本机 UE 5.8 的 `ProjectParams.cs`、`CommandEnvironment.cs` 与 `RunUAT.sh`；日志／UserDir／文件缓存全部指定项目内路径。第三方许可证与客户交付清单后续一并核对，不因本地构建成功宣称具有最终商业分发条件。

## 交付记录

实际完成官方分阶段工具、项目内可控输出／缓存、Mac ARM64 资格、独立 Game 实际构建与烘焙、所属进程组取消／失败证据、严格报告检查，以及 H264 早初始化断言修正。新增 6 个 JS 工具／测试文件按计划／执行／工具发现分工，最大 260 行以内，C++ 只读用例 31 行；入口 180 行以内，不修改 Rust／UI／锁文件、工程格式或真实设备。

最终 **34 Node（28 打包＋6 原信令）**、**12 UE 编辑器用例**全部 Success；Node 格式／语法、相关文档引用／严格 JSON／差异检查通过。编辑器最终复核在独立 `tmp/PREVIS-004/editor-user-final` 下顺序执行，与 Game 报告写入不并行；第一次编辑器检查与 Game 曾重叠，不用它替代最终复核。无需不相关 Rust／UI／固件全量，不冒记 GPU 或正式桌面操作。

第三／第四轮 UAT 均实际 Build／Cook／Stage／Package／Archive 成功（约 153／51 秒），Game `.app` 约 598 MiB。第三轮 Game 断言退出；第四轮正常 Engine Initialized／实际用例 Success，但无 JSON 报告，工具明确失败。预建输出目录的独立复核仍被沙盒拒绝，尚不登记 Game 自动化通过。签名与系统拒绝证据支持 [ADR-165](../decisions/PRODUCT-ADR-165-packaged-renderer-file-access.md)，未关闭／重签／绕过沙盒，也没有改全局引擎、部署或输出。

只读核对实际 Game 的 `LC_BUILD_VERSION` 为 macOS 14.0／SDK 26.1，桌面配置最低版本仍为 12.0；该差异及包内动态库解析需在整包增量明确，当前不能承诺带三维的客户包支持 macOS 12。未修改桌面兼容性承诺或降低引擎要求。

证据 `data/PREVIS-004/verification.json`、各实例 `build-record.json`；日志 `logs/previs-004-*`、`logs/PREVIS-004/`，最终 UE 报告 `tmp/PREVIS-004/editor-tests-final/index.json`。前期工具导入／取消用例失败、Zen／Metal／启动断言与报告失败原始日志保留；格式命令路径错误和 shell log 名称冲突以工具返回摘要保留在交付证据中，不冒充原始日志，不以重跑覆盖历史。原三份受保护工程／默认最近目录哈希保持，用户 output/ 保持，所有所属构建／Game／编辑器进程退出。

下一入口仍为本工单：按 ADR-165 收敛独立 Game 的最小文件权限与报告模板，完成实际 JSON 验收后再接 Node／信令／Tauri 整包。用户尚未答复 Xcode OS 临时文件例外；新 CLI 无明确授权时先拒绝。只暂停受影响原生构建，独立软件任务可继续；不自行推进 H6 或把 H5 记为完成。

## 接续：静态链接依赖资格（2026-10-05）

基线 main `0cf466887bfde3cac2cb9a43f8f834ea0a1f2414`，主工作区单写者；计划先纳入版本。上一轮实际提交构建工具和启动修正，独立 JSON 报告失败仍未解决。本次仅补齐 `tools/previs/` 的只读 ARM64 Mach-O 静态依赖检查，并接在 UAT 后、独立程序启动前；不运行新的 Xcode／UAT／Game，不重签原包，不改沙盒或 Rust／UI／工程／时间语义。

- 解析实际 `lipo`／`otool` 结果；从独立 Game 递归追踪动态库，保留路径顺序与加载者／主程序路径。操作系统库单列，非系统依赖必须能在本包内解析；不使用开发机 UE 目录补齐缺项，不跟随越出包的符号链接。缺项、架构／平台不匹配、非法元数据或图预算超限均明确失败，不留下合格标记。
- 记录各镜像最低 macOS 版本及整体最高要求，不静默修改桌面最低版本。相关测试覆盖解析、间接依赖、循环、弱依赖、路径空格／移动、目录与符号链接、错误及预算；原打包／信令保护保留。用上一轮原始独立产物实际执行，并保存原始工具输出、解析图及受保护文件哈希，语法／格式／文档／JSON／差异实际检查。
- 这是静态链接闭包资格，不证明运行期 `dlopen`、GPU、沙盒写入、客户整包或未安装 UE 的实际运行；此前独立报告失败仍是未解决事实，不以此检查代替正式出口。

成熟机制参考：Apple [可移动程序的运行路径依赖](https://developer.apple.com/library/archive/documentation/DeveloperTools/Conceptual/DynamicLibraries/100-Articles/RunpathDependentLibraries.html)及本机 `otool`／`lipo` 的真实输出。当前原包的开发机 fallback 路径可见，必须区分“带有 fallback 路径”与“实际缺库而依赖 fallback”，不能仅凭路径出现或本机启动成功判客户包自包含。

### 静态资格交付

计划先提交 `e0ffd9a`；结果为本次 `feat(previs): verify standalone binary dependencies` 提交。本次 5 个新源码／测试文件按 Mach-O 解析、包内解析图、只读 CLI 及测试职责拆分，均不超过 240 行；原入口仅接入检查，生成完整原始工具输出和边清单。UAT 后、Game 前执行资格；日志／错误仍沿原失败路径，不放宽 JSON 门槛，子进程环境去掉 DYLD 覆盖且不改 HOME／SDK 检查。

实际 **62 Node（28 新依赖＋34 原打包／信令）**、11 文件语法／Prettier、相关引用／4 配置严格 JSON及交付证据 JSON／差异检查通过。自审发现主程序自引用可被循环去重放过，新增保护先失败后修复；红／绿日志保留。初始研究路径与一次格式后补丁上下文错误是工具返回摘要，不伪装成原始日志或原生构建失败。没有 UE／Rust／UI／固件源代码改动，不重复这些编译／全量测试，原 12 编辑器用例仍只是上一轮证据。

用原始第四轮 `.app` 实际只读检查 **6 镜像／66 依赖：6 条包内、60 条系统**，全部包含 ARM64，最高最低 macOS 为 14.0。完整应用的中文空格移位副本得到相同镜像和依赖数；仅在所属副本移走 `libtbb.12.dylib` 时明确缺库拒绝，归回后恢复。六镜像副本与原始字节哈希相同，原 Game／Pak、三份受保护工程／最近目录及用户 output/ 保持。原包 `codesign --verify --deep --strict` 实际退出 0，未重签、改权限或运行 Game；四份历史 pipeline 仍 failed。

证据 `data/PREVIS-004/dependency-verification.json`，日志 `logs/previs-004-dependencies-*`，移位副本留项目 tmp 下并忽略。自包含图只是忽略开发机 fallback 的包内静态候选解析，不宣称完整 dyld 运行、运行期 `dlopen`、旧系统兼容、GPU、沙盒或客户整包通过。本次没有新的 Xcode／UAT／Game／编辑器运行；OS 临时例外尚未答复，只暂停相关构建。下一步仍按 ADR-165 处理最小文件资格和独立报告，不为当前绿灯关闭 H5 或完整 goal。

## 接续：Development 副本的限定文件资格

状态：**Development 限定文件资格增量完成，自审通过**。基线 main `47e24daa35a70bab0df3b19278fc87768d4dd84f`，计划／ADR 先提交 `946d70c`；结果为本次 `fix(previs): qualify sandboxed development reports` 提交。主工作区单写者，用户 output/ 保持。上一轮 PREVIS-005 已交付信令组件，本轮另取真实原生报告。按 ADR-165 验证不依赖新 Xcode 的现有 Development Game 副本路径；本工单不选定客户权限／签名团队，不更改引擎、桌面、UE 源码、播放器或桥协议。

- 新工具仅接受项目内已生成的唯一 `StageMasterPreview.app` 主程序；先校验原签名、ARM64 包内闭包和来源。新建 `data/PREVIS-004/previs-file-access-*/` 副本，只对副本组装签名，不覆盖或修改原包／Pak／历史失败记录。
- 使用原 Apple App Sandbox／网络／Development 调试资格，增加**本次实例**的 runtime-user、runtime-cache、runtime-temp、logs 和 runtime-report 五个目录的官方绝对路径 read-write 资格（目录尾 `/`）。禁止项目根、整个 tmp／data／logs、源码／工程／导航／密钥／设备资格和越界链接。签名为本地 ad-hoc／无时间戳，保持沙盒，不使用 noEntitlements、inherit 或关闭保护；范围具体化先记录在 ADR。
- 把本机引擎官方 `Report-Template.html` 原样放到副本 `Contents/UE/Engine/Content/Automation/`，记录来源和哈希，保留官方资源许可边界，不加入 Git 或改引擎。模板只为自动化导出，不打开 HTML 或新建外部依赖。
- 实际用副本 Game 而非编辑器执行原必需资源／H264 用例（NullRHI／NoSound），要求新的非空 JSON 和 HTML，原 successfulReport 门槛保持；独立实例缓存／UserDir／TMPDIR／日志，无新 Xcode／UAT／编译／烘焙。实测外部 sibling 报告目录拒绝写入并保持哨兵，再回到允许目录确认恢复，不拿退出 0 或 Success 文本当验收。
- 签名前后核对资格、签名和非修改内容；坏输入、缺模板／来源、异常签名／权限、范围扩大、报告缺／空／失败均拒绝且记录 failed。完整原生报告未通过前不接 Tauri。开发绝对路径资格不是客户可移位权限，仍开放客户签名／目录／最低系统／实际 GPU／H3／H4 门槛。

验收：实际脚本测试与原 87 项保护；实际源包签名／哈希不变、副本限定签名、真实 Game 正／负／恢复报告和所属进程退出；相关语法／格式、引用、严格 JSON、差异检查；证据和失败都留项目内，更新 STATE 后提交。若不能取得真实 JSON或发现受控产物写入项目外，停止受影响运行，不能以关闭沙盒或放宽文件范围迁就。

### 限定文件资格交付

新增 5 个小文件，计划／报告、系统资格／目录、脚本实施及两份测试分职责，均不超过 242 行；原所属进程执行器仅返回实际 PID／退出码／信号终态，原取消／失败／环境保护不变，不新增进程管理平台或正式播放器。入口 `tools/previs/qualify-renderer-files.mjs` 只接受一个项目内 Game 主程序，先校验原包／架构／模板来源（已验证 UE 5.8），写入前核对父目录真实路径；只签新副本，未改原包、嵌套库、引擎或全局配置。

最终 **119 Node（87 原保护＋32 文件范围／所属 PID）**、7 文件语法／Prettier、相关文档引用／4 配置与证据严格 JSON／差异检查通过。系统日志的松散匹配可能误收其他 PID／程序／目录／单文件或命令回显，自审新增 6 个拒绝用例先失败，补关联实际完成的 Game PID、JSON与 HTML两项拒绝后通过；红／绿原始日志保留。额外目录保护验证首次创建，以及链接父目录在任何新文件写入之前拒绝，不等写出后才发现越界。

最终实例 `previs-file-access-N1THSh`，实际独立 Game 顺序运行三次：

1. `runtime-report/allowed`：原 `StageMaster.Previs.RuntimeAssets` 的资源／H264 用例，真实 JSON **1 Success／0 failed／0 notRun／0 inProcess**，非空 HTML和实际成功导出日志，退出 0。
2. `denied-report`：同一用例仍文字 Success／退出 0，但非授权 sibling 目录的 JSON 哨兵哈希保持、HTML未创建；系统实际 `deny(1) file-write-data`／`file-write-create` 与此次 Game PID一致。该运行**不是报告通过**，明确证明不能只接收退出码或 Success 字样。
3. `runtime-report/restored`：回到既定资格内目录，重新取得同一用例的 Success JSON／非空 HTML，退出 0，无扩大资格或重试写操作。

两份成功报告是同一用例的不同运行，不计为两个独立功能。前两轮实例 `TKtyHP`、`85swsG` 也取得正／负／恢复结果；最终源码增加 PID和目录保护后用新实例实际复核，不拿早期运行代替。日志保留既有 VolumetricFog 配置优先级警告，不关闭保护以消除警告；无 GPU画面或声音验收。

原来源 **32 文件**逐项哈希保持；副本 **33 文件**只增加原样 `Report-Template.html` 并变化主程序签名／CodeResources，未改变 Pak或嵌套库。原／副本 codesign --verify --deep --strict 实际退出 0；实际资格保持原 Sandbox／network client／server／Development get-task-allow，加五个带尾 `/` 的实例目录，无 inherit、项目根／工程／设备资格。源码与签名副本的 6 镜像／66 依赖及 macOS 14.0 要求保持。本地 ad-hoc 签名无时间戳，**不是客户开发者身份、公证或可移位发行权限证明**。

证据 `data/PREVIS-004/file-access-verification.json`、各 `previs-file-access-*/build-record.json`；日志 `logs/previs-004-file-access-*`／`logs/PREVIS-004/`。原 Game／Pak、原两个 PREVIS-004 证据、PREVIS-005 证据和三份受保护工程／默认最近目录保持，用户 output/ 未动，所属进程退出。诊断 verifier 首次未经实测假定 45 文件，实际清单为 32，依据原记录／当前逐项清单更正诊断预期，错误日志保留，不是产品验收保护放宽。没有新 Xcode／UAT／编译／烘焙／编辑器、Rust／UI／UE／固件源文件变化、设备或输出操作。

接续客户可移动目录／签名与宿主组装：新的开发报告证据解除了“所有已有 Game都无法验收”的假设，但不把本机绝对路径副本放进 Tauri，也不自动把四份历史默认 pipeline 改为 component-verified。新的 Xcode 构建仍待 OS 临时例外；组合最低系统、许可、客户无编辑器／实际 GPU、H3 资料／听音、H4 差分／实灯／长期仍开放。完整 goal active。

### 实施中实际发现

- 首轮 ZenStore 隐式启用，断开所属构建进程并保留失败记录；项目明确文件 Pak／非 Zen 烘焙，构建禁用 UBA 网络执行，按实际所属进程组取消。原 RunUAT 的 `ps --ppid` 取消逻辑在 macOS 无效，实际子孙终止测试保护本工具，不改用户全局脚本。
- macOS .NET 忽略 XDG 对 ApplicationData 的重定向，采用进程级 CoreFoundation 用户缓存路径，不改 HOME；实际 Xcode／UBT 缓存进入项目 tmp。隔离发现缓存后 Metal 自动发现失效，显式读取既有工具链并放入所属 PATH，不下载／升级／跳过编译。
- 第三轮真实 Build／Cook／Archive 成功，但 Game 以 `AVCodecsCore` 包未注册断言退出。`nm` 确认其代码已在包内，不是缺二进制模块；LLDB 实际栈为 `LoadConsoleVariablesFromINI → VerifyCVarVideoSettings → StaticEnum<EVideoCodec> → ConstructUPackage`。原因是项目在 UObject 初始化前设置原本已缺省为 H264 的编码 CVar；移除该冗余启动设置，新增编辑器／Game 实际 H264 断言。第四轮完整重建后 Game 正常启动／用例实际成功，独立报告门槛仍失败；不改插件／模块依赖、引擎或公开协议。
- 已观察到 Apple `NSTemporaryDirectory()` 不服从显式 TMPDIR，Xcode scheme 临时脚本仍使用系统 `/var/folders/.../T`；本工具主动创建的临时／缓存／日志／包均在项目内，但尚不能宣称第三方工具零系统临时写入。保留偏差，进一步核对受支持的本地配置，不用修改全局系统临时目录或隐藏证据。
