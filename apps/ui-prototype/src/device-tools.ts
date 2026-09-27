import type {
  DeviceCandidate,
  DevicePhase,
  DeviceSnapshot,
} from "./device-types";
export const deviceLabels: Record<DevicePhase, string> = {
  idle: "未连接",
  preparing: "正在准备蓝牙",
  scanning: "正在搜索",
  connecting: "正在连接",
  connected: "已连接",
  stopping: "正在释放连接",
  fault: "连接中断",
  blocked: "连接需要处理",
};
export function newerDeviceSnapshot(
  current: DeviceSnapshot | null,
  next: DeviceSnapshot,
): DeviceSnapshot {
  return current && current.revision > next.revision ? current : next;
}
export function deviceMatches(
  candidate: DeviceCandidate,
  query: string,
): boolean {
  return `${candidate.name} ${candidate.id}`
    .toLocaleLowerCase()
    .includes(query.trim().toLocaleLowerCase());
}
export function deviceError(error: unknown): string {
  if (
    typeof error === "object" &&
    error &&
    "message" in error &&
    typeof error.message === "string"
  )
    return error.message;
  return typeof error === "string" ? error : "设备状态读取失败，请重试";
}
export function canStartDeviceOperation(phase: DevicePhase): boolean {
  return phase === "idle" || phase === "fault";
}

export async function deviceDeadline<T>(
  work: Promise<T>,
  milliseconds = 5000,
): Promise<T> {
  let timer: ReturnType<typeof setTimeout>;
  try {
    return await Promise.race([
      work,
      new Promise<never>((_, reject) => {
        timer = setTimeout(
          () => reject(new Error("设备服务未及时响应，正在重新读取状态")),
          milliseconds,
        );
      }),
    ]);
  } finally {
    clearTimeout(timer!);
  }
}
