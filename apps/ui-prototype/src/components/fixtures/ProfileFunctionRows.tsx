import { ProfileWheelAppearance } from "./ProfileWheelAppearance";
import type { FunctionDraft } from "../../fixture-function-draft";
export function ProfileFunctionRows({
  functions,
  prefix,
  channelLabel,
  colorWheel,
  max,
  onChange,
  fixedOnly = false,
  preserveKey,
  kindLabel,
}: {
  functions: FunctionDraft[];
  prefix: string;
  channelLabel: string;
  colorWheel: boolean;
  max: number;
  onChange(functions: FunctionDraft[]): void;
  fixedOnly?: boolean;
  preserveKey?: string;
  kindLabel?(key: string): string;
}) {
  return (
    <div className="profile-function-rows">
      {functions.map((f, j) => {
        const field = `${prefix}-function-${j}`;
        const change = (patch: Partial<FunctionDraft>) =>
          onChange(
            functions.map((old, i) => (i === j ? { ...old, ...patch } : old)),
          );
        return (
          <div className="profile-function-row" key={f.key}>
            {kindLabel && <span>{kindLabel(f.key)}</span>}
            <label>
              功能名称
              <input
                name={`${field}-name`}
                aria-label={`${channelLabel}功能 ${j + 1} 名称`}
                required
                maxLength={256}
                placeholder="按通道表填写"
                value={f.name}
                onChange={(e) => change({ name: e.target.value })}
              />
            </label>
            <label>
              控制方式
              <select
                name={`${field}-mode`}
                disabled={fixedOnly}
                value={f.mode}
                aria-label={`${channelLabel}功能 ${j + 1} 控制方式`}
                onChange={(e) =>
                  change({ mode: e.target.value as "slot" | "range" })
                }
              >
                <option value="slot">固定档位</option>
                {!fixedOnly && <option value="range">区间调节</option>}
              </select>
            </label>
            {(
              [
                ["dmxFrom", "起点"],
                ["dmxTo", "终点"],
                ["dmxDefault", "代表值"],
              ] as const
            ).map(([key, label]) => (
              <label key={key}>
                {label}
                <input
                  type="number"
                  name={`${field}-${key}`}
                  aria-label={`${channelLabel}功能 ${j + 1} ${label}`}
                  min={0}
                  max={max}
                  step={1}
                  value={f[key]}
                  required
                  onChange={(e) => change({ [key]: e.target.value })}
                />
              </label>
            ))}
            <button
              type="button"
              aria-label={`删除${channelLabel}功能 ${j + 1}`}
              disabled={f.key === preserveKey}
              onClick={() => onChange(functions.filter((_, i) => i !== j))}
            >
              删除
            </button>
            {colorWheel && (f.mode === "slot" || f.appearance) && (
              <ProfileWheelAppearance
                field={field}
                label={`${channelLabel}功能 ${j + 1} `}
                value={f.appearance}
                onChange={(appearance) => change({ appearance })}
              />
            )}
          </div>
        );
      })}
    </div>
  );
}
