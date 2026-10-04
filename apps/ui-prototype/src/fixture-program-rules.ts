import type {
  FunctionDefinition,
  FunctionSelection,
} from "./fixture-function-types";
import { FixtureFieldError } from "./fixture-field-error.ts";

export const programKey = "fixture-program";
export const programLabel = "内置程序";
export function programSelectionAllowed(key: string) {
  return key === "external";
}
export function programKind(key: string) {
  return key === "external"
    ? "外部通道控制"
    : key.startsWith("auto.")
      ? "内置自走（禁用）"
      : key.startsWith("sound.")
        ? "声控（禁用）"
        : "未知程序";
}
export function validateProgramDefinition(
  functions: FunctionDefinition[],
  selection: FunctionSelection,
  index: number,
) {
  const prefix = `channel-${index}`;
  if (!functions.some((f) => f.key === "external"))
    throw new FixtureFieldError(
      `${prefix}-function-add`,
      "内置程序须定义外部通道控制档位",
    );
  functions.forEach((f, j) => {
    if (f.mode !== "slot")
      throw new FixtureFieldError(
        `${prefix}-function-${j}-mode`,
        "内置程序只支持固定档位，不能渐变或调节区间",
      );
    if (programKind(f.key) === "未知程序")
      throw new FixtureFieldError(
        `${prefix}-function-${j}-name`,
        "内置程序只支持外部控制、自动或声控，不能承载复位",
      );
  });
  if (selection.functionKey !== "external" || selection.position !== 0)
    throw new FixtureFieldError(
      `${prefix}-default-function`,
      "内置程序默认须为外部通道控制，不能默认自走或声控",
    );
}
