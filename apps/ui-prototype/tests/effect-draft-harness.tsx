import { useMemo, useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import {
  EffectEditor,
  type EffectHandle,
} from "../src/components/workbench/EffectEditor";
import { PositionEffectEditor } from "../src/components/workbench/PositionEffectEditor";
import { useEffectDraftPreview } from "../src/components/workbench/useEffectDraftPreview";
import { createEffect, type EffectTemplate } from "../src/effect-tools";
import type {
  ApplicationHost,
  FixtureView,
  Snapshot,
} from "../src/application-host";
import type { PreviewSnapshot } from "../src/sequence-types";
import "../src/base.css";
import "../src/workbench.css";
function Harness() {
  const editor = useRef<EffectHandle>(null);
  const [effect, setEffect] = useState(
    createEffect("breathe", "effect", ["lamp", "second", "missing"]),
  );
  const [reset, setReset] = useState(0);
  const [mode, setMode] = useState<EffectTemplate>("breathe");
  const [dirty, setDirty] = useState(false),
    [applied, setApplied] = useState(0),
    [period, setPeriod] = useState(0);
  const host = useMemo(() => {
    let epoch = 1;
    let snapshot: PreviewSnapshot = { epoch, controlSerial: 0, loaded: null };
    return {
      request: async () => ({ generation: 1 }) as Snapshot,
      preview: async (r) => {
        if (r.kind === "beginEffectDraft")
          snapshot = {
            epoch: ++epoch,
            controlSerial: 0,
            loaded: {
              draftEffectId: r.effect.id,
              status: "running",
            } as PreviewSnapshot["loaded"],
          };
        if (r.kind === "beginEffectDraft" || r.kind === "updateEffectDraft")
          setPeriod(r.effect.periodMs);
        if (r.kind === "endEffectDraft")
          snapshot = { epoch: ++epoch, controlSerial: 0, loaded: null };
        return snapshot;
      },
    } as Pick<ApplicationHost, "request" | "preview"> as ApplicationHost;
  }, []);
  const audition = useEffectDraftPreview({
    host,
    editor,
    target: mode,
    available: true,
    canStart: () => true,
    openPlayback: () => {},
  });
  const fixture = {
    id: "lamp",
    name: "测试灯",
    attributes: ["dimmer", "red", "green", "blue", "pan", "tilt"].map(
      (key) => ({ key, defaultValue: 0 }),
    ),
    positioning: {},
  } as FixtureView;
  const Editor = mode === "circle" ? PositionEffectEditor : EffectEditor;
  return (
    <main style={{ width: 280, padding: 12 }}>
      <label>
        测试类型
        <select
          value={mode}
          onChange={(e) => {
            const next = e.target.value as EffectTemplate;
            setMode(next);
            setEffect(
              createEffect(next, "effect", ["lamp", "second", "missing"]),
            );
            setDirty(false);
          }}
        >
          <option value="breathe">亮度</option>
          <option value="multicolor">关键帧</option>
          <option value="circle">位置</option>
        </select>
      </label>
      <p role="status">
        已应用 {applied} 次；草稿 {dirty ? "有" : "无"}；预演周期 {period}
        ；已存灯序 {effect.fixtureIds.join(",")}
      </p>
      <Editor
        placements={[
          {
            fixtureId: "lamp",
            spaceId: null,
            positionMeters: { x: "10", y: "0", z: "1" },
            rotationDegreesXYZ: { x: "0", y: "0", z: "0" },
          },
          {
            fixtureId: "second",
            spaceId: null,
            positionMeters: { x: "-1", y: "0", z: "3" },
            rotationDegreesXYZ: { x: "0", y: "0", z: "0" },
          },
        ]}
        key={`${mode}:${reset}`}
        ref={editor}
        effect={effect}
        sceneId="scene"
        fixtures={[
          fixture,
          { ...fixture, id: "second", name: "第二灯" },
          { ...fixture, id: "missing", name: "未布置灯" },
        ]}
        selected={["lamp", "second", "missing"]}
        isNew={false}
        busy={false}
        error=""
        onPending={setDirty}
        onCancel={() => {
          void audition.end();
          setEffect(structuredClone(effect));
          setMode(mode);
          setReset((v) => v + 1);
          setDirty(false);
        }}
        onApply={async () => {
          const commands = editor.current?.collect() ?? [];
          const first = commands[0];
          if (first?.op === "effect" && first.command.kind === "put") {
            setEffect(first.command.effect);
            setApplied((v) => v + 1);
            editor.current?.accept();
          }
          return true;
        }}
        onPreview={async () => false}
        audition={audition.controls}
        onDraftChange={audition.changed}
      />
    </main>
  );
}
createRoot(document.getElementById("root")!).render(<Harness />);
