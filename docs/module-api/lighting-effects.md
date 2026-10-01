# 已实现：场景动态灯光效果

EFFECT-001／002／003 / [ADR-021](../development/decisions/PRODUCT-ADR-021-lighting-effects.md)、[ADR-022](../development/decisions/PRODUCT-ADR-022-effect-keyframes.md)，2026-09-27。本文是当前产品契约，区别于主目录中的远程服务伪接口。

## 调用与状态归属

`EffectRack` / `EffectEditor` → `ApplicationHost.request(edit)` → `Document::edit(Effect)` → Schema／语义校验 → 工程历史和保存。效果属于场景，灯序冻结成 UUID 数组；编辑、复制、启停、删除都是原子事务。UI 的模板工厂只生成参数，不求值动态曲线。`compile_scene` 和 `compile_sequence` 共用编译器，生成 `Plan::with_effects`，`Player` 按单调毫秒求值。

```rust,ignore
let compiled = document.compile_scene(scene_id)?;
let mut player = stagemaster_playback::Player::new(compiled.plan, 0);
player.execute(0, 0)?;
player.advance(250)?;
let frame = compiled.output.render(player.values())?;
```

TS 编辑契约在 `apps/ui-prototype/src/effect-types.ts`，Rust 对应 `crates/stagemaster-project/src/effects.rs`。`EffectEdit::Put` 用稳定 ID 新建或替换，`Remove` 删除；批次可与静态亮度修改合并。复制效果用新 ID，复制场景由 Rust 递归更新效果 ID。灯具不存在、属性不兼容、启用效果重叠或超限时，整个事务失败。

## 存储模型

`lighting.scenes[].effects` 可省略；有内容必须声明 `requires: lighting.effects.basic@1`。

| 字段 | 含义／范围 |
| --- | --- |
| id、name、enabled | UUID、非空名称、启用状态；停用仍保留引用和参数 |
| fixtureIds | 1–512 个互不重复的灯具 UUID，数组顺序就是效果灯序 |
| channels | 1–4 个互不重复的属性；基本曲线用 low／high，关键帧用 keyframes，互斥；基本／关键帧仅 dimmer／red／green／blue，位置模式仅 pan／tilt |
| periodMs | 一个完整往返／脉冲周期，整数 100–3600000 毫秒 |
| waveform | smooth：平滑往返；triangle：线性往返；pulse：亮段与低段切换；keyframes：逐帧循环；position：相对双轴运动 |
| spreadDegrees | 0–360 整数；按 i/N 展开，360 度时首尾不重合 |
| phaseDegrees | 0–359 整数，整体起始滞后 |
| reverse | 反转灯序的相位分布，安装位置不变 |
| dutyPercent | 脉冲高值段占周期的 1–99%；其余曲线保留此值但不使用 |

平滑／线性在相位 0 取 low、半周期取 high、整周期回到 low。脉冲在相位 0 取 high，到亮段边界切到 low。定点 Q16 求值，平滑为 smoothstep 而非正弦。RGB 按通道插值；颜色数值范围可反向。单个场景最多 32 个效果，启用效果只能在不同灯具属性上组合。

### 关键帧扩展

`waveform: "keyframes"` 额外声明 `lighting.effects.keyframes@1`；基本能力声明继续保留。每个 channel 的 `keyframes` 有 2–32 个 `{position, value, transition}`：position 是 0–9999 的周期万分比，首帧必须为 0，严格递增；value 为 u16；transition 为 hold／linear／smooth，决定本帧到下一帧如何变化。末帧连接下一轮首帧；同一效果的各属性位置和过渡方式必须一致。未知字段、混入 low／high、方式错配、点序错误全部拒绝。

编译将位置向上量化到 Q16，不早于指定相位，量化误差小于一相位单位。该规则让原脉冲转换为保持帧时保留比较边界；2018 个毫秒采样、33% 亮段及四灯相位对照通过。旧静态／三种两端曲线不重写或隐式迁移。

`KeyframeEditor` 独立维护帧草稿，上方选帧，下方编辑当前帧；切帧不丢草稿。支持位置／颜色／亮度、增删、重排与均分时间；重排交换帧内容，时间槽不变；删首帧时剩余首帧移到 0%。错误定位到对应帧及字段，整组修改在应用时形成一次历史。半速／倍速只调整周期草稿；新结果需重载预览，不是现场连续变速主控。

复用入口在效果组件中搜索当前工程所有场景，复制独立曲线和新 UUID，默认停用，可替换为当前所选灯具顺序；兼容检查与手工创建共用。它不增加新持久资源种类或隐式引用，修改副本不影响来源。

## 运行与预览

效果从步骤延时结束开始，在进入渐变中混合动态目标；上一可见值在延时期间冻结。每次执行或自动推进重置本步骤效果相位，效果不跨步骤继承，静态继承保持原契约。暂停／继续保留相位；停止回档案默认值；列表结束冻结精确边界帧。

桌面增加 `preview({kind:"loadScene", generation, sceneId})`，返回 `loaded.sceneId`；列表来源时为 null。两种来源共用唯一预览实例。单场景的 `stepId` 等于场景 ID。根快照增加 `controlSerial`；客户端发送当前代次和新序号，换来源后的旧命令必须失败。载入只准备，`execute` 才开始；编辑使旧内容过期，需重载。打开文件不会自动播放。

三维选择“跟随播放预览”消费同一 Rust 输出，编排页“三维监看”可直接切入；直接选择有动态效果的场景时，入口明确标为“静态值”。渲染器不保存效果、拥有时钟或输出硬件命令。

`Plan` 全部效果属性累计最多 16384，单步骤最多 512；关键帧累计最多 131072 点，编译时逐步核算并提前拒绝。`effect_buffer_bytes()` / 桌面 `effectBufferBytes` 统计效果结构（含内联曲线信息）与关键帧数组，不包含外层 Vec 头及分配器额外容量。逐帧求值不分配。设备包经既有独立 package 编码，不能直接写入 Rust 内存布局。相对位置先生成同一关键帧计划，不增加设备端运动求解器。

验收与限制见[基础效果](../development/tasks/EFFECT-001-basic-effects.md)和[关键帧与复用](../development/tasks/EFFECT-002-keyframes-reuse.md)工单。EFFECT-003 的相对位置如下；其他属性的相对效果、效果跨步骤跟踪、节拍主控、随机、世界轨迹及像素仍非隐含支持项。


## 相对位置扩展（EFFECT-003）

见 [ADR-058](../development/decisions/PRODUCT-ADR-058-relative-position-effects.md)。`waveform: "position"` 额外要求 `lighting.effects.position@1`；每轴 `{attribute: "pan" | "tilt", amplitudeDegrees: Decimal, offsetDegrees: Decimal, phaseDegrees: 0..359}`。幅度 0..3600°、偏移 ±3600°，相位正值沿用滞后语义；不能混用范围或作者关键帧。具备有效两轴模型的灯具才能使用，轴可独立启用。

中心取该步骤静态目标解码角度加偏移；静态目标已解析字面值、预设、隔离／跟踪与释放。连续中心 ± 幅度必须处于机械范围，编译失败定位场景／效果／灯具／轴，不裁剪。编译生成每轴 32 点等距正弦的线性循环曲线，轴相位加到既有整体／灯序相位；沿用单调时钟、渐变、暂停和执行包。静态跟踪不继承动态终帧。

`PositionEffectEditor` 只维护角度参数和有序灯具，复用效果草稿原子应用／撤销与唯一预演。三种模板：水平摆动、垂直摆动、双轴圆形；双轴圆形两轴等幅且滞后相差 90°，属于轴角度空间，不保证舞台投影为圆。不自动改变亮度或静态位置；幅度 0 仍占用该轴且输出中心，取消该轴或停用效果才释放占用。结构和兼容性在事务中校验，依赖执行上下文的机械行程在编译／工程检查时校验。


## 单场景效果时间偏移（AUDIO-010）

`Plan::with_effect_time_offset(offset_ms)` 消费并返回不可变单场景计划；只允许单步、手动保持、无循环、无延时，偏移不超过 MAX_TIME_MS。`effect_time_offset_ms()` 只读查询。Player 的每条效果先按各自周期对 elapsed 与 offset 取模再合成，保留不同周期与 16 位相位精度；渐变／离散属性仍按局部时钟，零值保持旧行为。设备包编码器对非零偏移明确拒绝，现有包字节与执行语义不升级，元数据预算另有尺寸保护。用途与格式边界见 [ADR-073](../development/decisions/PRODUCT-ADR-073-audio-clip-effect-offset.md)。

## 效果草稿即时预演（EFFECT-004）

按 [ADR-074](../development/decisions/PRODUCT-ADR-074-effect-draft-preview.md)，桌面唯一预演增加 `beginEffectDraft {generation,epoch,sceneId,effect,illuminate}`、`updateEffectDraft {generation,epoch,serial,effect,illuminate}`、`endEffectDraft {epoch}`。begin 显式替换离线播放并停止音乐，update 只作用于同一效果身份，end 恢复进入前已应用的该场景参数，不恢复前一音乐／列表。加载快照用可选 `draftEffectId` 标识临时草稿。

Rust 在短锁内提取 Document 克隆与归属，锁外完成正常编辑校验及单场景编译，重新验证代次／内容版本／预演 epoch／草稿 serial 后安装；无效或过时候选无副作用。更新／恢复保持累计效果时间和暂停状态，停止后更新不会重新启动；参数变化按同一累计时间重算，不承诺周期变化时相位连续。

界面显式“即时预演”，250 ms 合并输入，一个更新在途加一个最新候选；错误保留最近有效参数，不抢焦点。关键帧、普通曲线与双轴共用调度，正式应用／撤销／保存仍走原工程事务。内容变更清除临时预演，退出编辑器按所属 epoch 释放；旧回执不能夺回新播放。预演草稿不写工程、恢复文件或设备包，只有应用形成历史。与现场编程器、总速度主控、多执行器无混同。

## 共同空间目标的直线往返（EFFECT-007）

按 [ADR-084](../development/decisions/PRODUCT-ADR-084-world-line-effects.md)，`waveform:"worldLine"` 同时声明 `lighting.effects.world-line@1`，两条通道只有 `attribute:"pan"`／`"tilt"`。`targetPath` 为 `{kind:"line",fromMeters:SpatialVector3,toMeters:SpatialVector3,branch:"auto"|"front"|"back",maxErrorMeters:Decimal}`；其他效果不得带该字段。点在世界坐标中，±100000 米，允许误差 0.001–1 米。两轴作为同一目标占用，不能分开修改轴相位。可与其他属性效果并行，同轴竞争仍拒绝。

静态步骤值只用于确定全路径可行的起始解族／圈数。整条线段解析检查行程及奇点，路径途中不切解；世界端点不随静态轴值变化。目标进度按余弦缓入缓出往返，32 个等距相位关键帧计入已有预算。编译按区间导数包络叠加量化和 Q16 相位误差，计算目标距离处的光线偏差上界；超过允许值拒绝并返回场景、效果和灯具。它是保守的数值误差界，可能拒绝实际误差较小的路径，不是实灯命中精度。8 位高字节输出实际精度被计入，不假装 16 位。

`WorldLineEffectEditor` 独立维护起终点、交换、允许误差和解分支，复用周期／节拍、灯序、草稿与预演。单灯安装改变后重编译，旧计划不可变；世界几何不下放 ESP32，播放包仍仅含已有曲线。线性关节节点、进入／退出轨迹的过渡、机械速度／加速度、碰撞、观众避让和真实输出不由该效果自动保证。
