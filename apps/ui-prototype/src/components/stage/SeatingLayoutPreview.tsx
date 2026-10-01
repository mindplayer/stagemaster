import type { SeatingShape } from "../../stage-types";
import { seatingLayout } from "../../seating-tools";
import { SeatingPlanObject } from "./SeatingPlanObject";
/** The same chair projection as the main plan, centered only for this form thumbnail. */
export function SeatingLayoutPreview({ shape }: { shape: SeatingShape }) {
  const local: SeatingShape = {
    ...shape,
    positionMeters: { x: "0", y: "0", z: "0" },
    yawDegrees: "0",
  };
  const layout = seatingLayout(local);
  if (!layout) return null;
  const margin = Math.max(0.7, layout.width * 0.05);
  return (
    <svg
      className="seating-layout-preview"
      role="img"
      aria-label={`${shape.arc ? "弧形" : "直排"}座区排列预览，${shape.rows} 排，每排 ${shape.columns} 座`}
      viewBox={`${-layout.width / 2 - margin} ${-layout.depth / 2 - margin} ${layout.width + 2 * margin} ${layout.depth + 2 * margin}`}
    >
      <SeatingPlanObject
        id="seating-draft"
        name="座区草稿"
        shape={local}
        selected={false}
        unit={margin / 3}
      />
    </svg>
  );
}
