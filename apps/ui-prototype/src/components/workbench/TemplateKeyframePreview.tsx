import type { EffectKeyframe } from "../../effect-types";
import { curvePath } from "../../keyframe-geometry";
import { frameDrafts, valuePercent } from "../../keyframe-tools";

const transitionNames = { hold: "保持", linear: "线性", smooth: "平滑" };
export function TemplateKeyframePreview({
  frames,
}: {
  frames: EffectKeyframe[];
}) {
  const curve = curvePath(
    frameDrafts([{ attribute: "dimmer", keyframes: frames }]),
    "dimmer",
  );
  return (
    <section className="template-keyframe-preview" aria-label="模板关键帧曲线">
      <strong>亮度曲线 · {frames.length} 帧</strong>
      <svg
        viewBox="-2 -5 104 110"
        preserveAspectRatio="none"
        role="img"
        aria-label="一个循环内的亮度变化，包括末帧到下一轮的过渡"
      >
        {[0, 25, 50, 75, 100].map((v) => (
          <g key={v} className="template-curve-grid">
            <line x1="0" x2="100" y1={v} y2={v} />
            <line x1={v} x2={v} y1="0" y2="100" />
          </g>
        ))}
        <path
          d={curve}
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
          vectorEffect="non-scaling-stroke"
        />
      </svg>
      <div className="template-curve-labels">
        <span>0%</span>
        <span>50%</span>
        <span>100% · 循环</span>
      </div>
      <details>
        <summary>查看 {frames.length} 帧精确数值</summary>
        <div className="template-frame-values">
          <table>
            <thead>
              <tr>
                <th>周期位置</th>
                <th>亮度</th>
                <th>到下一帧</th>
              </tr>
            </thead>
            <tbody>
              {frames.map((f) => (
                <tr key={f.position}>
                  <td>{f.position / 100}%</td>
                  <td>{valuePercent(f.value)}%</td>
                  <td>{transitionNames[f.transition]}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </details>
    </section>
  );
}
