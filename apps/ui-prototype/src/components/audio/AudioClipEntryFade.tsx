import type { ClipEntryFade } from "../../audio-types";
export function AudioClipEntryFade({
  fade,
  disabled,
  onReset,
}: {
  fade: ClipEntryFade;
  disabled: boolean;
  onReset(): void;
}) {
  return (
    <section aria-label="保留原渐变">
      <strong>保留原渐变</strong>
      <p>
        原长 {(fade.durationMs / 1000).toFixed(3)} 秒 · 已进行{" "}
        {(fade.offsetMs / 1000).toFixed(3)} 秒
      </p>
      <small>
        沿用剪辑时的起始灯光。移动、复制保留此状态；重新计算后，从当前位置的前段或灯具默认值渐变。
      </small>
      <button type="button" disabled={disabled} onClick={onReset}>
        重新计算进入渐变
      </button>
    </section>
  );
}
