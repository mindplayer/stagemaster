import type { ReactNode } from "react";
import type { ChannelDraft } from "../../fixture-function-draft";
import { channelLabels } from "../../fixture-tools";
import { numberInput } from "./ProfileNumberInput";
export function ProfileChannelFields({
  channel: c,
  index: i,
  change,
  children,
  label: suppliedLabel,
}: {
  channel: ChannelDraft;
  index: number;
  change(patch: Partial<ChannelDraft>): void;
  children?: ReactNode;
  label?: string;
}) {
  const label = suppliedLabel ?? channelLabels[c.attribute];
  return (
    <div className="profile-channel">
      <strong>{label}</strong>
      <label>
        精度
        <select
          aria-label={`${label}精度`}
          value={c.bits}
          onChange={(e) => change({ bits: e.target.value as "8" | "16" })}
        >
          <option value="8">8 位</option>
          <option value="16">16 位</option>
        </select>
      </label>
      {numberInput(
        `channel-${i}-coarse`,
        `${label}粗调通道`,
        c.coarse,
        1,
        512,
        (v) => change({ coarse: v }),
      )}
      {c.bits === "16" ? (
        numberInput(
          `channel-${i}-fine`,
          `${label}细调通道`,
          c.fine,
          1,
          512,
          (v) => change({ fine: v }),
        )
      ) : (
        <span className="wb-dim">—</span>
      )}
      {children ??
        numberInput(
          `channel-${i}-percent`,
          `${label}默认值（%）`,
          c.percent,
          0,
          100,
          (v) => change({ percent: v }),
        )}
    </div>
  );
}
