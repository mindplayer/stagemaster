import { AudioClipLane } from "../src/components/audio/AudioClipLane";
import { clipsInRange } from "../src/components/audio/clip-selection";
import { useClipSelection } from "../src/components/audio/useClipSelection";
// Isolated actual library: no file, sound or device I/O; edits deliberately rejected.
import { useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import { AudioClipLibrary } from "../src/components/audio/AudioClipLibrary";
import { PerformanceLayout } from "../src/components/layout/PerformanceLayout";
import { DockPane } from "../src/components/layout/DockPane";
import type { AudioTimeline } from "../src/audio-types";
import "../src/base.css";
import "../src/workbench.css";
const track: AudioTimeline = {
  asset: {
    digest: "ab".repeat(32),
    fileName: "隔离选择",
    extension: "wav",
    durationMs: 30000,
  },
  inMs: 0,
  outMs: 30000,
  markers: [],
  lightingClips: Array.from({ length: 20 }, (_, i) => ({
    id: `clip-${i}`,
    name: `${i % 2 ? "蓝色" : "金色"}长名称片段 ${i + 1}`,
    sceneId: "s",
    startMs: i * 1000,
    endMs: i * 1000 + 700,
    fadeMs: 200,
    locked: i === 2,
    enabled: i !== 1,
  })),
};
function Harness() {
  const selection = useClipSelection("test", track.lightingClips);
  const cursor = useRef<HTMLDivElement>(null);
  const [selected, setSelected] = useState("");
  const [batch, setBatch] = useState(true),
    [visible, setVisible] = useState(true);
  return (
    <main className="workbench" style={{ height: "100vh" }}>
      <PerformanceLayout
        mode="audio"
        toolbar={<button onClick={() => setVisible(!visible)}>切换页面</button>}
      >
        <DockPane region="library" visible={visible}>
          <AudioClipLibrary
            selectionState={selection}
            track={track}
            scenes={[]}
            selected={selected}
            busy={false}
            onSelect={setSelected}
            onSeek={() => {}}
            onAdd={() => {}}
            onConvert={() => {}}
            batch={batch}
            onBatch={() => setBatch(!batch)}
            visible={visible}
            onEdit={async () => null}
          />
        </DockPane>
        <DockPane region="editor" visible={visible}>
          <div style={{ width: 600, height: 100, position: "relative" }}>
            <AudioClipLane
              track={track}
              scenes={[]}
              viewport={{ start: 0, end: 30000, width: 600 }}
              selected={selected}
              disabled={false}
              snap={false}
              cursor={cursor}
              onSelect={setSelected}
              onSeek={() => {}}
              onMove={() => {}}
              clipSelection={{
                active: batch,
                ids: selection.ids,
                onMode: () => setBatch(!batch),
                onPick: (id, range) =>
                  selection.toggle(id, track.lightingClips!, range),
                onRange: (start, end, append) =>
                  selection.replace([
                    ...(append ? selection.ids : []),
                    ...clipsInRange(track.lightingClips!, start, end),
                  ]),
                onClear: () => selection.replace([]),
              }}
            />
          </div>
        </DockPane>
        <DockPane region="viewport">
          <p>仅验收选择与属性布局</p>
        </DockPane>
      </PerformanceLayout>
    </main>
  );
}
createRoot(document.getElementById("root")!).render(<Harness />);
