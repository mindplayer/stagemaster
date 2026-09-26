# Prototype Instructions

2026-09-24：用户授权 goal 模式丰满正式工作台，并要求对照成熟软件补齐细节。按 DESKTOP-002 / PRODUCT-ADR-012 将已收敛的组件式布局接到真实工程；选择、搜索、批量编辑、取消、错误恢复、撤销与保存均须闭环。之前的“先设计、暂不介入业务”不再阻止本轮明确范围的实现；仍禁止用设计夹具或占位按钮冒充功能。

面向用户的业务术语统一中文，按 `../../docs/product-terminology.md` 执行。页面、辅助功能标签、提示和说明采用“场景”“场景列表”“执行”“渐变”“延时”，时间单位显示“秒”；原始参考图中的旧英文标签由此规则替代，代码标识符不作机械翻译。

StageMaster context: the user selected the first 舞台画布 visual direction and then requested a 剪映-like timeline workflow. Follow `../../docs/ui-design/visual-directions.md` and the refined `../../data/ui-design/2026-09-21/01-stage-canvas-timeline.png`. Keep runtime state, editing and offline preview distinct. The current Workbench is the real Tauri project editor backed by Rust. The older App/Timeline sources are retained as an in-memory interaction reference for later integration, not a TypeScript lighting engine. The root StageMaster rules take precedence: no subagents, real device output, deployment or files outside the project. Use TypeScript for app code. Keep generated caches under project `tmp/` and runtime logs under `logs/`.

User feedback: timeline dragging must feel continuous and follow the pointer. Do not quantize every move to coarse time steps. Snap only near visible alignment targets, keep scrubbing separate from live GO, and commit one undoable edit per completed clip gesture.

Run the local server yourself and open the preview in the browser available to this environment. Do not give the user server-start instructions when you can run it.

Before making substantial visual changes, use the Product Design plugin's `get-context` skill when the visual source is unclear or no longer matches the current goal. When the user gives durable prototype-specific design feedback, preferences, or decisions, record them in `AGENTS.md`.

When implementing from a selected generated mock, treat that image as the source of truth for layout, component anatomy, density, spacing, color, typography, visible content, and hierarchy.

Build app UI in `src/`. Keep `.openai/hosting.json`, `worker/index.js`, `scripts/prepare-sites-build.mjs`, and `tests/sites-worker.test.mjs` intact so the same local prototype can be handed to Sites. Before a Sites handoff, run `npm run build` and `npm run test:sites`; the build must leave `dist/client/index.html`, `dist/server/index.js`, and `dist/.openai/hosting.json`.

2026-09-23：用户确认先做电脑端，并优先交付可看见、可操作的功能以持续反馈。桌面宿主复用本目录唯一界面，平台调用通过 ApplicationHost 适配；工程语义与文件写入由 Rust 负责，普通组件不直接调用 Tauri。

2026-09-23 最新要求：直接按真实交付标准实现，不增加演示样例、教学说明或功能占位。正式桌面入口只提供已经接通的操作；保留旧交互原型代码作为待接入参考及回归测试，不能把其内存模拟保存当成文件保存。

2026-09-23 后续设计优先级：用户要求先重新定义好用的主界面与效果编辑，暂不急于扩展业务层。按 `../../docs/ui-design/effect-editor-design.md` 和 PRODUCT-ADR-007，以舞台／效果／时间编排组织工作流，配适和工程管理作为辅助入口。三张新视觉稿尚未被用户选择，不把设计建议当作已确认的像素标准。UE 可作为专业空间搭建和预演后端；普通组件仍经稳定对象与应用命令协作，不能在 TS 或 UE 内另造正式节目时钟／效果语义。生成图片和测试夹具留在设计资料，正式产品继续遵循真实能力原则。

2026-09-23 三维交互反馈：用户特别重视三维拖动的顺畅程度。分别设计镜头导航、物体摆位和光束指向；支持 Mac 触控板，无需中键才可完成常用操作。普通选灯不隐式移动安装位置，旋转中心稳定，可一键聚焦／找回舞台；一次拖动一次撤销。具体手势仍须实机验证，不把方案写成已实现功能。

2026-09-24 最新平台澄清：用户目前没有 iPad，当前仍按 MacBook 设计、开发和验收；未来 iPad 主力定位仅用于提前准备，见修订后的 PRODUCT-ADR-008。继续优化桌面布局、鼠标／触控板、效果编辑与三维拖动，保留共享命令、可适配布局、平台与渲染边界。不要改成平板优先页面、前置 iOS 宿主／真机测试或等待 iPad 型号；触控／Pencil 具体实现留到后续移动阶段。

2026-09-24 总体框架反馈：三维工作区必须准备“共同指向”——选中整组摇头灯，拖动目标点时光束持续跟随；每台灯分别求解角度，安装位置不变。选择／布置／指向工具、镜头聚焦与光学调焦分别表达；普通拖动不默认录制轨迹。依据 docs/ui-design/workspace-framework.md，当前仍是设计阶段，不把独立交互设计稿或理想指向线接入正式入口冒充灯具求解。

2026-09-24 场地创建反馈：参考成熟舞台设计／建模软件，采用“平面定义轮廓和尺寸、三维同步生成”。舞台、观众区与过道是持续可编辑的对象，观众按区域排布、过道自动避让，不逐把摆椅子；见 docs/ui-design/venue-layout-design.md。尺寸化场地、生成规则、导入与模板版本需正式契约后实施；本设计不授权在正式入口加入未接通的创建按钮或样例场地。

2026-09-24 异形室内补充：场地可能由多个长宽、形状、高度不同的房间组成。按 `../../docs/development/decisions/PRODUCT-ADR-010-composable-spaces.md` 与场地设计，用可组合空间、独立地面标高／净高、物理墙体／洞口／连接表达；不要固化为全局宽深高或强制每个工程有舞台。界面准备房间／可选楼层定位、当前空间／全场和隐藏屋顶；查看切换不改变灯具输出。当前单空间稿不代表多房间已实现，具体契约后续审查。

UX-012 后续交互迭代已在独立组件稿验证房间列表／画布选择、矩形与 L 形属性、标高／净高、平面边界拖动与立体查看；不再以单空间稿为现行设计上限。正式入口仍须接真实契约、命令与工程保存，不能直接搬入三个验证房间；完整墙体／门洞和自由三维能力尚未实现。

2026-09-24 灯具定义反馈：用户认可上述场地方向，并要求统一抽象亮度、颜色、图案、频闪等功能，允许自行定义不同年份／模式的通道。按 `../../docs/ui-design/fixture-definition-design.md` 与 PRODUCT-ADR-009，复用标准能力，分开档案变体／修订和工程灯具；借鉴 MA／Titan 的档案编辑与功能档位。界面用选功能、填通道、标区间和验证的直接流程，不能仅把通道改名、用 TS 私自解释复杂映射或给普通界面开放原始输出旁路。当前仅设计，真实编辑器和测试台按契约及设备能力逐步接入。

2026-09-24 最新 UI 要求：用户担心旧布局无法承载完整功能，要求基于上述需求重新设计。按 `../../docs/ui-design/modular-workspace-design.md`，比较任务工作区、对象标签页与大舞台布局；三图为未选定的静态提案。布置／灯具／编排／现场各有完整任务空间，监看／资源／设备按需展开；灯具定义不挤进普通片段属性栏，切换页面不触发执行且应保留编辑上下文。保持既有深色舞台风格和真实交付原则，未选型前不把新图直接替换正式产品。

2026-09-24 组件化偏好：用户明确倾向通过前端组件封装功能来保持页面清晰。以基础控件、功能组件和工作区容器组织界面；常用功能就地呈现，详细编辑按任务展开，避免常驻全部功能和层层弹窗。组件接收明确编辑目标与能力、发出应用命令，复用同一核心语义；收起／切换保留编辑上下文，不因卸载组件丢失草稿或改变现场执行。该偏好不等于选定某张视觉稿或要求任意窗口拼装。

2026-09-24 最新体验方向：用户要求接近现代、熟悉、易上手的创作软件，明确不希望沿用 MA 式陈旧、层层菜单的界面组织。以画布内容、选中对象和当前任务为中心，收敛边框／高亮和重复信息，属性在固定位置按需展开，复杂建档使用完整编辑面；直接操作、撤销与返回保持一致。参考 `../../docs/ui-design/component-workspace-design.md` 的独立交互验证。无需等待 UX-008 静态三选一才可迭代；正式接入仍按真实工程／命令逐步完成，不把局部设计状态作为产品业务。

2026-09-26：用户再次强调模块开发、功能解耦与扩展性。DESKTOP-003 采用独立列表容器／输入属性组件／草稿命令转换／预览宿主接口；Rust 工程编译、纯时间执行和 DMX 编码各司其职。新功能沿这些模块边界扩展，不在 Workbench 或 TS 内集中实现领域语义。


2026-09-27：用户反馈正式建模界面仍难用，要求借鉴大厂。UX-013 将常用尺寸、形状和直接操作放在前面，顶点坐标折叠为高级编辑；对象按空间组织。新建先确认尺寸再提交，取消不留对象；画布草稿、应用／撤销与保存同一路径。借鉴 SketchUp、Vectorworks 的成熟机制，不新增未接通的工具按钮。三维剖视属于查看状态，不删除工程墙体或改变实际灯光。
