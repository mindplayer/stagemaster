import { filteredAudioMarkers } from "../../audio-group-tools";
import type { AudioTimeline } from "../../audio-types";
import type { SceneView } from "../../application-host";
import { audioTime } from "../../audio-tools";
export function AudioMarkerList({
  track,
  scenes,
  selected,
  query,
  busy,
  onQuery,
  onSelect,
  onSeek,
}: {
  track: AudioTimeline;
  scenes: SceneView[];
  selected: string;
  query: string;
  busy: boolean;
  onQuery(query: string): void;
  onSelect(id: string): void;
  onSeek(timeMs: number): void;
}) {
  const markers = filteredAudioMarkers(track, scenes, query);
  return (
    <section className="audio-markers">
      <div className="audio-list-title">
        <h3>
          节奏与灯光 <small>{track.markers.length}</small>
        </h3>
        <input
          aria-label="搜索卡点"
          placeholder="搜索卡点或场景"
          value={query}
          onChange={(e) => onQuery(e.target.value)}
        />
      </div>
      {query && <button onClick={() => onQuery("")}>清除筛选</button>}
      <div className="audio-marker-list">
        {markers.map((m) => (
          <button
            key={m.id}
            data-reveal-id={m.id}
            aria-pressed={selected === m.id}
            className={selected === m.id ? "selected" : ""}
            disabled={busy}
            onClick={() => onSelect(m.id)}
            onDoubleClick={() => onSeek(m.timeMs)}
          >
            <time>{audioTime(m.timeMs)}</time>
            <strong>{m.name}</strong>
            <span>
              {scenes.find((s) => s.id === m.sceneId)?.name ?? "节奏标记"}
              {m.fadeMs ? ` · 渐变 ${(m.fadeMs / 1000).toFixed(3)} 秒` : ""}
            </span>
          </button>
        ))}
        {!markers.length && (
          <p>{track.markers.length ? "未找到卡点" : "播放时按 M 添加卡点"}</p>
        )}
      </div>
    </section>
  );
}
