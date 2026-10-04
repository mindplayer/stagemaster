import { useEffect, useRef } from "react";
import type { ManualCapture } from "../../manual-capture-types";
import type { ProjectView } from "../../application-host";
import { ManualCaptureValues } from "./ManualCaptureValues";
import { sceneUsages } from "../workbench/scene-usage";
export function ManualSceneReview({
  capture,
  project,
  name,
  working,
  disabled,
  onName,
  onConfirm,
  onCancel,
}: {
  capture: ManualCapture;
  project: ProjectView;
  name: string;
  working: boolean;
  disabled: boolean;
  onName(name: string): void;
  onConfirm(): void;
  onCancel(): void;
}) {
  const review = useRef<HTMLDivElement>(null);
  useEffect(() => {
    review.current?.focus();
  }, []);
  const merge = capture.merge;
  const usages = merge ? sceneUsages(project, merge.sceneId) : [];
  return (
    <div
      ref={review}
      tabIndex={-1}
      className="manual-scene-review"
      aria-label={merge ? "确认合并场景" : "确认录入场景"}
    >
      <strong>
        {merge ? `合并到“${merge.sceneName}”` : "录入新场景"} ·{" "}
        {capture.fixtures.length} 台灯／{capture.readings.length} 项属性
      </strong>
      <p>
        已冻结“{capture.sourceName}
        ”的电平前设定值。现场继续变化不会改变本次记录。
      </p>
      {merge ? (
        <>
          <p>
            新增 {merge.added} 项 · 更新 {merge.replaced} 项 · 相同{" "}
            {merge.unchanged} 项；另保留 {merge.preserved} 项属性和{" "}
            {merge.effects} 个效果。
          </p>
          {merge.rows.some((r) => r.previousPreset) && (
            <p>涉及的预设引用将转为本场景独立值，共享预设保持。</p>
          )}
          {merge.rows.some((r) => r.effectNames.length) && (
            <p>下列属性仍受原动态效果控制，合并后的播放结果可能随效果变化。</p>
          )}
          <p>只更新工程中的场景；后台正在运行的版本和手动层保持。</p>
          <details>
            <summary>工程内使用位置（{usages.length}）</summary>
            <div className="manual-merge-usages">
              {usages.map((u) => (
                <p key={u.key}>
                  {u.title} · {u.detail}
                </p>
              ))}
              {!usages.length && <p>尚未被执行步骤或音乐编排引用</p>}
            </div>
          </details>
        </>
      ) : (
        <label>
          场景名称
          <input
            aria-label="录入场景名称"
            value={name}
            disabled={working}
            onChange={(e) => onName(e.target.value)}
          />
        </label>
      )}
      <ManualCaptureValues capture={capture} />
      <div className="execution-buttons">
        <button className="wb-primary" disabled={disabled} onClick={onConfirm}>
          {merge ? "确认合并" : "确认录入"}
        </button>
        <button disabled={working} onClick={onCancel}>
          {merge ? "取消合并" : "取消录入"}
        </button>
      </div>
    </div>
  );
}
