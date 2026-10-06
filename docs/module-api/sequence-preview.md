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

MIX-002 新增[列表贡献适配与同步转换观察](live-sequence-contributions.md)，供多来源宿主复用同一个 Player。原入口通过空观察者执行，数值行为和 Plan／设备包格式保持；列表贡献中的释放是归还来源，与原单列表完整向量回落默认值分开。

执行计划上限：512 属性、1024 步、262144 个 u16 目标值，每项时间最多 24 小时。`value_buffer_bytes()` 是目标／默认／当前／起点值数组的有效数据字节数，不包括元数据、分配器额外容量、宿主工程和 DMX／IPC 缓冲；PLAYER-002A 已测 ESP32-S3 上 2 步／512 属性的执行与堆占用，见[设备诊断接口](device-link-probe.md)；未验证全部上限容量，也未包含 DMX 发送预算。

## 离散属性执行基础

FIXTURE-003A／[ADR-059](../development/decisions/PRODUCT-ADR-059-discrete-playback-attributes.md) 增加 `Plan::with_snap_attributes(defaults, steps, repeat, effects, indices)`。严格递增的 u16 索引声明不可渐变的属性，普通构造函数默认空列表，原有行为不变；`snap_attributes()` 只读，`snap_buffer_bytes()` 返回索引有效存储字节。

步骤延时内保留当前值，延时结束直接切到新目标；其他属性继续原渐变。暂停保持、恢复不重放，打断后新步骤延时内保留被打断时的值；零渐变、自动推进、跳转和循环采用同一规则。停止恢复各自默认值。离散属性与任何动态效果冲突会在计划构造／包扫描时拒绝，不能由曲线重新引入中间值；命名档位追逐待独立语义。

FIXTURE-003B 已接命名功能区间、类型化属性与 UI，详见[灯具建档](fixture-authoring.md)；未据此确认实板或真实光学效果。

## 桌面预览服务

独立 `preview_request` 接口：snapshot；load（工程 generation、sequenceId）；loadScene（generation、sceneId）；control（预览 epoch、递增 serial、command）。根快照带 controlSerial，避免不同面板独立递增产生冲突。控制命令为 execute（stepId）、next、pause、resume、stop。UI 每次只保留一个监看请求，控制期间忽略此前发出的旧监看结果。

打开／新建成功清空预览，取消文件操作保留；保存不使计划过期。编辑／撤销／重做改变内容代次，旧计划保持独立，但拒绝新的执行和继续；仍允许暂停、停止、重新载入。UI 未应用草稿在载入前统一验证。运行状态不进入工程和编辑历史。

[EXEC-019](../development/tasks/EXEC-019-preview-intent-lifecycle.md)：界面操作冻结宿主、场景／列表和可见工作区绑定。草稿处理、工程／预演读取后的异步边界重新核对，隐藏、切换／往返或卸载后不继续发送旧载入／控制；旧回执和错误不附着新绑定。已送出的动作不声称取消／回滚、不自动重发。仍用草稿后的宿主 generation、冻结的播放器 epoch 与当前 serial；共享已载入内容的暂停／停止可从当前可见面板操作，不因所选编辑对象不同而禁用。原单播放器、草稿与时间语义保持。

输出是完整 512 字节槽位和归一化灯具属性；当前编译器要求工程内灯具全部配适在同一输出域／线路。PREVIS-001 已通过独立[预演适配](previsualization.md)把该输出用于应用内三维。播放模块本身不包含 RS485 驱动、设备发送确认、物理时序、3D 光学、专业灯具分段或多源混合；这些通过相应适配与契约扩展，不改写 UI 播放算法。

## 剧本提示

[ADR-062](../development/decisions/PRODUCT-ADR-062-sequence-script-prompts.md) 增加 `SequenceEdit::UpdateStepScript { id, step_id, script: Option<StepScript> }`；JSON 命令为 `updateStepScript`，对象含 `section`／`trigger`／`notes`。null 或全空清除；旧 updateStep 保留提示。工程带对象时要求 `lighting.sequence-script@1`，单字段 80／1024／4096 个 Unicode 字符，每列表 UTF-8 总量最多 64 KiB。提示复制、删除、撤销和保存沿用工程事务。

ProjectView.StepView 与 CompiledStep 带可选 script，缺省时序列化不增加字段。执行台当前／下一步读取编译元数据；编辑后的提示不覆盖旧运行快照。文字不改变 delay／fade／advance，也不进入纯 Plan 或设备播放包，硬件执行容量不因此增加。检索仅影响界面选择，幕场标签不构成嵌套调度。

## 会话预演总控

[ADR-068](../development/decisions/PRODUCT-ADR-068-preview-output-master.md) 的 `OutputMaster` 在 Player 之后、DMX 编码之前应用。`CompiledOutput::render_with_master(values, master)` 根据已编译强度掩码生成同源槽位／监看；原 render 等价于 100%／未熄灯。数值调光优先，无调光时完整数值 RGB 同比缩放；运动、功能与未知属性不缩放。整数四舍五入，0% 或 blackout 使支持的强度归零；Player 和音乐时钟不变。

`ApplicationHost.output` 对接 `output_request`，请求为 snapshot 或 set（epoch、serial、percent、blackout），快照返回同字段与 uncontrolledFixtures。专用会话代次和序号拒绝旧工程／重放；成功打开／新建重置，编辑／保存／载入节目／切页不重置。不写入工程、历史或设备包。停止恢复默认值仍受总控抑制，解除熄灯恢复记忆百分比；错误草稿不能阻止操作。

静态三维仅缩放渲染强度，播放三维使用已缩放属性而不重复衰减。全局界面明确“预演”，没有真实设备输出或安全急停的含义。

## 步骤成组整理

[ADR-069](../development/decisions/PRODUCT-ADR-069-sequence-step-groups.md) 增加 `editSteps` 命令：`id` 为列表、`stepIds` 为非空唯一的步骤身份，`operation` 为 `copy`／`move`（`beforeId` 为目标 UUID，null 表示末尾）或 `remove`。Rust 按原执行顺序处理，最多 1024 步；跨列表身份、失效目标、移动到组内、复制超限、删空列表均原子拒绝。编号只是展示编号，移动不重新编号，复制分配新 UUID 和未占用整数编号。

复制保留场景引用、时间与剧本，未展开继承后的最终灯光；顺序或数量改变会重新解析跟踪，因此界面明确提醒重新预演。操作只改工程，一次撤销整个组，不改变已载入计划。`SequenceGroupEditor` 只持有筛选、选择、目标和确认状态，复用工程事务／资源搜索；切页保留、换列表重建、隐藏选项计入操作且明确显示数量。

[ADR-070](../development/decisions/PRODUCT-ADR-070-sequence-group-timing.md) 扩展 `operation: {kind:"timing",patch}`，其中 `delayMs`／`fadeMs`／`advance` 均可省略，省略代表保留每步原值。advance 仅为 `{kind:"manual"}` 或 `{kind:"after",waitMs}`；空 patch、未知字段与不合法整数／超限时间拒绝。手动形式不能夹带 waitMs。每项最多 86400000 毫秒；所有所选步骤一次原子提交与历史。UI 统一时间表单只生成选中字段，冻结步骤身份，通过既有 collect／accept 参加保存和上下文切换，不在 TS 推演播放时序。

效果草稿另提供 beginEffectDraft／updateEffectDraft／endEffectDraft，loaded.draftEffectId 标识临时内容；与普通载入共用同一 Player 和 epoch，详见[效果草稿契约](lighting-effects.md#效果草稿即时预演effect-004)。

## 场景／列表预演速率（EXEC-002）

[ADR-076](../development/decisions/PRODUCT-ADR-076-preview-rate.md) 新增 `control.command = {kind:"setRate",percent:25..400 整数}`；loaded 快照新增 `ratePercent`。沿用同一 epoch／controlSerial 与严格旧工程拒绝，不新增第二套播放会话。改变倍率不清零步骤经过时间；暂停中可调整而不会恢复，停止保留倍率，新载入默认 100%。草稿更新／结束保留逻辑进度和倍率。

纯 Rust `RateClock::new(now_ms)`、`advance(now_ms)`、`set_rate(now_ms,percent)` 在无 I/O／无分配的固定精度中映射单调时间。内部保留 1/100 毫秒余数，拒绝倒退与溢出；速率变更点以前按旧倍率、以后按新倍率。桌面 Loaded 持有一个映射并供 Player 的控制、监看、UE、草稿替换共用。整个列表的延时／渐变／自动等待／循环及效果一同改变，界面显示的“编排”时间保持原计划尺度。

UI 两个预演面复用 PreviewRateControls，快捷 50／100／200%、精确输入和取消只操作运行状态；setRate 不自动应用工程草稿，不能被误当效果参数。音乐仍由音频采样时钟驱动，音频与列表既有互斥不变。无工程／包格式或固件变更；多执行器、独立效果速率、节拍输入和真实输出主控仍后续。

## 批量剧本提示

[ADR-082](../development/decisions/PRODUCT-ADR-082-sequence-group-script.md) 在 `editSteps` 中增加 `operation: { kind: "script", patch: { section?, trigger?, notes? } }`。省略保留逐步原值，空字符串清除指定字段；此 patch 不接受 null、未知字段或空对象。依既有非空／唯一步骤身份选择，整条命令原子成功或拒绝；复用提示字符数和列表 64 KiB 上限，预检后写回。全空提示移除对象，能力声明自动同步。播放器和设备包不变。

桌面批量属性分别选择保留、统一填写或清空；空白混合值不自动解释成清空。时间与提示句柄统一收集为一个编辑批次，局部取消只清自己的草稿，组合待修改状态取并集；失败保持两个草稿。运行中的剧本提示仍来自已载入快照。
