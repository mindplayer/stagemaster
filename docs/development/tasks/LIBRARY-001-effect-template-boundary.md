# LIBRARY-001：云端灯效复用边界准备

状态：设计增量完成，运行实现未开始。2026-10-02，基线 `7befdf7`；主工作区，当前会话单写者；`output/` 不动。结果为本次 `docs(effects): define capability-bound reusable template library` 提交。

用户希望后续云端灯效库复用于摇头、星空、激光、帕灯、射灯和多种 LED。核对 ADR-009、已有动态效果与云端架构后，补 [ADR-090](../decisions/PRODUCT-ADR-090-reusable-effect-library.md)，将语义模板、灯具映射、工程绑定和目标产物分层；不重复技术选型，也不新增运行占位按钮。

已查阅 MA3 2.4 配方、Titan 19 效果预设及 GDTF 通道定义；参考机制、范围、数据归属、离线版本、更新与取消／失败政策见 ADR。新增独立 `effect-library-contracts.ts` 及正反例，接口仅覆盖有界方向，不能据类型存在声称跨型号绑定／云端服务已实现。未来 DTO 仍以 Rust 核心为权威。

验证：`apps/ui-prototype/node_modules/.bin/tsc -p docs/module-api/tsconfig.json` 通过，含阻断结果不能提交、原始 DMX 不能伪装语义颜色的反例（`logs/effect-library-contracts-check.log`）。当前工程、实际播放器与设备格式均未改。首个模板实现须冻结 schema、范围、预算、依赖／派生编辑策略并验证 ADR 的跨灯型矩阵；激光、多单元和控制宏仍需独立语义增量。

持续 goal 接续 FIXTURE-008 色盘变体通道值审阅与真实灯具能力缺口，不把云服务前置于桌面可用性。
