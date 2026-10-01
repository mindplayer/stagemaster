import { useRef } from "react";
import type { FixtureView, SceneView } from "../../application-host";
import { attributeState, type ParameterDraft } from "../../editor-tools";
export function ContinuousParameter({
  attribute,
  fixtures,
  scene,
  draft,
  resolvedValue,
  put,
  cancel,
  capture,
  onApply,
}: {
  attribute: FixtureView["attributes"][number];
  fixtures: FixtureView[];
  scene: SceneView;
  draft: ParameterDraft | undefined;
  resolvedValue(key: string): number;
  put(key: string, draft: ParameterDraft): void;
  cancel(key: string): void;
  capture(): () => void;
  onApply(): void;
}) {
  const gesture = useRef<{ rollback(): void; cancelled: boolean } | null>(null);
  const state = attributeState(scene, fixtures, attribute.key);
  const mode =
    typeof draft === "object" && "mode" in draft
      ? draft.mode
      : draft !== undefined
        ? "literal"
        : state.mode;
  const mixed = state.mixed && draft === undefined;
  const numeric = resolvedValue(attribute.key);
  const percent =
    typeof draft === "string"
      ? draft
      : draft === undefined && mixed
        ? ""
        : String(Math.round((numeric / 65535) * 10000) / 100);
  const status = mixed
    ? "混合值"
    : mode === "release"
      ? "释放"
      : mode === "remove" || mode === "absent"
        ? "未记录"
        : mode === "preset"
          ? `预设 · ${state.presetName}`
          : "已记录";
  const effected = scene.effects.some(
    (effect) =>
      effect.enabled &&
      effect.channels.some((channel) => channel.attribute === attribute.key) &&
      effect.fixtureIds.some((id) =>
        fixtures.some((fixture) => fixture.id === id),
      ),
  );
  return (
    <div className="wb-parameter" key={attribute.key}>
      <div className="wb-section-title">
        <label htmlFor={`param-${attribute.key}`}>{attribute.label}</label>
        <span
          title={
            effected
              ? "此处编辑静态值；受效果控制的灯具在停用效果后使用此值"
              : undefined
          }
        >
          {effected ? "效果覆盖 · 静态值" : status}
        </span>
      </div>
      <div className="wb-value-row">
        <input
          type="range"
          aria-label={`${attribute.label}滑块`}
          aria-valuetext={mixed ? "混合值" : `${percent}%`}
          min={0}
          max={65535}
          step={1}
          value={Math.min(65535, Math.max(0, numeric))}
          onChange={(e) => {
            if (!gesture.current?.cancelled)
              put(attribute.key, Number(e.target.value));
          }}
          onPointerDown={(e) => {
            gesture.current = {
              rollback: capture(),
              cancelled: false,
            };
            e.currentTarget.setPointerCapture(e.pointerId);
          }}
          onPointerUp={(e) => {
            const cancelled = gesture.current?.cancelled;
            gesture.current = null;
            e.currentTarget.releasePointerCapture(e.pointerId);
            if (!cancelled) onApply();
          }}
          onPointerCancel={() => {
            const start = gesture.current;
            gesture.current = null;
            if (start) start.rollback();
          }}
          onLostPointerCapture={() => {
            const start = gesture.current;
            gesture.current = null;
            if (start) start.rollback();
          }}
          onKeyDown={(e) => {
            if (e.key === "Escape") {
              e.preventDefault();
              e.stopPropagation();
              const start = gesture.current;
              if (start) {
                start.cancelled = true;
                start.rollback();
              } else cancel(attribute.key);
            }
          }}
          onKeyUp={(e) => {
            if (
              [
                "ArrowLeft",
                "ArrowRight",
                "ArrowUp",
                "ArrowDown",
                "Home",
                "End",
                "PageUp",
                "PageDown",
              ].includes(e.key)
            )
              onApply();
          }}
        />
        <div className="wb-percent">
          <input
            id={`param-${attribute.key}`}
            aria-label={`${attribute.label}百分比`}
            type="number"
            min={0}
            max={100}
            step="any"
            placeholder={mixed ? "混合" : ""}
            required={!mixed || draft !== undefined}
            value={percent}
            onChange={(e) => put(attribute.key, e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Escape") {
                e.preventDefault();
                e.stopPropagation();
                cancel(attribute.key);
              }
            }}
          />
          <span>%</span>
        </div>
      </div>
      <div className="wb-parameter-actions">
        <button
          type="button"
          onClick={() => {
            put(attribute.key, 0);
            onApply();
          }}
        >
          归零
        </button>
        <button
          type="button"
          onClick={() => {
            put(attribute.key, 65535);
            onApply();
          }}
        >
          {attribute.key === "dimmer" ? "全亮" : "100%"}
        </button>
        <button
          type="button"
          title="释放此属性的场景控制权"
          onClick={() => {
            put(attribute.key, { mode: "release" });
            onApply();
          }}
        >
          释放
        </button>
        <button
          type="button"
          title="从场景中移除此属性记录"
          onClick={() => {
            put(attribute.key, { mode: "remove" });
            onApply();
          }}
        >
          清除
        </button>
      </div>
    </div>
  );
}
