# Prototype Instructions

面向用户的业务术语统一中文，按 `../../docs/product-terminology.md` 执行。页面、辅助功能标签、提示和说明采用“场景”“场景列表”“执行”“渐变”“延时”，时间单位显示“秒”；原始参考图中的旧英文标签由此规则替代，代码标识符不作机械翻译。

StageMaster context: the user selected the first 舞台画布 visual direction and then requested a 剪映-like timeline workflow. Follow `../../docs/ui-design/visual-directions.md` and the refined `../../data/ui-design/2026-09-21/01-stage-canvas-timeline.png`. Keep runtime state, editing and offline preview distinct. The current Workbench is the real Tauri project editor backed by Rust. The older App/Timeline sources are retained as an in-memory interaction reference for later integration, not a TypeScript lighting engine. The root StageMaster rules take precedence: no subagents, real device output, deployment or files outside the project. Use TypeScript for app code. Keep generated caches under project `tmp/` and runtime logs under `logs/`.

User feedback: timeline dragging must feel continuous and follow the pointer. Do not quantize every move to coarse time steps. Snap only near visible alignment targets, keep scrubbing separate from live GO, and commit one undoable edit per completed clip gesture.

Run the local server yourself and open the preview in the browser available to this environment. Do not give the user server-start instructions when you can run it.

Before making substantial visual changes, use the Product Design plugin's `get-context` skill when the visual source is unclear or no longer matches the current goal. When the user gives durable prototype-specific design feedback, preferences, or decisions, record them in `AGENTS.md`.

When implementing from a selected generated mock, treat that image as the source of truth for layout, component anatomy, density, spacing, color, typography, visible content, and hierarchy.

Build app UI in `src/`. Keep `.openai/hosting.json`, `worker/index.js`, `scripts/prepare-sites-build.mjs`, and `tests/sites-worker.test.mjs` intact so the same local prototype can be handed to Sites. Before a Sites handoff, run `npm run build` and `npm run test:sites`; the build must leave `dist/client/index.html`, `dist/server/index.js`, and `dist/.openai/hosting.json`.

2026-09-23：用户确认先做电脑端，并优先交付可看见、可操作的功能以持续反馈。桌面宿主复用本目录唯一界面，平台调用通过 ApplicationHost 适配；工程语义与文件写入由 Rust 负责，普通组件不直接调用 Tauri。

2026-09-23 最新要求：直接按真实交付标准实现，不增加演示样例、教学说明或功能占位。正式桌面入口只提供已经接通的操作；保留旧交互原型代码作为待接入参考及回归测试，不能把其内存模拟保存当成文件保存。
