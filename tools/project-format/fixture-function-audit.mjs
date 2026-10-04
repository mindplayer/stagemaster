// Offline authoring-contract checks only. Runtime mapping belongs to Rust.
import { auditFixtureProgram } from "./fixture-program-audit.mjs";
import { functionSelectionAllowed } from "./fixture-function-safety.mjs";
import {
  auditEmitterFunction,
  baseAttribute,
  isEmitterFunction,
} from "./fixture-emitter-function-audit.mjs";
const assert = (ok, message) => {
  if (!ok) throw new Error(message);
};
const keys = new Set([
  "color-wheel",
  "gobo-wheel",
  "shutter",
  "prism",
  "fixture-program",
]);
export function auditFixtureFunctions(project) {
  const profiles = new Map(
    (project.lighting?.profiles ?? []).map((p) => [p.id, p]),
  );
  const fixtures = new Map(
    (project.lighting?.fixtures ?? []).map((f) => [f.id, f]),
  );
  const declared = project.requires.some(
    (c) => c.key === "lighting.fixture-functions" && c.version === 1,
  );
  function selection(profile, key, value) {
    const attribute = profile?.attributes.find((a) => a.key === key);
    if (attribute?.valueType.kind !== "function") return;
    const channel = profile.channels.find((c) => c.attribute === key);
    const f = channel?.functions?.find((f) => f.key === value.functionKey);
    assert(value.kind === "function" && f, "未知灯具功能或值类型错误");
    if (key === "fixture-program")
      assert(
        value.functionKey === "external",
        "声控和内置自走档位已屏蔽，演出仅允许外部通道控制",
      );
    assert(
      functionSelectionAllowed(key, f),
      "声控、自走、自动轮盘、复位及未知控制宏已屏蔽",
    );
    assert(f.mode !== "slot" || value.position === 0, "固定档位的位置必须为零");
  }
  for (const profile of profiles.values()) {
    for (const a of profile.attributes)
      if (a.key === "fixture-program")
        auditFixtureProgram(
          project,
          a,
          profile.channels.find((c) => c.attribute === a.key),
        );
    for (const c of profile.channels) {
      const a = profile.attributes.find((a) => a.key === c.attribute);
      if (a?.valueType.kind !== "function") {
        assert(
          !keys.has(a?.key) && !isEmitterFunction(a?.key ?? ""),
          "功能通道必须明确选择功能，旧普通百分比映射已屏蔽",
        );
        assert(!c.functions, "线性属性不能带功能区间");
        continue;
      }
      assert(
        declared &&
          (keys.has(a.key) || isEmitterFunction(a.key)) &&
          a.mix === "ltp",
        "功能属性需要支持的种类、LTP 和能力声明",
      );
      auditEmitterFunction(project, a, c);
      assert(c.functions?.length > 0, "功能通道缺少区间");
      const maximum = c.encoding === "u8" ? 255 : 65535,
        seen = new Set();
      c.functions.forEach((f, i) => {
        if (f.appearance) {
          assert(
            project.requires.some(
              (c) =>
                c.key === "lighting.fixture-wheel-appearance" &&
                c.version === 1,
            ),
            "色盘外观缺少能力声明",
          );
          assert(
            baseAttribute(c.attribute) === "color-wheel" && f.mode === "slot",
            "外观只能用于色盘固定档位",
          );
        }
        assert(!seen.has(f.key), "功能标识重复");
        seen.add(f.key);
        assert(
          f.name.trim().length > 0 && !/[\u0000-\u001f\u007f]/.test(f.name),
          "功能名称无效",
        );
        assert(
          f.dmxFrom <= f.dmxTo &&
            f.dmxTo <= maximum &&
            (f.mode !== "range" || f.dmxFrom < f.dmxTo),
          "功能区间或精度无效",
        );
        assert(
          f.dmxDefault >= f.dmxFrom && f.dmxDefault <= f.dmxTo,
          "功能代表值不在区间内",
        );
        assert(
          !c.functions
            .slice(0, i)
            .some((g) => g.dmxFrom <= f.dmxTo && f.dmxFrom <= g.dmxTo),
          "功能区间重叠",
        );
      });
    }
    for (const a of profile.attributes) selection(profile, a.key, a.default);
  }
  const validate = (target, value) =>
    selection(
      profiles.get(fixtures.get(target.fixtureId)?.profileId),
      target.attribute,
      value,
    );
  for (const preset of project.lighting?.presets ?? [])
    for (const v of preset.values) validate(v.target, v.value);
  for (const scene of project.lighting?.scenes ?? [])
    for (const a of scene.assignments)
      if (a.source?.kind === "literal") validate(a.target, a.source.value);
}
