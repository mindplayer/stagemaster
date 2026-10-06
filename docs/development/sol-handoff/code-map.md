# 代码地图与模块调用框架

2026-10-07：本页用于查现有职责和调用链，当前工单排序见[首发完善计划](../first-commercial-release-plan.md)。下方 AUDIO-020 的“建议新增”结构是早期规划示例，已有实现先核对，不据此创建重复组件。

本文件描述 `bab1789` 的实际代码组织及后续扩展约束。代码路径以仓库根为基准。优先延长已存在的调用链；这里的简图与伪代码不是新建一套同名服务的指令。全功能远期接口仍在 `docs/module-api/`，不能把那里的设计类型当作已实现方法。

## 实际目录与职责

| 入口 | 职责／唯一状态归属 | 后续改动原则 |
| --- | --- | --- |
| `apps/ui-prototype/src/components/` | React 功能组件、局部输入／选择／展开 | 不实现灯效求值、物理角度转换或直接调用 Tauri |
| `apps/ui-prototype/src/application-host.ts` | 界面到平台适配的应用请求契约 | 保持宿主无关；新端提供适配，普通组件复用 |
| `apps/ui-prototype/src/Workbench.tsx` | 已有工作台组装 | 新业务放独立动作／草稿模块，入口只传递依赖 |
| `apps/desktop/src/session/` | 当前工程、代次、编辑安装、历史与保存协调 | 复用 `install_edit`、guard 和原有请求队列，不自建第二撤销栈 |
| `crates/stagemaster-project/` | `Document`、校验、编辑、编译、场地／灯具／音频编排语义 | 按领域拆文件；JSON 是边界格式，运行不反复解析全工程 |
| `crates/stagemaster-project-store/` | 工程持久化、原子保存与恢复 | 内存已修改和磁盘已保存明确区分 |
| `crates/stagemaster-domain/`、`stagemaster-show/` | 基础领域与节目结构 | 不引入 UI、存储和设备依赖 |
| `crates/stagemaster-playback/` | 受限 `Player`、渐变／步骤／循环状态 | 显式时间输入；复用唯一规则，不在固件另写一套 |
| `crates/stagemaster-live/` | 多来源贡献、合成、媒体跟随组 | 合成语义值后统一编码，所有权和数值分离 |
| `crates/stagemaster-runtime/` | 安装后运行、实例、控制权、维护 | 连接会话生命周期不等同节目生命周期 |
| `crates/stagemaster-runtime-host/`、`stagemaster-live-host/` | 独立调度线程、有界命令、回执和快照 | 准备／I/O 与执行隔离，慢观察不阻塞输出 |
| `apps/execution-host/`、`crates/stagemaster-execution-client/` | 本机独立进程与客户端协议 | 本机受保护入口不是可公开暴露的云 API |
| `crates/stagemaster-time/`、`stagemaster-audio/` | 时间域映射；有界解码／实际消费与声音后端 | 消费位置不冒作声卡实际呈现时刻；音频可选组装 |
| `crates/stagemaster-spatial/`、`stagemaster-previs/` | 位置求解、渲染中立快照 | 灯具安装、校准、动作与光学投影分离 |
| `apps/previs-unreal/`、`tools/previs/` | UE 渲染／交互与信令适配 | 只读观察运行，编辑动作送回正式命令／历史 |
| `crates/stagemaster-package/`、`stagemaster-transfer/` | 受限节目包与传输记录 | 版本、能力、长度与预算校验；不直接接纳完整工程到 MCU |
| `crates/stagemaster-install/`、`stagemaster-install-worker/`、`stagemaster-install-store/`、`stagemaster-nor-store/` | 安装事务、持久提交／恢复、Flash 适配 | 校验、安装、激活分开；维护必须等输出静默 |
| `crates/stagemaster-device-*` | 发现／连接、应用会话、权限、运行和上传等具体模块 | 按名称查契约，业务不能与 BLE 绑定；不要又加通用大 DeviceManager |
| `crates/stagemaster-output-port/`、`stagemaster-dmx/` | 端口权威、帧交接与 DMX 逻辑 | 接纳不等于发送完成，故障不能发布虚假新帧 |
| `apps/esp32-player/` | 板级时钟、BLE、Flash、UART／看门狗组装 | 遵守目标能力与资源预算；启用物理输出须有对应授权和测量 |
| `crates/stagemaster-delivery/`、`stagemaster-device-auth/` | 内容取得／许可接缝及现有设备认证 | 开发机制不是生产安全；云端与 U 盘复用后续安装链 |

`stagemaster-engine`／`apps/engine-demo` 中的早期混合原型不应代替当前 Player／live／Host 权威链。工程已有多个 crate，不以“模块化”为由再平行创建全套空模块。

## 三条调用链

```text
编辑
React 表单／手势 → ApplicationHost → 桌面 Session
  → Document 的纯编辑／校验 → install_edit → 修订＋一次历史
  → 新快照返回界面；显式保存 → project-store

执行
显式选择工程快照／目标能力 → 编译／准备 → 独立 Host
  → 已授权且带版本的命令 → Player／live 合成
  → 完整帧 → 输出端口 → 驱动完成／故障回执
  → 只读观察 → UI／UE

设备自主播放
主机工程 → 按目标能力编译受限包 → 内容校验
  → GATT 或后续承载 → 安装／提交／恢复
  → 显式激活 → Runtime／Player → 板级 DMX
```

云端将来接在“内容取得与身份／权益”边界，推子接在“语义命令”边界，视频提供方接在“媒体／外部动作”边界。它们都不能绕过控制权直写 DMX。

## 一个真实功能的写法

EXEC-011 是最新范例，相关 API 确实存在：

```rust
// A/W：只读审阅。capture 来自已校验的权威手动值。
let merge = document.prepare_manual_scene_merge(&capture, scene_id)?;
let summary = merge.summary();
// 返回 summary 给界面；宿主保留合并计划和随机票据。

// 确认后由 Session 克隆候选、校验、一次安装历史。
// 下行展示 Document 的真实领域方法，不是 UI 可直接调用的远程入口。
candidate.merge_manual_scene(&merge)?;
```

对应路径：`crates/stagemaster-project/src/manual_scene_merge.rs` → `apps/desktop/src/manual_capture.rs`／`session/edit.rs` → `src/manual-recording-actions.ts` → `components/execution/ManualSceneDestination.tsx`／`ManualSceneReview.tsx`。前端只提交 `{ kind: "mergeManualScene", generation, token }`，不能篡改审阅数值或换目标。

这种“领域规则→宿主事务→界面组件→行为验收”模式可复用；不要给普通单字段编辑一律增加票据，只有需要冻结跨步骤审阅的操作才采用。

## 下一任务 AUDIO-020 的简单结构

已有领域类型：`AudioLoopRegion`、`AudioLoopPlays`、`AudioLoopEdit`／`AudioLoopGroupAction`，位于 `stagemaster-project/src/audio_loops.rs` 和 `audio_loop_edit.rs`；已有运行 `LoopSchedule`／`LoopPlayback` 和 `PerformanceAudio`。具体 DTO 看 [循环契约](../../module-api/performance-loops.md)，不得再实现第二计时器。

建议新增独立组件，名称可局部调整，以下尚非已有文件：

```text
components/audio/
  PerformanceLoopList.tsx       区段检索／选择／多选
  PerformanceLoopProperties.tsx 精确起止、次数／持续、启停与锁定
  PerformanceLoopLane.tsx       时间线上展示和编辑手势
audio-loop-draft.ts              固定工程／目标的草稿与提交协调
tests/                          规则测试与真实组件验收入口
```

调用伪代码仅表示层次；落地时使用现有 ApplicationHost 的音频请求形状，不按下文新建重名通用接口：

```text
选择区段 → 从真实工程建立草稿（工程代次＋区段身份）
修改表单／拖动 → 本地显示，保持运行游标独立
确认／松手 → 已有 AudioLoopEdit → 原子编辑＋一次历史
失败 → 保留草稿、定位错误；取消 → 撤回本地草稿
计划变化 → 原有音频准备规则；名称／锁定变化不重启音源
```

## 扩展边界和禁止捷径

| 新能力 | 正确落点 | 不采用的捷径 |
| --- | --- | --- |
| 新灯型 | 档案能力、物理量、功能区间、独立档案修订；编译器验证 | 按型号名称在 React／UE 中写特判 |
| 通用灯效库 | 参数化语义模板＋目标绑定＋能力报告 | 把某台灯的原始 DMX 数组当跨灯型模板 |
| 控制面 | 设备输入适配→绑定／拾取→同一控制命令；反馈走快照 | UI 鼠标事件转发或串口直接改执行内存 |
| 新传输 | 发现／记录承载适配→现有应用会话 | 在每种蓝牙／USB／网口重新实现安装／许可 |
| 新渲染器 | 已有场地／灯值／时间快照和编辑提案 | 从屏幕颜色推断正式灯值或渲染器自行播放节目 |
| 多端 | 共享 DTO／业务组件；平台 Host 与任务布局分别适配 | 假设 iPad、浏览器直接运行桌面插件或完整 UE |
| 云资产 | 不可变内容摘要、版本／清单、权限，取得后原链校验 | 登录状态或下载成功直接等同播放许可 |
| 电机／激光 | 类型化动作、确认、现场控制器反馈／联锁与明确故障策略 | 普通亮度渐变替代限位、急停或安全控制 |

## 具体契约按需读取

- 编辑与录入：[desktop-background-execution](../../module-api/desktop-background-execution.md)、[editing-library](../../module-api/editing-library.md)。
- 时间与媒体：[performance-loops](../../module-api/performance-loops.md)、[media-source-groups](../../module-api/media-source-groups.md)、[controlled-media-provider](../../module-api/controlled-media-provider.md)。
- 灯具与位置：[fixture-authoring](../../module-api/fixture-authoring.md)、[positioning](../../module-api/positioning.md)、[position-reference](../../module-api/position-reference.md)。
- 设备与端口：[device-runtime-application](../../module-api/device-runtime-application.md)、[runtime-output-coordination](../../module-api/runtime-output-coordination.md)、[dmx-transmission](../../module-api/dmx-transmission.md)。
- 渲染：[previsualization](../../module-api/previsualization.md)、[background-previsualization](../../module-api/background-previsualization.md)。
- 交付：[package-delivery](../../module-api/package-delivery.md)、[application-record-carriers](../../module-api/application-record-carriers.md)。

一个任务只读其中相关的两三份；发现过期描述按实际代码和最新 ADR 核对并局部修正，不以旧伪 API 为由大规模重写已验收实现。
