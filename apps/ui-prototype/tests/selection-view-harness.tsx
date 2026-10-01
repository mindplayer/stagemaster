import { createRoot } from "react-dom/client";
import { useRef, useState } from "react";
import { AudioWaveform } from "../src/components/audio/AudioWaveform";
import type { AudioTimeline, AudioPosition } from "../src/audio-types";
import "../src/base.css";
import "../src/workbench.css";
const track: AudioTimeline = {
  asset: {
    digest: "ab".repeat(32),
    fileName: "验收.wav",
    extension: "wav",
    durationMs: 122000,
  },
  inMs: 2000,
  outMs: 122000,
  markers: [
    { id: "beat", name: "末拍", timeMs: 119999, sceneId: null },
    { id: "legacy", name: "旧段落", timeMs: 90000, sceneId: "s" },
  ],
  lightingClips: [
    {
      id: "first",
      name: "首段",
      sceneId: "s",
      startMs: 1000,
      endMs: 3000,
      fadeMs: 0,
      locked: false,
    },
    {
      id: "last",
      name: "末段",
      sceneId: "s",
      startMs: 119000,
      endMs: 119950,
      fadeMs: 0,
      locked: true,
      enabled: false,
    },
  ],
};
const waveform = {
  durationMs: 122000,
  bucketMs: 10,
  channels: [
    Array.from(
      { length: 24400 },
      (_, i) => (i % 2 ? 1 : -1) * (0.1 + 0.4 * Math.abs(Math.sin(i / 100))),
    ),
  ],
};
function Harness() {
  const [selected, setSelected] = useState(""),
    [group, setGroup] = useState<string[] | null>(null);
  const [legacy, setLegacy] = useState(false),
    [busy, setBusy] = useState(false),
    [seeks, setSeeks] = useState(0),
    [edits, setEdits] = useState(0);
  const sample = useRef<{ position: AudioPosition; at: number }>({
    position: {
      positionMs: 5000,
      durationMs: 120000,
      playing: false,
      volumePercent: 100,
      problem: null,
    },
    at: performance.now(),
  });
  const actual = legacy ? { ...track, lightingClips: undefined } : track;
  return (
    <main className="workbench" style={{ display: "block", padding: 20 }}>
      <button
        onClick={() => {
          setGroup(null);
          setSelected("first");
          setLegacy(false);
        }}
      >
        选首段
      </button>
      <button
        onClick={() => {
          setGroup(null);
          setSelected("last");
          setLegacy(false);
        }}
      >
        选末段
      </button>
      <button
        onClick={() => {
          setGroup(["first", "last"]);
          setLegacy(false);
        }}
      >
        选择两端
      </button>
      <button onClick={() => setGroup([])}>清空组</button>
      <button
        onClick={() => {
          setGroup(null);
          setSelected("beat");
        }}
      >
        选末拍
      </button>
      <button
        onClick={() => {
          setGroup(null);
          setSelected("legacy");
          setLegacy(true);
        }}
      >
        选旧段落
      </button>
      <button
        onClick={() => {
          sample.current = {
            position: {
              ...sample.current.position,
              playing: !sample.current.position.playing,
            },
            at: performance.now(),
          };
        }}
      >
        切换模拟播放
      </button>
      <label>
        <input
          type="checkbox"
          checked={busy}
          onChange={(e) => setBusy(e.target.checked)}
        />
        忙状态
      </label>
      <input aria-label="外部输入" defaultValue="输入不触发缩放" />
      <p role="status">
        定位命令 {seeks} · 编辑命令 {edits} · 所选 {selected} · 组{" "}
        {JSON.stringify(group)}
      </p>
      <div style={{ width: 900, maxWidth: "100%" }}>
        <AudioWaveform
          track={actual}
          scenes={[{ id: "s", name: "暖场", values: [], effects: [] }]}
          waveform={waveform}
          sample={sample}
          selected={group ? "" : selected}
          disabled={busy}
          onSeek={() => setSeeks((n) => n + 1)}
          onSelect={setSelected}
          onMove={() => setEdits((n) => n + 1)}
          onClipMove={() => setEdits((n) => n + 1)}
          clipSelection={{
            active: !!group,
            ids: group ?? [],
            onMode: () => {},
            onPick: () => {},
            onRange: () => {},
            onClear: () => setGroup([]),
            onMove: () => setEdits((n) => n + 1),
          }}
        />
      </div>
    </main>
  );
}
createRoot(document.getElementById("root")!).render(<Harness />);
