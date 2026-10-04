// Isolated component acceptance. No device or physical audio output; native decoder/output has its own probe.
import { PerformanceLayout } from "../src/components/layout/PerformanceLayout";
import { DockPane } from "../src/components/layout/DockPane";
import { AudioPreviewTransport } from "../src/components/audio/AudioPreviewTransport";
import React, { useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import {
  AudioWorkspace,
  type AudioHandle,
} from "../src/components/audio/AudioWorkspace";
import { useAudio } from "../src/components/audio/useAudio";
import { applicationHost } from "../src/hosts/application-host";
import type {
  ApplicationHost,
  EditCommand,
  ProjectView,
} from "../src/application-host";
import type { AudioTimeline, AudioPosition } from "../src/audio-types";
import "../src/base.css";
import "../src/workbench.css";
const duration = new URLSearchParams(location.search).has("long")
  ? 3_600_000
  : 32000;
const track: AudioTimeline = {
  asset: {
    digest: "ab".repeat(32),
    fileName: "节奏验收.wav",
    extension: "wav",
    durationMs: duration,
  },
  inMs: 0,
  outMs: duration,
  markers: [
    {
      id: crypto.randomUUID(),
      name: "开场·蓝色光束",
      timeMs: 2000,
      sceneId: "blue",
    },
    {
      id: crypto.randomUUID(),
      name: "节奏追逐",
      timeMs: 8000,
      sceneId: "chase",
    },
  ],
  loopRegions: new URLSearchParams(location.search).has("loops") ? [
    { id: "hold", name: "对话等待", startMs: 2000, endMs: 4000,
      plays: { kind: "untilExit" }, enabled: true, locked: false },
    { id: "repeat", name: "返场三遍", startMs: 8000, endMs: 10000,
      plays: { kind: "count", count: 3 }, enabled: false, locked: false },
  ] : [],
};
let position: AudioPosition = {
  volumePercent: 100,
  playing: false,
  positionMs: 0,
  durationMs: duration,
  problem: null,
};
let anchor = performance.now();
const host: ApplicationHost = {
  ...applicationHost,
  kind: "desktop",
  audioPrepare: async () => ({
    asset: track.asset,
    waveform: {
      durationMs: duration,
      bucketMs: 10,
      channels: [0, 1].map((c) =>
        Array.from(
          { length: duration / 5 },
          (_, i) =>
            (i % 2 ? 1 : -1) *
            (0.12 + (c ? 0.5 : 0.7) * Math.exp(-(Math.floor(i / 2) % 50) / 6)),
        ),
      ),
    },
  }),
  audioCancel: async () => {},
  audio: async (_generation, command) => {
    if (position.playing) {
      position.positionMs = Math.min(
        duration,
        position.positionMs + performance.now() - anchor,
      );
      if (position.positionMs === duration) position.playing = false;
    }
    anchor = performance.now();
    switch (command.kind) {
      case "play":
        position.playing = true;
        break;
      case "pause":
        position.playing = false;
        break;
      case "stop":
        position.playing = false;
        position.positionMs = 0;
        break;
      case "seek":
        position.positionMs = command.positionMs;
        break;
      case "volume":
        position.volumePercent = command.percent;
    }
    return { ...position, positionMs: Math.round(position.positionMs) };
  },
};
const initial: ProjectView = {
  audio: track,
  id: "audio-component",
  name: "音频验收",
  description: "",
  profiles: [],
  domains: [],
  fixtures: [],
  groups: [],
  presets: [],
  sequences: [],
  stage: { attachments: [], spaces: [], constructions: [], placements: [] },
  scenes: [
    { id: "blue", name: "深蓝光束", values: [], effects: [] },
    { id: "chase", name: "对称追逐", values: [], effects: [] },
  ],
};
function Harness() {
  const [project, setProject] = useState(initial);
  const [error, setError] = useState("");
  const [history, setHistory] = useState<ProjectView[]>([]);
  const [visible, setVisible] = useState(true);
  const [waiting, setWaiting] = useState(false);
  const delay = useRef(false);
  const release = useRef<(() => void) | null>(null);
  const handle = useRef<AudioHandle>(null);
  const current = useRef(project);
  current.current = project;
  async function edit(command: EditCommand): Promise<ProjectView | null> {
    const next = structuredClone(current.current);
    const commands = command.op === "batch" ? command.commands : [command];
    for (const c of commands) {
      if (c.op !== "audio") continue;
      const a = c.command;
      if (a.kind === "clear") next.audio = null;
      else if (a.kind === "setAsset")
        next.audio = {
          asset: a.asset,
          inMs: 0,
          outMs: a.asset.durationMs,
          markers: [],
        };
      else if (next.audio) {
        if (a.kind === "putMarker") {
          next.audio.markers = next.audio.markers
            .filter((m) => m.id !== a.marker.id)
            .concat(a.marker)
            .sort((x, y) => x.timeMs - y.timeMs);
        }
        if (a.kind === "removeMarker")
          next.audio.markers = next.audio.markers.filter((m) => m.id !== a.id);
        if (a.kind === "trim") {
          next.audio.inMs = a.inMs;
          next.audio.outMs = a.outMs;
        }
        if (a.kind === "loopRegions") {
          const list = next.audio.loopRegions ??= [];
          const command = a.command;
          if (command.kind === "add") list.push({
            id: crypto.randomUUID(),
            name: command.name,
            startMs: command.startMs,
            endMs: command.endMs,
            plays: command.plays,
            enabled: true,
            locked: false,
          });
          if (command.kind === "put") {
            const index = list.findIndex((r) => r.id === command.region.id);
            list[index] = command.region;
          }
          if (command.kind === "edit") {
            const action = command.action;
            for (const id of command.ids) {
              const index = list.findIndex((r) => r.id === id);
              if (action.kind === "remove") list.splice(index, 1);
              if (action.kind === "locked") list[index].locked = action.locked;
              if (action.kind === "enabled") list[index].enabled = action.enabled;
            }
          }
          list.sort((a, b) => a.startMs - b.startMs);
        }
      }
    }
    const previous = current.current;
    if (JSON.stringify(previous) !== JSON.stringify(next))
      setHistory((h) => h.concat(previous));
    current.current = next;
    setProject(next);
    setError("");
    return next;
  }
  async function flush() {
    try {
      if (delay.current) {
        delay.current = false;
        setWaiting(true);
        await new Promise<void>((resolve) => { release.current = resolve; });
        setWaiting(false);
        release.current = null;
      }
      const commands = handle.current?.collect() ?? [];
      if (commands.length) await edit({ op: "batch", commands });
      handle.current?.accept();
      return true;
    } catch (e) {
      setError(String(e));
      return false;
    }
  }
  const session = useAudio(host, () => 1, project.audio, true);
  return (
    <main className="workbench">
      <header className="wb-top">
        <strong>音频组件隔离验收 · 不输出声音</strong>
        <button
          onClick={() => {
            const previous = history.at(-1);
            if (previous) {
              setProject(previous);
              setHistory(history.slice(0, -1));
              handle.current?.accept();
            }
          }}
        >
          撤销验收操作
        </button>
        <button onClick={() => setVisible(!visible)}>隐藏／显示工作区</button>
        <button onClick={() => { delay.current = true; }}>延迟下一次操作</button>
        <button disabled={!waiting} onClick={() => release.current?.()}>释放延迟操作</button>
        <output aria-label="验收历史">历史 {history.length} · 区段 {project.audio?.loopRegions?.length ?? 0}</output>
      </header>
      {error && <p role="alert">{error}</p>}
      <PerformanceLayout
        mode="audio"
        toolbar={<span>工作台音乐编排验收</span>}
        beforeChange={flush}
      >
        <DockPane region="viewport">
          <div style={{ flex: 1 }}>三维区域尺寸占位（隔离验收）</div>
          <AudioPreviewTransport
            session={session}
            track={project.audio}
            busy={false}
          />
        </DockPane>
        <AudioWorkspace
          sharedTransport
          ref={handle}
          session={session}
          project={project}
          host={host}
          generation={() => 1}
          visible={visible}
          busy={false}
          onEdit={async (command) => ((await flush()) ? edit(command) : null)}
          beforeChange={flush}
          onPending={() => {}}
        />
      </PerformanceLayout>
    </main>
  );
}
createRoot(document.getElementById("root")!).render(<Harness />);
