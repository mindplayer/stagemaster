import { checkedAppearance } from "./wheel-appearance.ts";
import type {
  FunctionDefinition,
  FunctionSelection,
} from "./fixture-function-types";
import type { ProfileDraft } from "./fixture-tools";
import { FixtureFieldError, fixtureInteger } from "./fixture-field-error.ts";
import { functionLabels, initialFunction } from "./fixture-function-types.ts";
import {
  programKey,
  validateProgramDefinition,
} from "./fixture-program-rules.ts";
export type FunctionDraft = Omit<
  FunctionDefinition,
  "dmxFrom" | "dmxTo" | "dmxDefault"
> & {
  dmxFrom: string;
  dmxTo: string;
  dmxDefault: string;
};
export interface ChannelDraft {
  attribute: string;
  coarse: string;
  fine: string;
  bits: "8" | "16";
  percent: string;
  functions?: FunctionDraft[];
  defaultFunction?: { functionKey: string; position: string };
}
export function functionsDraft(functions: FunctionDefinition[]) {
  return functions.map((f) => ({
    ...f,
    dmxFrom: String(f.dmxFrom),
    dmxTo: String(f.dmxTo),
    dmxDefault: String(f.dmxDefault),
  }));
}
export function functionDefinition(
  c: ChannelDraft,
  i: number,
): { functions: FunctionDefinition[]; defaultValue: FunctionSelection } {
  const prefix = `channel-${i}`,
    max = c.bits === "16" ? 65535 : 255;
  if (!c.functions?.length || c.functions.length > 64)
    throw new FixtureFieldError(
      `${prefix}-function-add`,
      "每个功能通道需要 1–64 个区间",
    );
  const keys = new Set<string>();
  const functions = c.functions.map((f, j) => {
    const field = `${prefix}-function-${j}`;
    if (
      !/^[a-z][a-z0-9]*([._-][a-z0-9]+)*$/.test(f.key) ||
      f.key.length > 128 ||
      keys.has(f.key)
    )
      throw new FixtureFieldError(
        `${field}-name`,
        "功能标识重复或无效，请删除后重新添加此功能",
      );
    keys.add(f.key);
    const name = f.name.trim();
    if (!name || [...name].length > 256 || /[\u0000-\u001f\u007f]/.test(name))
      throw new FixtureFieldError(
        `${field}-name`,
        "功能名称需要填写 1–256 个有效字符",
      );
    const dmxFrom = fixtureInteger(
      f.dmxFrom,
      0,
      max,
      `${field}-dmxFrom`,
      "区间起点",
    );
    const dmxTo = fixtureInteger(
      f.dmxTo,
      dmxFrom + (f.mode === "range" ? 1 : 0),
      max,
      `${field}-dmxTo`,
      "区间终点",
    );
    const dmxDefault = fixtureInteger(
      f.dmxDefault,
      dmxFrom,
      dmxTo,
      `${field}-dmxDefault`,
      "代表值",
    );
    const appearance = checkedAppearance(
      f.appearance,
      c.attribute,
      f.mode,
      field,
    );
    return {
      key: f.key,
      name,
      mode: f.mode,
      dmxFrom,
      dmxTo,
      dmxDefault,
      ...(appearance ? { appearance } : {}),
    };
  });
  for (let j = 0; j < functions.length; j++) {
    const f = functions[j];
    if (
      functions
        .slice(0, j)
        .some((g) => f.dmxFrom <= g.dmxTo && g.dmxFrom <= f.dmxTo)
    )
      throw new FixtureFieldError(
        `${prefix}-function-${j}-dmxFrom`,
        `“${f.name}”与前面的功能区间重叠`,
      );
  }
  const chosen = functions.find(
    (f) => f.key === c.defaultFunction?.functionKey,
  );
  if (!chosen)
    throw new FixtureFieldError(`${prefix}-default-function`, "请选择默认功能");
  const position = fixtureInteger(
    c.defaultFunction!.position,
    0,
    chosen.mode === "slot" ? 0 : 65535,
    `${prefix}-default-position`,
    "默认区间位置",
  );
  const defaultValue = { functionKey: chosen.key, position };
  if (c.attribute === programKey)
    validateProgramDefinition(functions, defaultValue, i);
  return { functions, defaultValue };
}
/** New fields start unassigned semantically: the user must name the function from their manual. */
export function newFunction(from = 0): FunctionDraft {
  return {
    key: `function-${crypto.randomUUID()}`,
    name: "",
    mode: "slot",
    dmxFrom: String(from),
    dmxTo: String(from),
    dmxDefault: String(from),
  };
}
export function draftInitial(f: FunctionDraft) {
  const a = Number(f.dmxFrom),
    b = Number(f.dmxTo),
    d = Number(f.dmxDefault);
  const position =
    f.mode === "range" && b > a && d >= a && d <= b
      ? initialFunction({ ...f, dmxFrom: a, dmxTo: b, dmxDefault: d }).position
      : 0;
  return { functionKey: f.key, position: String(position) };
}
export function nextChannel(draft: ProfileDraft) {
  return (
    Math.max(
      0,
      ...draft.channels.flatMap((c) => [
        Number(c.coarse) || 0,
        c.bits === "16" ? Number(c.fine) || 0 : 0,
      ]),
    ) + 1
  );
}
export function addFunctionChannel(
  draft: ProfileDraft,
  attribute: string,
): ProfileDraft {
  if (
    !(attribute in functionLabels) ||
    attribute === programKey ||
    draft.channels.some((c) => c.attribute === attribute)
  )
    return draft;
  const coarse = nextChannel(draft),
    first = newFunction();
  return {
    ...draft,
    footprint: String(Math.max(Number(draft.footprint) || 0, coarse)),
    channels: [
      ...draft.channels,
      {
        attribute,
        coarse: String(coarse),
        fine: "",
        bits: "8",
        percent: "0",
        functions: [first],
        defaultFunction: draftInitial(first),
      },
    ],
  };
}
export function withLinearFamily(
  draft: ProfileDraft,
  family: string,
): ProfileDraft {
  const keys =
    family === "none"
      ? []
      : family === "dimmer"
        ? ["dimmer"]
        : family === "rgb"
          ? ["red", "green", "blue"]
          : ["dimmer", "red", "green", "blue"];
  const preserved = draft.channels.filter(
    (c) => !["dimmer", "red", "green", "blue"].includes(c.attribute),
  );
  const channels = [
    ...preserved,
    ...draft.channels.filter((c) => keys.includes(c.attribute)),
  ];
  for (const attribute of keys)
    if (!channels.some((c) => c.attribute === attribute)) {
      const coarse = nextChannel({ ...draft, channels });
      channels.push({
        attribute,
        coarse: String(coarse),
        fine: "",
        bits: "8",
        percent: "0",
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
