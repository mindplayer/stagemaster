import { planeDistance } from "../../rigging-tools";
export function StageMeasureOverlay({
  measurement,
  unit,
}: {
  measurement: { from: [number, number]; to: [number, number] } | null;
  unit: number;
}) {
  if (!measurement) return null;
  const { from: a, to: b } = measurement,
    m = planeDistance(a, b);
  return (
    <g
      className="stage-ruler"
      pointerEvents="none"
      aria-label={`平面距离 ${m.distance.toFixed(3)} 米`}
    >
      <line
        x1={a[0]}
        y1={-a[1]}
        x2={b[0]}
        y2={-b[1]}
        strokeWidth={unit * 0.15}
      />
      {[a, b].map((p, i) => (
        <circle
          key={i}
          cx={p[0]}
          cy={-p[1]}
          r={unit * 0.35}
          strokeWidth={unit * 0.12}
        />
      ))}
      <text
        x={(a[0] + b[0]) / 2}
        y={-(a[1] + b[1]) / 2 - unit * 1.1}
        fontSize={unit * 1.15}
        textAnchor="middle"
      >
        {m.distance.toFixed(3)} 米
      </text>
      <text
        x={(a[0] + b[0]) / 2}
        y={-(a[1] + b[1]) / 2 + unit * 1.7}
        fontSize={unit * 0.9}
        textAnchor="middle"
      >
        ΔX {m.dx.toFixed(2)} · ΔY {m.dy.toFixed(2)} 米
      </text>
    </g>
  );
}
