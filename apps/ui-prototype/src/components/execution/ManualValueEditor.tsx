import type { ManualDraft, ManualFixture } from "../../execution-manual";
import { initialFunction } from "../../fixture-function-types";
import { WheelSlotChoices } from "../workbench/WheelSlotChoices";
import { functionSelectionAllowed } from "../../fixture-function-safety";
export function ManualValueEditor({
  attribute,
  draft,
  disabled,
  onChange,
}: {
  attribute: ManualFixture["attributes"][number];
  draft: ManualDraft;
  disabled: boolean;
  onChange(next: ManualDraft): void;
}) {
  const functions = attribute.function?.functions.filter((f) =>
    functionSelectionAllowed(attribute.key, f),
  );
  const selected = functions?.find((f) => f.key === draft.functionKey);
  const choose = (key: string) => {
    const f = functions?.find((f) => f.key === key);
    if (f)
      onChange({
        ...draft,
        functionKey: key,
        value: String((initialFunction(f).position * 100) / 65535),
      });
  };
  return (
    <fieldset disabled={disabled} className="execution-manual-value">
      <legend>待设置 · {attribute.label}</legend>
      {functions && (
        <>
          <label>
            功能
            <select
              aria-label="手动设置功能"
              value={draft.functionKey}
              onChange={(e) => choose(e.target.value)}
            >
              <option value="" disabled>
                选择功能
              </option>
              {functions.map((f) => (
                <option key={f.key} value={f.key}>
                  {f.name}
                </option>
              ))}
            </select>
          </label>
          {attributeBase(attribute.key) === "color-wheel" && (
            <WheelSlotChoices
              functions={functions}
              selected={draft.functionKey}
              onSelect={(f) => choose(f.key)}
            />
          )}
        </>
      )}
      {(!functions || selected?.mode === "range") && (
        <label>
          {functions ? "功能区间位置" : "百分比"}
          <input
            aria-label="手动设置百分比"
            type="number"
            min={0}
            max={100}
            step="any"
            value={draft.value}
            onChange={(e) => onChange({ ...draft, value: e.target.value })}
          />
          %
        </label>
      )}
    </fieldset>
  );
}
import { attributeBase } from "../../fixture-emitter-keys";
