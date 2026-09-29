// Isolated component acceptance. No device or physical audio output; native decoder/output has its own probe.
import React, { useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import {
  AudioWorkspace,
  type AudioHandle,
} from "../src/components/audio/AudioWorkspace";
import { applicationHost } from "../src/hosts/application-host";
import type {
  ApplicationHost,
  EditCommand,
  ProjectView,
} from "../src/application-host";
import type { AudioTimeline, AudioPosition } from "../src/audio-types";
import "../src/base.css";
import "../src/workbench.css";
const track: AudioTimeline = {
  asset: {
    digest: "ab".repeat(32),
    fileName: "节奏验收.wav",
    extension: "wav",
    durationMs: 32000,
  },
  inMs: 0,
  outMs: 32000,
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
};
let position: AudioPosition = {
  volumePercent: 100,
  playing: false,
  positionMs: 0,
  durationMs: 32000,
  problem: null,
};
let anchor = performance.now();
const host: ApplicationHost = {
  ...applicationHost,
  kind: "desktop",
  audioPrepare: async () => ({
    asset: track.asset,
    waveform: {
      durationMs: 32000,
      bucketMs: 20,
      peaks: Array.from(
        { length: 1600 },
        (_, i) => 0.12 + 0.7 * Math.exp(-(i % 25) / 3),
      ),
    },
  }),
  audioCancel: async () => {},
  audio: async (_generation, command) => {
    if (position.playing) {
      position.positionMs = Math.min(
        32000,
        position.positionMs + performance.now() - anchor,
      );
      if (position.positionMs === 32000) position.playing = false;
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
      }
    }
    setHistory((h) => h.concat(current.current));
    current.current = next;
    setProject(next);
    setError("");
    return next;
  }
  async function flush() {
    try {
      const commands = handle.current?.collect() ?? [];
      if (commands.length) await edit({ op: "batch", commands });
      handle.current?.accept();
      return true;
    } catch (e) {
      setError(String(e));
      return false;
    }
  }
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
      </header>
      {error && <p role="alert">{error}</p>}
      <AudioWorkspace
        ref={handle}
        project={project}
        host={host}
        generation={() => 1}
        visible
        busy={false}
        onEdit={async (command) => ((await flush()) ? edit(command) : null)}
        beforeChange={flush}
        onPending={() => {}}
        previs={null}
      />
    </main>
  );
}
createRoot(document.getElementById("root")!).render(<Harness />);
