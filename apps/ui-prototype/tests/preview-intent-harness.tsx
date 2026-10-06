// Actual PreviewPanel with controlled editor waits. No audio, renderer or devices.
import { useState } from "react";
import { createRoot } from "react-dom/client";
import { PreviewPanel } from "../src/components/workbench/PreviewPanel";
import { applicationHost } from "../src/hosts/application-host";
import type {
  ApplicationHost,
  SceneView,
  Snapshot,
} from "../src/application-host";
import type { PreviewSnapshot, SequenceView } from "../src/sequence-types";
import "../src/base.css";
import "../src/workbench.css";

type Target = "甲" | "乙";
const scenes: Record<Target, SceneView> = {
  甲: { id: "scene-甲", name: "预演甲场景", effects: [], values: [] },
  乙: { id: "scene-乙", name: "预演乙场景", effects: [], values: [] },
};
const sequences: Record<Target, SequenceView> = Object.fromEntries(
  (["甲", "乙"] as const).map((id) => [
    id,
    {
      id,
      name: `预演${id}列表`,
      tracking: "isolated",
      repeat: "once",
      steps: [
        {
          id: `${id}-step`,
          name: `预演${id}步骤`,
          number: "1",
          sceneId: `scene-${id}`,
          delayMs: 0,
          fadeMs: 0,
          waitMs: null,
        },
      ],
    },
  ]),
) as Record<Target, SequenceView>;
function loaded(id: Target, epoch: number): PreviewSnapshot {
  return {
    epoch,
    controlSerial: 0,
    loaded: {
      sequenceId: id,
      sceneId: null,
      name: sequences[id].name,
      sourceRevision: "test-revision",
      status: "idle",
      stepId: null,
      elapsedMs: 0,
      ratePercent: 100,
      delayMs: 0,
      fadeMs: 0,
      waitMs: null,
      stale: false,
      canNext: true,
      bufferBytes: 0,
      effectBufferBytes: 0,
      steps: sequences[id].steps,
      output: { universe: 1, slots: Array<number>(512).fill(0), fixtures: [] },
    },
  };
}
let state = loaded("甲", 1),
  generation = 7,
  observe = 0,
  notify = () => {};
let waitForDraft = true,
  holdRead = false;
const pendingDraft: ((value: boolean) => void)[] = [];
const pendingRead: {
  resolve(value: PreviewSnapshot): void;
  reject(error: Error): void;
}[] = [];
const commands: unknown[] = [];
function log(value: unknown) {
  if (commands.length >= 64) throw Error("测试命令超过上限");
  commands.push(value);
  notify();
}
const host: ApplicationHost = {
  ...applicationHost,
  request: async (request) => {
    if (request.kind !== "snapshot") throw Error("测试不允许工程变更");
    log({ project: request.kind, generation });
    return { generation } as Snapshot;
  },
  preview: async (request) => {
    if (request.kind === "snapshot") {
      observe = Math.min(100000, observe + 1);
      if (holdRead)
        return new Promise((resolve, reject) => {
          if (pendingRead.length >= 4) throw Error("测试读取超过上限");
          pendingRead.push({ resolve, reject });
          notify();
        });
      return structuredClone(state);
    }
    log(request);
    if (request.kind === "load")
      state = loaded(request.sequenceId as Target, state.epoch + 1);
    else if (request.kind === "loadScene") {
      const id = request.sceneId === scenes.甲.id ? "甲" : "乙";
      state = loaded(id, state.epoch + 1);
      state.loaded!.sceneId = request.sceneId;
      state.loaded!.name = scenes[id].name;
      state.loaded!.steps = [
        { id: request.sceneId, name: scenes[id].name, number: "1" },
      ];
    } else if (request.kind === "control") {
      state.controlSerial = request.serial;
      const command = request.command;
      if (command.kind === "execute" || command.kind === "next") {
        state.loaded!.status = "running";
        state.loaded!.stepId =
          state.loaded!.sceneId ??
          sequences[state.loaded!.sequenceId as Target].steps[0].id;
      } else if (command.kind === "pause") state.loaded!.status = "paused";
      else if (command.kind === "resume") state.loaded!.status = "running";
      else if (command.kind === "stop") {
        state.loaded!.status = "idle";
        state.loaded!.stepId = null;
      }
    } else throw Error("测试未实现此预演请求");
    return structuredClone(state);
  },
};
function Harness() {
  const [target, setTarget] = useState<Target>("甲"),
    [visible, setVisible] = useState(true);
  const [sceneMode, setSceneMode] = useState(false);
  const [, update] = useState(0);
  notify = () => update((v) => v + 1);
  const beforeAction = async () => {
    if (!waitForDraft) return true;
    return new Promise<boolean>((resolve) => {
      if (pendingDraft.length) throw Error("测试已有在途草稿");
      pendingDraft.push(resolve);
      notify();
    });
  };
  return (
    <main
      className="workbench"
      style={{
        display: "block",
        padding: 12,
        height: "100vh",
        overflow: "auto",
      }}
    >
      <h1>离线预演意图隔离验收</h1>
      <p role="status">
        当前目标：{target}；面板：{visible ? "显示" : "隐藏"}
      </p>
      <button onClick={() => setTarget((v) => (v === "甲" ? "乙" : "甲"))}>
        切换目标
      </button>
      <button onClick={() => setVisible((v) => !v)}>切换显示</button>
      <button onClick={() => setSceneMode((v) => !v)}>
        {sceneMode ? "切回列表模式" : "切到场景模式"}
      </button>
      <button
        disabled={!pendingDraft.length}
        onClick={() => {
          generation = 48;
          pendingDraft.shift()?.(true);
          notify();
        }}
      >
        完成草稿
      </button>
      <button
        disabled={!pendingDraft.length}
        onClick={() => {
          pendingDraft.shift()?.(false);
          notify();
        }}
      >
        拒绝草稿
      </button>
      <button
        onClick={() => {
          waitForDraft = !waitForDraft;
          notify();
        }}
      >
        {waitForDraft ? "关闭草稿等待" : "开启草稿等待"}
      </button>
      <button
        onClick={() => {
          holdRead = !holdRead;
          notify();
        }}
      >
        {holdRead ? "关闭读取等待" : "开启读取等待"}
      </button>
      <button
        disabled={!pendingRead.length}
        onClick={() => {
          pendingRead.shift()?.resolve(structuredClone(state));
          notify();
        }}
      >
        完成读取
      </button>
      <button
        disabled={!pendingRead.length}
        onClick={() => {
          pendingRead.shift()?.reject(Error("测试预演读取失败"));
          notify();
        }}
      >
        拒绝读取
      </button>
      <pre aria-label="测试操作记录">
        {JSON.stringify({
          commands,
          pendingDraft: pendingDraft.length,
          pendingRead: pendingRead.length,
          observe,
          generation,
        })}
      </pre>
      <div hidden={!visible}>
        <PreviewPanel
          host={host}
          sequence={sceneMode ? undefined : sequences[target]}
          scene={sceneMode ? scenes[target] : undefined}
          stepId={sequences[target].steps[0].id}
          generation={generation}
          busy={false}
          beforeAction={beforeAction}
          visible={visible}
          execution={!sceneMode}
        />
      </div>
    </main>
  );
}
createRoot(document.getElementById("root")!).render(<Harness />);
