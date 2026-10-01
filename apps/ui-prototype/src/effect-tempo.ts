import { secondsToMs } from "./sequence-tools.ts";
export const cycleBeats = [0.25, 0.5, 1, 2, 4, 8, 16] as const;

export function effectPeriodMs(value: string): number {
  const ms = secondsToMs(value, "循环周期");
  if (ms < 100 || ms > 3_600_000)
    throw new Error("循环周期应在 0.1–3600 秒之间");
  return ms;
}
export function tempoPeriodMs(bpmText: string, beats: number): number {
  const text = bpmText.trim();
  if (!/^\d+(?:\.\d)?$/.test(text))
    throw new Error("每分钟拍数请输入 30–300，最多一位小数");
  const bpm = Number(text);
  if (!Number.isFinite(bpm) || bpm < 30 || bpm > 300)
    throw new Error("每分钟拍数须在 30–300 之间");
  if (!(cycleBeats as readonly number[]).includes(beats))
    throw new Error("请选择支持的每轮拍数");
  const ms = Math.round((60_000 * beats) / bpm);
  if (ms < 100 || ms > 3_600_000)
    throw new Error("换算周期不足 0.1 秒，请增加每轮拍数或降低拍速");
  return ms;
}
export function scaledEffectPeriod(value: string, factor: 0.5 | 2): number {
  const ms = Math.round(effectPeriodMs(value) * factor);
  if (ms < 100 || ms > 3_600_000)
    throw new Error("调整后的周期须在 0.1–3600 秒之间");
  return ms;
}
export interface TapEstimate {
  count: number;
  bpm: number | null;
  state: "collecting" | "estimated" | "tooFast" | "invalid";
}
/** Bounded authoring input estimator, never a playback clock. */
export class EffectTapTempo {
  private taps: number[] = [];
  reset() {
    this.taps = [];
  }
  tap(now: number): TapEstimate {
    const last = this.taps.at(-1);
    if (
      !Number.isFinite(now) ||
      now < 0 ||
      (last !== undefined && now < last)
    ) {
      this.reset();
      return { count: 0, bpm: null, state: "invalid" };
    }
    const interval = last === undefined ? null : now - last;
    if (interval !== null && interval < 200) return this.estimate("tooFast");
    if (interval !== null && interval > 2000) this.reset();
    this.taps.push(now);
    if (this.taps.length > 9) this.taps.shift();
    return this.estimate(this.taps.length > 1 ? "estimated" : "collecting");
  }
  private estimate(state: TapEstimate["state"]): TapEstimate {
    const count = this.taps.length;
    const bpm =
      count > 1
        ? Math.round(
            (60_000 * (count - 1) * 10) / (this.taps[count - 1] - this.taps[0]),
          ) / 10
        : null;
    return { count, bpm, state };
  }
}
