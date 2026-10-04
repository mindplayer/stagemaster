# FIXTURE-009：两轴速度控制的建档与编排

状态：限定功能、自审与相关验收完成；最终全工作区仍有既有后台管理锁失败，不能称全量出口通过。H3／FIXTURE-006 的一个内聚软件增量，非完整实灯适配。基线 main `ce0447d6cf1f0fd190aca191492c6f2087597f3d`，计划／ADR 先提交 `fe04e5b`，结果为本次 `feat(fixtures): author independent pan tilt speed channels` 提交；主工作区单写者，只有用户未跟踪 `output/`，不创建其他会话、子代理或工作器。

依据 [ADR-158](../decisions/PRODUCT-ADR-158-pan-tilt-speed-control.md)及[用户通道证据](../../fixtures/user-supplied/README.md)。先提交本计划和 ADR，再实现。

## 范围与失败行为

Rust 独立两轴速度建档规则、原 Profile／投影与相关测试；UI 独立通道草稿／组件及灯库接线，复用线性字段、场景／预设／文件／历史。`pan-tilt-speed` 是共同两轴速度的 normalized／LTP 原始控制位置，须具备两轴，不推断物理速度或改变渐变时钟。草稿默认留空；非法范围、重复属性／通道、功能表、缺轴以及不相容替换由原事务整批拒绝。错误定位、取消保持原工程，零值与释放分开。

不实现未知图案子区间、频闪物理量、自动程序、持时复位、18CH 多单元、实际输出或 UE 机械动力学；不发布测试档案为实灯。持久结构、执行包、控制权和第三方依赖保持。

## 验收与证据计划

1. 先用现有真实 authoring API 复现两轴速度被拒绝；新增 Rust 矩阵覆盖非零起址、8／16 位端点和任意粗细、全占用默认、场景／预设／释放／渐变、总控隔离、导入导出、两个灯具换模式和原子错误。
2. UI 草稿往返、默认必填／重复定位、基础组合／几何变更保持、移除依赖与取消；实际组件检查精确输入及正式 ProfileWorkspace 接线。已有选择／搜索／复制／取消／保存流程复用，不另造入口。
3. 正式 `.app` 使用项目内独立副本，通过真实 Rust 宿主建档、配适两个灯、场景精确参数、撤销重做、模式文件交换和工程保存重开；不连接设备、不播放音乐、不开启 UE。原来源、默认最近目录和用户 output 保持。
4. 相关 Rust／UI、类型、格式／严格检查、模式格式检查与正式构建实际执行；依据变更影响决定全工作区回归，不写未执行的通过。源码、证据 `data/FIXTURE-009/`、日志 `logs/fixture-009-*`、缓存和临时文件均在本项目。

完成后自审、更新本单与 STATE 顶部、逐项提交并核对工作区。完整 H3 及 H1 听音、H4 物理／长期、H5 客户 UE 包仍为未关闭出口；goal active。优先处理本轮后台管理锁回归失败，再接续完整灯型其他独立语义，不自行编造未知参数。

## 实现与自审

Rust 私有 `fixture_axis_speed.rs` 维护属性／中文／全范围连续值规则；原建档组合增加完整两轴依赖，投影复用原通道映射与 LTP。不扩展播放器、动态效果或 UE；总控、渐变、稀疏预设、有界执行包及换模式走既有路径。UI 草稿工具与组件分别独立，添加默认留空，取消／错误定位复用 ProfileWorkspace；场景“控制”和“仅两轴速度控制”预设不混入位置／强度。移除两轴有说明并移除依赖草稿，基础组合和几何变更保持映射。

核对事务原子拒绝、8／16 位任意粗细、非零起址、空余／复位通道、默认与清除／释放、已用模式保护及文件往返。入口 ProfileWorkspace 324 行只增加组件组装，fixture-tools 306 行只增加既有建档校验接线；相关新增职责已独立，不继续堆入入口，无新增超过 500 行文件。公共结构、能力版本、依赖、时钟与控制权保持；语义选择先记录 ADR-158。

## 实际检查

所有缓存／临时命令均使用项目 tmp/，日志留 `logs/fixture-009-*`，证据 `data/FIXTURE-009/verification.json`。

| 实际检查 | 结果／日志 |
| --- | --- |
| 原 authoring API＋新增两轴速度回归 | 原代码拒绝不支持属性，退出 101；`authoring-red.log`，没有改验收适应旧实现 |
| 新 Rust 两文件专项 | 7 通过；`core-second.log` |
| `cargo test -p stagemaster-project -p stagemaster-previs --locked --offline` | 309 通过，0 失败／ignored；`related.log`，包含新 7 项，不重复计数 |
| 原 `cargo test --workspace --locked --offline` | **退出 101**；桌面 120 通过／1 失败／1 子进程入口 ignored，工作区中断，不能写完整通过；`workspace.log` |
| 失败旧测试单独诊断 | 1 通过，121 filtered；`management-lock-diagnostic.log`，仅诊断，不覆盖全量失败 |
| `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` | 通过；`clippy-first.log` |
| 原 UI 全量／类型 | 418 通过／0 失败／0 skip，类型通过；`ui-test.log`／`ui-check.log` |
| Rust／相关 UI 格式、模式格式、差异与文档引用 | 通过；`fmt.log`／`ui-format.log`／`format-contract.log`／`delivery-check.log` |
| `node tools/desktop/run.mjs build` | 正式 `.app` 通过，原大包提示保留；`desktop-build.log` |

失败现场是旧 `execution::manager::media_tests::missing_configuration_and_damaged_music_do_not_publish_background_records` 在 `media_tests.rs:183` 顺序预留编辑音频时返回“另一个应用正在管理此后台”，独立目录 `tmp/.tmp6byohb/execution`。`files::lock` 当前仅返回 File 并将所有锁错误合并为忙，复制／继承句柄是待受控复现的候选机制，不是本次现场已证明根因。未删除锁、等待重试、忽略测试或改音频保护；后继发布 EXEC-015，先复现再修。历史音频预留问题仍未确认。

新测试初次 API 名称、测试 helper 对非法 JSON 的解码、序列默认记录预期以及原生证据误把保存修订元数据当撤销内容的问题均已修正，初次日志保留。撤销逐字段比较全部业务内容，保存重开仍严格比较文件字节；不关闭保护或缩减真实验收。

## 真实组件与正式桌面

真实 ProfileWorkspace 组件经独立 Vite 5186 验收：空默认／冲突定位且不提交，基础 RGB／几何保持，16 位 9／10 通道精确 37.5%，复制／移除／取消和移除两轴依赖，Escape 恢复及搜索无结果仍保持所选上下文；7 项证据 `component-checks.json`，控制台错误为空。组件模拟宿主不作为真实 Rust 提交或文件证据。

正式 `tmp/framework-001-light-target/debug/bundle/macos/舞台大师.app` 使用 `STAGEMASTER_ACCEPTANCE_INSTANCE=fixture-009-kLRnxY`：

1. 从 FIXTURE-006 来源复制独立 `axis-speed.project.json`；复制旧模式后明确标注“11CH 软件子集，非实灯档案”，无虚构几何。真实 Rust 建档拒绝空默认，填写 37.5% 后保存，两轴 1／2、3／4，调光 8、速度 9，占用 11。
2. 配适两灯起址 17／28；按搜索整组选中，场景控制精确 62.5% 对应 40959。一次撤销恢复两灯未记录／默认 24576，一次重做恢复记录；20% 草稿取消不改工程。原旧灯三项记录保持。
3. 真正导出 `axis-speed.smfixture.json`，文件选择器导入、检查、改名后保存独立模式；身份／修订与原模式不同、映射／默认／LTP 相同。两灯整组换到导入模式，场景／预设／起址／灯具身份保持；一次撤销恢复原模式，一次重做恢复导入模式。`native-pre-swap`／`native-swapped`／`native-swap-undone.project.json` 实际文件核对通过。
4. 保存后退出、同实例重启，经隔离最近工程打开：两灯仍为导入模式、控制已记录 62.5%，设备未连、音乐 0、预演未载入、三维未启用。重开前后 SHA-256 `6d060768b2b6ebfbfc6312358f7a2a12f6fd7f31100278aadd1dbff54a548142` 相同；截图 `native-reopened-speed.png`、完整 AX `native-final-ax.txt`、文件断言 `logs/fixture-009-native-files.log`。

来源 SHA-256 保持 `8ffe1bf55286b2ba2a5401a7233ba6343cc8be488dcb93fa30441853f7be4ea4`，默认用户最近目录保持 `69d620946c1bc483de120ab9570ede44ca50e663e4e93a3fd09071f5405dc637`。未启动执行后台，无 execution/current；原生实例及自有组件页／Vite 收尾退出，用户 output/ 不动。数据／截图／原始日志保持项目内忽略，不加入主仓库。

## 结果与接续

限定软件功能可提交；完整工作区门槛仍失败，下一项先补 EXEC-015 后台管理锁生命周期确定性回归，恢复出口。保存提交后实际版本到 verification.json 并核对工作区。FIXTURE-006 完整实灯、听音、GPU 首帧、物理差分／实灯／长期和客户无 UE 编辑器包均未关闭，goal active。
