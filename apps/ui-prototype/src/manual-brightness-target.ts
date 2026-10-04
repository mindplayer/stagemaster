import type { ExecutionView } from "./execution-types";
import type { ExecutionGestureTarget } from "./execution-gesture-target.ts";
import {
  commonManualAttributes,
  manualAvailable,
  manualChanges,
} from "./execution-manual.ts";
import { manualRows, selectedManualReading } from "./manual-readings.ts";

export function manualBrightnessKey(
  source: string,
  selected: readonly string[],
) {
  return `brightness:${source}:${JSON.stringify(selected)}`;
}

export function manualBrightnessReading(
  runtime: ExecutionView,
  source: string,
  selected: string[],
) {
  const state = runtime.observation.snapshot?.state.sources.find(
    (s) => s.id === source,
  );
  const rows = manualRows(runtime.catalog.fixtures ?? [], state);
  if (!rows) return null;
  return selectedManualReading(rows, selected, "dimmer");
}

export function manualBrightnessTarget(
  runtime: ExecutionView,
  source: string,
  selected: readonly string[],
  current: () => boolean,
): ExecutionGestureTarget {
  const targets = [...selected];
  const project = runtime.catalog.projectId;
  const compatible = (value: ExecutionView) => {
    const attribute = commonManualAttributes(
      value.catalog.fixtures ?? [],
      targets,
    ).find((a) => a.key === "dimmer");
    return (
      value.catalog.projectId === project &&
      value.catalog.sources.some(
        (s) => s.id === source && s.selection.kind === "manual",
      ) &&
      manualAvailable(value) &&
      !!value.catalog.capabilities?.includes("manualValues") &&
      !!attribute &&
      !attribute.function &&
      manualBrightnessReading(value, source, targets) !== null
    );
  };
  if (!compatible(runtime))
    throw Error("所选灯具未提供连续共同亮度及实际设定值");
  // Maximum normalized value also verifies the complete atomic wire budget once.
  const changes = manualChanges(runtime, source, {
    targets,
    attribute: "dimmer",
    value: "100",
    functionKey: "",
  });
  const limits = { ...runtime.catalog.limits! };
  return {
    key: manualBrightnessKey(source, targets),
    source,
    label: "共同亮度",
    initial: 32768,
    valid: (value) =>
      current() &&
      compatible(value) &&
      value.catalog.limits?.manualChanges === limits.manualChanges &&
      value.catalog.limits.requestBytes === limits.requestBytes,
    value: (value) => {
      const reading = manualBrightnessReading(value, source, targets);
      return reading?.unheld === 0 ? reading.raw : undefined;
    },
    request: (hostId, revision, value) => ({
      kind: "apply",
      hostId,
      revision,
      source,
      action: {
        kind: "patch",
        changes: changes.map((change) => ({
          ...change,
          value: { kind: "normalized", value },
        })),
      },
    }),
  };
}
