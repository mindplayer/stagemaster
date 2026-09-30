import type { StageObject } from "../../stage-types";
import { objectOutline } from "../../stage-tools";
import { displayMeters } from "./stage-display";
import { footprintBounds, resizedByHandle } from "../../stage-geometry";
export function StageSelectionOverlay({
  object,
  unit,
  disabled,
  onResize,
}: {
  object: StageObject;
  unit: number;
  disabled: boolean;
  onResize(value: StageObject): void;
}) {
  const outline = objectOutline(object);
  if (!outline) return null;
  const b = footprintBounds(outline),
    mx = (b.minX + b.maxX) / 2,
    my = (b.minY + b.maxY) / 2;
  const handles = [
    ["sw", b.minX, b.minY],
    ["s", mx, b.minY],
    ["se", b.maxX, b.minY],
    ["e", b.maxX, my],
    ["ne", b.maxX, b.maxY],
    ["n", mx, b.maxY],
    ["nw", b.minX, b.maxY],
    ["w", b.minX, my],
  ] as const;
  const names: Record<string, string> = {
    sw: "左下",
    s: "下",
    se: "右下",
    e: "右",
    ne: "右上",
    n: "上",
    nw: "左上",
    w: "左",
  };
  return (
    <g className="stage-selection-overlay">
      <rect
        pointerEvents="none"
        fill="none"
        stroke="#70b7ab"
        strokeOpacity={0.5}
        strokeWidth={unit * 0.07}
        strokeDasharray={`${unit * 0.4} ${unit * 0.4}`}
        x={b.minX}
        y={-b.maxY}
        width={b.maxX - b.minX}
        height={b.maxY - b.minY}
      />
      <g
        className="stage-measurements"
        pointerEvents="none"
        fill="none"
        strokeWidth={unit * 0.07}
      >
        <path
          d={`M ${b.minX} ${-b.maxY - unit} v ${-unit * 2.5} m 0 ${unit} H ${b.maxX} m 0 ${unit * 1.5} v ${-unit * 2.5}`}
        />
        <path
          d={`M ${b.maxX + unit} ${-b.minY} h ${unit * 2.5} m ${-unit} 0 V ${-b.maxY} m ${-unit * 1.5} 0 h ${unit * 2.5}`}
        />
        <text
          x={mx}
          y={-b.maxY - unit * 3.4}
          fontSize={unit * 1.25}
          textAnchor="middle"
        >
          {displayMeters(b.maxX - b.minX)} 米
        </text>
        <text
          x={b.maxX + unit * 4}
          y={-my}
          fontSize={unit * 1.25}
          dominantBaseline="middle"
        >
          {displayMeters(b.maxY - b.minY)} 米
        </text>
      </g>
      {handles.map(([handle, x, y]) => (
        <rect
          key={handle}
          data-handle={handle}
          data-kind={object.kind}
          data-id={
            object.kind === "placement"
              ? object.value.fixtureId
              : object.value.id
          }
          role="button"
          tabIndex={disabled ? -1 : 0}
          aria-disabled={disabled}
          aria-label={`调整${names[handle]}边界`}
          className={`stage-resize stage-resize-${handle}`}
          x={x - unit * 0.5}
          y={-y - unit * 0.5}
          width={unit}
          height={unit}
          rx={unit * 0.14}
          strokeWidth={unit * 0.1}
          onKeyDown={(e) => {
            if (disabled) return;
            const vectors: Record<string, [number, number]> = {
              ArrowLeft: [-1, 0],
              ArrowRight: [1, 0],
              ArrowUp: [0, 1],
              ArrowDown: [0, -1],
            };
            const v = vectors[e.key];
            if (
              !v ||
              (v[0] && !/[we]/.test(handle)) ||
              (v[1] && !/[ns]/.test(handle))
            )
              return;
            e.preventDefault();
            e.stopPropagation();
            const step = e.shiftKey ? 1 : 0.1;
            onResize(resizedByHandle(object, handle, v[0] * step, v[1] * step));
          }}
        >
          <title>拖动调整{names[handle]}边界</title>
        </rect>
      ))}
    </g>
  );
}
