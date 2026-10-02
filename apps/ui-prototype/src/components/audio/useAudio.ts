import { useEffect, useRef, useState } from "react";
import type { ApplicationHost } from "../../application-host";
import type {
  AudioCommand,
  AudioLoopRange,
  AudioPosition,
  AudioTimeline,
  AudioWaveform,
} from "../../audio-types";
import { AudioActionQueue } from "../../audio-action-queue";
import { runAudioCommands } from "../../audio-command-sequence";
import {
  audioMediaKey,
  enabledAudioLoops,
} from "../../audio-performance-tools";
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
  const preparationSerial = useRef(0);
  const preparedKey = useRef("");
  const mediaKey = audioMediaKey(track, projectId);
  const queue = useRef(new AudioActionQueue());
  const intentSerial = useRef(0);
  const [seekTarget, setSeekTarget] = useState<{
    key: string;
    token: number;
    value: number;
  } | null>(null);
  const [volumeTarget, setVolumeTarget] = useState<{
    key: string;
    token: number;
    value: number;
  } | null>(null);
  const latestSeek = useRef(0);
  const latestVolume = useRef(0);
  const state = useRef({ generation, track, mediaKey });
  state.current = { generation, track, mediaKey };
  const playingSample = useRef({ position: idle, at: performance.now() });
  const mounted = useRef(true);
  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
      epoch.current++;
      queue.current.invalidate();
    };
  }, []);
  function accept(next: AudioPosition) {
    setPosition(next);
    playingSample.current = { position: next, at: performance.now() };
  }
  useEffect(() => {
    epoch.current++;
    queue.current.invalidate();
    setSeekTarget(null);
    setVolumeTarget(null);
    preparedKey.current = "";
    preparationSerial.current++;
    setPreparing(false);
    setProblem("");
    setWaveform(null);
    accept(idle);
  }, [mediaKey]);
  async function prepare(kind: "import" | "load" | "locate") {
    const version = ++epoch.current;
    queue.current.invalidate();
    setSeekTarget(null);
    setVolumeTarget(null);
    const loadingKey = mediaKey;
    const preparation = ++preparationSerial.current;
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
      if (mounted.current && preparation === preparationSerial.current)
        setPreparing(false);
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
          const observation = queue.current.observation();
          if (observation === null) return;
          const next = await host.audio(state.current.generation(), {
            kind: "snapshot",
          });
          if (
            !active ||
            version !== epoch.current ||
            !queue.current.acceptsObservation(observation)
          )
            return;
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
        const observation = queue.current.observation();
        if (observation !== null) {
          const next = await host.audio(state.current.generation(), {
            kind: "snapshot",
          });
          if (
            active &&
            version === epoch.current &&
            queue.current.acceptsObservation(observation)
          )
            accept(next);
        }
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
  async function enqueue(sequence: AudioCommand[]) {
    const targetGeneration = state.current.generation();
    const targetKey = state.current.mediaKey;
    const token = ++intentSerial.current;
    const seek = sequence.find((value) => value.kind === "seek");
    const volume = sequence.find((value) => value.kind === "volume");
    if (seek?.kind === "seek") {
      latestSeek.current = token;
      setSeekTarget({ key: targetKey, token, value: seek.positionMs });
    }
    if (volume?.kind === "volume") {
      latestVolume.current = token;
      setVolumeTarget({ key: targetKey, token, value: volume.percent });
    }
    try {
      return await queue.current.enqueue(
        sequence,
        `${targetKey}:${targetGeneration}`,
        (values, current) =>
          runAudioCommands(
            values,
            () =>
              current() &&
              mounted.current &&
              targetKey === state.current.mediaKey &&
              targetGeneration === state.current.generation(),
            (value) => send(value, targetGeneration),
          ),
      );
    } catch (error) {
      if (mounted.current && targetKey === state.current.mediaKey)
        setProblem(error instanceof Error ? error.message : String(error));
      return false;
    } finally {
      if (mounted.current) {
        if (latestSeek.current === token) setSeekTarget(null);
        if (latestVolume.current === token) setVolumeTarget(null);
      }
    }
  }
  async function command(command: AudioCommand) {
    if (command.kind !== "stop" && command.kind !== "pause") {
      await enqueue([command]);
      return;
    }
    const targetGeneration = state.current.generation();
    const targetKey = state.current.mediaKey;
    epoch.current++;
    setSeekTarget(null);
    setVolumeTarget(null);
    await queue.current.interrupt([command], (values, current) =>
      runAudioCommands(
        values,
        () =>
          current() &&
          mounted.current &&
          targetKey === state.current.mediaKey &&
          targetGeneration === state.current.generation(),
        (value) => send(value, targetGeneration),
      ),
    );
  }
  function previewAt(positionMs: number) {
    return enqueue([{ kind: "seek", positionMs }, { kind: "play" }]);
  }
  async function send(command: AudioCommand, targetGeneration: number) {
    const version = ++epoch.current;
    const preparation =
      enabledAudioLoops(state.current.track).length &&
      (command.kind === "seek" || command.kind === "play")
        ? ++preparationSerial.current
        : null;
    if (preparation !== null) setPreparing(true);
    setProblem("");
    try {
      const next = await host.audio(targetGeneration, command);
      if (mounted.current && version === epoch.current) {
        accept(next);
        return true;
      }
    } catch (error) {
      if (mounted.current && version === epoch.current)
        setProblem(error instanceof Error ? error.message : String(error));
    } finally {
      if (
        mounted.current &&
        preparation !== null &&
        preparation === preparationSerial.current
      )
        setPreparing(false);
    }
    return false;
  }
  return {
    position,
    requestedPosition: seekTarget?.key === mediaKey ? seekTarget.value : null,
    requestedVolume: volumeTarget?.key === mediaKey ? volumeTarget.value : null,
    playingSample,
    waveform,
    preparing,
    problem,
    prepare,
    command,
    previewAt,
    configureLoop: (range: AudioLoopRange | null) =>
      enqueue([{ kind: "setLoop", range }]),
    cancel: () => host.audioCancel(),
  };
}
