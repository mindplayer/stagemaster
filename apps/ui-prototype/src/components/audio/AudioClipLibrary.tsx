import "./audio-clips.css";
import { AudioClipBatch } from "./AudioClipBatch";
import { filteredClips } from "./clip-group-tools";
import type { AudioEdit } from "../../audio-types";
import type { ProjectView } from "../../application-host";
import { useState } from "react";
import type { AudioTimeline } from "../../audio-types";
import type { SceneView } from "../../application-host";
import { audioTime } from "../../audio-tools";
export function AudioClipLibrary({
  track,
  scenes,
  selected,
  busy,
  onSelect,
  onSeek,
  onAdd,
  onConvert,
  batch,
  onBatch,
  visible,
  onEdit,
}: {
  batch: boolean;
  onBatch(): void;
  visible: boolean;
  onEdit(command: AudioEdit): Promise<ProjectView | null>;
  track: AudioTimeline;
  scenes: SceneView[];
  selected: string;
  busy: boolean;
  onSelect(id: string): void;
  onSeek(time: number): void;
  onAdd(): void;
  onConvert(): void;
}) {
  const [query, setQuery] = useState("");
  const clips = track.lightingClips;
  if (!clips)
    return (
      <section className="audio-clips-library">
        <button disabled={busy} onClick={onConvert}>
          转换为独立灯光片段
        </button>
        <small>
          保留节奏标记，转换后可单独移动、复制与调整片段长度；可以撤销。
        </small>
      </section>
    );
  const items = filteredClips(clips, scenes, query);
  return (
    <section className="audio-clips-library" aria-label="灯光片段目录">
      <header>
        <h3>灯光片段 · {clips.length}</h3>
        <button
          disabled={busy || clips.length >= 512 || !scenes.length}
          onClick={onAdd}
        >
          添加片段
        </button>
      </header>
      <button disabled={busy} aria-pressed={batch} onClick={onBatch}>
        {batch ? "返回单片段编辑" : "批量整理片段"}
      </button>
      <input
        aria-label="搜索灯光片段"
        placeholder="搜索片段或场景"
        value={query}
        onChange={(e) => setQuery(e.target.value)}
      />
      {query && <button onClick={() => setQuery("")}>清除片段筛选</button>}
      {batch ? (
        <AudioClipBatch
          track={track}
          items={items}
          busy={busy}
          visible={visible}
          onEdit={onEdit}
        />
      ) : (
        <>
          {clips.some((c) => c.id === selected) &&
            !items.some((c) => c.id === selected) && (
              <small>所选片段在筛选范围外</small>
            )}
          <div className="audio-clip-list">
            {items.map((c) => (
              <button
                key={c.id}
                aria-pressed={selected === c.id}
                className={selected === c.id ? "selected" : ""}
                disabled={busy}
                onClick={() => onSelect(c.id)}
                onDoubleClick={() => onSeek(c.startMs)}
              >
                <time>
                  {audioTime(c.startMs)} — {audioTime(c.endMs)}
                </time>
                <strong>
                  {c.name}
                  {c.locked ? " · 已锁定" : ""}
                </strong>
                <span>{scenes.find((s) => s.id === c.sceneId)?.name}</span>
              </button>
            ))}
          </div>
          {!items.length && (
            <small>
              {clips.length ? "未找到片段" : "添加场景片段，空隙使用灯具默认值"}
            </small>
          )}
        </>
      )}
    </section>
  );
}
