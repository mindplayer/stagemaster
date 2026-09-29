import type { AudioMarker, AudioTimeline } from "./audio-types.ts";
export function audioTime(ms: number) {
  return `${Math.floor(ms / 60000)
    .toString()
    .padStart(2, "0")}:${Math.floor((ms / 1000) % 60)
    .toString()
    .padStart(2, "0")}.${Math.round(ms % 1000)
    .toString()
    .padStart(3, "0")}`;
}
export function audioMilliseconds(value: string, field: string) {
  if (!/^\d+(\.\d{1,3})?$/.test(value.trim()))
    throw new Error(`${field}需要非负秒数，最多三位小数`);
  const number = Math.round(Number(value) * 1000);
  if (!Number.isSafeInteger(number) || number > 3600000)
    throw new Error(`${field}不能超过 3600 秒`);
  return number;
}
export function validateMarker(marker: AudioMarker, track: AudioTimeline) {
  if (!marker.name.trim()) throw new Error("卡点名称不能为空");
  if (
    !Number.isSafeInteger(marker.timeMs) ||
    marker.timeMs < 0 ||
    marker.timeMs >= track.outMs - track.inMs
  )
    throw new Error("卡点须位于音乐裁切范围内");
  if (
    track.markers.some((m) => m.id !== marker.id && m.timeMs === marker.timeMs)
  )
    throw new Error("这个时间已有卡点，请移动播放头或修改现有卡点");
  return marker;
}
export function snapAudioTime(
  ms: number,
  track: AudioTimeline,
  snap: boolean,
  except?: string,
  tolerance = 80,
) {
  const bounded = Math.max(
    0,
    Math.min(track.outMs - track.inMs - 1, Math.round(ms)),
  );
  if (!snap) return bounded;
  const nearby = track.markers
    .filter((m) => m.id !== except)
    .reduce<
      number | null
    >((best, m) => (Math.abs(m.timeMs - bounded) <= tolerance && (best === null || Math.abs(m.timeMs - bounded) < Math.abs(best - bounded)) ? m.timeMs : best), null);
  return nearby ?? bounded;
}
