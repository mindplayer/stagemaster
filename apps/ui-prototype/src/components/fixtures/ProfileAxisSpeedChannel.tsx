import type { ProfileDraft } from "../../fixture-tools";
import {
  addAxisSpeedChannel,
  axisSpeedKey,
  axisSpeedLabel,
  hasBothAxes,
} from "../../fixture-axis-speed";
import { ProfileChannelFields } from "./ProfileChannelFields";

export function ProfileAxisSpeedChannel({
  value,
  setDraft,
}: {
  value: ProfileDraft;
  setDraft(value: ProfileDraft): void;
}) {
  const present = value.channels.some((c) => c.attribute === axisSpeedKey);
  return (
    <section aria-label="两轴速度控制通道">
      <h3>{axisSpeedLabel}</h3>
      <button
        type="button"
        disabled={!hasBothAxes(value) || present}
        onClick={() => setDraft(addAxisSpeedChannel(value))}
      >
        添加{axisSpeedLabel}
      </button>
      <p className="wb-dim">
        先定义完整两轴，按说明书填写全范围速度通道和默认值。百分比只表示通道控制位置，不代表实际速度或渐变时长；未知快慢方向不自动推断。取消两轴会移除这个速度通道。
      </p>
      {value.channels.map(
        (c, i) =>
          c.attribute === axisSpeedKey && (
            <div key={c.attribute}>
              <ProfileChannelFields
                channel={c}
                index={i}
                change={(patch) =>
                  setDraft({
                    ...value,
                    channels: value.channels.map((old, j) =>
                      i === j ? { ...old, ...patch } : old,
                    ),
                  })
                }
              />
              <button
                type="button"
                onClick={() =>
                  setDraft({
                    ...value,
                    channels: value.channels.filter(
                      (old) => old.attribute !== axisSpeedKey,
                    ),
                  })
                }
              >
                移除{axisSpeedLabel}
              </button>
            </div>
          ),
      )}
    </section>
  );
}
