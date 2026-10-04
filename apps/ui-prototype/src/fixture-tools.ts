import {
  FixtureFieldError,
  fixtureInteger as integer,
} from "./fixture-field-error.ts";
export { FixtureFieldError } from "./fixture-field-error.ts";
export { withMotion } from "./profile-motion.ts";
import { opticsLabels } from "./fixture-optics.ts";
import { splitEmitterAttribute, validateEmitters } from "./fixture-emitters.ts";
import { attributeBase, isEmitterFunction } from "./fixture-emitter-keys.ts";
import {
  axisSpeedKey,
  axisSpeedLabel,
  validateAxisSpeedDraft,
} from "./fixture-axis-speed.ts";
import {
  functionLabels,
  sameFunctionMapping,
} from "./fixture-function-types.ts";
import {
  functionsDraft,
  functionDefinition,
} from "./fixture-function-draft.ts";
import type { ChannelDraft } from "./fixture-function-draft";
import type { FixtureView, ProjectView } from "./application-host";
import type { ProfileDefinition, Repatch } from "./fixture-types";
export const channelLabels: Record<string, string> = {
  ...functionLabels,
  ...opticsLabels,
  dimmer: "亮度",
  pan: "水平轴",
  tilt: "垂直轴",
  [axisSpeedKey]: axisSpeedLabel,
  red: "红色",
  green: "绿色",
  blue: "蓝色",
  white: "白光",
};
export function profileChannelLabel(draft: ProfileDraft, key: string) {
  const split = splitEmitterAttribute(key);
  if (split)
    return `${draft.emitters?.find((e) => e.key === split.owner)?.name || "光源"} · ${channelLabels[split.attribute] ?? "未支持属性"}`;
  return key === "dimmer" && draft.emitters?.length
    ? "总亮度"
    : channelLabels[key];
}
export type ProfileDraft = Omit<ProfileDefinition, "footprint" | "channels"> & {
  footprint: string;
  channels: ChannelDraft[];
};
export function profileDraft(profile?: ProfileDefinition): ProfileDraft {
  const p: ProfileDefinition = profile ?? {
    name: "新灯具模式",
    manufacturer: "自定义",
    model: "新灯具",
    mode: "调光与 RGB",
    footprint: 4,
    channels: ["dimmer", "red", "green", "blue"].map((attribute, i) => ({
      attribute,
      coarse: i + 1,
      fine: null,
      defaultValue: 0,
    })),
  };
  return {
    ...(p.emitters ? { emitters: structuredClone(p.emitters) } : {}),
    ...(p.positioning ? { positioning: structuredClone(p.positioning) } : {}),
    name: p.name,
    manufacturer: p.manufacturer,
    model: p.model,
    mode: p.mode,
    footprint: String(p.footprint),
    channels: p.channels.map((c) => ({
      attribute: c.attribute,
      coarse: String(c.coarse),
      fine: c.fine === null ? "" : String(c.fine),
      bits: c.fine === null ? "8" : "16",
      percent:
        typeof c.defaultValue === "number"
          ? String(Number(((c.defaultValue * 100) / 65535).toFixed(6)))
          : "0",
      ...(c.functions
        ? {
            functions: functionsDraft(c.functions),
            defaultFunction:
              typeof c.defaultValue === "object"
                ? {
                    functionKey: c.defaultValue.functionKey,
                    position: String(c.defaultValue.position),
                  }
                : undefined,
          }
        : {}),
    })),
  };
}
export function profileDefinition(draft: ProfileDraft): ProfileDefinition {
  const meta = {} as Pick<
    ProfileDefinition,
    "name" | "manufacturer" | "model" | "mode"
  >;
  for (const [key, label] of [
    ["name", "模式名称"],
    ["manufacturer", "厂家"],
    ["model", "型号"],
    ["mode", "模式标识"],
  ] as const) {
    const value = draft[key].trim();
    if (!value || [...value].length > 256)
      throw new FixtureFieldError(key, `${label}需要填写 1–256 个字符`);
    meta[key] = value;
  }
  const footprint = integer(draft.footprint, 1, 512, "footprint", "模式占用");
  validateEmitters(draft);
  const keys = draft.channels
    .map((c) => c.attribute)
    .filter(
      (k) =>
        !k.startsWith("emitter.") &&
        !(k in functionLabels) &&
        !Object.hasOwn(opticsLabels, k) &&
        k !== axisSpeedKey,
    )
    .filter((k) => k !== "pan" && k !== "tilt")
    .sort()
    .join(",");
  if (
    !(keys === "" && draft.emitters?.length) &&
    !["dimmer", "blue,green,red", "blue,dimmer,green,red"].includes(keys)
  )
    throw new FixtureFieldError("family", "请选择调光、RGB 或调光加 RGB");
  validateAxisSpeedDraft(draft);
  if (
    draft.positioning ||
    draft.channels.some((c) => c.attribute === "pan" || c.attribute === "tilt")
  ) {
    if (
      draft.channels.filter((c) => c.attribute === "pan").length !== 1 ||
      draft.channels.filter((c) => c.attribute === "tilt").length !== 1
    )
      throw new FixtureFieldError("family", "两轴映射必须包含水平和垂直通道");
  }
  if (draft.positioning) {
    for (const axis of ["pan", "tilt"] as const) {
      const a = draft.positioning[axis];
      for (const key of ["minDegrees", "maxDegrees"] as const) {
        if (
          !/^-?(0|[1-9]\d*)(\.\d+)?$/.test(a[key]) ||
          !Number.isFinite(Number(a[key])) ||
          Math.abs(Number(a[key])) > 3600
        )
          throw new FixtureFieldError(
            `${axis}-${key}`,
            `${channelLabels[axis]}角度须为 -3600 至 3600 度`,
          );
      }
      if (Number(a.minDegrees) >= Number(a.maxDegrees))
        throw new FixtureFieldError(
          `${axis}-maxDegrees`,
          `${channelLabels[axis]}最大角度必须大于最小角度`,
        );
    }
  }
  if (
    new Set(draft.channels.map((c) => c.attribute)).size !==
    draft.channels.length
  )
    throw new FixtureFieldError("family", "同一种属性只能定义一次");
  const occupied = new Set<number>();
  const channels = draft.channels.map((c, i) => {
    const slot = (raw: string, part: "coarse" | "fine") => {
      const field = `channel-${i}-${part}`,
        label = `${profileChannelLabel(draft, c.attribute)}${part === "coarse" ? "粗调" : "细调"}通道`;
      const n = integer(raw, 1, footprint, field, label);
      if (occupied.has(n))
        throw new FixtureFieldError(field, `通道 ${n} 重复，请修改${label}`);
      occupied.add(n);
      return n;
    };
    const coarse = slot(c.coarse, "coarse"),
      fine = c.bits === "16" ? slot(c.fine, "fine") : null;
    if (c.attribute in functionLabels || isEmitterFunction(c.attribute))
      return {
        attribute: c.attribute,
        coarse,
        fine,
        ...functionDefinition(c, i),
      };
    if (c.functions)
      throw new FixtureFieldError(
        `channel-${i}-coarse`,
        "此属性不支持功能区间",
      );
    if (!/^\d+(\.\d+)?$/.test(c.percent.trim()) || Number(c.percent) > 100)
      throw new FixtureFieldError(
        `channel-${i}-percent`,
        `${profileChannelLabel(draft, c.attribute)}默认值应为 0–100%`,
      );
    return {
      attribute: c.attribute,
      coarse,
      fine,
      defaultValue: Math.round((Number(c.percent) * 65535) / 100),
    };
  });
  return {
    ...meta,
    footprint,
    channels,
    ...(draft.emitters ? { emitters: structuredClone(draft.emitters) } : {}),
    ...(draft.positioning
      ? { positioning: structuredClone(draft.positioning) }
      : {}),
  };
}
export function profileMatches(p: ProfileDefinition, query: string) {
  return `${p.name} ${p.manufacturer} ${p.model} ${p.mode} ${p.footprint}`
    .toLocaleLowerCase()
    .includes(query.trim().toLocaleLowerCase());
}
export function compatibleProfile(
  fixtures: FixtureView[],
  p: ProfileDefinition,
  allowColorSlotRemap = false,
) {
  const keys = p.channels
    .map((c) => c.attribute)
    .sort()
    .join(",");
  return fixtures.every(
    (f) =>
      f.attributes.every((a) =>
        sameFunctionMapping(
          a.function?.functions,
          p.channels.find((c) => c.attribute === a.key)?.functions,
          allowColorSlotRemap && attributeBase(a.key) === "color-wheel",
        ),
      ) &&
      Boolean(f.positioning) === Boolean(p.positioning) &&
      (!f.positioning ||
        (f.positioning.kind === p.positioning!.kind &&
          ["pan", "tilt"].every((key) => {
            const a = f.positioning![key as "pan" | "tilt"],
              b = p.positioning![key as "pan" | "tilt"];
            return (
              a.minDegrees === b.minDegrees &&
              a.maxDegrees === b.maxDegrees &&
              a.reversed === b.reversed
            );
          }))) &&
      f.attributes
        .map((a) => a.key)
        .sort()
        .join(",") === keys,
  );
}
export interface PatchRow {
  fixture: FixtureView;
  universe: number;
  address: number;
  end: number;
}
export function patchPlan(
  project: ProjectView,
  ids: string[],
  layout: Repatch,
  profile?: ProfileDefinition,
): PatchRow[] {
  if (!ids.length || ids.length > 256 || new Set(ids).size !== ids.length)
    throw new FixtureFieldError("selection", "请选择 1–256 台不重复的灯具");
  for (const [key, min, max, label] of [
    ["universe", 1, 65535, "线路"],
    ["address", 1, 512, "起始地址"],
    ["gap", 0, 511, "灯间空余通道"],
  ] as const)
    integer(String(layout[key]), min, max, key, label);
  let address = layout.address;
  const rows = ids.map((id) => {
    const fixture = project.fixtures.find((f) => f.id === id);
    if (!fixture) throw new FixtureFieldError("selection", "所选灯具已不存在");
    const end = address + (profile?.footprint ?? fixture.footprint) - 1;
    if (end > 512)
      throw new FixtureFieldError(
        "address",
        `“${fixture.name}”的地址 ${address}–${end} 超出 512 通道`,
      );
    const row = { fixture, universe: layout.universe, address, end };
    address = end + 1 + layout.gap;
    return row;
  });
  for (const row of rows) {
    const conflict = project.fixtures.find(
      (f) =>
        !ids.includes(f.id) &&
        f.domainId === row.fixture.domainId &&
        f.universe === row.universe &&
        f.address !== null &&
        f.address <= row.end &&
        f.address + f.footprint - 1 >= row.address,
    );
    if (conflict)
      throw new FixtureFieldError(
        "address",
        `“${row.fixture.name}”的 ${row.address}–${row.end} 与“${conflict.name}”的 ${conflict.address}–${conflict.address! + conflict.footprint - 1} 重叠`,
      );
  }
  return rows;
}
export function availablePatch(
  project: ProjectView,
  ids: string[],
  universe: number,
  gap: number,
  profile?: ProfileDefinition,
): number | null {
  for (let address = 1; address <= 512; address++) {
    try {
      patchPlan(project, ids, { universe, address, gap }, profile);
      return address;
    } catch {
      /* Suggestion only: the Rust transaction remains authoritative. */
    }
  }
  return null;
}
