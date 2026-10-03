import type {
  DeviceProgramKey,
  DeviceRunReply,
  DeviceRunState,
  DeviceRunView,
} from "./device-runtime-types.ts";

export function programKey(key: DeviceProgramKey | null): string {
  return key ? `${key.kind}:${key.id}` : "";
}
export function acceptRunReply(
  current: DeviceRunReply | null,
  view: DeviceRunView,
  epoch: number,
): DeviceRunReply | null {
  if (view.epoch !== epoch || view.connectionEpoch !== epoch || !view.peer)
    return current;
  const next = view.reply ?? view.lastResponse;
  if (!next || next.boot !== view.peer.boot) return current;
  if (current?.boot === next.boot && BigInt(current.id) > BigInt(next.id))
    return current;
  return next;
}
export function runState(reply: DeviceRunReply | null): DeviceRunState | null {
  return reply?.body.kind === "state" ? reply.body.state : null;
}
export function ownsRun(
  state: DeviceRunState | null,
  lease: string | null,
): boolean {
  return !!lease && state?.owner?.lease === lease;
}
export function shouldRenew(
  state: DeviceRunState,
  observed: string,
  lease: string | null,
  renewed: string,
): boolean {
  return (
    ownsRun(state, lease) &&
    BigInt(observed) - BigInt(renewed) >= 20_000n &&
    BigInt(state.owner!.expiresMs) - BigInt(observed) <= 40_000n
  );
}
export function runLabel(state: DeviceRunState | null): string {
  if (!state) return "尚未读取节目状态";
  if (state.mode === "maintenance") return "安装维护中";
  if (state.mode === "quiescing") return "正在停止输出，准备维护";
  if (!state.loaded) return "尚未载入节目";
  return (
    (
      {
        running: "正在执行",
        paused: "已暂停",
        finished: "本次节目结束",
        idle: "已载入，待执行",
      } as const
    )[state.status ?? "idle"] ?? "待执行"
  );
}
export function elapsedLabel(milliseconds: string): string {
  const seconds = BigInt(milliseconds) / 1000n;
  return `${seconds / 60n}:${String(seconds % 60n).padStart(2, "0")}`;
}
