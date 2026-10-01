import {
  presetScopeId,
  presetScopeOptions,
} from "../../preset-attribute-scopes";

export function PresetScopeSelect({
  label,
  available,
  selected,
  disabled,
  onChange,
}: {
  label: string;
  available: { key: string }[];
  selected: string[] | null;
  disabled?: boolean;
  onChange(keys: string[] | null): void;
}) {
  const choices = presetScopeOptions(available);
  const value = presetScopeId(selected, available);
  const empty =
    selected !== null && !available.some((a) => selected.includes(a.key));
  return (
    <select
      aria-label={label}
      disabled={disabled}
      value={value}
      onChange={(e) => {
        const scope = choices.find((s) => s.id === e.target.value);
        if (scope) onChange(scope.id === "all" ? null : [...scope.keys]);
      }}
    >
      {choices.map((scope) => (
        <option key={scope.id} value={scope.id} disabled={!scope.keys.length}>
          {scope.name}
        </option>
      ))}
      {value === "custom" && (
        <option value="custom" disabled>
          {empty ? "未选择属性" : "自选属性"}
        </option>
      )}
    </select>
  );
}
