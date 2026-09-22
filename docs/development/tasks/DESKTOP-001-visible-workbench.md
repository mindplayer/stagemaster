# DESKTOP-001：可试用桌面工作台与工程入口

- 状态：done；负责人：当前 Astra 会话
- 基线／分支：603f40f／main；工作区：/Users/sunqi/projects/stagemaster；开始时干净
- 依据：[PRODUCT-ADR-006](../decisions/PRODUCT-ADR-006-visible-desktop-increments.md)

## 范围

按用户最新要求交付真实工程编辑：空白新建、打开、灯具配适、场景属性、命名、撤销重做、保存／另存为及重开。删除本轮尚未提交的演示入口和只读样例页；原时间线交互原型及测试保留但不混入正式入口。

写入范围：apps/desktop/**、apps/ui-prototype/src/**、其 index.html、package 清单／锁文件、vite 配置、tsconfig、AGENTS、必要测试；tools/desktop/**、crates/stagemaster-project/**、crates/stagemaster-project-store/**；Cargo.toml／Cargo.lock；本工单、ADR-006、execution-plan、STATE、README、implementation-status、桌面使用说明、桌面迭代计划、PROJECT-001 状态、工程格式说明／Schema、格式测试和 design-qa。缓存／日志／打包结果仅在项目内，加入必要忽略。

工程编辑和保存限已实现灯光子集；不实现调度、云端或设备操作。产品工程格式不变，未知能力拒绝而不丢弃。Tauri、API 和 Rust 依赖锁定后记录实际版本。

## 验收

- 严格 JSON、结构校验、身份和引用、配适越界／重叠、属性和预设类型；不支持能力明确拒绝。
- 编辑原子性、撤销重做、修订、保存重开、外部文件变更拒绝、保存失败保留旧文件；核心旧保护测试保持。
- 类型检查、原有交互测试、生产构建和 Sites 兼容检查；工作区 fmt、离线测试、严格 Clippy。
- 本机 .app 无开发服务器启动；原生文件选择、实际灯具／场景编辑、保存重开和未保存取消实查。
- 没有样例展示页和假功能按钮；本轮不声称 Windows／移动端／真实设备或崩溃恢复已验收。

## 完成记录

结果为本次 `feat(desktop): deliver real lighting project editing and persistence` 提交；基线 603f40f，main 原地执行，开工前干净。用户中途明确按真实交付标准开发，最终实现以 ADR-006 修订范围为准；首版只读样例入口未保留。

- 分离 stagemaster-project（严格 JSON、嵌入 Schema、已支持领域校验、原子编辑、视图投影）与 stagemaster-project-store（限量读文件、冲突检测、同目录同步替换），薄 Tauri 宿主管理串行会话和原生窗口。工程模块不依赖 Tauri，普通 React 组件不直接调用平台 API。
- 正式界面为空白新建／打开、灯具配适、场景属性、工程信息、撤销重做及保存。无预置节目、假设备状态、未接通的执行按钮或教程展板。前端参数、配适表、检查器与确认对话框分组件，旧时间线原型及保护验收保留待接入。
- 工程版本仍为未发布 0.1.0-draft.1；修正 entryPoints 最小数量为 0，使空白工程可保存。首存没有虚构父修订；此后保存创建新修订并保留引用。未支持模块／未来字段拒绝，不剥离后重存。
- 版本：Tauri 2.11.6、CLI 2.11.5、JS API 2.11.1、dialog 2.7.3；serde 1.0.229、serde_json 1.0.151、jsonschema 0.57.0（禁默认网络特性）、uuid 1.26.1、tempfile 3.27.0；Cargo/npm 锁文件纳入版本控制。
- 验证：fmt check、离线工作区测试 43 项（旧 24＋工程 10＋存储 6＋会话 3）、严格 Clippy；前端类型、原 15 项交互回归、50 项格式检查、4 项 Sites 测试及实际 .app 构建通过。新增保护包括整数 4.0／1e0 的 Schema 语义、配适边界、预设引用保留、陈旧会话、磁盘竞争及保存失败保全。
- 原生窗口实查：新建工程、添加 RGB 灯具、场景亮度／蓝色均设置为 32768、未失焦改名后快捷键保存、进程重启后读取一致；取消打开、地址冲突与取消编辑、撤销回到已保存状态、重做、未保存退出取消均通过。无父窗口的消息提示未正常显示，修正为父窗口 sheet 后复测通过。实际窗口截图已在验收会话中检查，记录见 design-qa；未声称留存截图文件。
- 打包产物 `target/debug/bundle/macos/舞台大师.app`。验证使用 tauri://localhost 嵌入资源，无 1420 开发服务；本轮 4174 预览服务及临时页已关闭。运行日志在 logs/DESKTOP-001，验收工程在 data/development/DESKTOP-001，不纳入 Git。

限制：当前只支持灯光编辑子集；场景列表／时间线、编译播放、真实设备、其他端、签名公证／更新、崩溃恢复尚未完成。非协作外部编辑器仍存在最后比较到替换的竞态窗口，目录同步失败有明确提交后提示，不宣称断电绝对安全。未刷机、打开串口、输出或部署。下一项按[桌面迭代计划](../desktop-iteration-plan.md)推进，PROJECT-001 父任务不结项。
