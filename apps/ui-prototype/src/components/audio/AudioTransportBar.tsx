import type { AudioCommand, AudioPosition } from "../../audio-types";
import { audioTime } from "../../audio-tools";
export function AudioTransportBar({
  position,
  command,
  ready,
  duration,
  markerCount,
  blocked,
  addMarker,
  editingOnly = false,
}: {
  position: AudioPosition;
  command(value: AudioCommand): Promise<void>;
  ready: boolean;
  duration: number;
  markerCount: number;
  blocked: boolean;
  addMarker(): Promise<void>;
  editingOnly?: boolean;
}) {
  return (
    <div className="audio-transport">
      {editingOnly && <strong>音乐时间线</strong>}
      {!editingOnly && (
        <>
          <button
            className="primary"
            disabled={blocked || !ready}
            onClick={() =>
              command({
                kind: position.playing ? "pause" : "play",
              })
            }
          >
            {position.playing ? "暂停" : "播放"}
          </button>
          <button disabled={blocked} onClick={() => command({ kind: "stop" })}>
            停止
          </button>
          <output>
            {audioTime(position.positionMs)}
            <small> / {audioTime(duration)}</small>
          </output>
        </>
      )}
      <button
        disabled={blocked || markerCount >= 512}
        onClick={() => void addMarker()}
      >
        添加卡点 <kbd>M</kbd>
      </button>
      <label className="audio-volume">
        试听音量
        <input
          type="range"
          aria-label="试听音量"
          min="0"
          max="100"
          value={position.volumePercent}
          onChange={(e) =>
            void command({
              kind: "volume",
              percent: Number(e.target.value),
            })
          }
        />
      </label>
    </div>
  );
}
