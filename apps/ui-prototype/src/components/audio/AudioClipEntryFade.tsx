import type { ClipEntryFade, ClipEntryCrossfade } from "../../audio-types";
export function AudioClipEntryFade({
  fade,
  dynamic = false,
  sourceName,
  disabled,
  onReset,
}: {
  fade: ClipEntryFade | ClipEntryCrossfade;
  dynamic?: boolean;
  sourceName?: string;
  disabled: boolean;
  onReset(): void;
}) {
  return (
    <section aria-label={dynamic ? "保留原交叉" : "保留原渐变"}>
      <strong>{dynamic ? "保留原交叉" : "保留原渐变"}</strong>
      <p>
        原长 {(fade.durationMs / 1000).toFixed(3)} 秒 · 已进行{" "}
        {(fade.offsetMs / 1000).toFixed(3)} 秒
      </p>
      {dynamic && sourceName && <p>来源：{sourceName}</p>}
      <small>
        {dynamic ? "沿用剪辑时的来源场景与交叉进度，来源场景的效果修改仍会生效。" : "沿用剪辑时的起始灯光。"}
        重新计算后，使用当前位置的前段或灯具默认值。
      </small>
      <button type="button" disabled={disabled} onClick={onReset}>
        重新计算进入渐变
      </button>
    </section>
  );
}
