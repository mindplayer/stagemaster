import type { FixturePlacement } from "../../stage-types.ts";
import { translated } from "../../stage-tools.ts";
export function nudgedPlacements(
  placements: FixturePlacement[],
  key: string,
  fast: boolean,
): FixturePlacement[] {
  const step = fast ? 1 : 0.1;
  const dx = key === "ArrowLeft" ? -step : key === "ArrowRight" ? step : 0;
  const dy = key === "ArrowDown" ? -step : key === "ArrowUp" ? step : 0;
  return placements.map(
    (p) =>
      (
        translated({ kind: "placement", value: p }, dx, dy) as {
          kind: "placement";
          value: FixturePlacement;
        }
      ).value,
  );
}
