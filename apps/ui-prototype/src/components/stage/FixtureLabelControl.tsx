import type { PlanLabelMode } from "../../fixture-plan-display";
export function FixtureLabelControl({
  value,
  onChange,
}: {
  value: PlanLabelMode;
  onChange(value: PlanLabelMode): void;
}) {
  return (
    <label className="fixture-label-control">
      标注
      <select
        aria-label="平面灯位标注"
        value={value}
        onChange={(e) => onChange(e.target.value as PlanLabelMode)}
      >
        <option value="none">隐藏</option>
        <option value="name">名称</option>
        <option value="address">配适地址</option>
      </select>
    </label>
  );
}
