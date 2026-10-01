import type { StepScript, StepView } from "./sequence-types";
export const scriptFields = [
  ["scriptSection", "section", "幕／场", 80],
  ["scriptTrigger", "trigger", "台词／动作提示", 1024],
  ["scriptNotes", "notes", "排练备注", 4096],
] as const;
export function scriptSearchText(script?: StepScript): string {
  return script ? `${script.section} ${script.trigger} ${script.notes}` : "";
}
export function stepMatches(
  step: StepView,
  sceneName: string,
  query: string,
): boolean {
  return `${step.number} ${step.name} ${sceneName} ${scriptSearchText(step.script)}`
    .toLocaleLowerCase()
    .includes(query.trim().toLocaleLowerCase());
}
export function scriptProblem(
  value: string,
  label: string,
  limit: number,
): string | null {
  return Array.from(value).length > limit ||
    /[\u0000-\u0008\u000b\u000c\u000e-\u001f\u007f-\u009f]/.test(value)
    ? `${label}最多 ${limit} 个字符，且不能包含不可见控制字符`
    : null;
}
