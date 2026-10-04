import type { ProfileDraft } from "../../fixture-tools";
import { addFunctionChannel } from "../../fixture-function-draft";
import { functionLabels } from "../../fixture-function-types";
import { programKey } from "../../fixture-program";
import { ProfileFunctionCard } from "./ProfileFunctionCard";
export function ProfileFunctionChannels({
  value,
  setDraft,
}: {
  value: ProfileDraft;
  setDraft(value: ProfileDraft): void;
}) {
  return (
    <section className="profile-functions" aria-label="灯具功能通道">
      <h3>功能通道</h3>
      <div className="profile-actions">
        {Object.entries(functionLabels)
          .filter(([key]) => key !== programKey)
          .map(([key, label]) => (
            <button
              type="button"
              key={key}
              disabled={value.channels.some((c) => c.attribute === key)}
              onClick={() => setDraft(addFunctionChannel(value, key))}
            >
              添加{label}
            </button>
          ))}
      </div>
      {value.channels.map((c, i) =>
        c.attribute in functionLabels && c.attribute !== programKey ? (
          <ProfileFunctionCard
            key={c.attribute}
            channel={c}
            index={i}
            value={value}
            setDraft={setDraft}
            label={functionLabels[c.attribute]}
          />
        ) : null,
      )}
    </section>
  );
}
