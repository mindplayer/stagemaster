// Offline contract checks; semantic evaluation still belongs exclusively to Rust.
const assert = (ok, message) => {
  if (!ok) throw new Error(message);
};
export const baseAttribute = (key) =>
  /^emitter\.[^.]+\.(.+)$/.exec(key)?.[1] ?? key;
export const isEmitterFunction = (key) =>
  key.startsWith("emitter.") &&
  ["shutter", "color-wheel", "gobo-wheel"].includes(baseAttribute(key));
export function auditEmitterFunction(project, attribute, channel) {
  if (!isEmitterFunction(attribute.key)) return;
  assert(
    project.requires.some(
      (r) => r.key === "lighting.fixture-emitter-functions" && r.version === 1,
    ),
    "独立光源功能缺少能力声明",
  );
  assert(
    attribute.valueType.kind === "function" &&
      attribute.mix === "ltp" &&
      channel?.functions?.length > 0,
    "独立光源功能需要明确功能值、LTP和区间定义",
  );
  const base = baseAttribute(attribute.key);
  for (const f of channel.functions) {
    const slot = f.mode === "slot";
    const allowed =
      base === "shutter"
        ? (["open", "closed"].includes(f.key) && slot) ||
          (f.key === "strobe" && !slot)
        : ((f.key === "open" ||
            (f.key.startsWith("slot-") && f.key.length > 5)) &&
            slot) ||
          (base === "gobo-wheel" &&
            f.key.startsWith("shake-") &&
            f.key.length > 6 &&
            !slot);
    assert(
      allowed,
      "独立光源仅允许明确受控功能；声控、自走、复位及未知宏已屏蔽",
    );
  }
}
