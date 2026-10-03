// Isolated component interaction fixture; no backend, audio or lighting output.
import { useState } from "react";
import { createRoot } from "react-dom/client";
import { MediaLoopControls } from "../src/components/execution/MediaLoopControls";
import type { ExecutionAudioState, ExecutionMediaAction } from "../src/execution-media-types";
import "../src/base.css";
import "../src/workbench.css";
import "../src/components/execution/execution.css";

const original: ExecutionAudioState = {
  output: "software", status: "paused", positionMs: 2500, durationMs: 5000,
  instance: "9007199254741001", problem: null,
  loopState: {region: 0, name: "演员候场", pass: "9007199254741003", exitRequested: false, pendingExit: null},
};
function Harness() {
  const [audio, setAudio] = useState(original);
  const [disabled, setDisabled] = useState(false);
  const [commands, setCommands] = useState<ExecutionMediaAction[]>([]);
  return <main className="workbench" style={{display:"block",padding:24}}>
    <h1>后台循环组件验收</h1>
    <label><input type="checkbox" checked={disabled} onChange={e=>setDisabled(e.target.checked)}/>只读观察</label>
    <article className="execution-source execution-music" style={{maxWidth:600}}>
      <MediaLoopControls audio={audio} disabled={disabled} onAction={command=>{
        setCommands(old=>[...old,command]);
        if(command.kind === "exitLoop") setAudio(old=>({...old,loopState:{...old.loopState!,pendingExit:command.requested}}));
      }}/>
    </article>
    <button onClick={()=>setAudio(old=>({...old,loopState:old.loopState?{...old.loopState,exitRequested:old.loopState.pendingExit??old.loopState.exitRequested,pendingExit:null}:null}))}>模拟回调确认</button>
    <button onClick={()=>setAudio(old=>({...old,status:"preparing"}))}>模拟准备中</button>
    <button onClick={()=>setAudio(old=>({...old,loopState:null,status:"playing"}))}>模拟离开循环</button>
    <button onClick={()=>setAudio(original)}>恢复循环</button>
    <pre aria-label="已提交指令">{JSON.stringify(commands,null,2)}</pre>
  </main>;
}
createRoot(document.getElementById("root")!).render(<Harness/>);
