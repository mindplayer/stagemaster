import { opticsLabels } from "../../fixture-optics";
import { axisSpeedKey } from "../../fixture-axis-speed";
import { profileChannelLabel, type ProfileDraft } from "../../fixture-tools";
import { functionLabels } from "../../fixture-function-types";
import { ProfileChannelFields } from "./ProfileChannelFields";
export function ProfileLinearChannels({
  value,
  setDraft,
}: {
  value: ProfileDraft;
  setDraft(value: ProfileDraft): void;
}) {
  return (
    <>
      <h3>线性属性与物理通道</h3>
      <div className="profile-channels">
        {value.channels.map((c, i) =>
          c.attribute.startsWith("emitter.") ||
          c.attribute in functionLabels ||
          c.attribute === axisSpeedKey ||
          Object.hasOwn(opticsLabels, c.attribute) ? null : (
            <ProfileChannelFields
              key={c.attribute}
              channel={c}
              index={i}
              label={profileChannelLabel(value, c.attribute)}
              change={(patch) =>
                setDraft({
                  ...value,
                  channels: value.channels.map((old, j) =>
                    i === j ? { ...old, ...patch } : old,
                  ),
                })
              }
            />
          ),
        )}
      </div>
    </>
  );
}
