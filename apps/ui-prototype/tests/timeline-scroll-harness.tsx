import { trimmedEffectOffset } from "../src/components/audio/clip-trim-tools";
import { validateMarker } from "../src/audio-tools";
// Isolated real waveform/lane components. No sound, files, engine or device I/O.
import { useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import { AudioWaveform } from "../src/components/audio/AudioWaveform";
import { clipsInRange } from "../src/components/audio/clip-selection";
import type {
  AudioTimeline,
  AudioPosition,
  AudioWaveform as Wave,
} from "../src/audio-types";
import "../src/base.css";
import "../src/components/audio/audio.css";
const markerMode = new URLSearchParams(location.search).has("markers");
const initial: AudioTimeline = {
  asset: {
    digest: "ab".repeat(32),
    fileName: "手势隔离验收",
    extension: "wav",
    durationMs: 60000,
  },
  inMs: 0,
  outMs: 60000,
  markers: [],
  lightingClips: [
    {
      id: "one",
      name: "金色",
      sceneId: "s",
      startMs: 1000,
      endMs: 2000,
      fadeMs: 0,
      locked: false,
    },
    {
      id: "two",
      name: "蓝色",
      sceneId: "s",
      startMs: 3000,
      endMs: 4000,
      fadeMs: 0,
      locked: false,
    },
    {
      id: "three",
      name: "后段",
      sceneId: "s",
      startMs: 40000,
      endMs: 42000,
      fadeMs: 0,
      locked: false,
    },
    {
      id: "locked",
      name: "固定段",
      sceneId: "s",
      startMs: 50000,
      endMs: 52000,
      fadeMs: 0,
      locked: true,
    },
  ],
};
const waveform = {
  durationMs: 60000,
  bucketMs: 10,
  channels: [
    Array.from(
      { length: 12000 },
      (_, i) => (i % 2 ? 1 : -1) * (0.1 + Math.abs(Math.sin(i / 180)) * 0.7),
    ),
  ],
} satisfies Wave;
if (markerMode) {
  delete initial.lightingClips;
  initial.markers = [
    { id: "m1", name: "节奏一", timeMs: 1000, sceneId: null },
    { id: "m2", name: "邻近节奏", timeMs: 1050, sceneId: null },
    { id: "start", name: "开场", timeMs: 3000, sceneId: "s", fadeMs: 500 },
    { id: "end", name: "收束", timeMs: 40000, sceneId: "t", fadeMs: 1000 },
  ];
}
function Harness() {
  const [track, setTrack] = useState(initial),
    [selected, setSelected] = useState("one"),
    [ids, setIds] = useState<string[]>([]);
  const [batch, setBatch] = useState(false),
    [visible, setVisible] = useState(true),
    [moves, setMoves] = useState(0),
    [seeks, setSeeks] = useState(0);
  const [error, setError] = useState("");
  const sample = useRef<{ position: AudioPosition; at: number }>({
    position: {
      positionMs: 505,
      playing: false,
      durationMs: 60000,
      volumePercent: 100,
      problem: null,
    } satisfies AudioPosition,
    at: performance.now(),
  });
  return (
    <main className="workbench" style={{ padding: 24, width: 900 }}>
      <button
        onClick={() => {
          setTrack(initial);
          setMoves(0);
          setIds([]);
        }}
      >
        还原
      </button>
      <button onClick={() => setVisible((v) => !v)}>切换显示</button>
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
        切换播放
      </button>
      <output role="status">
        提交 {moves} 次；定位 {seeks} 次；选择 {ids.join(",")}；
        {track.lightingClips
          ?.map(
            (c) =>
              `${c.name} ${c.startMs}—${c.endMs}（效果起点 ${c.effectOffsetMs ?? 0}）`,
          )
          .join("；")}
        {track.markers.map((m) => `${m.name} ${m.timeMs}`).join("；")}
      </output>
      {error && <p role="alert">{error}</p>}
      <div style={{ display: visible ? "block" : "none" }}>
        <AudioWaveform
          track={track}
          scenes={[
            { id: "s", name: "蓝色", values: [], effects: [] },
            { id: "t", name: "金色", values: [], effects: [] },
          ]}
          waveform={waveform}
          sample={sample}
          selected={selected}
          disabled={!visible}
          compact
          onSeek={() => setSeeks((v) => v + 1)}
          onSelect={setSelected}
          onMove={(marker) => {
            try {
              validateMarker(marker, track);
              setTrack({
                ...track,
                markers: track.markers
                  .map((m) => (m.id === marker.id ? marker : m))
                  .sort((a, b) => a.timeMs - b.timeMs),
              });
              setMoves((v) => v + 1);
              setError("");
            } catch (error) {
              setError(String(error));
            }
          }}
          onClipMove={(clip, mode) => {
            const previous = track.lightingClips!.find(
              (c) => c.id === clip.id,
            )!;
            if (mode !== "move")
              clip = {
                ...clip,
                effectOffsetMs: trimmedEffectOffset(previous, clip.startMs),
              };
            setTrack({
              ...track,
              lightingClips: track.lightingClips!.map((c) =>
                c.id === clip.id ? clip : c,
              ),
            });
            setMoves((v) => v + 1);
          }}
          clipSelection={{
            active: batch,
            ids,
            onMode: () => setBatch((v) => !v),
            onClear: () => setIds([]),
            onPick: (id) =>
              setIds((v) =>
                v.includes(id) ? v.filter((i) => i !== id) : [...v, id],
              ),
            onRange: (a, b, append) =>
              setIds([
                ...new Set([
                  ...(append ? ids : []),
                  ...clipsInRange(track.lightingClips!, a, b),
                ]),
              ]),
            onMove: (chosen, destination) => {
              const start = Math.min(
                ...track
                  .lightingClips!.filter((c) => chosen.includes(c.id))
                  .map((c) => c.startMs),
              );
              setTrack({
                ...track,
                lightingClips: track.lightingClips!.map((c) =>
                  chosen.includes(c.id)
                    ? {
                        ...c,
                        startMs: c.startMs + destination - start,
                        endMs: c.endMs + destination - start,
                      }
                    : c,
                ),
              });
              setMoves((v) => v + 1);
            },
          }}
        />
      </div>
    </main>
  );
}
createRoot(document.getElementById("root")!).render(<Harness />);
