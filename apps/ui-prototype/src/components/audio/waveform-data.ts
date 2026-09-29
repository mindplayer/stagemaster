import type { AudioWaveform } from "../../audio-types";

/** Rendering-only envelopes. File decoding and the playback clock stay in Rust. */
export function clipEnvelope(
  wave: AudioWaveform,
  inMs: number,
  outMs: number,
): Float32Array[] {
  if (
    !Number.isFinite(inMs) ||
    !Number.isFinite(outMs) ||
    inMs < 0 ||
    outMs <= inMs ||
    outMs > wave.durationMs ||
    wave.bucketMs !== 10 ||
    wave.channels.length < 1 ||
    wave.channels.length > 2
  )
    throw new Error("波形数据或音乐范围无效，请重新准备音乐");
  const first = Math.floor(inMs / wave.bucketMs) * 2;
  const last = Math.ceil(outMs / wave.bucketMs) * 2;
  return wave.channels.map((channel) => {
    if (channel.length > 720_000 || channel.length % 2 || channel.length < last)
      throw new Error("波形数据不完整，请重新准备音乐");
    const data = new Float32Array(last - first);
    for (let i = first; i < last; i++) {
      const value = channel[i];
      if (!Number.isFinite(value) || Math.abs(value) > 1)
        throw new Error("波形包含无效采样值");
      data[i - first] = value;
    }
    return data;
  });
}

export function zoomAround(
  anchorMs: number,
  anchorX: number,
  pixelsPerSecond: number,
): number {
  return Math.max(0, (anchorMs / 1000) * pixelsPerSecond - anchorX);
}

export type WaveViewport = { start: number; end: number; width: number };
export function viewTime(
  clientX: number,
  left: number,
  view: WaveViewport,
): number {
  return (
    view.start +
    ((clientX - left) / Math.max(1, view.width)) * (view.end - view.start)
  );
}
