import type { ProfileDraft } from "../../fixture-tools";
import { addOpticsChannel, opticsLabels } from "../../fixture-optics";
import { ProfileChannelFields } from "./ProfileChannelFields";
export function ProfileOpticsChannels({
  value,
  setDraft,
}: {
  value: ProfileDraft;
  setDraft(value: ProfileDraft): void;
}) {
  return (
    <section aria-label="镜头与光圈通道">
      <h3>镜头与光圈</h3>
      <div className="profile-actions">
        {Object.entries(opticsLabels).map(([key, label]) => (
          <button
            type="button"
            key={key}
            disabled={value.channels.some((c) => c.attribute === key)}
            onClick={() => setDraft(addOpticsChannel(value, key))}
          >
            添加{label}
          </button>
        ))}
      </div>
      <p className="wb-dim">
        按说明书填写全范围线性通道。百分比是通道控制位置，不代表实际光束角或光圈开度；三维暂不模拟这些光学效果。
      </p>
      {value.channels.map(
        (c, i) =>
          Object.hasOwn(opticsLabels, c.attribute) && (
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
                      (old) => old.attribute !== c.attribute,
                    ),
                  })
                }
              >
                移除{opticsLabels[c.attribute]}
              </button>
            </div>
          ),
      )}
    </section>
  );
}
