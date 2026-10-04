import { attributeBase, isEmitterFunction } from "./fixture-emitter-keys.ts";
import { FixtureFieldError } from "./fixture-field-error.ts";
import type { FunctionDraft } from "./fixture-function-draft";
import type { FunctionDefinition } from "./fixture-function-types";

export const emitterFunctionLabels = {
  shutter: "快门与频闪",
  "color-wheel": "色盘",
  "gobo-wheel": "图案盘",
};
export function controlledFunctionOptions(attribute: string) {
  const base = attributeBase(attribute);
  if (base === "prism")
    return [
      { id: "off", name: "退出", mode: "slot" as const },
      { id: "on", name: "插入", mode: "slot" as const },
    ];
  if (!Object.hasOwn(emitterFunctionLabels, base)) return [];
  return base === "shutter"
    ? [
        { id: "open", name: "开光", mode: "slot" as const },
        { id: "closed", name: "闭光", mode: "slot" as const },
        { id: "strobe", name: "受控频闪", mode: "range" as const },
      ]
    : [
        { id: "open", name: "通光", mode: "slot" as const },
        { id: "slot", name: "固定档位", mode: "slot" as const },
        ...(base === "gobo-wheel"
          ? [{ id: "shake", name: "单图案抖动", mode: "range" as const }]
          : []),
      ];
}
export function newControlledFunction(
  attribute: string,
  kind: string,
): FunctionDraft {
  const type = controlledFunctionOptions(attribute).find((o) => o.id === kind);
  if (!type) throw new Error("不支持此受控功能种类");
  return {
    key:
      kind === "slot" || kind === "shake"
        ? `${kind}-${crypto.randomUUID()}`
        : kind,
    name: type.name,
    mode: type.mode,
    dmxFrom: "",
    dmxTo: "",
    dmxDefault: "",
  };
}
export function controlledKind(key: string) {
  return key.startsWith("slot-")
    ? "slot"
    : key.startsWith("shake-")
      ? "shake"
      : key;
}
export function validateEmitterFunctions(
  attribute: string,
  functions: FunctionDefinition[],
  prefix: string,
) {
  if (!isEmitterFunction(attribute)) return;
  const options = controlledFunctionOptions(attribute);
  functions.forEach((f, i) => {
    const kind = controlledKind(f.key);
    const type = options.find((o) => o.id === kind);
    if (
      !type ||
      type.mode !== f.mode ||
      ((kind === "slot" || kind === "shake") && f.key.length <= kind.length + 1)
    )
      throw new FixtureFieldError(
        `${prefix}-function-${i}-mode`,
        "独立光源仅允许明确开闭、受控频闪、固定档位或单图案抖动；声控、自走、复位和未知宏已屏蔽",
      );
  });
}
