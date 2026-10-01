import { FixtureFieldError, fixtureInteger } from "./fixture-field-error.ts";
import type { ChannelDraft, FunctionDraft } from "./fixture-function-draft";
export interface SlotBatchDraft {
  start: string;
  width: string;
  count: string;
  name: string;
}
const untouched = (channel: ChannelDraft) => {
  const f = channel.functions?.[0];
  return (
    channel.functions?.length === 1 &&
    f?.name === "" &&
    f.mode === "slot" &&
    f.dmxFrom === "0" &&
    f.dmxTo === "0" &&
    f.dmxDefault === "0" &&
    !f.appearance &&
    channel.defaultFunction?.functionKey === f.key &&
    channel.defaultFunction.position === "0"
  );
};
export function planSlotBatch(
  channel: ChannelDraft,
  draft: SlotBatchDraft,
  prefix: string,
) {
  const max = channel.bits === "16" ? 65535 : 255;
  const start = fixtureInteger(
    draft.start,
    0,
    max,
    `${prefix}-batch-start`,
    "起始值",
  );
  const width = fixtureInteger(
    draft.width,
    1,
    max + 1,
    `${prefix}-batch-width`,
    "每档宽度",
  );
  const replaceEmpty = untouched(channel);
  const existing = replaceEmpty ? [] : (channel.functions ?? []);
  const count = fixtureInteger(
    draft.count,
    1,
    64,
    `${prefix}-batch-count`,
    "档位数量",
  );
  if (count + existing.length > 64)
    throw new FixtureFieldError(
      `${prefix}-batch-count`,
      `当前已有 ${existing.length} 个区间，总数不能超过 64`,
    );
  if (start + width * count - 1 > max)
    throw new FixtureFieldError(
      `${prefix}-batch-count`,
      `生成区间超出 0–${max}，请减少档位数量或宽度`,
    );
  const name = draft.name.trim();
  if (!name || [...name].length > 240 || /[\u0000-\u001f\u007f]/.test(name))
    throw new FixtureFieldError(
      `${prefix}-batch-name`,
      "请填写 1–240 个有效字符作为名称前缀",
    );
  for (const f of existing) {
    if (
      !/^\d+$/.test(f.dmxFrom) ||
      !/^\d+$/.test(f.dmxTo) ||
      Number(f.dmxFrom) > Number(f.dmxTo)
    )
      throw new FixtureFieldError(
        `${prefix}-batch-start`,
        "请先补齐已有区间的起止值",
      );
    if (
      Number(f.dmxFrom) <= start + width * count - 1 &&
      Number(f.dmxTo) >= start
    )
      throw new FixtureFieldError(
        `${prefix}-batch-start`,
        `生成区间与“${f.name || "未命名功能"}”重叠，请调整起点或先修改原区间`,
      );
  }
  const slots = Array.from({ length: count }, (_, i) => {
    const from = start + width * i,
      to = from + width - 1;
    return {
      name: `${name} ${i + 1}`,
      from,
      to,
      representative: Math.floor((from + to) / 2),
    };
  });
  return { slots, replaceEmpty };
}
export function addSlotBatch(
  channel: ChannelDraft,
  draft: SlotBatchDraft,
  prefix: string,
): ChannelDraft {
  const { slots, replaceEmpty } = planSlotBatch(channel, draft, prefix);
  const generated: FunctionDraft[] = slots.map((s, i) => ({
    key:
      replaceEmpty && i === 0
        ? channel.functions![0].key
        : `function-${crypto.randomUUID()}`,
    name: s.name,
    mode: "slot",
    dmxFrom: String(s.from),
    dmxTo: String(s.to),
    dmxDefault: String(s.representative),
  }));
  const existing = replaceEmpty ? [] : (channel.functions ?? []);
  return {
    ...channel,
    functions: [...existing, ...generated],
    defaultFunction: existing.length
      ? channel.defaultFunction
      : { functionKey: generated[0].key, position: "0" },
  };
}
