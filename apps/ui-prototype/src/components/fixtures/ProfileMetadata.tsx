import type { ProfileDraft } from "../../fixture-tools";
import { withLinearFamily } from "../../fixture-function-draft";
import { numberInput } from "./ProfileNumberInput";
export function ProfileMetadata({
  value,
  setDraft,
}: {
  value: ProfileDraft;
  setDraft(value: ProfileDraft): void;
}) {
  return (
    <div className="profile-meta">
      {(
        [
          ["name", "模式名称"],
          ["manufacturer", "厂家"],
          ["model", "型号"],
          ["mode", "模式标识"],
        ] as const
      ).map(([key, label]) => (
        <label key={key}>
          {label}
          <input
            name={key}
            aria-label={label}
            required
            maxLength={256}
            value={value[key]}
            onChange={(e) => setDraft({ ...value, [key]: e.target.value })}
          />
        </label>
      ))}
      {numberInput("footprint", "占用通道数", value.footprint, 1, 512, (v) =>
        setDraft({ ...value, footprint: v }),
      )}
      <label>
        功能组合
        <select
          name="family"
          aria-label="功能组合"
          value={
            value.channels.some((c) => c.attribute === "red")
              ? value.channels.some((c) => c.attribute === "dimmer")
                ? "rgbd"
                : "rgb"
              : "dimmer"
          }
          onChange={(e) => setDraft(withLinearFamily(value, e.target.value))}
        >
          <option value="dimmer">调光</option>
          <option value="rgb">RGB 三原色</option>
          <option value="rgbd">调光与 RGB</option>
        </select>
      </label>
    </div>
  );
}
