import type { AudioMarker, AudioTimeline } from "../../audio-types.ts";
import { snapAudioTime } from "../../audio-tools.ts";
import { constrainBoundaryTime } from "./lighting-segments.ts";

export interface MarkerGrip {
  marker?: AudioMarker;
  original: number;
  anchor: number;
  boundary: boolean;
  x: number;
  moved: boolean;
}

/** A click preserves the original marker, even if another snap point is near. */
export function markerMotion(
  grip: MarkerGrip,
  raw: number,
  clientX: number,
  track: AudioTimeline,
  snap: boolean,
  tolerance: number,
) {
  const moved = grip.moved || Math.abs(clientX - grip.x) >= 3;
  if (grip.marker && !moved) return { moved, time: grip.original };
  const time = snapAudioTime(
    grip.marker ? grip.original + raw - grip.anchor : raw,
    track,
    snap,
    grip.marker?.id,
    tolerance,
  );
  return {
    moved,
    time:
      grip.boundary && grip.marker
        ? constrainBoundaryTime(track, grip.marker.id, time)
        : time,
  };
}
