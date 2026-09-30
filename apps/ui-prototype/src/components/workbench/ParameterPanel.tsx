import { useImperativeHandle, useRef, useState } from "react";
import type { Ref } from "react";
import type {
  EditOperation,
  FixtureView,
  SceneView,
} from "../../application-host";
import {
  attributeState,
  commonAttributes,
  parameterCommands,
} from "../../editor-tools";
import type { ParameterDraft } from "../../editor-tools";

import { ParameterColor } from "./ParameterColor";
import { validateEditorForm } from "./form-validation";

export interface ParameterHandle {
  collect(): EditOperation[];
  accept(): void;
}
export function ParameterPanel({
  ref,
  fixtures,
  scene,
  busy,
  onApply,
  onPending,
}: {
  ref: Ref<ParameterHandle>;
  fixtures: FixtureView[];
  scene: SceneView;
  busy: boolean;
  onApply(): void;
  onPending(value: boolean): void;
}) {
  const [drafts, setDrafts] = useState<Record<string, ParameterDraft>>({});
  const draftRef = useRef(drafts);
  const fields = useRef<HTMLFormElement>(null);
  const gesture = useRef<{
    key: string;
    drafts: Record<string, ParameterDraft>;
    hex: string | null;
    cancelled: boolean;
  } | null>(null);
  const [hex, setHex] = useState<string | null>(null);
  const hexRef = useRef<string | null>(null);
  function change(
    next: Record<string, ParameterDraft>,
    color = hexRef.current,
  ) {
    if (!Object.keys(next).length && color === null) {
      fields.current
        ?.querySelectorAll<HTMLInputElement>("input")
        .forEach((input) => input.setCustomValidity(""));
    }
    draftRef.current = next;
    setDrafts(next);
    hexRef.current = color;
    setHex(color);
    onPending(Object.keys(next).length > 0 || color !== null);
  }
  function put(key: string, value: ParameterDraft) {
    change(
      { ...draftRef.current, [key]: value },
      ["red", "green", "blue"].includes(key) ? null : hexRef.current,
    );
  }
  function cancel(key?: string) {
    if (!key) change({}, null);
    else {
      const next = { ...draftRef.current };
      delete next[key];
      change(next);
    }
  }
  const attributes = commonAttributes(fixtures).filter(
    (a) =>
      !fixtures.every((f) => f.positioning) ||
      (a.key !== "pan" && a.key !== "tilt"),
  );
  const rgb = ["red", "green", "blue"].every((key) =>
    attributes.some((a) => a.key === key),
  );
  function resolvedValue(key: string) {
    const draft = drafts[key];
    if (typeof draft === "number") return draft;
    if (
      typeof draft === "string" &&
      draft.trim() &&
      Number.isFinite(Number(draft))
    )
      return Math.round((Number(draft) * 65535) / 100);
    return attributeState(scene, fixtures, key).value;
  }
  const currentColor = rgb
    ? `#${["red", "green", "blue"]
        .map((key) =>
          Math.round(resolvedValue(key) / 257)
            .toString(16)
            .padStart(2, "0"),
        )
        .join("")}`
    : "#000000";
  const mixedColor =
    rgb &&
    ["red", "green", "blue"].some(
      (key) =>
        drafts[key] === undefined && attributeState(scene, fixtures, key).mixed,
    );
  function colorDraft(value: string) {
    const next = { ...draftRef.current };
    if (/^#[a-f\d]{6}$/i.test(value))
      ["red", "green", "blue"].forEach(
        (key, i) =>
          (next[key] = parseInt(value.slice(1 + i * 2, 3 + i * 2), 16) * 257),
      );
    change(next, value);
  }
  useImperativeHandle(ref, () => ({
    collect() {
      if (!Object.keys(draftRef.current).length && hexRef.current === null)
        return [];
      validateEditorForm(fields.current);
      return parameterCommands(scene.id, fixtures, draftRef.current);
    },
    accept() {
      change({}, null);
    },
  }));
  function renderAttribute(attribute: FixtureView["attributes"][number]) {
    const state = attributeState(scene, fixtures, attribute.key);
    const draft = drafts[attribute.key];
    const mode =
      typeof draft === "object"
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
        effect.channels.some(
          (channel) => channel.attribute === attribute.key,
        ) &&
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
                key: attribute.key,
                drafts: { ...draftRef.current },
                hex: hexRef.current,
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
              if (start) change(start.drafts, start.hex);
            }}
            onLostPointerCapture={() => {
              const start = gesture.current;
              gesture.current = null;
              if (start) change(start.drafts, start.hex);
            }}
            onKeyDown={(e) => {
              if (e.key === "Escape") {
                e.preventDefault();
                e.stopPropagation();
                const start = gesture.current;
                if (start) {
                  start.cancelled = true;
                  change(start.drafts, start.hex);
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

  if (!fixtures.length)
    return <div className="wb-empty wb-compact">选择灯具以编辑属性</div>;
  return (
    <form
      ref={fields}
      noValidate
      onInputCapture={(e) => {
        if (e.target instanceof HTMLInputElement)
          e.target.setCustomValidity("");
      }}
      className="wb-parameters"
      onSubmit={(e) => {
        e.preventDefault();
        onApply();
      }}
    >
      <div className="wb-section-title">
        <h2>灯光属性</h2>
        <span>{fixtures.length} 台已选</span>
      </div>
      <p
        className="wb-target-names"
        title={fixtures.map((f) => f.name).join("、")}
      >
        {fixtures.map((f) => f.name).join("、")}
      </p>
      <fieldset disabled={busy}>
        {attributes.filter((a) => a.key === "dimmer").map(renderAttribute)}
        {rgb && (
          <ParameterColor
            color={currentColor}
            mixed={mixedColor}
            draft={hex}
            onChange={colorDraft}
            onCancel={() => {
              const next = { ...draftRef.current };
              for (const key of ["red", "green", "blue"]) delete next[key];
              change(next, null);
            }}
          />
        )}
        {attributes.filter((a) => a.key !== "dimmer").map(renderAttribute)}
        {!attributes.length && <p className="wb-dim">选中的灯具没有共同属性</p>}
        {(Object.keys(drafts).length > 0 || hex !== null) && (
          <div className="wb-form-actions">
            <button type="submit" className="wb-primary">
              应用属性
            </button>
            <button
              type="button"
              onMouseDown={(e) => e.preventDefault()}
              onClick={() => cancel()}
            >
              还原属性
            </button>
          </div>
        )}
      </fieldset>
    </form>
  );
}
