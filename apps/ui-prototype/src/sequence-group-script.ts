import type { SequenceEdit, StepView, StepScript } from "./sequence-types";
import { scriptFields, scriptProblem } from "./sequence-script-tools.ts";
export type ScriptChangeMode = "keep" | "replace" | "clear";
export interface GroupScriptField {
  mode: ScriptChangeMode;
  value: string;
  mixed: boolean;
}
export interface GroupScriptDraft {
  ids: string[];
  fields: Record<keyof StepScript, GroupScriptField>;
}
export function groupScriptDraft(steps: StepView[]): GroupScriptDraft {
  const field = (key: keyof StepScript): GroupScriptField => {
    const value = steps[0]?.script?.[key] ?? "";
    const mixed = steps.some((step) => (step.script?.[key] ?? "") !== value);
    return { mode: "keep", value: mixed ? "" : value, mixed };
  };
  return {
    ids: steps.map((s) => s.id),
    fields: {
      section: field("section"),
      trigger: field("trigger"),
      notes: field("notes"),
    },
  };
}
export class GroupScriptError extends Error {
  field: keyof StepScript;
  constructor(field: keyof StepScript, message: string) {
    super(message);
    this.field = field;
  }
}
export function groupScriptCommand(
  sequenceId: string,
  draft: GroupScriptDraft,
): SequenceEdit | null {
  const patch: Partial<StepScript> = {};
  for (const [, key, label, limit] of scriptFields) {
    const field = draft.fields[key];
    if (field.mode === "keep") continue;
    if (field.mode === "clear") {
      patch[key] = "";
      continue;
    }
    if (!field.value.trim())
      throw new GroupScriptError(key, `请填写${label}，或明确选择“清空”`);
    const problem = scriptProblem(field.value, label, limit);
    if (problem) throw new GroupScriptError(key, problem);
    patch[key] = field.value;
  }
  if (!Object.keys(patch).length) return null;
  if (
    !draft.ids.length ||
    draft.ids.length > 1024 ||
    new Set(draft.ids).size !== draft.ids.length
  )
    throw new Error("请重新选择需要修改剧本提示的步骤");
  return {
    kind: "editSteps",
    id: sequenceId,
    stepIds: [...draft.ids],
    operation: { kind: "script", patch },
  };
}
