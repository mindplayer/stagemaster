import assert from "node:assert/strict";
const unique = (values, label) =>
  assert(new Set(values).size === values.length, `重复${label}`);

export function auditEffects(scene, declared, get, attr) {
  const effects = scene.effects ?? [],
    activeTargets = [];
  if (effects.length)
    assert(declared.has("lighting.effects.basic@1"), "缺少动态效果能力声明");
  for (const effect of effects) {
    unique(
      effect.channels.map((c) => c.attribute),
      "效果属性",
    );
    const keyed = effect.waveform === "keyframes";
    const motion = effect.waveform === "position";
    const world = effect.waveform === "worldLine";
    assert(
      world === (effect.targetPath !== undefined),
      "空间轨迹与效果参数不一致",
    );
    if (world) {
      assert(
        declared.has("lighting.effects.world-line@1"),
        "缺少空间轨迹能力声明",
      );
      assert(
        effect.channels.length === 2 &&
          ["pan", "tilt"].every((axis) =>
            effect.channels.some((c) => c.attribute === axis),
          ),
        "空间轨迹必须同时控制两轴",
      );
      const path = effect.targetPath;
      for (const p of [path.fromMeters, path.toMeters])
        assert(
          Object.values(p).every((v) => Math.abs(Number(v)) <= 100000),
          "轨迹坐标越界",
        );
      assert(
        ["x", "y", "z"].reduce(
          (sum, axis) =>
            sum +
            (Number(path.fromMeters[axis]) - Number(path.toMeters[axis])) ** 2,
          0,
        ) >= 1e-12,
        "轨迹端点重合",
      );
      assert(
        Number(path.maxErrorMeters) >= 0.001 &&
          Number(path.maxErrorMeters) <= 1,
        "轨迹允许误差越界",
      );
    }
    if (motion)
      assert(
        declared.has("lighting.effects.position@1"),
        "缺少位置效果能力声明",
      );
    if (keyed)
      assert(
        declared.has("lighting.effects.keyframes@1"),
        "缺少关键帧效果能力声明",
      );
    let timing;
    for (const channel of effect.channels) {
      assert(
        world === (Object.keys(channel).length === 1),
        "空间轨迹与属性参数不一致",
      );
      assert(
        motion === (channel.amplitudeDegrees !== undefined),
        "位置效果与属性参数不一致",
      );
      if (motion) {
        assert(
          Number(channel.amplitudeDegrees) >= 0 &&
            Number(channel.amplitudeDegrees) <= 3600 &&
            Math.abs(Number(channel.offsetDegrees)) <= 3600,
          "位置效果角度超出范围",
        );
      }
      assert(
        keyed === Array.isArray(channel.keyframes),
        "变化方式与关键帧不一致",
      );
      if (keyed) {
        const frames = channel.keyframes;
        assert(
          frames[0].position === 0 &&
            frames.every(
              (f, i) => i === 0 || f.position > frames[i - 1].position,
            ),
          "关键帧须从零开始递增",
        );
        const current = JSON.stringify(
          frames.map((f) => [f.position, f.transition]),
        );
        assert(
          timing === undefined || timing === current,
          "关键帧位置与过渡方式须一致",
        );
        timing = current;
      }
    }
    for (const fixtureId of effect.fixtureIds)
      for (const channel of effect.channels) {
        if (motion || world)
          assert(
            get(get(fixtureId, "fixture").profileId, "profile").positioning,
            "位置效果需要两轴运动模型",
          );
        assert(
          attr({ fixtureId, attribute: channel.attribute }).valueType.kind ===
            "normalized",
          "效果仅支持归一化属性",
        );
        if (effect.enabled)
          activeTargets.push(`${fixtureId}/${channel.attribute}`);
      }
  }
  unique(activeTargets, "已启用效果目标");
}
