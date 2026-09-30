import { useEffect, useRef, useState } from "react";
import type { ApplicationHost } from "../../application-host";
import type {
  AudioCommand,
  AudioPosition,
  AudioTimeline,
  AudioWaveform,
} from "../../audio-types";
const idle: AudioPosition = {
  volumePercent: 100,
  playing: false,
  positionMs: 0,
  durationMs: 0,
  problem: null,
};
export function useAudio(
  host: ApplicationHost,
  generation: () => number,
  track: AudioTimeline | null,
  visible: boolean,
  projectId = "",
  prepareVisible = visible,
) {
  const [position, setPosition] = useState(idle);
  const [waveform, setWaveform] = useState<AudioWaveform | null>(null);
  const [preparing, setPreparing] = useState(false);
  const [problem, setProblem] = useState("");
  const epoch = useRef(0);
  const preparedKey = useRef("");
  const mediaKey = track
    ? `${projectId}:${track.asset.digest}:${track.inMs}:${track.outMs}`
    : `${projectId}:empty`;
  const commands = useRef(Promise.resolve());
  const state = useRef({ generation, track, mediaKey });
  state.current = { generation, track, mediaKey };
  const playingSample = useRef({ position: idle, at: performance.now() });
  const mounted = useRef(true);
  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
      epoch.current++;
    };
  }, []);
  function accept(next: AudioPosition) {
    setPosition(next);
    playingSample.current = { position: next, at: performance.now() };
  }
  useEffect(() => {
    epoch.current++;
    preparedKey.current = "";
    setPreparing(false);
    setProblem("");
    setWaveform(null);
    accept(idle);
  }, [mediaKey]);
  async function prepare(kind: "import" | "load" | "locate") {
    const version = ++epoch.current;
    const loadingKey = mediaKey;
    setPreparing(true);
    setProblem("");
    try {
      const result = await host.audioPrepare(state.current.generation(), kind);
      if (
        !mounted.current ||
        version !== epoch.current ||
        loadingKey !== state.current.mediaKey
      )
        return null;
      if (result) {
        setWaveform(result.waveform);
        if (kind !== "import") preparedKey.current = loadingKey;
      }
      return result;
    } catch (error) {
      if (mounted.current && version === epoch.current)
        setProblem(error instanceof Error ? error.message : String(error));
      return null;
    } finally {
      if (mounted.current && version === epoch.current) setPreparing(false);
    }
  }
  useEffect(() => {
    let active = true;
    if (!track) return;
    async function restore() {
      const version = epoch.current;
      if (preparedKey.current === mediaKey) {
        // A scene preview can release the audio owner while this page is hidden.
        // Query the native transport before reusing the waveform; never reload a live voice.
        try {
          const next = await host.audio(state.current.generation(), {
            kind: "snapshot",
          });
          if (!active || version !== epoch.current) return;
          if (next.durationMs === track!.outMs - track!.inMs) {
            accept(next);
            return;
          }
        } catch {
          /* Retry preparation to surface missing resources or stale ownership. */
        }
      }
      if (active && version === epoch.current) void prepare("load");
    }
    if (prepareVisible) void restore();
    return () => {
      active = false;
    };
    // Reload only after identity/range changes or native ownership was released.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [mediaKey, prepareVisible]);
  useEffect(() => {
    let active = true;
    let timer: ReturnType<typeof setTimeout>;
    async function poll() {
      const version = epoch.current;
      try {
        const next = await host.audio(state.current.generation(), {
          kind: "snapshot",
        });
        if (active && version === epoch.current) accept(next);
      } catch {
        /* Project mutations can invalidate an in-flight poll; next poll uses the new generation. */
      }
      if (active) timer = setTimeout(poll, 60);
    }
    if (visible && track && !preparing) void poll();
    return () => {
      active = false;
      clearTimeout(timer);
    };
  }, [host, visible, mediaKey, preparing]);
  function command(command: AudioCommand) {
    const targetGeneration = state.current.generation();
    const targetKey = state.current.mediaKey;
    const queued = commands.current.then(() => {
      if (
        targetKey === state.current.mediaKey &&
        targetGeneration === state.current.generation()
      )
        return send(command, targetGeneration);
    });
    commands.current = queued;
    return queued;
  }
  async function send(command: AudioCommand, targetGeneration: number) {
    const version = ++epoch.current;
    setProblem("");
    try {
      const next = await host.audio(targetGeneration, command);
      if (mounted.current && version === epoch.current) accept(next);
    } catch (error) {
      if (mounted.current && version === epoch.current)
        setProblem(error instanceof Error ? error.message : String(error));
    }
  }
  return {
    position,
    playingSample,
    waveform,
    preparing,
    problem,
    prepare,
    command,
    cancel: () => host.audioCancel(),
  };
}
