import type { AudioMarker } from "../../audio-types";
import { audioTime } from "../../audio-tools";
import "./audio-scene-return.css";
export function AudioSceneReturn({
  marker,
  busy,
  onBack,
}: {
  marker: AudioMarker;
  busy: boolean;
  onBack(): void;
}) {
  return (
    <div
      className="audio-scene-return"
      role="region"
      aria-label="音乐编排上下文"
    >
      <button disabled={busy} onClick={onBack}>
        返回音乐时间线
      </button>
      <span title={marker.name}>{marker.name}</span>
      <time>{audioTime(marker.timeMs)}</time>
    </div>
  );
}
