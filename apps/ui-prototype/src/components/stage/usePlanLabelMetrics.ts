import { useMemo } from "react";
import { fitPlanLabel } from "../../plan-label-text";
/** Geometry changes do not require measuring unchanged names again. */
export function usePlanLabelMetrics(items: { id: string; text: string }[]) {
  const key = JSON.stringify(items.map(({ id, text }) => [id, text]));
  return useMemo(() => {
    const context = document.createElement("canvas").getContext("2d");
    if (context) context.font = "14px sans-serif";
    const measure = (text: string) =>
      context
        ? (context.measureText(text).width / 14) * 0.9
        : [...text].length * 0.9;
    const entries: [string, string][] = JSON.parse(key);
    return new Map(
      entries.map(([id, text]) => [id, fitPlanLabel(text, measure)]),
    );
  }, [key]);
}
