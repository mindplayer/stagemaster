import type { ProfileDraft } from "../../fixture-tools";
import {
  draftInitial,
  newFunction,
  type ChannelDraft,
} from "../../fixture-function-draft";
import { attributeBase, isEmitterFunction } from "../../fixture-emitter-keys";
import {
  controlledFunctionOptions,
  controlledKind,
  newControlledFunction,
} from "../../fixture-emitter-functions";
import { ProfileChannelFields } from "./ProfileChannelFields";
import { ProfileFunctionRows } from "./ProfileFunctionRows";
import { ProfileSlotBatch } from "./ProfileSlotBatch";
import "./profile-functions.css";

export function ProfileFunctionCard({
  channel: c,
  index: i,
  value,
  setDraft,
  label,
}: {
  channel: ChannelDraft;
  index: number;
  value: ProfileDraft;
  setDraft(value: ProfileDraft): void;
  label: string;
}) {
  const prefix = `channel-${i}`,
    functions = c.functions ?? [],
    scoped = isEmitterFunction(c.attribute);
  const selected = functions.find(
    (f) => f.key === c.defaultFunction?.functionKey,
  );
  const change = (patch: Partial<ChannelDraft>) =>
    setDraft({
      ...value,
      channels: value.channels.map((old, j) =>
        i === j ? { ...old, ...patch } : old,
      ),
    });
  const colorWheel = attributeBase(c.attribute) === "color-wheel";
  return (
    <section className="profile-function-card" aria-label={`${label}功能定义`}>
      <ProfileChannelFields channel={c} index={i} label={label} change={change}>
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
      {colorWheel && (
        <p className="wb-dim">
          按实际档位标记；色块仅作屏幕识别，未确认档位保持未标记。
        </p>
      )}
      {scoped && (
        <p className="wb-dim">
          仅开放明确开闭、受控频闪、固定轮盘及单图案抖动。自动来回切换、声控、自走、随机、复位和未知宏不开放；原生区间必须按资料填写，百分比不是
          Hz。
        </p>
      )}
      {(!scoped || attributeBase(c.attribute) !== "shutter") && (
        <ProfileSlotBatch
          channel={c}
          prefix={prefix}
          label={label}
          onChange={change}
        />
      )}
      <ProfileFunctionRows
        prefix={prefix}
        channelLabel={label}
        colorWheel={colorWheel}
        max={c.bits === "16" ? 65535 : 255}
        functions={functions}
        lockedMode={scoped}
        kindLabel={
          scoped
            ? (key) =>
                controlledFunctionOptions(c.attribute).find(
                  (o) => o.id === controlledKind(key),
                )?.name ?? "不支持的功能"
            : undefined
        }
        onChange={(next) => change({ functions: next })}
      />
      <div className="profile-function-footer">
        {scoped ? (
          controlledFunctionOptions(c.attribute).map((option) => (
            <button
              type="button"
              key={option.id}
              name={`${prefix}-function-add`}
              disabled={
                functions.length >= 64 ||
                (!["slot", "shake"].includes(option.id) &&
                  functions.some((f) => f.key === option.id))
              }
              onClick={() =>
                change({
                  functions: [
                    ...functions,
                    newControlledFunction(c.attribute, option.id),
                  ],
                })
              }
            >
              添加{option.name}
            </button>
          ))
        ) : (
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
        )}
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
        {selected?.mode === "slot" && c.defaultFunction?.position !== "0" && (
          <button
            type="button"
            name={`${prefix}-default-position`}
            onClick={() => change({ defaultFunction: draftInitial(selected) })}
          >
            使用档位代表值
          </button>
        )}
      </div>
    </section>
  );
}
