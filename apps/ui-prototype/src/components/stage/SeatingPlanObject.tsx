import type { SeatingShape } from "../../stage-types";
import { seatingLayout } from "../../seating-tools";
import "./seating.css";
export function SeatingPlanObject({
  id,
  name,
  shape,
  selected,
  labels,
  unit,
}: {
  id: string;
  name: string;
  shape: SeatingShape;
  selected: boolean;
  labels: boolean;
  unit: number;
}) {
  const layout = seatingLayout(shape);
  if (!layout) return null;
  const { width, depth, w, d, centers, yaw } = layout;
  return (
    <g
      data-kind="construction"
      data-id={id}
      className={`stage-seating ${selected ? "is-selected" : ""}`}
      transform={`translate(${shape.positionMeters.x},${-Number(shape.positionMeters.y)}) rotate(${-yaw})`}
    >
      <rect
        className="seating-boundary"
        x={-width / 2}
        y={-depth / 2}
        width={width}
        height={depth}
        strokeWidth={unit * 0.12}
      />
      {centers.map(([x, y], i) => (
        <g key={i}>
          <rect
            className="seating-chair"
            x={x - w / 2}
            y={-y - d / 2}
            width={w}
            height={d}
            rx={0.03}
            strokeWidth={Math.min(0.025, unit * 0.09)}
          />
          <rect
            className="seating-back"
            x={x - w / 2}
            y={-y + d / 2 - 0.06}
            width={w}
            height={0.06}
          />
        </g>
      ))}
      <path
        className="seating-front"
        d={`M 0 ${-depth / 2 - 0.1} v -.4 m -.15 .15 l .15 -.15 l .15 .15`}
        strokeWidth={unit * 0.1}
      />
      {labels && (
        <text y={depth / 2 + unit * 1.4} fontSize={unit} textAnchor="middle">
          {name} · {centers.length} 座
        </text>
      )}
      <title>
        {name} · {shape.rows} 排 × {shape.columns} 座
      </title>
    </g>
  );
}
