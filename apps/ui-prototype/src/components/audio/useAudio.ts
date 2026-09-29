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
) {
  const [position, setPosition] = useState(idle);
  const [waveform, setWaveform] = useState<AudioWaveform | null>(null);
  const [preparing, setPreparing] = useState(false);
  const [problem, setProblem] = useState("");
  const epoch = useRef(0);
  const commands = useRef(Promise.resolve());
  const state = useRef({ generation, track });
  state.current = { generation, track };
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
  async function prepare(kind: "import" | "load" | "locate") {
    const version = ++epoch.current;
    setPreparing(true);
    setProblem("");
    try {
      const result = await host.audioPrepare(state.current.generation(), kind);
      if (!mounted.current || version !== epoch.current) return null;
      if (result) setWaveform(result.waveform);
      return result;
    } catch (error) {
      if (mounted.current && version === epoch.current)
        setProblem(error instanceof Error ? error.message : String(error));
      return null;
    } finally {
      if (mounted.current && version === epoch.current) setPreparing(false);
    }
  }
  const key = track ? `${track.asset.digest}:${track.inMs}:${track.outMs}` : "";
  useEffect(() => {
    if (!track) {
      epoch.current++;
      setWaveform(null);
      accept(idle);
      return;
    }
    if (visible) void prepare("load");
    // Loading is keyed to media identity/range, never to each marker edit.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [key, visible]);
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
  }, [host, visible, !!track, preparing]);
  function command(command: AudioCommand) {
    const queued = commands.current.then(() => send(command));
    commands.current = queued;
    return queued;
  }
  async function send(command: AudioCommand) {
    const version = ++epoch.current;
    setProblem("");
    try {
      const next = await host.audio(state.current.generation(), command);
      if (mounted.current && version === epoch.current) accept(next);
    } catch (error) {
      if (mounted.current)
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
