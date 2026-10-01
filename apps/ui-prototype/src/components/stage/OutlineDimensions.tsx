import { displayMeters } from "./stage-display";
import { useEffect, useRef, useState } from "react";
import type { StageObject } from "../../stage-types";
import { objectOutline, decimal } from "../../stage-tools";
import { footprintBounds, footprintArea, resized } from "../../stage-geometry";
function dimensions(object: StageObject) {
  const b = footprintBounds(objectOutline(object)!);
  return {
    width: decimal(b.maxX - b.minX),
    depth: decimal(b.maxY - b.minY),
    x: decimal(b.minX),
    y: decimal(b.minY),
  };
}
export function OutlineDimensions({
  object,
  onChange,
}: {
  object: StageObject;
  onChange(value: StageObject): void;
}) {
  const [values, setValues] = useState(() => dimensions(object));
  const emitted = useRef<StageObject | null>(null);
  useEffect(() => {
    if (object !== emitted.current) setValues(dimensions(object));
  }, [object]);
  const change = (key: keyof typeof values, text: string) => {
    const next = { ...values, [key]: text };
    setValues(next);
    let updated = structuredClone(object);
    const n = Object.fromEntries(
      Object.entries(next).map(([k, v]) => [k, Number(v)]),
    ) as Record<keyof typeof values, number>;
    if (
      Object.values(next).every(
        (v) =>
          v.trim() !== "" &&
          Number.isFinite(Number(v)) &&
          Math.abs(Number(v)) <= 100000,
      ) &&
      n.width >= 0.01 &&
      n.depth >= 0.01
    )
      updated = resized(object, {
        minX: n.x,
        minY: n.y,
        maxX: n.x + n.width,
        maxY: n.y + n.depth,
      });
    // Even incomplete text owns an unsaved draft; the parent validates this form before any command.
    emitted.current = updated;
    onChange(updated);
  };
  const input = (key: keyof typeof values, label: string, min: number) => (
    <label>
      {label}
      <input
        aria-label={label}
        type="number"
        step="any"
        required
        min={min}
        max={100000}
        value={values[key]}
        onChange={(e) => change(key, e.target.value)}
      />
    </label>
  );
  return (
    <section className="stage-dimensions">
      <h3>
        尺寸{" "}
        <span>
          {displayMeters(footprintArea(objectOutline(object)!))} 平方米
        </span>
      </h3>
      <div className="stage-pair">
        {input("width", "整体宽度（米）", 0.01)}
        {input("depth", "整体深度（米）", 0.01)}
      </div>
      <h3>平面位置</h3>
      <div className="stage-pair">
        {input("x", "左边界 X（米）", -100000)}
        {input("y", "下边界 Y（米）", -100000)}
      </div>
    </section>
  );
}
