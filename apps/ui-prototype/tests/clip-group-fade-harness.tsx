// Actual shared draft/library/form. Command collection only; no host/media/device I/O.
import { useState } from "react";
import { createRoot } from "react-dom/client";
import { AudioClipLibrary } from "../src/components/audio/AudioClipLibrary";
import { AudioClipGroupFadeInspector } from "../src/components/audio/AudioClipGroupFadeInspector";
import { useAudioWorkspaceDraft } from "../src/components/audio/useAudioWorkspaceDraft";
import { groupFadeDraft } from "../src/components/audio/clip-group-fade";
import { useClipSelection } from "../src/components/audio/useClipSelection";
import { PerformanceLayout } from "../src/components/layout/PerformanceLayout";
import { DockPane } from "../src/components/layout/DockPane";
import type { AudioTimeline } from "../src/audio-types";
import "../src/base.css";
import "../src/workbench.css";
const track: AudioTimeline = {
  asset: {
    digest: "ab".repeat(32),
    fileName: "隔离渐变",
    extension: "wav",
    durationMs: 10000,
  },
  inMs: 0,
  outMs: 10000,
  markers: [],
  lightingClips: [
    {
      id: "a",
      name: "长段",
      sceneId: "s",
      startMs: 0,
      endMs: 2000,
      fadeMs: 200,
      locked: false,
    },
    {
      id: "b",
      name: "短段",
      sceneId: "s",
      startMs: 3000,
      endMs: 3500,
      fadeMs: 0,
      locked: false,
      enabled: false,
    },
    {
      id: "c",
      name: "锁定段",
      sceneId: "s",
      startMs: 5000,
      endMs: 6000,
      fadeMs: 0,
      locked: true,
    },
  ],
};
function Harness() {
  const selection = useClipSelection("test", track.lightingClips);
  const [problem, setProblem] = useState("");
  const [pending, setPending] = useState(false);
  const [visible, setVisible] = useState(true);
  const [busy, setBusy] = useState(false);
  const [reject, setReject] = useState(false);
  const [commands, setCommands] = useState<string[]>([]);
  const { draft, form, change, cancel, collect } = useAudioWorkspaceDraft(
    track,
    setPending,
    setProblem,
    () => {},
  );
  async function beforeChange() {
    try {
      const next = collect();
      if (next.length && reject) throw new Error("隔离宿主拒绝提交");
      if (next.length) setCommands((old) => [...old, JSON.stringify(next)]);
      cancel();
      return true;
    } catch (e) {
      setProblem(e instanceof Error ? e.message : String(e));
      return false;
    }
  }
  return (
    <main className="workbench" style={{ height: "100vh" }}>
      <PerformanceLayout
        mode="audio"
        toolbar={
          <>
            <button
              onClick={async () => {
                if (await beforeChange()) setVisible(!visible);
              }}
            >
              切换页面
            </button>
            <button onClick={() => void beforeChange()}>保存验收</button>
            <label>
              <input
                type="checkbox"
                checked={busy}
                onChange={(e) => setBusy(e.target.checked)}
              />
              宿主忙
            </label>
            <label>
              <input
                type="checkbox"
                checked={reject}
                onChange={(e) => setReject(e.target.checked)}
              />
              拒绝提交
            </label>
          </>
        }
      >
        <DockPane region="library" visible={visible}>
          <AudioClipLibrary
            track={track}
            scenes={[]}
            selected=""
            busy={busy}
            visible={visible}
            batch={true}
            onSelect={() => {}}
            onSeek={() => {}}
            onAdd={() => {}}
            onConvert={() => {}}
            onBatch={() => void beforeChange()}
            onEdit={async () => null}
            selectionState={selection}
            groupEditor={{
              beforeChange,
              begin: () => change(groupFadeDraft(track, selection.ids)),
              content:
                draft?.kind === "clipGroupFade" ? (
                  <AudioClipGroupFadeInspector
                    draft={draft}
                    track={track}
                    form={form}
                    busy={busy}
                    problem={problem}
                    onChange={change}
                    onApply={() => void beforeChange()}
                    onCancel={cancel}
                  />
                ) : null,
            }}
          />
        </DockPane>
        <DockPane region="viewport">
          <pre aria-label="验收结果">
            {JSON.stringify(
              { pending, selected: selection.ids, visible, commands },
              null,
              2,
            )}
          </pre>
        </DockPane>
      </PerformanceLayout>
    </main>
  );
}
createRoot(document.getElementById("root")!).render(<Harness />);
