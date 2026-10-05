# PREVIS-005：自包含 Node／信令组件

状态：**有限软件增量完成，自审通过，客户整包未验收**。基线 main `bc97055ded850f50827acfc904faf4b2da8570af`，计划先提交 `9d97a7d`，结果为本次 `feat(previs): package self-contained signalling runtime` 提交。主工作区单写者；用户未跟踪 output/ 不触碰。属于 H5 已有分发组件的独立准备，不开放 H6 或宣布客户整包完成。

## 范围与依赖

桌面已有 `resource_dir/previs/{node,signalling.mjs,node_modules}` 组件路径，原官方信令、身份／来源隔离、一次观看及迟到连接语义保持；本工单仅在 `tools/previs/` 实现可重复的 **Mac ARM64 Node／信令组件组装和验证**。不写第二套信令服务、视频播放器或节目时钟，不改 Rust／UI／UE、持久格式或公共协议。

- 复用已安装且本机实际使用的 Node 24.17.0，以及现行 package.json／package-lock.json。全新项目内目录执行 npm ci，先离线、禁安装脚本、关闭审计／全局安装；不升级版本、改锁、覆盖原 node_modules 或下载另一运行时。缺缓存单独处理、保留失败，不用复制未校验的工作区依赖冒充锁定安装。
- 组件在 `data/PREVIS-005/<实例>/previs/`，暂存在项目 tmp、日志在 logs，生成产物忽略；每轮独立路径。保留 Node 完整 LICENSE、npm 原发布内容及许可清单，不把许可元数据当法律／商业发行审查通过。已核对 124 个锁定包，其中两份 Epic 包和 cookie-signature 无独立 LICENSE 文件，缺项必须留在清单。
- 复制后的 Node 以实际 Mach-O 元数据确认 ARM64、可执行角色、系统动态库依赖及最低系统要求；复制 byte hash 与版本必须一致。清理所属验证进程的 NODE／DYLD 加载覆盖，缓存路径指定项目内，不改 HOME 或全局配置。
- 使用包内 Node、原样复制的 signalling.mjs 及两份既有测试，在包的依赖上下文重跑原 6 项真实回环／发现测试。原断言不变；实测暴露的测试脚本文件 URL 路径转换修正见交付记录。测试副本只用于所属验证，结束移到 tmp，不进入最终组件；不得改弱原断言、用系统 Node／源目录 node_modules 顶替。移位副本也须通过并保持文件等价。
- 生命周期继续由父进程 stdin／EOF 拥有；补齐退出证明。额外参数、坏锁、缺 Node 许可／运行时、依赖与架构不匹配、报告空／失败、测试进程异常均拒绝，不把失败记录登记为合格。npm 发布包的许可文本缺项保留待审，不混同缺少 Node 必需许可。

PREVIS-004 的独立 Game JSON／最小文件资格／报告模板仍未通过，Xcode OS 临时例外仍待用户答复。本任务在不需要 Xcode 的部分独立推进；**最终 Tauri 整包的消费资格仍依赖 PREVIS-004 实际通过**。不会装进或改签现有正式应用，不运行 UE、音乐或设备，静态 Node 资格与回环连接不代表 GPU／客户环境。

## 验收

1. 计划／安装／许可清单／报告判断与失败行为实际脚本测试，原 62 项打包／依赖／信令保护保留。
2. 实际锁定离线安装、新组件及中文空格移位副本，用各自 Node 执行原 6 项；保存 Node／锁／源码／清单哈希、原始 Mach-O 输出和测试报告，所属子进程／回环端口退出。
3. 确认核心工程、默认最近目录、原 Game／Pak、源 node_modules 与用户 output/ 不变；相关语法／格式、引用／严格 JSON、git diff --check 实际执行。无 Rust／UI／UE 源码变更，不重复不相关编译或冒记原生／硬件验证。
4. 更新本工单与 STATE，提交相关源码／测试／文档，保留失败和许可缺项；不解除独立 Game、组合最低系统、客户正式包、听音／GPU／物理／长期门槛。

成熟机制：采用 [npm ci 锁定安装](https://docs.npmjs.com/cli/v11/commands/npm-ci/)和原官方信令包，不重写通用依赖解析器；保留已安装运行时对应的 [Node 24.17.0 完整 LICENSE](https://raw.githubusercontent.com/nodejs/node/v24.17.0/LICENSE)。实际安装与测试证据决定技术资格，开源／Epic 素材发行资格单列，不以本任务绿灯代替。

## 交付记录

新增 5 个小文件按安装计划／资格、文件／许可、组装、测试和生命周期分工，最大 257 行以内；不把实现堆入旧信令或桌面入口。复用原所属进程组执行器及官方 npm ci，不实现另一套依赖解析。正式工具 `tools/previs/package-signalling.mjs` 不接受额外参数，仅复用已验证 Node 24.17.0／Mac ARM64；记录实际源码哈希、Mach-O 原文、锁定包／许可和文件清单。失败状态保持，只有两个包内 Node 验证及移位清单均通过才登记 `signalling-component-verified`，合格记录仍明确 `customerPackageVerified:false`。

实际最终 **87 Node（62 原打包／依赖／信令保护＋25 新组装／生命周期）**通过；原目录和中文空格移位副本各用自己的 Node 运行 **7 项**，原 6 项认证／发现加 EOF 退出及两个真实端口释放。7 个修改／新增 JS 文件语法和 Prettier、相关文档引用、配置／证据严格 JSON与 git diff --check 通过；无 Rust／UI／UE／固件源码改动，不重复相关编译或冒记原 12 UE 编辑器结果。

最终实例 `data/PREVIS-005/previs-signalling-sR8DnA/previs/` 实际离线安装 **124 包**，**2,273 文件、135,002,876 字节**含 Node／源码／许可，全部移位字节等价。复制 Node 与实际安装二进制同哈希，包内 `--version` 为 v24.17.0；真实 lipo／otool 确认 ARM64 可执行、4 系统库和最低 macOS 13.5。Node 完整 LICENSE 原样保留；所有 npm 发布文件保留，124 项许可清单中以下无独立文本：

- `@epicgames-ps/lib-pixelstreamingcommon-ue5.8@0.1.0`
- `@epicgames-ps/lib-pixelstreamingsignalling-ue5.8@0.2.0`
- `cookie-signature@1.0.7`

它们的现行锁有 MIT 元数据，清单明确 `needsLicenseReview`，不补造许可文本或将元数据当作商业发行许可已审查。许可发行与 Epic 资源权限仍需后续收敛。

三份早期失败记录保持：`previs-signalling-yY07kn` 的 user／global npm 配置重复加载，改为两个独立空配置；`previs-signalling-YWrCRK` 的 ENOTCACHED，单独在项目 tmp 中按原锁无脚本联网补缓存，原锁不改，正式工具／最后验收始终 offline；`previs-signalling-2624zr` 的原目录 7 通过、移位 3 通过／4 取消，确认旧测试直接取 `.pathname` 把中文和空格变成百分号编码路径。两份旧测试仅实质替换成标准 `fileURLToPath`，同时执行机械格式化，原 6 项断言、6 秒期限、认证／发现与脱敏保护不改；生产 signalling.mjs 字节未改。修正后第四轮和最终轮两目录均实际通过，不回写前三轮结果。

证据 `data/PREVIS-005/verification.json`、各 `build-record.json`（含实际安装和测试命令）；日志 `logs/previs-005-*`／`logs/PREVIS-005/`。最终组装前后源 node_modules **2,267 文件**清单保持；这是最终轮快照比较，不冒作前几轮之前没有记录的全树哈希。生产信令／清单／锁与基线字节相同，Game／Pak／PREVIS-004 两份历史证据及受保护工程／默认最近目录哈希保持；用户 output/ 未动，所属进程退出。原四份 Game pipeline 仍 failed，未关沙盒、重签或改变文件权限。

下一步在具备系统临时例外权限后，继续 PREVIS-004 的 Game JSON、最小文件资格和报告模板；通过后才组装／消费此组件到 Tauri 正式包。桌面 12.0、Node 13.5、Game 14.0 的组合最低系统决定未作；许可审查、客户无编辑器／实际 GPU、听音、厂家资料、物理差分／8 小时仍未通过。完整 goal active，不扩大至 H6、不恢复旧工作器。
