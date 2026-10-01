import { useMemo } from "react";
import type { FixtureView } from "../../application-host";
import {
  fixturePlanLabel,
  type PlanLabelMode,
} from "../../fixture-plan-display";
import { fitPlanLabel } from "../../plan-label-text";
export function useFixtureLabelMetrics(
  fixtures: FixtureView[],
  mode: PlanLabelMode,
) {
  return useMemo(() => {
    const result = new Map<string, { text: string; width: number }>();
    if (mode === "none") return result;
    const context = document.createElement("canvas").getContext("2d");
    if (context) context.font = "14px sans-serif";
    const measure = (text: string) =>
      context
        ? (context.measureText(text).width / 14) * 0.9
        : [...text].length * 0.9;
    for (const f of fixtures)
      result.set(f.id, fitPlanLabel(fixturePlanLabel(f, mode), measure));
    return result;
  }, [fixtures, mode]);
}
