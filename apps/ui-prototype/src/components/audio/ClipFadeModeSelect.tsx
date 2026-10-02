import type { ClipFadeMode } from "../../audio-types";
export function ClipFadeModeSelect({ value, disabled, optional = false, onChange }: {
  value: ClipFadeMode | "";
  disabled: boolean;
  optional?: boolean;
  onChange(value: ClipFadeMode | ""): void;
}) {
  return <label>
    过渡方式
    <select name="clipFadeMode" aria-label="灯光片段过渡方式" value={value} disabled={disabled}
      onChange={e => onChange(e.target.value as ClipFadeMode | "")}>
      {optional && <option value="">保持各片段方式</option>}
      <option value="snapshot">从前段结束值渐变</option>
      <option value="dynamic">前后动态效果交叉</option>
    </select>
  </label>;
}
