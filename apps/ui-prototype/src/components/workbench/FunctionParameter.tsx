import { WheelSlotChoices } from "./WheelSlotChoices";
import type { FixtureView, SceneView } from "../../application-host";
import type { ParameterDraft } from "../../editor-tools";
import { initialFunction } from "../../fixture-function-types";
import { functionState } from "../../function-parameter-tools";
import { programKey } from "../../fixture-program-rules";
import {
  functionSelectionAllowed,
  functionSafetyMessage,
} from "../../fixture-function-safety";
export function FunctionParameter({
  attribute,
  fixtures,
  scene,
  draft,
  put,
  cancel,
  onApply,
}: {
  attribute: FixtureView["attributes"][number];
  fixtures: FixtureView[];
  scene: SceneView;
  draft: ParameterDraft | undefined;
  put(key: string, draft: ParameterDraft): void;
  cancel(key: string): void;
  onApply(): void;
}) {
  const spec = attribute.function!,
    state = functionState(scene, fixtures, attribute.key);
  const pending =
    typeof draft === "object" && "function" in draft ? draft : undefined;
  const selection = pending?.function ?? state.selection;
  const mixed = state.mixed && draft === undefined;
  const chosen = spec.functions.find((f) => f.key === selection.functionKey);
  const selectable = spec.functions.filter((f) =>
    functionSelectionAllowed(attribute.key, f),
  );
  const mode =
    typeof draft === "object" && "mode" in draft
      ? draft.mode
      : pending
        ? "literal"
        : state.mode;
  const status = mixed
    ? "混合值"
    : mode === "release"
      ? "释放"
      : mode === "remove" || mode === "absent"
        ? "未记录"
        : mode === "preset"
          ? `预设 · ${state.presetName}`
          : "已记录";
  const percent =
    pending?.percent ??
    String(Number(((selection.position * 100) / 65535).toFixed(6)));
  const escape = (e: React.KeyboardEvent) => {
    if (e.key === "Escape") {
      e.preventDefault();
      e.stopPropagation();
      cancel(attribute.key);
    }
  };
  return (
    <div className="wb-parameter wb-function-parameter">
      <div className="wb-section-title">
        <label htmlFor={`param-${attribute.key}`}>{attribute.label}</label>
        <span>{status}</span>
      </div>
      <select
        id={`param-${attribute.key}`}
        aria-label={`${attribute.label}功能`}
        value={mixed ? "" : selection.functionKey}
        onKeyDown={escape}
        onChange={(e) => {
          const f = selectable.find((f) => f.key === e.target.value);
          if (f) put(attribute.key, { function: initialFunction(f) });
        }}
      >
        {mixed && (
          <option value="" disabled>
            混合功能
          </option>
        )}
        {selectable.map((f) => (
          <option key={f.key} value={f.key}>
            {f.name}
          </option>
        ))}
      </select>
      {attribute.key === programKey && (
        <p className="wb-dim">
          声控、内置自走等自主档位已屏蔽，不能用于编排或播放。仅允许外部通道控制；释放遵循下层／默认，不是复位或机械急停。
        </p>
      )}
      {attribute.key !== programKey &&
        selectable.length < spec.functions.length && (
          <p className="wb-dim">
            {functionSafetyMessage}。禁用区间仅保留资料。
          </p>
        )}
      {attributeBase(attribute.key) === "color-wheel" && (
        <WheelSlotChoices
          functions={selectable}
          selected={mixed ? undefined : selection.functionKey}
          onSelect={(f) => {
            put(attribute.key, { function: initialFunction(f) });
            onApply();
          }}
        />
      )}
      {!mixed && chosen?.mode === "range" && (
        <label className="wb-function-position">
          区间位置
          <div className="wb-percent">
            <input
              type="number"
              min={0}
              max={100}
              step="any"
              required
              aria-label={`${attribute.label}区间百分比`}
              value={percent}
              onKeyDown={escape}
              onChange={(e) =>
                put(attribute.key, {
                  function: selection,
                  percent: e.target.value,
                })
              }
            />
            <span>%</span>
          </div>
        </label>
      )}
      <div className="wb-parameter-actions">
        <button
          type="button"
          onClick={() => {
            put(attribute.key, { function: spec.default });
            onApply();
          }}
        >
          默认功能
        </button>
        <button
          type="button"
          onClick={() => {
            put(attribute.key, { mode: "release" });
            onApply();
          }}
        >
          释放
        </button>
        <button
          type="button"
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
import { attributeBase } from "../../fixture-emitter-keys";
