// Isolated actual library: no file, sound or device I/O; edits deliberately rejected.
import { useState } from "react";
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
  })),
};
function Harness() {
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
            track={track}
            scenes={[]}
            selected=""
            busy={false}
            onSelect={() => {}}
            onSeek={() => {}}
            onAdd={() => {}}
            onConvert={() => {}}
            batch={batch}
            onBatch={() => setBatch(!batch)}
            visible={visible}
            onEdit={async () => null}
          />
        </DockPane>
        <DockPane region="viewport">
          <p>仅验收选择与属性布局</p>
        </DockPane>
      </PerformanceLayout>
    </main>
  );
}
createRoot(document.getElementById("root")!).render(<Harness />);
