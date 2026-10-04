import type { ExecutionView } from "./execution-types";
import type { ExecutionGestureTarget } from "./execution-gesture-target.ts";

export const outputMasterKey = "output:master";
export const outputLevel = (percent: number) => Math.round(percent * 655.35);
export function outputAvailable(runtime: ExecutionView) {
  const output = runtime.observation.snapshot?.state.output;
  const count = runtime.catalog.output?.uncontrolledFixtures;
  return (
    !!runtime.catalog.capabilities?.includes("outputMaster") &&
    Number.isInteger(count) &&
    count! >= 0 &&
    count! <= (runtime.catalog.fixtures?.length ?? -1) &&
    !!output &&
    Number.isInteger(output.percent) &&
    output.percent >= 0 &&
    output.percent <= 100 &&
    typeof output.blackout === "boolean"
  );
}
export function outputMasterTarget(
  runtime: ExecutionView,
  current = () => true,
): ExecutionGestureTarget {
  if (!outputAvailable(runtime)) throw Error("后台未提供有效输出总控");
  const project = runtime.catalog.projectId;
  const count = runtime.catalog.output!.uncontrolledFixtures;
  return {
    key: outputMasterKey,
    label: "后台总亮度",
    normalize: (value) => outputLevel(Math.round(value / 655.35)),
    valid: (value) =>
      current() &&
      outputAvailable(value) &&
      value.catalog.projectId === project &&
      value.catalog.output!.uncontrolledFixtures === count,
    value: (value) =>
      outputAvailable(value)
        ? outputLevel(value.observation.snapshot!.state.output!.percent)
        : undefined,
    request: (hostId, revision, value) => ({
      kind: "output",
      hostId,
      revision,
      action: { kind: "level", percent: Math.round(value / 655.35) },
    }),
  };
}
