import type { EditOperation } from "./application-host";
import type { SequenceView, StepView, SequenceEdit } from "./sequence-types";
// Input presentation only. Playback interpolation and schedule interpretation stay in Rust.
export function secondsToMs(input: string, label: string): number {
  const value = input.trim();
  if (!/^\d+(?:\.\d{1,3})?$/.test(value))
    throw new Error(`${label}请输入非负秒数，最多三位小数`);
  const [whole, fraction = ""] = value.split(".");
  const ms = Number(whole) * 1000 + Number(fraction.padEnd(3, "0"));
  if (!Number.isSafeInteger(ms) || ms > 86_400_000)
    throw new Error(`${label}须在 0–86400 秒内`);
  return ms;
}
export function seconds(ms: number): string {
  return String(ms / 1000);
}
export function fixtureAppearance(
  attributes: { key: string; value: number }[],
) {
  const get = (key: string) => attributes.find((a) => a.key === key)?.value;
  const dimmer = get("dimmer");
  const rgb = [get("red"), get("green"), get("blue")];
  return {
    level: dimmer === undefined ? null : dimmer / 65535,
    color: rgb.every((v) => v !== undefined)
      ? `rgb(${rgb.map((v) => Math.round((v! / 65535) * 255)).join(" ")})`
      : "#e1ecee",
  };
}

export interface SequenceDraft {
  sequenceName: string;
  tracking: SequenceView["tracking"];
  repeat: SequenceView["repeat"];
  name: string;
  number: string;
  position: string;
  sceneId: string;
  delay: string;
  fade: string;
  advance: "manual" | "after";
  wait: string;
}
export function sequenceDraft(
  sequence: SequenceView,
  step: StepView,
): SequenceDraft {
  return {
    sequenceName: sequence.name,
    tracking: sequence.tracking,
    repeat: sequence.repeat,
    name: step.name,
    number: step.number,
    position: String(sequence.steps.indexOf(step) + 1),
    sceneId: step.sceneId,
    delay: seconds(step.delayMs),
    fade: seconds(step.fadeMs),
    advance: step.waitMs === null ? "manual" : "after",
    wait: seconds(step.waitMs ?? 0),
  };
}

export class SequenceInputError extends Error {
  field: keyof SequenceDraft;
  constructor(field: keyof SequenceDraft, message: string) {
    super(message);
    this.field = field;
  }
}
export function sequenceCommands(
  d: SequenceDraft,
  sequence: SequenceView,
  step: StepView,
): EditOperation[] {
  for (const [key, label] of [
    ["sequenceName", "列表名称"],
    ["name", "步骤名称"],
  ] as const)
    if (!d[key].trim() || d[key].trim().length > 256)
      throw new SequenceInputError(key, `${label}请输入 1–256 个字符`);
  if (!/^(0|[1-9]\d{0,5})(?:\.\d{1,3})?$/.test(d.number))
    throw new SequenceInputError(
      "number",
      "步骤编号请输入 0–999999，最多三位小数",
    );
  if (
    sequence.steps.some(
      (s) => s.id !== step.id && Number(s.number) === Number(d.number),
    )
  )
    throw new SequenceInputError(
      "number",
      "步骤编号已被使用（1 与 1.0 视为相同）",
    );
  if (
    !/^\d+$/.test(d.position) ||
    Number(d.position) < 1 ||
    Number(d.position) > sequence.steps.length
  )
    throw new SequenceInputError(
      "position",
      `执行顺序请输入 1–${sequence.steps.length} 的整数`,
    );
  const time = (key: "delay" | "fade" | "wait", label: string) => {
    try {
      return secondsToMs(d[key], label);
    } catch (reason) {
      throw new SequenceInputError(
        key,
        reason instanceof Error ? reason.message : String(reason),
      );
    }
  };
  const commands: EditOperation[] = [];
  if (
    d.sequenceName.trim() !== sequence.name ||
    d.tracking !== sequence.tracking ||
    d.repeat !== sequence.repeat
  )
    commands.push({
      op: "sequence",
      command: {
        kind: "update",
        id: sequence.id,
        name: d.sequenceName.trim(),
        tracking: d.tracking,
        repeat: d.repeat,
      },
    });
  const update: SequenceEdit = {
    kind: "updateStep",
    id: sequence.id,
    stepId: step.id,
    name: d.name.trim(),
    number: d.number,
    sceneId: d.sceneId,
    delayMs: time("delay", "延时"),
    fadeMs: time("fade", "渐变"),
    waitMs: d.advance === "manual" ? null : time("wait", "自动等待"),
  };
  if (
    update.name !== step.name ||
    update.number !== step.number ||
    update.sceneId !== step.sceneId ||
    update.delayMs !== step.delayMs ||
    update.fadeMs !== step.fadeMs ||
    update.waitMs !== step.waitMs
  )
    commands.push({ op: "sequence", command: update });
  if (Number(d.position) !== sequence.steps.indexOf(step) + 1)
    commands.push({
      op: "sequence",
      command: {
        kind: "moveStep",
        id: sequence.id,
        stepId: step.id,
        index: Number(d.position) - 1,
      },
    });
  return commands;
}

export function channelWindow(slots: number[], query: string, page: number) {
  const range = /^\s*(\d+)(?:\s*[-–]\s*(\d+))?\s*$/.exec(query);
  const valid =
    !query.trim() ||
    !!(
      range &&
      Number(range[1]) >= 1 &&
      Number(range[2] ?? range[1]) <= 512 &&
      Number(range[1]) <= Number(range[2] ?? range[1])
    );
  const all = slots
    .map((value, index) => ({ value, address: index + 1 }))
    .filter(
      ({ address }) =>
        valid &&
        (!range ||
          (address >= Number(range[1]) &&
            address <= Number(range[2] ?? range[1]))),
    );
  const pages = Math.max(1, Math.ceil(all.length / 64));
  const index = Math.max(0, Math.min(pages - 1, Math.trunc(page)));
  return {
    valid,
    index,
    pages,
    total: all.length,
    rows: all.slice(index * 64, (index + 1) * 64),
  };
}
