import "./attribute-mask.css";
import { PresetScopeSelect } from "./PresetScopeSelect";

export function AttributeMask({
  available,
  selected,
  onChange,
}: {
  available: { key: string; label: string }[];
  selected: string[];
  onChange(ids: string[]): void;
}) {
  return (
    <div className="wb-attribute-mask" role="group" aria-label="属性范围">
      {available.map((a) => (
        <label key={a.key}>
          <input
            type="checkbox"
            checked={selected.includes(a.key)}
            onChange={(e) =>
              onChange(
                e.target.checked
                  ? [...selected, a.key]
                  : selected.filter((k) => k !== a.key),
              )
            }
          />
          {a.label}
        </label>
      ))}
      <PresetScopeSelect
        label="属性范围快捷选择"
        available={available}
        selected={selected}
        onChange={(keys) => onChange(keys ?? available.map((a) => a.key))}
      />
      <button
        type="button"
        disabled={!selected.length}
        onClick={() => onChange([])}
      >
        清空属性
      </button>
    </div>
  );
}
