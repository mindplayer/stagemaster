import type { FixtureView } from "./application-host";
import type { ExecutionView, ExecutionStatus } from "./execution-types";
import { sameFunctions, functionLabels } from "./fixture-function-types.ts";
import {
  functionSelectionAllowed,
  functionSafetyMessage,
} from "./fixture-function-safety.ts";
export type ManualFixture = Pick<
  FixtureView,
  "id" | "name" | "profileName" | "universe" | "address" | "attributes"
>;
export interface ManualTarget {
  fixtureId: string;
  attribute: string;
}
export type ManualValue =
  | { kind: "release" }
  | { kind: "normalized"; value: number }
  | { kind: "function"; functionKey: string; position: number };
export interface ManualEdit extends ManualTarget {
  value: ManualValue;
}
export interface ManualLimits {
  manualChanges: number;
  requestBytes: number;
}
export interface ManualDraft {
  targets: string[];
  attribute: string;
  value: string;
  functionKey: string;
}
export function manualAvailable(runtime: ExecutionView) {
  return (
    !!runtime.catalog.capabilities?.includes("manualOwnership") &&
    !!runtime.catalog.capabilities.includes("semanticManualPatch") &&
    !!runtime.catalog.fixtures &&
    !!runtime.catalog.limits
  );
}
export function commonManualAttributes(
  fixtures: ManualFixture[],
  selected: readonly string[],
) {
  const targets = selected.map((id) => fixtures.find((f) => f.id === id));
  if (!targets.length || targets.some((f) => !f)) return [];
  return targets[0]!.attributes.filter((a) =>
    targets.every((f) => {
      const other = f!.attributes.find((b) => b.key === a.key);
      return (
        !!other &&
        sameFunctions(a.function?.functions, other.function?.functions) &&
        a.function?.fine === other.function?.fine
      );
    }),
  );
}
export function manualPercent(text: string) {
  const value = Number(text);
  if (!text.trim() || !Number.isFinite(value) || value < 0 || value > 100)
    throw Error("请输入 0–100 之间的百分比");
  return Math.round((value * 65535) / 100);
}
export function manualChanges(
  runtime: ExecutionView,
  source: string,
  draft: ManualDraft,
  release = false,
): ManualEdit[] {
  if (
    !manualAvailable(runtime) ||
    !runtime.catalog.sources.some(
      (s) => s.id === source && s.selection.kind === "manual",
    )
  )
    throw Error("当前后台未提供手动编程能力");
  if (
    !draft.targets.length ||
    new Set(draft.targets).size !== draft.targets.length
  )
    throw Error("请选择不重复的后台灯具");
  const attr = commonManualAttributes(
    runtime.catalog.fixtures!,
    draft.targets,
  ).find((a) => a.key === draft.attribute);
  if (!attr) throw Error("所选灯具没有一致的此项属性定义，请缩小选灯范围");
  let value: ManualValue;
  if (release) value = { kind: "release" };
  else if (attr.function) {
    const f = attr.function.functions.find((f) => f.key === draft.functionKey);
    if (!f) throw Error("请选择后台已定义的功能");
    if (!functionSelectionAllowed(attr.key, f))
      throw Error(functionSafetyMessage);
    value = {
      kind: "function",
      functionKey: f.key,
      position: f.mode === "slot" ? 0 : manualPercent(draft.value),
    };
  } else {
    if (Object.hasOwn(functionLabels, attr.key))
      throw Error("功能通道必须明确选择功能，旧普通百分比映射已屏蔽");
    value = { kind: "normalized", value: manualPercent(draft.value) };
  }
  const changes = draft.targets.map((fixtureId) => ({
    fixtureId,
    attribute: draft.attribute,
    value,
  }));
  const limits = runtime.catalog.limits!;
  if (changes.length > limits.manualChanges)
    throw Error(`一次最多修改 ${limits.manualChanges} 个属性，请减少所选灯具`);
  // Conservative complete wire envelope: includes maximum next serial/revision, not only payload.
  const body = {
    serial: "18446744073709551615",
    ttlMs: 5000,
    command: {
      kind: "submit",
      expectedRevision: "18446744073709551615",
      action: { kind: "source", source, action: { kind: "patch", changes } },
    },
  };
  if (
    new TextEncoder().encode(JSON.stringify(body)).length > limits.requestBytes
  )
    throw Error("本次整组操作超过后台请求容量，请减少灯具；不会自动分批");
  return changes;
}
export function manualResult(status: ExecutionStatus | undefined) {
  const runtime = status?.runtime;
  if (!runtime?.record || !runtime.sessionId) return null;
  return {
    hostId: runtime.hostId,
    sessionId: runtime.sessionId,
    serial: runtime.record.serial,
  };
}
