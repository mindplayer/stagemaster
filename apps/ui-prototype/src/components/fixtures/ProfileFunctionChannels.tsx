import { ProfileSlotBatch } from "./ProfileSlotBatch";
import type { ProfileDraft } from "../../fixture-tools";
import {
  addFunctionChannel,
  draftInitial,
  newFunction,
} from "../../fixture-function-draft";
import type { ChannelDraft } from "../../fixture-function-draft";
import { functionLabels } from "../../fixture-function-types";
import { ProfileChannelFields } from "./ProfileChannelFields";
import { ProfileFunctionRows } from "./ProfileFunctionRows";
import "./profile-functions.css";
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
        {Object.entries(functionLabels).map(([key, label]) => (
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
      {value.channels.map((c, i) => {
        if (!(c.attribute in functionLabels)) return null;
        const label = functionLabels[c.attribute],
          prefix = `channel-${i}`,
          functions = c.functions ?? [];
        const change = (patch: Partial<ChannelDraft>) =>
          setDraft({
            ...value,
            channels: value.channels.map((old, j) =>
              i === j ? { ...old, ...patch } : old,
            ),
          });
        const selected = functions.find(
          (f) => f.key === c.defaultFunction?.functionKey,
        );
        return (
          <section
            className="profile-function-card"
            key={c.attribute}
            aria-label={`${label}功能定义`}
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
                移除{label}
              </button>
            </ProfileChannelFields>
            {c.attribute === "color-wheel" && (
              <p className="wb-dim">
                按实际档位标记；色块仅作屏幕识别，未确认的档位保持未标记。自动换色区间单独定义。
              </p>
            )}
            <ProfileSlotBatch
              channel={c}
              prefix={prefix}
              label={label}
              onChange={change}
            />
            <ProfileFunctionRows
              prefix={prefix}
              channelLabel={label}
              colorWheel={c.attribute === "color-wheel"}
              max={c.bits === "16" ? 65535 : 255}
              functions={functions}
              onChange={(next) => change({ functions: next })}
            />
            <div className="profile-function-footer">
              <button
                type="button"
                name={`${prefix}-function-add`}
                disabled={functions.length >= 64}
                onClick={() =>
                  change({
                    functions: [
                      ...functions,
                      newFunction(
                        Math.min(
                          c.bits === "16" ? 65535 : 255,
                          Math.max(
                            -1,
                            ...functions.map((f) => Number(f.dmxTo) || 0),
                          ) + 1,
                        ),
                      ),
                    ],
                  })
                }
              >
                添加区间
              </button>
              <label>
                默认功能
                <select
                  name={`${prefix}-default-function`}
                  aria-label={`${label}默认功能`}
                  value={selected?.key ?? ""}
                  onChange={(e) => {
                    const f = functions.find((f) => f.key === e.target.value);
                    if (f) change({ defaultFunction: draftInitial(f) });
                  }}
                >
                  <option value="" disabled>
                    请选择默认功能
                  </option>
                  {functions.map((f, j) => (
                    <option key={f.key} value={f.key}>
                      {f.name || `未命名功能 ${j + 1}`}
                    </option>
                  ))}
                </select>
              </label>
              {selected?.mode === "range" && (
                <label>
                  默认区间位置（0–65535）
                  <input
                    type="number"
                    min={0}
                    max={65535}
                    step={1}
                    required
                    name={`${prefix}-default-position`}
                    aria-label={`${label}默认区间位置`}
                    value={c.defaultFunction?.position ?? ""}
                    onChange={(e) =>
                      change({
                        defaultFunction: {
                          functionKey: selected.key,
                          position: e.target.value,
                        },
                      })
                    }
                  />
                </label>
              )}
              {selected?.mode === "slot" &&
                c.defaultFunction?.position !== "0" && (
                  <button
                    type="button"
                    name={`${prefix}-default-position`}
                    onClick={() =>
                      change({ defaultFunction: draftInitial(selected) })
                    }
                  >
                    使用档位代表值
                  </button>
                )}
            </div>
          </section>
        );
      })}
    </section>
  );
}
