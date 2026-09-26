# 已实现：列表编辑、编译与离线预览

基于 [ADR-013](../development/decisions/PRODUCT-ADR-013-sequence-preview.md)，由 DESKTOP-003 / PLAYER-001A 实现。本文是当前运行接口，区别于其他文件中的未来伪 API。

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

执行计划上限：512 属性、1024 步、262144 个 u16 目标值，每项时间最多 24 小时。`value_buffer_bytes()` 是目标／默认／当前／起点值数组的有效数据字节数，不包括元数据、分配器额外容量、宿主工程和 DMX／IPC 缓冲；没有做 MCU 内存或吞吐实测。

## 桌面预览服务

独立 `preview_request` 接口：snapshot；load（工程 generation、sequenceId）；control（预览 epoch、递增 serial、command）。控制命令为 execute（stepId）、next、pause、resume、stop。UI 每次只保留一个监看请求，控制期间忽略此前发出的旧监看结果。

打开／新建成功清空预览，取消文件操作保留；保存不使计划过期。编辑／撤销／重做改变内容代次，旧计划保持独立，但拒绝新的执行和继续；仍允许暂停、停止、重新载入。UI 未应用草稿在载入前统一验证。运行状态不进入工程和编辑历史。

输出是完整 512 字节槽位和归一化灯具属性；当前编译器要求工程内灯具全部配适在同一输出域／线路。它是离线数值预览，不包含 RS485 驱动、设备发送确认、物理时序、3D 光学、专业灯具分段或多源混合。未来这些通过相应适配与契约扩展，不改写 UI 播放算法。
