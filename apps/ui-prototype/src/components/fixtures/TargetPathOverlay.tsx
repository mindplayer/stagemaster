import { readAimPoint, type AimPoint } from "./aim-point";

/** Plan annotation only; physical path solving remains in Rust. */
export function TargetPathOverlay({
  active,
  activeLabel,
  other,
  unit,
}: {
  active: AimPoint | null;
  activeLabel: string;
  other: { x: string; y: string; label: string };
  unit: number;
}) {
  const point = readAimPoint(other.x, other.y);
  return (
    <g className="target-path-overlay">
      {active && point && (
        <line
          x1={active.x}
          y1={-active.y}
          x2={point.x}
          y2={-point.y}
          strokeWidth={unit * 0.25}
          strokeDasharray={`${unit} ${unit * 0.65}`}
        />
      )}
      {point && (
        <circle
          cx={point.x}
          cy={-point.y}
          r={unit * 1.15}
          strokeWidth={unit * 0.25}
        />
      )}
      {[
        { point: active, label: activeLabel, direction: -1 },
        { point, label: other.label, direction: 1 },
      ].map(({ point: at, label, direction }) =>
        at ? (
          <text
            key={label}
            x={at.x}
            y={-at.y + direction * unit * 4}
            fontSize={unit * 3.7}
            textAnchor="middle"
            dominantBaseline={direction > 0 ? "hanging" : "auto"}
            strokeWidth={unit * 0.6}
          >
            {label}
          </text>
        ) : null,
      )}
    </g>
  );
}
