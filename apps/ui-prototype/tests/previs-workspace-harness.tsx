// Isolated ownership regression: no UE process, audio device, project file or physical output.
import { PerformanceLayout } from "../src/components/layout/PerformanceLayout";
import { DockPane } from "../src/components/layout/DockPane";
import { useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import {
  SharedPrevis,
  type SharedPrevisHandle,
} from "../src/components/stage/SharedPrevis";
import { AudioPreviewTransport } from "../src/components/audio/AudioPreviewTransport";
import { useAudio } from "../src/components/audio/useAudio";
import { applicationHost } from "../src/hosts/application-host";
import type { ApplicationHost } from "../src/application-host";
import type { AudioPosition, AudioTimeline } from "../src/audio-types";
import type { PrevisStatus } from "../src/previs-types";
import "../src/base.css";
import "../src/workbench.css";
import "../src/components/stage/stage.css";
const track: AudioTimeline = {
  asset: {
    digest: "ab".repeat(32),
    fileName: "验收.wav",
    extension: "wav",
    durationMs: 30000,
  },
  inMs: 0,
  outMs: 30000,
  markers: [],
};
let loads = 0;
let sourceWrites = 0;
let prepareGate: Promise<void> | null = null;
let position: AudioPosition = {
  volumePercent: 100,
  playing: false,
  positionMs: 0,
  durationMs: 0,
  problem: null,
};
let status: PrevisStatus = {
  enabled: false,
  connected: false,
  port: null,
  viewerUrl: null,
  source: { kind: "defaults" },
  problem: null,
};
const host: ApplicationHost = {
  ...applicationHost,
  kind: "desktop",
  audioPrepare: async () => {
    loads++;
    if (prepareGate) await prepareGate;
    position = {
      ...position,
      durationMs: 30000,
      positionMs: 0,
      playing: false,
    };
    return {
      asset: track.asset,
      waveform: {
        durationMs: 30000,
        bucketMs: 10,
        channels: [
          [0, 0],
          [0, 0],
        ],
      },
    };
  },
  audio: async (_generation, command) => {
    if (command.kind === "seek")
      position = { ...position, positionMs: command.positionMs };
    if (command.kind === "play") position = { ...position, playing: true };
    if (command.kind === "pause") position = { ...position, playing: false };
    if (command.kind === "stop")
      position = { ...position, playing: false, positionMs: 0 };
    return { ...position };
  },
  previs: async (request) => {
    if (request.kind === "enable") status = { ...status, enabled: true };
    if (request.kind === "disable") status = { ...status, enabled: false };
    if (request.kind === "source") {
      sourceWrites++;
      status = { ...status, source: request.source };
    }
    return { ...status };
  },
};
const settle = () => new Promise((resolve) => setTimeout(resolve, 80));
function ensure(value: unknown, message: string) {
  if (!value) throw new Error(message);
}
function Harness() {
  const [page, setPage] = useState("audio");
  const [projectId, setProjectId] = useState("first");
  const [report, setReport] = useState("等待验收");
  const panel = useRef<SharedPrevisHandle>(null);
  const audio = useAudio(
    host,
    () => (projectId === "first" ? 1 : 2),
    track,
    page === "audio",
    projectId,
  );
  const currentAudio = useRef(audio);
  currentAudio.current = audio;
  async function verify() {
    try {
      await settle();
      ensure(loads === 1, "初始音乐应只加载一次");
      panel.current?.openPlayback();
      await settle();
      const viewport = document.querySelector('[aria-label="三维舞台视窗"]');
      ensure(viewport, "应打开公共三维");
      position = { ...position, playing: true, positionMs: 12340 };
      for (let i = 0; i < 20; i++) {
        setPage(i % 2 ? "audio" : "stage");
        await settle();
        ensure(
          document.querySelectorAll('[aria-label="三维舞台视窗"]').length === 1,
          "只能有一个视窗",
        );
        ensure(
          document.querySelector('[aria-label="三维舞台视窗"]') === viewport,
          "切页不可重建视频节点",
        );
        ensure(
          document.querySelectorAll(".previs-tools [aria-pressed]").length ===
            (i % 2 ? 1 : 2),
          "仅舞台显示移动控件",
        );
      }
      ensure(
        loads === 1 && position.playing && position.positionMs === 12340,
        "切页不可重载或重置音乐",
      );
      ensure(
        sourceWrites === 1 && status.source.kind === "playback",
        "切页不可覆盖播放来源",
      );
      setPage("stage");
      await settle();
      position = { ...position, playing: false, positionMs: 0, durationMs: 0 };
      setPage("audio");
      await settle();
      await settle();
      ensure(
        loads === 2 && position.durationMs === 30000,
        "被其他播放器释放后须重新准备音乐",
      );
      setPage("stage");
      await settle();
      let release!: () => void;
      prepareGate = new Promise<void>((resolve) => {
        release = resolve;
      });
      const oldPreparation = currentAudio.current.prepare("load");
      await settle();
      ensure(currentAudio.current.preparing, "应存在延迟的旧工程加载");
      setProjectId("second");
      await settle();
      ensure(
        !currentAudio.current.preparing &&
          currentAudio.current.waveform === null,
        "切换工程须清除旧加载状态及波形",
      );
      prepareGate = null;
      release();
      ensure((await oldPreparation) === null, "迟到的旧工程加载必须失效");
      ensure(
        currentAudio.current.waveform === null,
        "旧响应不能恢复新工程的波形",
      );
      setPage("audio");
      await settle();
      await settle();
      ensure(
        loads === 4 && currentAudio.current.waveform !== null,
        "新工程的同一资源须独立准备",
      );
      setReport(
        "通过：20 次切页同一视窗；来源未覆盖；音乐保持 12.340 秒；释放后重新加载；移动权限随页面变化；切换工程拒绝迟到加载",
      );
    } catch (error) {
      setReport(`失败：${String(error)}`);
    }
  }
  return (
    <main className="workbench">
      <button onClick={() => void verify()}>运行隔离验收</button>
      <output>{report}</output>
      <output>
        当前页面：{page} · 加载次数：{loads} · 音乐位置：
        {audio.position.positionMs}
      </output>
      <PerformanceLayout mode={page} toolbar={<span>隔离验收</span>}>
        <DockPane region="viewport" keepConnected>
          <SharedPrevis
            fixed
            ref={panel}
            transport={
              <AudioPreviewTransport
                session={audio}
                track={track}
                busy={false}
              />
            }
            host={host}
            scenes={[]}
            contextKey={page}
            allowPlacement={page === "stage"}
            busy={false}
            generation={() => 1}
            run={async (work) => {
              await work();
              return true;
            }}
            selectedId=""
            onSelect={async () => true}
            onPrepareMove={async () => true}
            onPlacement={async () => false}
          />
        </DockPane>
      </PerformanceLayout>
    </main>
  );
}
createRoot(document.getElementById("root")!).render(<Harness />);
