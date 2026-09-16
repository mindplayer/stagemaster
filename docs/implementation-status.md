# 实现状态

既有代码缺陷核查：2026-09-10；架构与接口再评估及 G0 实施更新：2026-09-11。技术框架为 Rust＋TypeScript、Tauri 2＋React；当前代码是 A0 静态语义验证原型，能完成一次内存求值和编码。没有初始化 Tauri／React，没有持续播放调度，没有连接真实控台或 DMX 设备。[架构审查](architecture-review.md)复现的问题中 R01、R02 已在 G0 修复并加入保护回归，R03–R09 仍待后续契约任务处理；[架构 v0.5](architecture.md)是设计补充，不是已实现能力。

| 能力 | 状态 | 当前证据 | 下一步 |
| --- | --- | --- | --- |
| 长期 AI 开发协作 | Sol 直接开发、Astra 高级调度；G0 按调整后的范围结项 | [DEV-ADR-001](development/decisions/DEV-ADR-001-sol-astra.md)取消 Qwen／工作器路线；Git 基础、两个核心修复及 Rust 24 项回归保留；工具残留缺陷仍未修复 | Astra 规划 G1 契约与首批工单，Sol 直接实施；不再补修工作器或累计模型资格样本 |
| 核心语言复评 | 保留 Rust 主核心；无两种语言性能对测 | [Rust／C++26 复评](core-language-rust-vs-cpp.md)核查官方支持状态与本机工具链，TS 只作接口／客户端及云端业务 | 按同一实时预算验证核心；SDK／固件有具体约束再局部采用 C／C++ |
| ESP32／ARM 独立 Cue 播放盒 | 使用场景已明确；方案评估，无固件／实机 | [播放盒评估](standalone-cue-player.md)覆盖离线选 Cue、目标执行包、Rust 复用、受限档位与恢复 | 先虚拟时间验证计划和 Cue 跳转，再选板验证独立播放与容量 |
| 模块伪 API | 接口草案；无服务实现 | [方案 0.3](module-api/README.md)含 Rust 伪接口、TS 声明、调用样例和编译期反例；8 个 TS 文件的严格检查通过 | 固定首批契约并生成 Rust／TS 对应类型，逐模块实现与联调 |
| 外部音视频／设备控制与监看 | 资料研究与接口草案；无协议接入 | [专项设计](audiovisual-stage-design.md)及 external-contracts／external-examples 区分控制、反馈和监看，已纳入 TS 检查 | 验证一个外部播放器、一个媒体返回源与灯光模拟的闭环 |
| 实体控台／双向控制面 | 产品目标与接口草案；无硬件或固件实现 | [硬件设计](hardware-control-surfaces.md)及 surface-contracts 定义输入、反馈、映射屏障与接管，已纳入 TS 检查 | 先虚拟输入，再验证已有设备；电动推子单独实测 |
| 架构扩展审查 | 设计与契约修订；无运行实现 | [C01—C10 与验证门槛](architecture-evolution-review.md)，包含源码依赖复核、正反类型样例 | 以单域、虚拟推子和假外部设备验证替换边界 |
| UE5 专业预演 | 优先原型候选；未安装／接入 | [专项设计](ue5-professional-previsualization.md)定义中立场景、渲染生命周期与验证场景；未加入 TS 契约检查范围 | 选择受控场景，在指定 Mac 上验证状态接入、灯具表现、资源预算与隔离 |
| 真实场地采集／重建 | 设计建议；无实现或实测 | [混合场景方案](venue-capture-design.md)定义高斯外观、几何、业务对象分层与修订 | 先导入小场地采集结果，验证尺度、重新布光与工具平台限制 |
| Depence R4 对照／多设备预演 | 公开资料重点对照；无仿真实现 | [12 项对照与架构补充](depence-r4-assessment.md)，含独立预演、仿真生命周期和图纸职责；未加入 TS 检查 | 先验证灯光／视频场景，再按项目扩展专项模型 |
| 归一化属性和类型化 ID | 原型 | u16 数值、u64 ID 包装；未定义跨工程身份与传输规则 | 建立带类型值、离线身份、单位、物理范围和子灯地址 |
| 有序 Group | 已有基础实现 | 保留选灯顺序并去重 | 增加二维／三维 Selection Layout 和选择变换 |
| Programmer | 已有基础实现 | 选择、字面量、Preset 调用、激活／释放分开 | 增加属性过滤、来源范围、撤销命令和多用户上下文 |
| Preset | Selective 原型 | Cue 保留引用；重新求值读取新值；已激活 Playback 不自动更新 | 作用域、循环检查、版本与现场更新策略 |
| Cue／Sequence Tracking | 静态基础求值；R01 已解决 | 顺序处理 Set／引用／Release；新增和替换 Cue 均校验另一稳定 ID 的编号，6 项保护回归通过 | 补 Part、Block、Cue Only、MIB 和时间；其他约束按后续契约处理 |
| 播放器 | 静态多 Playback 容器 | 激活数值集合、调电平、立即释放；没有时钟 | Cue 状态机、命令、编译计划、版本切换和时间推进 |
| 输出合成 | HTP／LTP 原型；R02 已解决 | HTP default 仅在无有效贡献时回退；8 项保护回归覆盖零值、权重、优先级和缩放；未知属性仍可被忽略 | 固定专业推杆／合成契约，编译前校验不支持属性；处理 R03、R07 |
| 来源追踪 | 初步贡献记录 | 可见 Playback 等来源；未保留 Cue／Preset／效果链 | 编译 sourceMap、获胜／抑制／回退原因与帧关联 |
| DMX 配适 | 简单平面档案 | 地址、占用、粗细通道及跨界检查 | 领域档案与编码分离；多单元、功能范围、版本及校准 |
| DMX 编码 | 8／16-bit 数据编码 | 一次快照生成 512-slot 通道负载，不是物理发送 | 中立数值帧、epoch／时序和真实输出适配器 |
| 模拟逻辑链路 | 可运行 | demo 中两个灯具 Intensity 255、Blue 166 | 转为带虚拟时钟、状态和失败场景的验收工程 |
| 工程持久化 | 未实现 | 只定义边界 | 版本化 schema、事务保存、迁移及恢复 |
| 时间、效果、音频参考 | 未实现 | 已进入产品蓝图；正式音频播放归外部系统 | A1 设计时钟、波形参考、外部播放控制和监听闭环 |
| Tauri／React 界面 | 未实现 | 技术框架已确定 | 核心命令稳定后初始化桌面壳并生成 TS 契约 |
| RS485／网络输出 | 未实现 | 仅有未实现的 `DmxSink` trait，接受／发送状态尚未细分 | 帧契约、输出所有权、队列与时序；再做设备测试 |
| 云端服务 | 未实现 | Fastify＋PostgreSQL＋对象存储职责已确定 | 单机发布格式稳定后落地 |

## 验证命令

架构 v0.5 与接口 0.3 已按[完整目标再评估](architecture-evolution-review.md)修正独立操作会话、监看稳定键、外部动作同步组代次、单域编译／激活及工程／执行两类包，增加无节目启动和跨端交换的类型样例。C-A1—C-A10 均为待实现的运行验收；时钟／deadline、资源清单及兼容矩阵仍需原型细化。G0 只修改 R01／R02 的局部 Rust 语义和开发工作器，没有真实设备验证。

```sh
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked --offline
cargo test --workspace --locked --offline
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
cargo run -p stagemaster-engine-demo
```

G0 的 CORE-002 集成提交 `81ed816fff5d8a358d5e1ecf933057a148d02e8f` 已运行 fmt、离线 workspace 测试和严格 Clippy并通过。workspace 当前 24 项测试：原有 10 项，加 CORE-001 的 6 项 Cue 唯一性保护回归及 CORE-002 的 8 项 HTP 回退保护回归。demo 与真实设备未在 G0 重跑，不能据此扩大能力声明。

新增接口检查命令：

```sh
npm exec --yes --package=typescript@5.9.3 -- tsc -p docs/module-api/tsconfig.json
```

严格类型检查通过，并核查相关文档的本地链接与代码围栏。该检查只验证声明、调用样例和编译期错误反例，不证明业务服务、RPC、鉴权、时序或设备输出已实现。

独立临时探针曾复现 Cue 编号替换碰撞、HTP 非零默认下限、未知属性被忽略、运行中 Preset 更新不传播、Cue 编号越界饱和行为。前两项现已由产品内保护回归验证修复；R03、R06、R09 仍保留。历史探针不计入当前 24 项测试，模拟运行也不代表现场实时性、完整确定性或商业可用性。
