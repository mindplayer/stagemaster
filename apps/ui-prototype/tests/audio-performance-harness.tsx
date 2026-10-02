// Isolated host contract fixture: no audio device, decoder, project write or UE process.
import React, { useState } from "react";
import { createRoot } from "react-dom/client";
import type { ApplicationHost } from "../src/application-host";
import type { AudioPosition, AudioTimeline } from "../src/audio-types";
import { applicationHost } from "../src/hosts/application-host";
import { useAudio } from "../src/components/audio/useAudio";
import { AudioPreviewTransport } from "../src/components/audio/AudioPreviewTransport";
import { AudioWorkspaceTransport } from "../src/components/audio/AudioWorkspaceTransport";
import "../src/base.css";
import "../src/workbench.css";
import "../src/components/audio/audio.css";

const original: AudioTimeline = {
  asset: {
    digest: "ab".repeat(32),
    extension: "wav",
    fileName: "循环运行验收.wav",
    durationMs: 5000,
  },
  inMs: 0,
  outMs: 5000,
  markers: [],
  loopRegions: [
    {
      id: "one",
      name: "演员候场",
      startMs: 1000,
      endMs: 3000,
      plays: { kind: "untilExit" },
      enabled: true,
      locked: false,
    },
  ],
};
const initial: AudioPosition = {
  playing: false,
  volumePercent: 100,
  positionMs: 1500,
  durationMs: 5000,
  problem: null,
  performance: {
    instance: "9007199254741001",
    pass: "18446744073709551615",
    region: 0,
    exitRequested: false,
    pendingExit: null,
    controlProblem: null,
    ended: false,
    snapshotPending: false,
    boundaryMs: 3000,
    cachedBytes: 10000,
  },
};
let position = structuredClone(initial),
  loads = 0;
let release: (() => void) | null = null;
let cancelled = false;
const commands: string[] = [];
let update = () => {};
const host: ApplicationHost = {
  ...applicationHost,
  kind: "desktop",
  audioPrepare: async () => {
    loads++;
    position = structuredClone(initial);
    update();
    return {
      asset: original.asset,
      waveform: { durationMs: 5000, bucketMs: 10, channels: [[0, 0]] },
    };
  },
  audioCancel: async () => {
    cancelled = true;
    release?.();
  },
  audio: async (_generation, command) => {
    if (command.kind !== "snapshot") commands.push(JSON.stringify(command));
    if (command.kind === "seek") {
      cancelled = false;
      await new Promise<void>((resolve) => {
        release = resolve;
        update();
      });
      release = null;
      if (cancelled) throw new Error("音乐操作已取消");
      position.positionMs = command.positionMs;
      position.performance!.pass = "1";
    }
    if (command.kind === "play") position.playing = true;
    if (command.kind === "pause" || command.kind === "stop") {
      cancelled = true;
      position.playing = false;
      if (command.kind === "stop") {
        position.positionMs = 0;
        position.performance!.instance = null;
        position.performance!.region = null;
        position.performance!.pass = null;
      }
    }
    if (command.kind === "exitLoop")
      position.performance!.pendingExit = {
        region: 0,
        pass: command.pass,
        requested: command.requested,
      };
    update();
    return structuredClone(position);
  },
};
function Harness() {
  const [track, setTrack] = useState(original);
  const [, redraw] = useState(0);
  update = () => redraw((n) => n + 1);
  const audio = useAudio(host, () => 1, track, true, "component");
  return (
    <main className="workbench" style={{ display: "block", padding: 20 }}>
      <h1>演出循环运行组件验收</h1>
      <p>隔离宿主，不输出声音</p>
      <AudioPreviewTransport session={audio} track={track} busy={false} />
      <AudioWorkspaceTransport
        track={track}
        session={audio}
        blocked={audio.preparing}
        shared
        selected=""
        addMarker={async () => {}}
      />
      <div className="wb-actions" style={{ margin: "20px 0" }}>
        <button onClick={() => void audio.previewAt(2000)}>
          延迟定位后播放
        </button>
        <button onClick={() => release?.()} disabled={!release}>
          完成待处理定位
        </button>
        <button
          onClick={() =>
            setTrack({
              ...track,
              loopRegions: track.loopRegions!.map((r) => ({
                ...r,
                name: "等待上场",
                locked: true,
              })),
            })
          }
        >
          修改名称和锁定
        </button>
        <button
          onClick={() =>
            setTrack({
              ...track,
              loopRegions: track.loopRegions!.map((r) => ({
                ...r,
                plays: { kind: "count", count: 3 },
              })),
            })
          }
        >
          改为三遍
        </button>
      </div>
      <p>音源准备次数：{loads}</p>
      <p>
        原生测试位置：{audio.position.positionMs}；播放：
        {String(audio.position.playing)}
      </p>
      <pre aria-label="宿主命令记录">{commands.join("\n")}</pre>
    </main>
  );
}
createRoot(document.getElementById("root")!).render(<Harness />);
