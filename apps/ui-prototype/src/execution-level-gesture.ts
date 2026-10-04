import type { ExecutionRequest, ExecutionStatus } from "./execution-types";
import {
  levelRuntime,
  sameLevelSession,
  sourceLevel,
  newLevelSerial,
  type LevelContext,
  type LevelIdentity,
} from "./execution-level-context.ts";

export interface LevelGestureView {
  source: string | null;
  target: number;
  phase: "idle" | "dragging" | "settling" | "cancelled";
  problem: string;
}
export interface LiveLevels {
  view: LevelGestureView;
  begin(source: string): boolean;
  change(source: string, value: number): void;
  finish(source: string): void;
  cancel(source?: string, reason?: string): void;
}
interface Gesture extends LevelIdentity {
  target: number;
  confirmed: number;
  ending: boolean;
  cancelled: boolean;
}
interface Dependencies {
  context(): LevelContext;
  request(command: ExecutionRequest): Promise<ExecutionStatus | undefined>;
  publish(view: LevelGestureView): void;
  now(): number;
  wait(ms: number): Promise<void>;
}
export const idleLevelGesture: LevelGestureView = {
  source: null,
  target: 0,
  phase: "idle",
  problem: "",
};

// One gesture, one request and one latest target. Never replay a pointer-event queue.
export class LevelGestureController {
  private gesture: Gesture | null = null;
  private pumping = false;
  private lastSent = -Infinity;
  private problem = "";
  private readonly deps: Dependencies;
  constructor(deps: Dependencies) {
    this.deps = deps;
  }
  get busy() {
    return this.gesture !== null;
  }
  begin(source: string): boolean {
    if (this.gesture) {
      if (this.gesture.source !== source || !this.valid(this.gesture))
        return false;
      this.gesture.ending = false;
      this.publish();
      return true;
    }
    const context = this.deps.context();
    const runtime = levelRuntime(context.status);
    const value = runtime && sourceLevel(runtime, source);
    if (
      !context.available ||
      context.busy ||
      !runtime ||
      runtime.pending ||
      value == null
    )
      return false;
    const identity = {
      host: runtime.hostId,
      layout: runtime.catalog.layout,
      session: runtime.sessionId!,
      source,
    };
    if (!sameLevelSession(identity, runtime)) return false;
    this.gesture = {
      ...identity,
      target: value,
      confirmed: value,
      ending: false,
      cancelled: false,
    };
    this.problem = "";
    this.publish();
    return true;
  }
  change(source: string, value: number) {
    const gesture = this.gesture;
    if (
      !gesture ||
      gesture.source !== source ||
      gesture.ending ||
      gesture.cancelled
    )
      return;
    if (!Number.isInteger(value) || value < 0 || value > 65535) {
      this.cancel(source, "电平超出范围，已停止连续调整");
      return;
    }
    gesture.target = value;
    this.publish();
    this.kick(gesture);
  }
  finish(source: string) {
    const gesture = this.gesture;
    if (!gesture || gesture.source !== source || gesture.cancelled) return;
    gesture.ending = true;
    this.publish();
    this.kick(gesture);
  }
  cancel(source?: string, reason = "已停止连续调整，请核对实际电平") {
    const gesture = this.gesture;
    if (!gesture || (source && gesture.source !== source)) return;
    gesture.cancelled = true;
    this.problem = reason;
    if (!this.pumping) this.gesture = null;
    this.publish();
  }
  validate() {
    if (this.gesture && !this.gesture.cancelled && !this.valid(this.gesture))
      this.cancel(undefined, "连接或控制状态已变化，已停止连续调整");
  }
  private valid(gesture: Gesture) {
    const context = this.deps.context();
    return (
      !gesture.cancelled &&
      context.available &&
      sameLevelSession(gesture, levelRuntime(context.status))
    );
  }
  private check(gesture: Gesture, status?: ExecutionStatus) {
    if (
      !this.valid(gesture) ||
      !sameLevelSession(gesture, levelRuntime(status))
    )
      throw new Error("连接或控制状态已变化，已停止连续调整");
    return status!.runtime!;
  }
  private publish() {
    const g = this.gesture;
    this.deps.publish({
      source: g?.source ?? null,
      target: g?.target ?? 0,
      phase: !g
        ? "idle"
        : g.cancelled
          ? "cancelled"
          : g.ending
            ? "settling"
            : "dragging",
      problem: this.problem,
    });
  }
  private kick(gesture: Gesture) {
    if (this.pumping) return;
    this.pumping = true;
    void this.pump(gesture)
      .catch((error: unknown) => {
        if (!gesture.cancelled)
          this.problem =
            error instanceof Error
              ? error.message
              : "连续调整未确认，请核对实际电平";
        gesture.cancelled = true;
      })
      .finally(() => {
        this.pumping = false;
        if (!gesture.cancelled && gesture.target !== gesture.confirmed) {
          this.kick(gesture);
          return;
        }
        if (gesture.cancelled || gesture.ending) this.gesture = null;
        this.publish();
      });
  }
  private async pause(gesture: Gesture, deadline: number, ms = 50) {
    if (this.deps.now() >= deadline)
      throw new Error("电平确认超时，已停止继续发送，请核对实际电平");
    await this.deps.wait(ms);
    if (!this.valid(gesture)) throw new Error("连续调整已中断，请核对实际电平");
  }
  private async pump(gesture: Gesture) {
    while (!gesture.cancelled) {
      const deadline = this.deps.now() + 5000;
      if (!this.valid(gesture))
        throw new Error("连续调整已中断，请核对实际电平");
      const interval = 50 - (this.deps.now() - this.lastSent);
      if (interval > 0) await this.pause(gesture, deadline, interval);
      let current = this.check(
        gesture,
        await this.deps.request({ kind: "snapshot" }),
      );
      while (current.pending) {
        await this.pause(gesture, deadline);
        current = this.check(
          gesture,
          await this.deps.request({ kind: "snapshot" }),
        );
      }
      const target = gesture.target;
      if (sourceLevel(current, gesture.source) === target) {
        gesture.confirmed = target;
        return;
      }
      const previous = current.record?.serial;
      if (this.deps.now() >= deadline)
        throw new Error("准备电平操作超时，请核对实际电平后重试");
      this.lastSent = this.deps.now();
      current = this.check(
        gesture,
        await this.deps.request({
          kind: "apply",
          hostId: gesture.host,
          revision: current.observation.snapshot!.state.revision,
          source: gesture.source,
          action: { kind: "level", value: target },
        }),
      );
      const serial = current.record?.serial;
      if (!newLevelSerial(serial, previous))
        throw new Error("未取得本次电平回执，已停止继续发送");
      for (;;) {
        const record = current.record;
        if (!record || record.serial !== serial)
          throw new Error("操作回执已变化，已停止继续发送");
        if (record.status === "complete") {
          if (record.outcome?.kind !== "applied")
            throw new Error(
              record.outcome?.message || "后台未应用电平，已停止连续调整",
            );
          if (
            !current.pending &&
            sourceLevel(current, gesture.source) === target
          )
            break;
        } else if (record.status !== "pending")
          throw new Error("电平结果未知，已停止继续发送");
        await this.pause(gesture, deadline);
        current = this.check(
          gesture,
          await this.deps.request({ kind: "snapshot" }),
        );
      }
      gesture.confirmed = target;
      if (gesture.target === target) return;
    }
  }
}
