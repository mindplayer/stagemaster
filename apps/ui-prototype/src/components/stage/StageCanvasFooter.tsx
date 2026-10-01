import type { FixtureView } from "../../application-host";
import { planeDistance } from "../../rigging-tools";
import { FixturePlanLegend } from "./FixturePlanLegend";
export function StageCanvasFooter({
  fixtures,
  ids,
  blocked,
  tool,
  measurement,
  step,
}: {
  fixtures: FixtureView[];
  ids: string[];
  blocked: string;
  tool: "select" | "move" | "pan" | "measure";
  measurement: { from: [number, number]; to: [number, number] } | null;
  step: number;
}) {
  return (
    <>
      <FixturePlanLegend fixtures={fixtures} ids={ids} />
      <footer>
        <span role="status">
          {blocked ||
            (tool === "select"
              ? "拖框选择灯具 · ⇧ 点击增减选择"
              : tool === "move"
                ? "拖动所选对象移动 · ⇧ 锁定方向 · Esc 取消"
                : tool === "measure"
                  ? "拖动两点测量平面距离 · ⇧ 锁定方向 · Esc 清除"
                  : "拖动平移视图")}
          {!blocked && tool !== "measure" && " · 方向键微调"}
        </span>
        <span>
          {measurement
            ? `平面距离 ${planeDistance(measurement.from, measurement.to).distance.toFixed(3)} 米 · `
            : ""}
          网格 {step} 米
        </span>
      </footer>
    </>
  );
}
