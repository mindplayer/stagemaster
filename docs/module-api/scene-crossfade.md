# 双动态场景交叉求值

AUDIO-019／[ADR-095](../development/decisions/PRODUCT-ADR-095-dynamic-crossfade-sampling.md)。Rust 核心、工程格式与宿主／界面已接入，完整回归、原生历史／剪辑／UE 播放与保存重开验收通过，见 AUDIO-019 工单。

```rust
use stagemaster_playback::{CrossfadeTiming, SceneCrossfade};

// 两个 Plan 由同一工程编译，属性顺序与离散映射必须相同。
let mut transition = SceneCrossfade::new(source_plan, target_plan, CrossfadeTiming {
    duration_ms: 2000,
    offset_ms: 0,
    source_elapsed_ms: 8000,
    target_elapsed_ms: 0,
})?;
let values = transition.sample(750)?;
// 旧效果运行至 8750 ms，新效果至 750 ms；连续属性按 37.5% 新值混合。
// 各 Plan 自己已有的效果偏移／进入渐变偏移仍独立生效。
```

`sample(t)` 从本可见区段起点计算，允许后退、重复或跳跃。连续值使用与 Player 相同的整数线性混合及取整；离散属性在交叉开始即采用新场景，不插值经过未选择的色盘／复位等通道区间。构造后 `values()` 等于 `sample(0)`；非法采样保留上次输出。

支持输入：两个无延时、无自动推进、非循环的单场景 Plan；1–512 属性，数量与递增离散掩码一致。该层没有灯具身份，语义属性索引完全一致必须由工程编译器确认。不能用此接口将两路独立 DMX 数组直接合并。

duration 为 1–86400000 ms；三个起点及加 t 后的位置都不超过 86400000 ms，checked 运算拒绝溢出。构造时通过可失败预分配创建两个固定 u16 输出缓冲，每个最多 1024 字节；两个 Plan 自身的默认值、目标和效果数据另计，受原 Plan 上限约束。成功采样不分配、无文件／网络／设备操作，也没有新的时钟。

剪辑时需同时平移 source_elapsed、target_elapsed 和 offset，保留 duration，才能完整保留原交叉采样。工程分割和内部截取通过下述历史来源记录实现这一保证。该类型不能直接交给现有单 Plan 设备编码器；目标适配完成前不得通过丢弃旧动态效果来导出。


## 工程、宿主与编辑

独立片段新增 `fadeMode?: "snapshot" | "dynamic"`，默认 snapshot，旧文件不变；dynamic 要求 `media.audio-clip-crossfade@1`。普通动态片段以相邻启用前段为来源，没有来源使用灯具默认值。`fadeMs=0` 直接切换。

```json
{
  "fadeMode": "dynamic",
  "fadeMs": 750,
  "effectOffsetMs": 250,
  "entryCrossfade": {
    "durationMs": 1000,
    "offsetMs": 250,
    "source": {
      "sceneId": "b0000000-0000-4000-8000-000000000001",
      "effectOffsetMs": 0,
      "elapsedMs": 4250
    }
  }
}
```

上例是原转场已进行 250 ms 的切片；目标效果从 250 ms 开始，来源从 4250 ms 开始，剩余过渡 750 ms。外层片段身份、范围、目标场景等字段沿用 audio-editing 契约，此片段展示并非完整工程。source.sceneId 可为 null 表示默认值；source 可含独立 `entryFade`，结构同既有连续属性快照；禁止嵌套交叉。两种历史互斥，引用必须存在，即使片段停用也不能删除历史来源场景。

`Document::compile_audio_segment(id)` 返回 `CompiledAudioSegment {playback, output}`；`playback.into_player()` 构建宿主采样器，`advance(elapsed)`、`values()` 供唯一音频游标调用；向后定位重建采样器。旧 `compile_audio_lighting` 遇到动态交叉明确拒绝，禁止丢弃来源退化为单 Plan。映射由编译器按灯具身份、属性键、索引、离散位逐项核对。

新增／单片段编辑支持 fadeMode；`editLightingClips` 的 fade 动作接受可选 fadeMode，省略保持各自模式。修改模式、场景、时长或显式重新计算会清除历史，统一编辑有任一锁定／超限／三源冲突则整体拒绝。分割、裁切、复制沿用现有事务与历史，来源时间仅由核心维护；普通 put 不允许伪造 entryCrossfade。

来源时间和历史交叉偏移加片段长都受一小时限制；历史静态快照与交叉内嵌快照合计不超过 32768 项，单个不超过 512。新交叉不能接在未完成的另一双动态转场后；同一历史的切片续接合法。上述能力仅为宿主时间线，设备节目导出仍仅场景／场景列表。
