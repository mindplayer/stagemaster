# 已实现：列表编辑、编译与离线预览

基于 [ADR-013](../development/decisions/PRODUCT-ADR-013-sequence-preview.md)，由 DESKTOP-003 / PLAYER-001A 实现。EFFECT-001 已扩展[单场景与动态效果](lighting-effects.md)：共用同一执行器和单调时钟。本文是当前运行接口，区别于其他文件中的未来伪 API。

## 单向依赖

界面功能组件 → `ApplicationHost` → Tauri 会话适配 → Rust 工程／编译；编译生成不可变 `stagemaster-playback::Plan`，纯执行器只读计划并计算属性数组；独立 `CompiledOutput` 复用 DMX 编码。设备、网络、文件、界面不能成为执行模块依赖。

工程保存原始引用和顺序；执行计划内将预设和继承解析成完整目标。临时数字灯具／属性索引仅属于编译结果，不替换工程 UUID，也不是设备持久协议。已保存修订号不能单独标识未保存编辑快照；桌面另维护内容代次，预览安装代次用于拒绝旧控制请求。

## 工程编辑

沿用 `Document::edit(EditCommand)`，新增 `Sequence { command: SequenceEdit }`，也可作为原子批次的一项。

```json
{"op":"sequence","command":{"kind":"updateStep","id":"列表 UUID","stepId":"步骤 UUID","name":"入场","number":"1","sceneId":"场景 UUID","delayMs":500,"fadeMs":3000,"waitMs":2000}}
```

上述 ID 为接口说明，实际请求必须使用当前工程的稳定 ID。`waitMs: null` 表示手动推进，非空则从渐变结束计时。其他命令见 `crates/stagemaster-project/src/sequence.rs`：add、update、duplicate、remove、insertStep、moveStep、duplicateStep、removeStep。一次命令／批次只有一个撤销节点。

TS 的 `sequence-tools.ts` 只处理输入草稿、秒数的精确转换和命令构造；Rust 是最终校验者。`SequenceInspector` 管理控件展示，`SequenceWorkspace` 协调选择与编辑，`PreviewPanel` 管理独立预览传输与监看，不在组件里创建第二套播放语义。

## 编译与纯执行

```rust,ignore
let compiled = document.compile_sequence(sequence_id)?;
let mut player = stagemaster_playback::Player::new(compiled.plan, 0);
player.execute(0, 0)?;
player.advance(2000)?;
let output = compiled.output.render(player.values())?;
```

`Player` 提供 execute、next、pause、resume、stop、advance，所有调用使用外部单调毫秒；没有休眠、线程和物理输出。直接执行中间步骤也具有确定目标，渐变从当时值开始。停止恢复灯具默认属性，未配适通道为零；不能把整帧全零当所有灯具或机构的安全状态。

执行计划上限：512 属性、1024 步、262144 个 u16 目标值，每项时间最多 24 小时。`value_buffer_bytes()` 是目标／默认／当前／起点值数组的有效数据字节数，不包括元数据、分配器额外容量、宿主工程和 DMX／IPC 缓冲；PLAYER-002A 已测 ESP32-S3 上 2 步／512 属性的执行与堆占用，见[设备诊断接口](device-link-probe.md)；未验证全部上限容量，也未包含 DMX 发送预算。

## 离散属性执行基础

FIXTURE-003A／[ADR-059](../development/decisions/PRODUCT-ADR-059-discrete-playback-attributes.md) 增加 `Plan::with_snap_attributes(defaults, steps, repeat, effects, indices)`。严格递增的 u16 索引声明不可渐变的属性，普通构造函数默认空列表，原有行为不变；`snap_attributes()` 只读，`snap_buffer_bytes()` 返回索引有效存储字节。

步骤延时内保留当前值，延时结束直接切到新目标；其他属性继续原渐变。暂停保持、恢复不重放，打断后新步骤延时内保留被打断时的值；零渐变、自动推进、跳转和循环采用同一规则。停止恢复各自默认值。离散属性与任何动态效果冲突会在计划构造／包扫描时拒绝，不能由曲线重新引入中间值；命名档位追逐待独立语义。

FIXTURE-003B 已接命名功能区间、类型化属性与 UI，详见[灯具建档](fixture-authoring.md)；未据此确认实板或真实光学效果。

## 桌面预览服务

独立 `preview_request` 接口：snapshot；load（工程 generation、sequenceId）；loadScene（generation、sceneId）；control（预览 epoch、递增 serial、command）。根快照带 controlSerial，避免不同面板独立递增产生冲突。控制命令为 execute（stepId）、next、pause、resume、stop。UI 每次只保留一个监看请求，控制期间忽略此前发出的旧监看结果。

打开／新建成功清空预览，取消文件操作保留；保存不使计划过期。编辑／撤销／重做改变内容代次，旧计划保持独立，但拒绝新的执行和继续；仍允许暂停、停止、重新载入。UI 未应用草稿在载入前统一验证。运行状态不进入工程和编辑历史。

输出是完整 512 字节槽位和归一化灯具属性；当前编译器要求工程内灯具全部配适在同一输出域／线路。PREVIS-001 已通过独立[预演适配](previsualization.md)把该输出用于应用内三维。播放模块本身不包含 RS485 驱动、设备发送确认、物理时序、3D 光学、专业灯具分段或多源混合；这些通过相应适配与契约扩展，不改写 UI 播放算法。

## 剧本提示

[ADR-062](../development/decisions/PRODUCT-ADR-062-sequence-script-prompts.md) 增加 `SequenceEdit::UpdateStepScript { id, step_id, script: Option<StepScript> }`；JSON 命令为 `updateStepScript`，对象含 `section`／`trigger`／`notes`。null 或全空清除；旧 updateStep 保留提示。工程带对象时要求 `lighting.sequence-script@1`，单字段 80／1024／4096 个 Unicode 字符，每列表 UTF-8 总量最多 64 KiB。提示复制、删除、撤销和保存沿用工程事务。

ProjectView.StepView 与 CompiledStep 带可选 script，缺省时序列化不增加字段。执行台当前／下一步读取编译元数据；编辑后的提示不覆盖旧运行快照。文字不改变 delay／fade／advance，也不进入纯 Plan 或设备播放包，硬件执行容量不因此增加。检索仅影响界面选择，幕场标签不构成嵌套调度。
