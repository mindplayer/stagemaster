import { createRoot } from "react-dom/client";
import { useRef, useState } from "react";
import { AudioMarkerLibrary } from "../src/components/audio/AudioMarkerBatch";
import { AudioWaveform } from "../src/components/audio/AudioWaveform";
import { useOrderedSelection } from "../src/components/selection/useOrderedSelection";
import { PerformanceLayout } from "../src/components/layout/PerformanceLayout";
import { DockPane } from "../src/components/layout/DockPane";
import type { AudioPosition, AudioTimeline } from "../src/audio-types";
import { stageProject } from "./stage-organization-fixture";
import "../src/base.css";
import "../src/workbench.css";
const initial: AudioTimeline = {
  asset: {
    digest: "ab".repeat(32),
    fileName: "选择验收.wav",
    extension: "wav",
    durationMs: 30000,
  },
  inMs: 0,
  outMs: 30000,
  markers: Array.from({ length: 8 }, (_, i) => ({
    id: `m${i}`,
    name: `${i % 2 ? "蓝" : "金"}拍 ${i + 1}`,
    timeMs: 1000 + i * 2000,
    sceneId: null,
  })),
  lightingClips: [],
};
const waveform = {
  durationMs: 30000,
  bucketMs: 10,
  channels: [
    Array.from(
      { length: 6000 },
      (_, i) => (i % 2 ? 1 : -1) * (0.2 + 0.3 * Math.abs(Math.sin(i / 20))),
    ),
  ],
};
function Harness() {
  const [track, setTrack] = useState(initial),
    [identity, setIdentity] = useState(1);
  const selection = useOrderedSelection(String(identity), track.markers);
  const [batch, setBatch] = useState(false),
    [pending, setPending] = useState(false),
    [busy, setBusy] = useState(false),
    [visible, setVisible] = useState(true),
    [failure, setFailure] = useState(false);
  const [query, setQuery] = useState(""),
    [selected, setSelected] = useState("");
  const [edits, setEdits] = useState(0),
    [seeks, setSeeks] = useState(0);
  const sample = useRef({
    position: {
      positionMs: 0,
      durationMs: 30000,
      playing: false,
      volumePercent: 100,
      problem: null,
    } as AudioPosition,
    at: performance.now(),
  });
  function mode(value: boolean) {
    if (!pending) setBatch(value);
  }
  return (
    <main className="workbench" style={{ height: "100vh" }}>
      <PerformanceLayout
        mode="audio"
        toolbar={
          <>
            <button onClick={() => setVisible(!visible)}>切换页面</button>
            <button onClick={() => setIdentity((n) => n + 1)}>
              切换音源身份
            </button>
            <button
              onClick={() =>
                setTrack((t) => ({
                  ...t,
                  markers: t.markers.filter((m) => m.id !== "m0"),
                }))
              }
            >
              移除首卡点
            </button>
            <label>
              <input
                type="checkbox"
                checked={busy}
                onChange={(e) => setBusy(e.target.checked)}
              />
              忙状态
            </label>
            <label>
              <input
                type="checkbox"
                checked={failure}
                onChange={(e) => setFailure(e.target.checked)}
              />
              宿主失败
            </label>
          </>
        }
      >
        <DockPane region="library" visible={visible}>
          <section className="audio-resources">
            <AudioMarkerLibrary
              key={identity}
              track={track}
              scenes={[]}
              selected={selected}
              query={query}
              busy={busy}
              onQuery={setQuery}
              onSelect={setSelected}
              onSeek={() => setSeeks((n) => n + 1)}
              batch={batch}
              onBatch={mode}
              beforeChange={async () => true}
              workspaceVisible={visible}
              selectionState={selection}
              pending={pending}
              onPending={setPending}
              onEdit={async (command) => {
                if (failure || command.kind !== "editMarkers") return null;
                const chosen = new Set(command.ids),
                  action = command.action;
                const first = track.markers.find((m) =>
                  chosen.has(m.id),
                )!.timeMs;
                const markers =
                  action.kind === "remove"
                    ? track.markers.filter((m) => !chosen.has(m.id))
                    : action.kind === "copy"
                      ? [
                          ...track.markers,
                          ...track.markers
                            .filter((m) => chosen.has(m.id))
                            .map((m) => ({
                              ...m,
                              id: crypto.randomUUID(),
                              timeMs: m.timeMs + action.destinationMs - first,
                            })),
                        ].sort((a, b) => a.timeMs - b.timeMs)
                      : track.markers
                          .map((m) =>
                            chosen.has(m.id)
                              ? {
                                  ...m,
                                  timeMs:
                                    m.timeMs + action.destinationMs - first,
                                }
                              : m,
                          )
                          .sort((a, b) => a.timeMs - b.timeMs);
                const next = { ...track, markers };
                setTrack(next);
                setEdits((n) => n + 1);
                return { ...stageProject(), audio: next };
              }}
            />
          </section>
        </DockPane>
        <DockPane region="viewport">
          <p role="status">
            选择 {JSON.stringify(selection.ids)} · 编辑 {edits} · 定位 {seeks} ·{" "}
            {pending ? "目标待处理" : "空闲"}
          </p>
        </DockPane>
        <DockPane region="editor" visible={visible}>
          <AudioWaveform
            track={track}
            scenes={[]}
            waveform={waveform}
            sample={sample}
            selected={selected}
            disabled={busy || !visible}
            onSeek={() => setSeeks((n) => n + 1)}
            onSelect={setSelected}
            onMove={() => setEdits((n) => n + 1)}
            onClipMove={() => setEdits((n) => n + 1)}
            markerSelection={{
              active: batch,
              ids: selection.ids,
              blocked: pending,
              onMode: () => mode(!batch),
              onPick: (id, range) => selection.toggle(id, track.markers, range),
              onClear: () => selection.replace([]),
            }}
          />
        </DockPane>
      </PerformanceLayout>
    </main>
  );
}
createRoot(document.getElementById("root")!).render(<Harness />);
