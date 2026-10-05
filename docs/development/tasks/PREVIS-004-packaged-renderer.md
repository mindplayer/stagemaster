# PREVIS-004：独立预演组件的可复现打包

状态：**进行中，基础工具／启动修正已提交，完整独立验收未通过**。基线 main `0885847e3de629ac0ead1eb498b4e84cc30082b7`，计划先提交 `b937b1a`；本轮结果为 `fix(previs): prepare standalone renderer packaging and safe codec startup` 提交。主工作区单写者；用户未跟踪 `output/` 不触碰。属于 H5 客户无 UE 编辑器路径的准备，不关闭 H5 或整个持续 goal。

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

### 实施中实际发现

- 首轮 ZenStore 隐式启用，断开所属构建进程并保留失败记录；项目明确文件 Pak／非 Zen 烘焙，构建禁用 UBA 网络执行，按实际所属进程组取消。原 RunUAT 的 `ps --ppid` 取消逻辑在 macOS 无效，实际子孙终止测试保护本工具，不改用户全局脚本。
- macOS .NET 忽略 XDG 对 ApplicationData 的重定向，采用进程级 CoreFoundation 用户缓存路径，不改 HOME；实际 Xcode／UBT 缓存进入项目 tmp。隔离发现缓存后 Metal 自动发现失效，显式读取既有工具链并放入所属 PATH，不下载／升级／跳过编译。
- 第三轮真实 Build／Cook／Archive 成功，但 Game 以 `AVCodecsCore` 包未注册断言退出。`nm` 确认其代码已在包内，不是缺二进制模块；LLDB 实际栈为 `LoadConsoleVariablesFromINI → VerifyCVarVideoSettings → StaticEnum<EVideoCodec> → ConstructUPackage`。原因是项目在 UObject 初始化前设置原本已缺省为 H264 的编码 CVar；移除该冗余启动设置，新增编辑器／Game 实际 H264 断言。第四轮完整重建后 Game 正常启动／用例实际成功，独立报告门槛仍失败；不改插件／模块依赖、引擎或公开协议。
- 已观察到 Apple `NSTemporaryDirectory()` 不服从显式 TMPDIR，Xcode scheme 临时脚本仍使用系统 `/var/folders/.../T`；本工具主动创建的临时／缓存／日志／包均在项目内，但尚不能宣称第三方工具零系统临时写入。保留偏差，进一步核对受支持的本地配置，不用修改全局系统临时目录或隐藏证据。
