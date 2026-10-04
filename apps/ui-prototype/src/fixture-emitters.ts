import type { ProfileDraft } from "./fixture-tools";
import { nextChannel } from "./fixture-function-draft.ts";
import { FixtureFieldError } from "./fixture-field-error.ts";

export const emitterFamilies = [
  { id: "dimmer", name: "独立调光", keys: ["dimmer"] },
  { id: "rgb", name: "RGB 三原色", keys: ["red", "green", "blue"] },
  { id: "rgbw", name: "RGBW 四色", keys: ["red", "green", "blue", "white"] },
  {
    id: "rgbd",
    name: "独立调光与 RGB",
    keys: ["dimmer", "red", "green", "blue"],
  },
  {
    id: "rgbwd",
    name: "独立调光与 RGBW",
    keys: ["dimmer", "red", "green", "blue", "white"],
  },
] as const;
export function splitEmitterAttribute(key: string) {
  const match = /^emitter\.([a-z][a-z0-9-]{0,31})\.([a-z-]+)$/.exec(key);
  return match ? { owner: match[1], attribute: match[2] } : null;
}
export function attributeBase(key: string) {
  return splitEmitterAttribute(key)?.attribute ?? key;
}
export function emitterFamily(draft: ProfileDraft, owner: string) {
  const keys = draft.channels
    .filter((c) => splitEmitterAttribute(c.attribute)?.owner === owner)
    .map((c) => attributeBase(c.attribute))
    .sort()
    .join(",");
  return (
    emitterFamilies.find((f) => [...f.keys].sort().join(",") === keys)?.id ?? ""
  );
}
export function validateEmitters(draft: ProfileDraft) {
  const units = draft.emitters ?? [];
  const scoped = draft.channels.filter((c) =>
    c.attribute.startsWith("emitter."),
  );
  if (!draft.emitters && !scoped.length) return;
  if (!units.length || units.length > 32)
    throw new FixtureFieldError("emitter-add", "独立光源需要 1–32 个单元");
  const keys = new Set<string>();
  units.forEach((unit, i) => {
    if (
      !/^[a-z][a-z0-9]*(-[a-z0-9]+)*$/.test(unit.key) ||
      unit.key.length > 32 ||
      keys.has(unit.key)
    )
      throw new FixtureFieldError(
        `emitter-${i}-key`,
        "光源标识须唯一，以小写字母开头，只含字母、数字或连字符，最多 32 位",
      );
    keys.add(unit.key);
    if (!unit.name.trim() || [...unit.name].length > 64)
      throw new FixtureFieldError(
        `emitter-${i}-name`,
        "光源名称需要 1–64 个字符",
      );
    if (!emitterFamily(draft, unit.key))
      throw new FixtureFieldError(
        `emitter-${i}-family`,
        "每个光源需要完整的调光、RGB 或 RGBW 组合",
      );
  });
  for (const c of scoped) {
    const split = splitEmitterAttribute(c.attribute);
    if (!split || !keys.has(split.owner) || c.functions)
      throw new FixtureFieldError(
        "emitter-add",
        "光源归属无效；不支持单元功能、声控、自走或复位宏",
      );
  }
  if (
    draft.channels.some((c) =>
      ["red", "green", "blue", "white"].includes(c.attribute),
    )
  )
    throw new FixtureFieldError(
      "family",
      "使用独立光源时，根级只保留可选总调光，RGBW 请归入光源",
    );
}
export function withEmitterFamily(
  draft: ProfileDraft,
  owner: string,
  family: string,
) {
  const bases = emitterFamilies.find((f) => f.id === family)?.keys;
  if (!bases || !draft.emitters?.some((e) => e.key === owner)) return draft;
  const keys = bases.map((key) => `emitter.${owner}.${key}`);
  const channels = draft.channels.filter(
    (c) =>
      splitEmitterAttribute(c.attribute)?.owner !== owner ||
      keys.includes(c.attribute),
  );
  for (const attribute of keys)
    if (!channels.some((c) => c.attribute === attribute)) {
      const coarse = nextChannel({ ...draft, channels });
      channels.push({
        attribute,
        coarse: String(coarse),
        fine: "",
        bits: "8",
        percent: "",
      });
    }
  return {
    ...draft,
    channels,
    footprint: String(
      Math.max(
        Number(draft.footprint) || 0,
        nextChannel({ ...draft, channels }) - 1,
      ),
    ),
  };
}
export function addEmitter(draft: ProfileDraft) {
  if (
    (draft.emitters?.length ?? 0) >= 32 ||
    draft.channels.some((c) => ["red", "green", "blue"].includes(c.attribute))
  )
    return draft;
  let n = 1;
  while (draft.emitters?.some((e) => e.key === `unit${n}`)) n++;
  const key = `unit${n}`;
  return withEmitterFamily(
    {
      ...draft,
      emitters: [...(draft.emitters ?? []), { key, name: `光源 ${n}` }],
    },
    key,
    "dimmer",
  );
}
export function removeEmitter(draft: ProfileDraft, owner: string) {
  const units = draft.emitters?.filter((e) => e.key !== owner) ?? [];
  const { emitters: _, ...rest } = draft;
  return {
    ...rest,
    ...(units.length ? { emitters: units } : {}),
    channels: draft.channels.filter(
      (c) => splitEmitterAttribute(c.attribute)?.owner !== owner,
    ),
  };
}
