# PROJECT-001A：声光电与互动机构 JSON 格式设计

- 状态：done（格式设计；产品能力未实现）
- 负责人：当前 Astra 会话；基线／分支：`fc14472`／`main`
- 工作区：`/Users/sunqi/projects/stagemaster`；基线工作区干净
- 依据：[PRODUCT-ADR-005](../decisions/PRODUCT-ADR-005-multidomain-project-format.md)

## 交付与边界

交付 `0.1.0-draft.1` 规范、工程／现场绑定／部署清单 JSON Schema、纯灯光及声光电／密室示例、可重复运行的格式校验和负例验收。保留现有 Rust 和 UI，不访问板卡、网络设备或真实机构。

允许修改：`docs/project-format/**`、`tools/project-format/**`、本工单与 ADR-005、PROJECT-001、STATE、execution-plan、README、implementation-status，以及必要的根 `.gitignore`。生成用临时脚本和安装缓存只放项目 `tmp/`。

允许在设计期工具独立使用固定版本 Ajv 与 JSON 语法解析器，不向产品依赖注入。工程的生产领域语义仍由 Rust 维护。

## 验收

- 所有 Schema 可按 JSON Schema 2020-12 编译，样例严格解析、结构校验和跨文件引用审查通过。
- 坏引用、重复键／身份、非法类型、未知字段、时间／单位错误、运行状态／授权混入正文等关键反例被检查；明确哪些跨对象和执行约束仍需 Rust 后续验证。
- 文档覆盖模块映射、身份／时间／值、编辑与运行分离、媒体外部执行、机构保护、迁移／扩展、资源与现场绑定、设备发布及 24 小时授权边界。
- 校验工具无网络设备调用，不将 Schema 合格视为实际可播放或安全认证。
- 本任务完成只代表格式设计增量完成，不将 PROJECT-001 的持久化任务标为完成。

## 完成记录

- 结果提交：`feat(project): define modular audiovisual and motion JSON format`；格式版本 `0.1.0-draft.1`。
- 已交付：[规范](../../project-format/README.md)、4 份 Schema、纯灯光／密室工程与现场绑定、ESP32 部署清单共 5 份设计样例，及固定依赖的开发期检查工具。样例占位资源／程序／授权明确不可部署。
- 验证：Schema 编译、5 份样例的严格解析／结构／引用检查、49 项正反测试、本地文档引用与差异空白检查。曾发现严格解析结果对象原型不一致，已修正并通过原验收；没有削弱测试。
- 用户补充已连接 ESP32；只读枚举 `303A:1001`、`USB JTAG_serial debug unit`、`/dev/cu.usbmodem2101`。未打开端口、读取固件、复位、刷机或操作灯具／机构。
- 审查：条件未知值传播、互动规则归属入口、机构与普通输入输出保护边界、控制权失效与商业授权收尾分别定义。完整依赖闭合、有限结束证明、时序和生产安全仍须 Rust／现场实现。
- Rust／UI 源码未变，不重复其测试；PROJECT-001 保存／重开及 PLAYER／AUTH 保持未完成。
