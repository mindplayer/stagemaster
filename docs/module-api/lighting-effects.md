# 已实现：场景动态灯光效果

EFFECT-001 / [ADR-021](../development/decisions/PRODUCT-ADR-021-lighting-effects.md)，2026-09-27。本文是当前产品契约，区别于主目录中的远程服务伪接口。

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
| channels | 1–4 个互不重复的属性，两端值 low／high 均为 0–65535；仅 dimmer／red／green／blue |
| periodMs | 一个完整往返／脉冲周期，整数 100–3600000 毫秒 |
| waveform | smooth：平滑往返；triangle：线性往返；pulse：亮段与低段切换 |
| spreadDegrees | 0–360 整数；按 i/N 展开，360 度时首尾不重合 |
| phaseDegrees | 0–359 整数，整体起始滞后 |
| reverse | 反转灯序的相位分布，安装位置不变 |
| dutyPercent | 脉冲高值段占周期的 1–99%；其余曲线保留此值但不使用 |

平滑／线性在相位 0 取 low、半周期取 high、整周期回到 low。脉冲在相位 0 取 high，到亮段边界切到 low。定点 Q16 求值，平滑为 smoothstep 而非正弦。RGB 按通道插值；颜色数值范围可反向。单个场景最多 32 个效果，启用效果只能在不同灯具属性上组合。

## 运行与预览

效果从步骤延时结束开始，在进入渐变中混合动态目标；上一可见值在延时期间冻结。每次执行或自动推进重置本步骤效果相位，效果不跨步骤继承，静态继承保持原契约。暂停／继续保留相位；停止回档案默认值；列表结束冻结精确边界帧。

桌面增加 `preview({kind:"loadScene", generation, sceneId})`，返回 `loaded.sceneId`；列表来源时为 null。两种来源共用唯一预览实例。单场景的 `stepId` 等于场景 ID。根快照增加 `controlSerial`；客户端发送当前代次和新序号，换来源后的旧命令必须失败。载入只准备，`execute` 才开始；编辑使旧内容过期，需重载。打开文件不会自动播放。

三维选择“跟随播放预览”消费同一 Rust 输出，编排页“三维监看”可直接切入；直接选择有动态效果的场景时，入口明确标为“静态值”。渲染器不保存效果、拥有时钟或输出硬件命令。

`Plan` 全部效果属性累计最多 16384，单步骤最多 512；额外提供 `effect_buffer_bytes()` / 桌面 `effectBufferBytes`，表示编译效果数据的有效字节，不含 Vec 元数据／分配器。逐帧求值不分配。当前没有设备持久包协议，不可以直接把 Rust 内存布局写进 ESP32 文件。

验收与限制见[工单](../development/tasks/EFFECT-001-basic-effects.md)。相对效果、多关键帧、效果跟踪、节拍主控、随机、运动及像素不是本能力的隐含支持项。
