import type {
  AudioCommand,
  AudioPosition,
  AudioTimeline,
} from "./audio-types.ts";

export function enabledAudioLoops(track: AudioTimeline | null) {
  return track?.loopRegions?.filter((region) => region.enabled) ?? [];
}

/** Match native playback identity: names, locks and inactive regions do not reload audio. */
export function audioMediaKey(track: AudioTimeline | null, projectId = "") {
  return JSON.stringify(
    track
      ? [
          projectId,
          track.asset.digest,
          track.inMs,
          track.outMs,
          enabledAudioLoops(track).map((r) => [
            r.id,
            r.startMs,
            r.endMs,
            r.plays.kind === "count" ? r.plays.count : "untilExit",
          ]),
        ]
      : [projectId, null],
  );
}

export function audioLoopRuntime(
  track: AudioTimeline,
  position: AudioPosition,
) {
  const state = position.performance;
  if (!state || state.region === null) return null;
  const region = enabledAudioLoops(track)[state.region];
  if (
    !region ||
    position.positionMs < region.startMs ||
    position.positionMs >= region.endMs
  )
    return null;
  const pending = state.pendingExit;
  const exiting =
    pending?.region === state.region && pending.pass === state.pass
      ? pending.requested
      : state.exitRequested;
  return { state, region, exiting };
}

export function audioLoopExitCommand(
  track: AudioTimeline,
  position: AudioPosition,
): AudioCommand | null {
  const current = audioLoopRuntime(track, position);
  if (!current || position.problem) return null;
  const { state, region, exiting } = current;
  if (!state.instance || !state.pass || state.ended || state.snapshotPending)
    return null;
  // Do not round a native u64 or send commands to another pass based on UI interpolation.
  return {
    kind: "exitLoop",
    instance: state.instance,
    regionId: region.id,
    pass: state.pass,
    requested: !exiting,
  };
}
