import { createRoot } from "react-dom/client";
import { useState } from "react";
import { SceneUsagePanel } from "../src/components/workbench/SceneUsagePanel";
import { AudioClipLibrary } from "../src/components/audio/AudioClipLibrary";
import {
  useRevealItem,
  type RevealItem,
} from "../src/components/layout/useRevealItem";
import type { SceneUsageTarget } from "../src/components/workbench/scene-usage";
import { useClipSelection } from "../src/components/audio/useClipSelection";
import { usageProject } from "./scene-usage-fixture";
import "../src/base.css";
import "../src/workbench.css";
const project = usageProject();
project.audio!.lightingClips = Array.from({ length: 40 }, (_, i) => ({
  ...project.audio!.lightingClips![0],
  id: `clip-${i}`,
  name: `片段 ${i + 1}`,
  startMs: i * 2000,
  endMs: i * 2000 + 1000,
  sceneId: i === 39 ? "s" : "other",
  enabled: i !== 39,
}));
function Harness() {
  const [request, setRequest] = useState<RevealItem | null>(null);
  const [selected, setSelected] = useState("");
  const [reject, setReject] = useState(false),
    [busy, setBusy] = useState(false);
  const [status, setStatus] = useState("");
  const root = useRevealItem(request, true, busy);
  const selection = useClipSelection("test", project.audio!.lightingClips);
  function locate(target: SceneUsageTarget) {
    if (reject) {
      setStatus("草稿拒绝，页面保持");
      return;
    }
    setStatus(`${target.kind}:${target.id}`);
    if (target.kind === "clip") {
      setSelected(target.id);
      setRequest((old) => ({ id: target.id, serial: (old?.serial ?? 0) + 1 }));
    }
  }
  return (
    <main className="workbench" style={{ display: "block", padding: 20 }}>
      <label>
        <input
          type="checkbox"
          checked={reject}
          onChange={(e) => setReject(e.target.checked)}
        />
        拒绝草稿
      </label>
      <label>
        <input
          type="checkbox"
          checked={busy}
          onChange={(e) => setBusy(e.target.checked)}
        />
        忙状态
      </label>
      <p role="status">
        {status} · 选中 {selected}
      </p>
      <div style={{ display: "flex", height: 640, gap: 20 }}>
        <aside style={{ width: 310, overflow: "auto" }}>
          <SceneUsagePanel
            project={project}
            sceneId="s"
            busy={busy}
            onLocate={locate}
          />
        </aside>
        <div
          aria-label="外层导航验收"
          style={{ width: 360, height: 600, overflow: "auto" }}
        >
          <div style={{ height: 70 }}>外层滚动隔离</div>
          <div ref={root} style={{ height: 430, overflow: "auto" }}>
            <AudioClipLibrary
              revealRequest={request}
              track={project.audio!}
              scenes={project.scenes}
              selected={selected}
              busy={busy}
              onSelect={setSelected}
              onSeek={() => setStatus("意外播放定位")}
              onAdd={() => {}}
              onConvert={() => {}}
              batch={false}
              onBatch={() => {}}
              visible={true}
              onEdit={async () => null}
              selectionState={selection}
            />
          </div>
          <div style={{ height: 400 }}>外层结尾</div>
        </div>
      </div>
    </main>
  );
}
createRoot(document.getElementById("root")!).render(<Harness />);
