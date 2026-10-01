// Actual directory + selection hook; records proposed commands, no core or output I/O.
import { useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import {
  SceneLibrary,
  type SceneLibraryHandle,
} from "../src/components/workbench/SceneLibrary";
import { sceneCopyCommands } from "../src/components/workbench/scene-copy-tools";
import type { SceneView } from "../src/application-host";
import "../src/base.css";
import "../src/workbench.css";
function Harness() {
  const [scenes, setScenes] = useState<SceneView[]>(
    ["暖场", "蓝色", "暖场 副本", "收束"].map((name, i) => ({
      id: `s${i}`,
      name,
      values: [],
      effects: [],
    })),
  );
  const [query, setQuery] = useState("");
  const [selected, setSelected] = useState("s0");
  const [reject, setReject] = useState(false);
  const [busy, setBusy] = useState(false);
  const [commands, setCommands] = useState<unknown[]>([]);
  const [single, setSingle] = useState(0);
  const library = useRef<SceneLibraryHandle>(null);
  return (
    <main
      className="workbench"
      style={{
        height: "100vh",
        display: "flex",
        flexDirection: "row",
        gap: 20,
      }}
    >
      <div style={{ width: 320 }}>
        <SceneLibrary
          ref={library}
          scenes={scenes}
          selected={selected}
          query={query}
          onQuery={setQuery}
          busy={busy}
          canCreate={true}
          beforeChange={async () => !reject}
          onSelect={(s) => setSelected(s.id)}
          onAdd={() => {}}
          onDuplicate={() => setSingle((n) => n + 1)}
          onCopyMany={async (ids) => {
            if (reject) return null;
            const next = sceneCopyCommands(scenes, ids);
            setCommands((old) => [...old, next]);
            const copies = next.map((c, i) => ({
              id: `copy-${scenes.length + i}`,
              name: "name" in c ? c.name : "",
              values: [],
              effects: [],
            }));
            setScenes((old) => [...old, ...copies]);
            setSelected(copies[0].id);
            setQuery("");
            return copies.map((s) => s.id);
          }}
        />
      </div>
      <section>
        <label>
          <input
            type="checkbox"
            checked={reject}
            onChange={(e) => setReject(e.target.checked)}
          />
          拒绝操作
        </label>
        <label>
          <input
            type="checkbox"
            checked={busy}
            onChange={(e) => setBusy(e.target.checked)}
          />
          宿主忙
        </label>
        <button
          onClick={() => {
            if (!library.current?.duplicateSelected()) setSingle((n) => n + 1);
          }}
        >
          调用复制快捷入口
        </button>
        <pre aria-label="验收状态">
          {JSON.stringify({ selected, query, commands, single }, null, 2)}
        </pre>
      </section>
    </main>
  );
}
createRoot(document.getElementById("root")!).render(<Harness />);
