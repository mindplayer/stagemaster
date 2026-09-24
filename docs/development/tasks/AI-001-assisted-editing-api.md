# AI-001：AI 辅助编辑伪 API

- 状态：done（方向与伪 API 设计）。
- 负责人：当前 Astra；基线 `ba613d8`／main；工作区 `/Users/sunqi/projects/stagemaster`，开始时干净。
- 需求：用户提出为 AI 辅助完成灯光编辑预留接口。
- 范围：ADR-011、独立 TS 接口／调用与编译期反例、相关文档导航和状态；不改正式产品、工程格式或已存在的 draft-0.3 调用。

## 验收

复用现有工程命令与事务；明确读取、提案、差异／隔离预演、范围授权、应用／撤销、冲突与幂等对账。示例体现真实 DTO 形状；反例拒绝原始输出、模型自行授权、任意补丁和未具备能力。既有全部伪接口类型检查通过，新设计不冒充服务已经运行。

## 验证与结果

结果为本次 `docs(api): reserve scoped AI-assisted editing contracts` 提交。

- 先形成 [ADR-011](../decisions/PRODUCT-ADR-011-assisted-editing.md)，再增加 [辅助编辑说明](../../module-api/assisted-editing.md)、3 个 TS 文件和检查入口；主协议／StageClient 与工程 Schema 未变。
- 模型工具与可信宿主入口分开，限定对象、目标灯具、属性、类型、数量及有效期；复用 EditOperation 子集和 EditReceipt。提案差异、隔离预演、人工应用／预授权自动应用、撤销、隐式写入、并发修订与重复请求均有规则。
- 亮度 30% 的例子使用已读灯具地址和现有场景，仅生成 `update-cue`／`cue-only`；不捏造 DMX 通道或将模型调用当成实时执行。MCP 工具发现／Schema 与取消语义仅借鉴可选适配，未安装服务。
- 使用项目已安装的 TypeScript 7.0.2 执行 `apps/ui-prototype/node_modules/.bin/tsc -p docs/module-api/tsconfig.json`：全部 11 个 TS 输入通过；新增 11 个编译期反例均产生预期拒绝，包含完整有效 TimelineDraft 也不能作为初批 AI 编辑操作。
- 本次 Markdown 引用／围栏、能力编号 CAP-01—21、TS 文件清单、差异空白与范围检查通过；没有 apps／crates 改动。README、模块索引、能力总表、实现状态及 STATE 已同步。

审查结论：本轮伪接口可交付，未增加第二套工程语义或模型供应商依赖。未解决项为运行服务、Rust 权威 DTO／Wire Schema 生成、候选快照／预演、持久回执、真实鉴权与模型调用，以及对应运行验收。类型检查不能证明这些已实现。

没有模型服务安装／调用、外部工程数据发送或设备操作。未运行与文档／伪接口无关的 Rust、界面或硬件测试；后续从真实场景参数编辑闭环接入，不影响当前 UI 与首版交付顺序。
