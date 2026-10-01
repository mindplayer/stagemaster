import { ContinuousParameter } from "./ContinuousParameter";
import { FunctionParameter } from "./FunctionParameter";
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
import {
  availableParameterCategories,
  visibleParameterAttributes,
} from "../../parameter-categories";
import { ParameterCategories } from "./ParameterCategories";

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
  beforeChange,
}: {
  ref: Ref<ParameterHandle>;
  fixtures: FixtureView[];
  scene: SceneView;
  busy: boolean;
  onApply(): void;
  onPending(value: boolean): void;
  beforeChange(): Promise<boolean>;
}) {
  const [category, setCategory] = useState("");
  const [switching, setSwitching] = useState(false);
  const switchingRef = useRef(false);
  const [drafts, setDrafts] = useState<Record<string, ParameterDraft>>({});
  const draftRef = useRef(drafts);
  const fields = useRef<HTMLFormElement>(null);
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
  const incompatible = [
    ...new Set(
      fixtures
        .flatMap((f) => f.attributes)
        .filter((a) => a.function && !attributes.some((b) => b.key === a.key))
        .map((a) => a.label),
    ),
  ];
  const categories = availableParameterCategories(attributes);
  const selectedCategory =
    category === "all" || categories.some((c) => c.id === category)
      ? category
      : (categories[0]?.id ?? "");
  const visibleAttributes = visibleParameterAttributes(
    attributes,
    selectedCategory,
  );
  const colorVisible =
    selectedCategory === "all" || selectedCategory === "color";
  async function changeCategory(next: string) {
    if (busy || switchingRef.current || next === selectedCategory) return;
    switchingRef.current = true;
    setSwitching(true);
    try {
      if (await beforeChange()) {
        setCategory(next);
        fields.current?.closest(".wb-properties")?.scrollTo({ top: 0 });
      }
    } finally {
      switchingRef.current = false;
      setSwitching(false);
    }
  }
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
    const shared = {
      attribute,
      fixtures,
      scene,
      draft: drafts[attribute.key],
      put,
      cancel,
      onApply,
    };
    return attribute.function ? (
      <FunctionParameter key={attribute.key} {...shared} />
    ) : (
      <ContinuousParameter
        key={attribute.key}
        {...shared}
        resolvedValue={resolvedValue}
        capture={() => {
          const before = { ...draftRef.current },
            color = hexRef.current;
          return () => change(before, color);
        }}
      />
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
      <ParameterCategories
        categories={categories}
        value={selectedCategory}
        busy={busy || switching}
        onChange={(next) => void changeCategory(next)}
      />
      <fieldset disabled={busy}>
        {visibleAttributes
          .filter((a) => a.key === "dimmer")
          .map(renderAttribute)}
        {rgb && colorVisible && (
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
        {visibleAttributes
          .filter((a) => a.key !== "dimmer")
          .map(renderAttribute)}
        {!!incompatible.length && (
          <p className="wb-dim">
            {incompatible.join("、")}
            未在全部所选灯具中使用相同定义，请分组选灯编辑。
          </p>
        )}
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
