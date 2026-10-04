import { profileChannelLabel, type ProfileDraft } from "../../fixture-tools";
import { addFunctionChannel } from "../../fixture-function-draft";
import { emitterFunctionLabels } from "../../fixture-emitter-functions";
import {
  isEmitterFunction,
  splitEmitterAttribute,
} from "../../fixture-emitter-keys";
import { ProfileFunctionCard } from "./ProfileFunctionCard";
export function ProfileEmitterFunctions({
  value,
  setDraft,
  owner,
}: {
  value: ProfileDraft;
  setDraft(value: ProfileDraft): void;
  owner: string;
}) {
  const name = value.emitters?.find((u) => u.key === owner)?.name ?? "光源";
  return (
    <section className="profile-functions" aria-label={`${name}功能通道`}>
      <div className="profile-actions">
        {Object.entries(emitterFunctionLabels).map(([base, label]) => {
          const key = `emitter.${owner}.${base}`;
          return (
            <button
              type="button"
              key={key}
              disabled={value.channels.some((c) => c.attribute === key)}
              onClick={() => setDraft(addFunctionChannel(value, key))}
            >
              添加{name} · {label}
            </button>
          );
        })}
      </div>
      {value.channels.map((c, i) =>
        isEmitterFunction(c.attribute) &&
        splitEmitterAttribute(c.attribute)?.owner === owner ? (
          <ProfileFunctionCard
            key={c.attribute}
            channel={c}
            index={i}
            value={value}
            setDraft={setDraft}
            label={profileChannelLabel(value, c.attribute)}
          />
        ) : null,
      )}
    </section>
  );
}
