import type { FixtureView } from "../../application-host";
import type { ImportedEffectTemplate } from "../../effect-template-types";
import { TemplateKeyframePreview } from "./TemplateKeyframePreview";
import { LibraryDialog } from "./LibraryDialog";
const waves = { smooth: "平滑呼吸", triangle: "线性往返", pulse: "亮度脉冲" };
export function EffectTemplateReview({
  file,
  fixtures,
  sceneName,
  busy,
  error,
  onApply,
  onCancel,
}: {
  file: ImportedEffectTemplate;
  fixtures: FixtureView[];
  sceneName: string;
  busy: boolean;
  error: string;
  onApply(): Promise<boolean>;
  onCancel(): void;
}) {
  const effect = file.review.effect,
    source = effect.templateSource!;
  const recipe = source.template.definition.recipe;
  const percent = (value: number) =>
    ((value * 100) / 65535).toLocaleString("zh-CN", {
      maximumFractionDigits: 3,
    }) + "%";
  return (
    <LibraryDialog
      title="导入灯效模板"
      focusTitle
      busy={busy}
      error={error}
      submit="应用到场景"
      onSubmit={onApply}
      onCancel={onCancel}
    >
      <div className="effect-template-review">
        <strong>{effect.name}</strong>
        <p>
          {file.fileName} ·{" "}
          {recipe.kind === "intensity-wave"
            ? waves[recipe.waveform]
            : "关键帧亮度"}
        </p>
        <dl>
          <dt>目标场景</dt>
          <dd>{sceneName}</dd>
          <dt>亮度范围</dt>
          <dd>
            {recipe.kind === "intensity-wave"
              ? `${percent(recipe.low)} → ${percent(recipe.high)}`
              : `${percent(Math.min(...recipe.keyframes.map((f) => f.value)))}–${percent(Math.max(...recipe.keyframes.map((f) => f.value)))}`}
          </dd>
          <dt>变化节奏</dt>
          <dd>
            {effect.periodMs / 1000} 秒／轮 · 起始相位 {effect.phaseDegrees}° ·
            展开 {effect.spreadDegrees}°{effect.reverse ? " · 反向灯序" : ""}
          </dd>
          {recipe.kind === "intensity-wave" && recipe.waveform === "pulse" && (
            <>
              <dt>亮段比例</dt>
              <dd>{recipe.dutyPercent}%</dd>
            </>
          )}
        </dl>
        {recipe.kind === "intensity-keyframes" && (
          <TemplateKeyframePreview frames={recipe.keyframes} />
        )}
        <p>将添加已启用的亮度效果。颜色和位置保持原值，应用后再手动预演。</p>
        <h3>绑定灯序 · {effect.fixtureIds.length} 台</h3>
        <ol className="effect-template-targets">
          {effect.fixtureIds.map((id) => {
            const f = fixtures.find((f) => f.id === id);
            return (
              <li key={id}>
                {f?.name ?? "灯具已不存在"}
                <small>{f?.profileName}</small>
              </li>
            );
          })}
        </ol>
        <details>
          <summary>模板来源与检查结果</summary>
          <p>
            当前场景使用 {file.review.usage.effectChannels}{" "}
            项动态属性，核心编译检查通过。
          </p>
          <p>模板版本：{source.template.revision}</p>
          <p>内容摘要：{source.sha256}</p>
        </details>
      </div>
    </LibraryDialog>
  );
}
