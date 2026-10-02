import type { AudioTimeline } from "../../audio-types";
import { enabledAudioLoops } from "../../audio-performance-tools";
import { AudioLoopControls } from "./AudioLoopControls";
import { AudioPerformanceControls } from "./AudioPerformanceControls";
import { AudioTransportBar } from "./AudioTransportBar";
import type { useAudio } from "./useAudio";

export function AudioWorkspaceTransport({
  track,
  session,
  blocked,
  shared,
  selected,
  addMarker,
}: {
  track: AudioTimeline;
  session: ReturnType<typeof useAudio>;
  blocked: boolean;
  shared: boolean;
  selected: string;
  addMarker(): Promise<void>;
}) {
  const ready =
    !!session.waveform &&
    session.position.durationMs === track.outMs - track.inMs;
  return (
    <>
      <AudioTransportBar
        position={session.position}
        requestedPosition={session.requestedPosition}
        requestedVolume={session.requestedVolume}
        command={session.command}
        ready={ready}
        duration={track.outMs - track.inMs}
        markerCount={track.markers.length}
        blocked={blocked}
        addMarker={addMarker}
        editingOnly={shared}
      />
      {!shared && (
        <AudioPerformanceControls
          track={track}
          position={session.position}
          disabled={!ready || session.preparing}
          command={session.command}
        />
      )}
      {!enabledAudioLoops(track).length && (
        <AudioLoopControls
          track={track}
          selected={selected}
          position={session.position}
          disabled={blocked || !ready}
          configure={session.configureLoop}
        />
      )}
    </>
  );
}
