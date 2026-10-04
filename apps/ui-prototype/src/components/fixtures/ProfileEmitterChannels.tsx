import { profileChannelLabel, type ProfileDraft } from "../../fixture-tools";
import {
  addEmitter,
  emitterFamilies,
  emitterFamily,
  removeEmitter,
  splitEmitterAttribute,
  withEmitterFamily,
} from "../../fixture-emitters";
import { ProfileChannelFields } from "./ProfileChannelFields";
import { isEmitterFunction } from "../../fixture-emitter-keys";
import { ProfileEmitterFunctions } from "./ProfileEmitterFunctions";

export function ProfileEmitterChannels({
  value,
  setDraft,
}: {
  value: ProfileDraft;
  setDraft(value: ProfileDraft): void;
}) {
  const rootColor = value.channels.some((c) =>
    ["red", "green", "blue"].includes(c.attribute),
  );
  return (
    <section aria-label="独立光源">
      <h3>独立光源</h3>
      <p className="wb-dim">
        同一灯具内分别编排调光与
        RGBW，不创建第二台灯或地址。先将基础功能组合改为调光再添加；没有真实总调光时选择“无独立总调光通道”。白光及受控快门／轮盘分别编排，三维暂不模拟这些光源。
      </p>
      <button
        type="button"
        name="emitter-add"
        disabled={rootColor || (value.emitters?.length ?? 0) >= 32}
        onClick={() => setDraft(addEmitter(value))}
      >
        添加独立光源
      </button>
      {value.emitters?.map((unit, index) => (
        <section key={index} aria-label={`光源 ${index + 1} 设置`}>
          <div className="profile-meta">
            <label>
              光源名称
              <input
                aria-label={`光源 ${index + 1} 名称`}
                name={`emitter-${index}-name`}
                maxLength={64}
                value={unit.name}
                onChange={(e) =>
                  setDraft({
                    ...value,
                    emitters: value.emitters!.map((u, i) =>
                      i === index ? { ...u, name: e.target.value } : u,
                    ),
                  })
                }
              />
            </label>
            <label>
              稳定标识
              <input
                aria-label={`光源 ${index + 1} 稳定标识`}
                name={`emitter-${index}-key`}
                readOnly
                value={unit.key}
              />
            </label>
            <label>
              光源功能
              <select
                aria-label={`光源 ${index + 1} 功能`}
                name={`emitter-${index}-family`}
                value={emitterFamily(value, unit.key)}
                onChange={(e) =>
                  setDraft(withEmitterFamily(value, unit.key, e.target.value))
                }
              >
                {emitterFamilies.map((f) => (
                  <option key={f.id} value={f.id}>
                    {f.name}
                  </option>
                ))}
              </select>
            </label>
            <button
              type="button"
              title="无总调光时至少保留一个光源；新增总调光须有实际通道依据"
              disabled={
                value.emitters!.length === 1 &&
                !value.channels.some((c) => c.attribute === "dimmer")
              }
              onClick={() => setDraft(removeEmitter(value, unit.key))}
            >
              移除光源 {index + 1}
            </button>
          </div>
          <p className="wb-dim">
            稳定标识只读，名称可改；使用中的模式须复制再替换。新增属性的默认值必须按资料明确填写。
          </p>
          <div className="profile-channels">
            {value.channels.map(
              (c, i) =>
                splitEmitterAttribute(c.attribute)?.owner === unit.key &&
                !isEmitterFunction(c.attribute) && (
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
          <ProfileEmitterFunctions
            value={value}
            setDraft={setDraft}
            owner={unit.key}
          />
        </section>
      ))}
    </section>
  );
}
