# EXEC-013 后台总亮度与熄灯

状态：本轮功能验收完成；最终全量回归复现既有最近工程写锁失败，出口未关闭，接续 PROJECT-003。负责人：当前 6.1 Sol，单写者。2026-10-04，基线 main `31e4b981c963d6f0a10a625f1a3c4028325ad9f1`，项目主工作区；用户 `output/` 未跟踪并保护。

依据：[路线 H2](../sol-handoff/roadmap.md#h2-补齐必要现场操作)、[ADR-156](../decisions/PRODUCT-ADR-156-background-output-master.md)、[后台契约](../../module-api/desktop-background-execution.md)。EXEC-012 已完成，不重复。

## 可用增量

- 核心合成后强度总控，后台权威状态／能力与独立百分比／熄灯命令；复用 OutputMaster、原认证租约与回执。
- 正式现场后台区连续推子、整数精确输入、熄灯与明确提示；复用原连续调度，不写工程历史、不联动编辑预演、不释放或停止节目。
- 未识别亮度的灯具数量、只读／最后已知、旧能力禁用、草稿与连续互斥、取消／失联／抢占保护。

## 验收

1. 两来源＋手动层，合成后仅识别强度缩放一次，颜色／功能不变；黑场继续节目，恢复读取当时状态；0 仍持有，明确释放才归还。
2. 实际控制权／版本／序号／重放、非法输入原子拒绝、旧能力和 malformed 状态拒绝、软件最终帧与只读再连；不把软件采样当实灯。
3. 单在途最新目标／迟到回执、轨道外松手、键盘／取消、精确草稿、窄屏与错误、正式 `.app` 两动态节目／退出重开／释放。
4. Rust 相关及工作区／UI／类型／严格检查／格式／正式打包按实际执行记结果，失败日志保留；源工程与用户 output/ 保持。

证据存 `data/EXEC-013/`，日志 `logs/exec-013-*`，临时／缓存在项目 tmp/。物理差分／听音／长期及客户无 UE 编辑器打包保持未通过；完整 goal active。

## 实现与自审

- Rust 复用 `OutputMaster` 与 `LiveOutput` 的强度掩码，缓存掩码在准备期生成，合成路径不增加逐帧分配；旧 `render` 默认输出保持。`Session::values` 明确为总控后值，来源贡献和赢家保持。百分比／熄灯独立操作，防止迟到亮度意图清除熄灯。
- 组级操作接入原宿主、共享客户端和桌面入口；保持认证、租约、序号、版本、期限及原回执。不改变工程格式、依赖或第三方版本；旧 v1 拒绝总控，v2 能力／目录／状态必须一致。
- 原连续调度器提取目标请求策略，来源与手动目标保持既有请求，总控无需伪造来源。全局互斥以目标 key 而非来源身份判断。新总控组件／策略独立，入口只组装；相关容器 `BackgroundExecution` 302、`ManualControls` 309 行已评估，职责未扩入业务，未新增超过 500 行文件。
- 自审覆盖合成顺序、整数精度、亮度掩码、状态校验、回执保护、历史不变与失败行为。部分灯型不能识别亮度时显示数量；熄灯不作全场物理黑场或机械急停保证。

## 实际验证

所有项目命令显式使用根目录 `tmp/`；Rust 使用项目 `tmp/cargo-home` 与 `tmp/framework-001-light-target`，npm 使用项目缓存。相关检查与失败日志均保留，没有削弱保护、跳过失败或禁用严格检查。

- 初次完整 `cargo test --workspace --locked --offline` 通过 **1196 项 Rust＋2 项文档测试**，日志 `logs/exec-013-workspace-final.log`。随后新增租约到期保护和严格检查等价修正后，实际宿主最终 5 项及相关核心／桌面专项通过。交付前再次执行原完整命令，退出 101：桌面 113 通过、1 项旧 `recent::store::tests::multiple_instances_merge_latest_data_and_busy_writer_preserves_it` 在第二次 remember（第 90 行）遇到写锁拒绝，`logs/exec-013-workspace-delivery.log`。不能称最终全量通过，也不以首次通过覆盖本次失败。
- 最终 `cargo clippy --workspace --all-targets --locked --offline -- -D warnings`、`cargo fmt --all -- --check` 通过，日志 `logs/exec-013-clippy-final.log`／`exec-013-rust-format-check.log`。覆盖合成后仅强度衰减、RGB 回退、粗细字节、零值持有、黑场内步骤推进、非法值／倒退时间拒绝，以及真实宿主回放／版本／接管／租约到期／只读再连。
- 原生音频软件后端专项确认熄灯后仍为同一音乐实例与代次，消费帧继续；桌面实际后台桥确认预演总控为零且熄灯时，后台 50% 仍输出 `128/255`，无二次缩放，后台熄灯只改变强度。日志 `logs/exec-013-host-final.log`／`exec-013-targeted.log`；不等于本轮现场听音或 UE 画面验收。
- **409 UI**、类型、相关 14 个 TS 文件格式及正式 `.app` 打包通过，日志 `logs/exec-013-ui-final.log`／`exec-013-types-final.log`／`exec-013-ui-format-check.log`／`exec-013-build.log`。新总控 5 项测试复用已有矩阵，纳入默认测试命令。
- 真实组件：延迟 1500ms 时快速 0→100→99 只提交首个与最终目标；精确 37%、草稿禁用推子、取消与焦点返回、熄灯锁存后改零再解除、实际拒绝不重试、失联最后已知且禁用、未识别数量和只读通过。窄栏宽度与滚动宽度同为 266，无溢出；最终新页 warn/error 为 0。证据 `data/EXEC-013/component-*`。
- 正式桌面：独立工程两动态节目＋较高优先级手动层，两项亮度原值 `52428`（80%）。真实总控拖动 50→84→28%、区域外松手、左箭头 50→49%、精确 0／37% 均得到实际状态。原生 macOS `Home` 未改变滑块，未记为通过；键盘细调与精确零值分别实际验证。0%／熄灯时两节目继续，手动值／来源电平不变；37% 时对应 8 位强度 75，熄灯为 0，功能／位置等槽不变。
- 退出编辑器后实际后台 PID 96583 继续，采样时钟增长；重开同一启动身份 `210f8d2a-5d00-44aa-b841-4678ff272878` 只读保留 37%／熄灯及两项持有，不自动抢权或重播。明确取得控制权解除熄灯；编辑预演熄灯不影响后台 37%。明确整层释放后持有 0、节目继续；归还控制权不复位总控。`native-*-state.json`、`native-*-ax.txt`／截图与 `logs/exec-013-native-*` 为对应证据。
- 源工程 `data/EXEC-012/manual-brightness.project.json` 与独立 `data/EXEC-013/output-master.project.json` 前后 SHA-256 同为 `a27abb8b90de6bf80884d6bb61d6b3e0cff243144660c8f54f29d515e123b097`，撤销／保存始终禁用。最后通过正式界面确认关闭后台，PID 已退出；临时 Vite 92332 与组件页关闭，保留已保存的本轮工程，音乐／UE／设备未启用。用户 `output/` 未改。

首次宿主测试在缺少能力时真实失败（`exec-013-red-host.log`），之后修正新增测试中旧 acquire pending／release 回执、手动优先级、真实字段等前提，并修正 CloneOnCopy／过长测试与通配模式严格问题，保留对应 `*-first-failure`／`*-second-failure`／字段失败日志；未降低产品断言。组件热更新曾产生重复 root 警告，补标准 dispose 卸载，最终新页零告警，不将热更新全过程称为无告警。

最近工程旧锁失败已有 EXEC-009 历史记录，本轮生产文件未改。原 `update` 依靠 File 最后关闭释放锁；本地受控实验确认仍有复制句柄时，原句柄关闭后下一写者返回真实 `WouldBlock`，显式 unlock 后可取得（`logs/exec-013-lock-probe.log`）。原最近工程六项隔离通过（`cargo test -p stagemaster-desktop --locked --offline recent::store::tests`，`logs/exec-013-recent-isolated.log`），不覆盖全量失败。这支持并发子进程继承句柄的候选原因，不证明本次现场根因；下一项 [PROJECT-003](PROJECT-003-recent-write-lock-lifecycle.md) 先补锁错误与受控重现，不盲重试或改测试来隐去失败。

## 结果与下一步

相关文件提交为本次 `feat(execution): control background output intensity and blackout`，基线／工作区如上，提交后核对实际 HEAD 和状态。最终证据索引为 `data/EXEC-013/verification.json`（运行产物忽略，不把原始快照／缓存纳入 Git）。

先接续 PROJECT-003 最近工程锁问题，恢复完整回归出口，再推进 H2 执行工作面的代表演出批量操作／保护；共同亮度与后台主控／明确释放不重做，不前置全部专业控台页面。AUDIO-020 现场听音、历史音频预留偶发失败原因、GPU 首帧日志、H3 指定真实灯型、H4 物理差分／实灯／长期及 H5 客户无编辑器 UE 打包仍未关闭，不宣称完整闭环或商业产品完成。
