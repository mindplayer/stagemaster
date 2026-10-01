// UI-only fault injection; actual mutations/playback are verified in Rust and native desktop.
import { useState } from "react";
import { createRoot } from "react-dom/client";
import { AudioClipSplit } from "../src/components/audio/AudioClipSplit";
import "../src/base.css";
import "../src/workbench.css";

function Harness() {
  const [locked, setLocked] = useState(false);
  const [fullFade, setFullFade] = useState(false);
  const [preserved, setPreserved] = useState(false);
  const [reject, setReject] = useState(false);
  const [result, setResult] = useState("无命令");
  const [submits, setSubmits] = useState(0);
  const clip = {
    id: "test",
    name: "分割验收",
    sceneId: "scene",
    startMs: 1000,
    endMs: 5000,
    fadeMs: preserved ? 300 : fullFade ? 4000 : 500,
    ...(preserved
      ? { entryFade: { durationMs: 500, offsetMs: 200, from: [] } }
      : {}),
    effectOffsetMs: 1250,
    locked,
  };
  return (
    <main className="workbench" style={{ padding: 24 }}>
      <div style={{ width: 330 }}>
        <button onClick={() => setLocked(!locked)}>切换锁定</button>
        <button onClick={() => setFullFade(!fullFade)}>切换全段渐变</button>
        <button onClick={() => setPreserved(!preserved)}>切换保留渐变</button>
        <button onClick={() => setReject(!reject)}>切换宿主拒绝</button>
        <p>
          锁定：{String(locked)}；全段渐变：{String(fullFade)}；宿主拒绝：
          {String(reject)}
        </p>
        <form
          onSubmit={(e) => {
            e.preventDefault();
            setSubmits(submits + 1);
          }}
        >
          <AudioClipSplit
            key={String(fullFade)}
            clip={clip}
            disabled={false}
            ready
            actions={{
              async split(time) {
                await Promise.resolve();
                setResult(`分割命令：${time}`);
                return !reject;
              },
              async resetEntryFade() {
                setResult("重新计算渐变");
                return !reject;
              },
              async resetOffset() {
                await Promise.resolve();
                setResult("重置命令");
                return !reject;
              },
              async readPosition() {
                await Promise.resolve();
                if (reject) throw new Error("原生游标暂不可用");
                return 2333;
              },
            }}
          />
        </form>
        <p role="status">
          {result}；外层表单提交：{submits}
        </p>
      </div>
    </main>
  );
}
createRoot(document.getElementById("root")!).render(<Harness />);
