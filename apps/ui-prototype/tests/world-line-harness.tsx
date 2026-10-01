// Real editor and draft-preview lifecycle; isolated host records requests without I/O.
import { useMemo, useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import type {
  ApplicationHost,
  FixtureView,
  Snapshot,
} from "../src/application-host";
import type { EffectHandle } from "../src/components/workbench/EffectEditor";
import { WorldLineEffectEditor } from "../src/components/workbench/WorldLineEffectEditor";
import { useEffectDraftPreview } from "../src/components/workbench/useEffectDraftPreview";
import { createWorldLine } from "../src/world-line-tools";
import type { PreviewSnapshot } from "../src/sequence-types";
import "../src/base.css";
import "../src/workbench.css";

const fixtures = ["左灯", "右灯", "未布置灯"].map(
  (name, i) =>
    ({
      id: String(i),
      name,
      profileName: "摇头灯",
      domainName: "灯光",
      universe: 1,
      address: i * 5 + 1,
      positioning: {},
      attributes: ["pan", "tilt", "dimmer"].map((key) => ({
        key,
        defaultValue: 0,
      })),
    }) as FixtureView,
);
const placements = fixtures.slice(0, 2).map((f, i) => ({
  fixtureId: f.id,
  spaceId: null,
  positionMeters: { x: String(i * 2 - 1), y: "-2", z: "3" },
  rotationDegreesXYZ: { x: "0", y: "0", z: "0" },
}));
function Harness() {
  const editor = useRef<EffectHandle>(null);
  const [saved, setSaved] = useState(() =>
    createWorldLine("effect", ["0", "1"]),
  );
  const [reset, setReset] = useState(0),
    [dirty, setDirty] = useState(false),
    [applied, setApplied] = useState(0);
  const [busy, setBusy] = useState(false),
    [failing, setFailing] = useState(false),
    [error, setError] = useState("");
  const [preview, setPreview] = useState("");
  const host = useMemo(() => {
    let snapshot: PreviewSnapshot = {
      epoch: 1,
      controlSerial: 0,
      loaded: null,
    };
    return {
      request: async () => ({ generation: 1 }) as Snapshot,
      preview: async (r) => {
        if (r.kind === "beginEffectDraft")
          snapshot = {
            epoch: snapshot.epoch + 1,
            controlSerial: 0,
            loaded: {
              draftEffectId: r.effect.id,
              status: "running",
            } as PreviewSnapshot["loaded"],
          };
        if (r.kind === "beginEffectDraft" || r.kind === "updateEffectDraft")
          setPreview(JSON.stringify(r.effect.targetPath));
        if (r.kind === "endEffectDraft")
          snapshot = {
            epoch: snapshot.epoch + 1,
            controlSerial: 0,
            loaded: null,
          };
        return snapshot;
      },
    } as Pick<ApplicationHost, "request" | "preview"> as ApplicationHost;
  }, []);
  const audition = useEffectDraftPreview({
    host,
    editor,
    target: "world",
    available: true,
    canStart: () => true,
    openPlayback: () => {},
  });
  async function apply() {
    try {
      const command = editor.current!.collect()[0];
      if (failing) {
        setError("提交失败，保留草稿");
        return false;
      }
      if (command?.op === "effect" && command.command.kind === "put") {
        setSaved(command.command.effect);
        setApplied((n) => n + 1);
        editor.current!.accept();
        setError("");
      }
      return true;
    } catch {
      return false;
    }
  }
  return (
    <main
      className="workbench"
      style={{
        padding: 16,
        display: "flex",
        flexDirection: "row",
        gap: 24,
        alignItems: "flex-start",
      }}
    >
      <aside style={{ width: 288 }}>
        <WorldLineEffectEditor
          key={reset}
          ref={editor}
          effect={saved}
          sceneId="scene"
          fixtures={fixtures}
          placements={placements}
          selected={["1", "0"]}
          isNew={false}
          busy={busy}
          error={error}
          onPending={setDirty}
          onApply={apply}
          onPreview={apply}
          audition={audition.controls}
          onDraftChange={audition.changed}
          onCancel={() => {
            void audition.end();
            setReset((n) => n + 1);
            setError("");
          }}
        />
      </aside>
      <section style={{ maxWidth: 500 }}>
        <label>
          <input
            type="checkbox"
            checked={busy}
            onChange={(e) => setBusy(e.target.checked)}
          />
          模拟忙状态
        </label>
        <label>
          <input
            type="checkbox"
            checked={failing}
            onChange={(e) => setFailing(e.target.checked)}
          />
          模拟提交失败
        </label>
        <p role="status">
          已应用 {applied} 次；草稿 {dirty ? "有" : "无"}
        </p>
        <pre aria-label="保存内容">{JSON.stringify(saved, null, 2)}</pre>
        <pre aria-label="预演内容">{preview}</pre>
      </section>
    </main>
  );
}
const root = createRoot(document.getElementById("root")!);
root.render(<Harness />);
import.meta.hot?.dispose(() => root.unmount());
