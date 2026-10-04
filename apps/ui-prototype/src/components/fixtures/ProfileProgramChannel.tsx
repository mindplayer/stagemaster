import type { ProfileDraft } from "../../fixture-tools";
import type { ChannelDraft } from "../../fixture-function-draft";
import {
  addProgramChannel,
  newProgramFunction,
  programKey,
  programLabel,
  programKind,
} from "../../fixture-program";
import { ProfileChannelFields } from "./ProfileChannelFields";
import { ProfileFunctionRows } from "./ProfileFunctionRows";

export function ProfileProgramChannel({
  value,
  setDraft,
}: {
  value: ProfileDraft;
  setDraft(value: ProfileDraft): void;
}) {
  return (
    <section className="profile-functions" aria-label="内置程序通道">
      <h3>外部控制与禁用程序区间</h3>
      <button
        type="button"
        disabled={value.channels.some((c) => c.attribute === programKey)}
        onClick={() => setDraft(addProgramChannel(value))}
      >
        添加内置程序
      </button>
      <p className="wb-dim">
        演出仅允许外部通道控制。声控、内置自走等会接管灯具的档位只作禁用资料，不能选择、记录或播放。按说明书填写确认区间，未知控制宏、复位或锁存请勿填入。
      </p>
      {value.channels.map((c, i) => {
        if (c.attribute !== programKey) return null;
        const prefix = `channel-${i}`,
          functions = c.functions ?? [];
        const change = (patch: Partial<ChannelDraft>) =>
          setDraft({
            ...value,
            channels: value.channels.map((old, j) =>
              i === j ? { ...old, ...patch } : old,
            ),
          });
        return (
          <section
            className="profile-function-card"
            aria-label="内置程序功能定义"
            key={c.attribute}
          >
            <ProfileChannelFields channel={c} index={i} change={change}>
              <button
                type="button"
                onClick={() =>
                  setDraft({
                    ...value,
                    channels: value.channels.filter((_, j) => i !== j),
                  })
                }
              >
                移除内置程序
              </button>
            </ProfileChannelFields>
            <p>
              默认及唯一可执行档位：外部通道控制。自动／声控区间已屏蔽，不会作为场景功能。
            </p>
            <ProfileFunctionRows
              prefix={prefix}
              channelLabel={programLabel}
              colorWheel={false}
              max={c.bits === "16" ? 65535 : 255}
              functions={functions}
              onChange={(next) => change({ functions: next })}
              fixedOnly
              preserveKey="external"
              kindLabel={programKind}
            />
            <div className="profile-function-footer">
              {(["auto", "sound"] as const).map((kind) => (
                <button
                  type="button"
                  key={kind}
                  name={`${prefix}-function-add`}
                  disabled={functions.length >= 64}
                  onClick={() =>
                    change({
                      functions: [...functions, newProgramFunction(kind)],
                    })
                  }
                >
                  登记{kind === "auto" ? "自走" : "声控"}禁用区间
                </button>
              ))}
            </div>
          </section>
        );
      })}
    </section>
  );
}
