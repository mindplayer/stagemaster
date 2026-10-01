import type {
  InstallationPhase,
  InstallationTask,
  InstallationView,
} from "./installation-types";
export const installationLabels: Record<InstallationPhase, string> = {
  querying: "正在核对设备",
  transferring: "正在传输",
  verifying: "正在校验",
  committing: "正在提交",
  cancelling: "正在确认取消",
  reconnect: "等待重新连接",
  failed: "安装需要处理",
  installed: "已安装",
  cancelled: "已取消",
  notStarted: "未开始安装",
};
export function installationFinished(phase: InstallationPhase) {
  return (
    phase === "installed" || phase === "cancelled" || phase === "notStarted"
  );
}
export function startInstallationReason(
  view: InstallationView | null,
  communicationError: string,
): string | null {
  if (communicationError) return communicationError;
  if (!view) return "正在读取设备状态";
  if (!view.destination.allowed)
    return view.destination.reason || "当前设备尚未取得安装权限";
  if (
    view.installation.task &&
    !installationFinished(view.installation.task.phase)
  )
    return "请先处理当前安装任务";
  return null;
}
export function mergeInstallation(
  current: InstallationView | null,
  next: InstallationView,
): InstallationView {
  if (!current) return next;
  return {
    installation:
      current.installation.revision > next.installation.revision
        ? current.installation
        : next.installation,
    destination:
      current.destination.revision > next.destination.revision
        ? current.destination
        : next.destination,
  };
}
export function resumeInstallationReason(
  task: InstallationTask,
  destination: InstallationView["destination"],
): string | null {
  if (task.running || installationFinished(task.phase))
    return "当前任务不需要恢复";
  if (!destination.allowed) return destination.reason || "设备尚未取得安装权限";
  if (
    (task.package.executionSemantics ?? 1) >
    (destination.executionSemantics ?? 1)
  )
    return "此设备固件不支持节目的灯具功能，请更新设备固件后再下发";
  if (destination.deviceId !== task.deviceId) return "请连接原定安装设备";
  if (task.phase === "reconnect" && task.connectionEpoch === destination.epoch)
    return "请先断开并重新连接原设备";
  return null;
}

export function packageTargetReason(
  required = 1,
  supported = 1,
): string | null {
  return required > supported
    ? "此设备固件不支持节目的灯具功能，请更新设备固件后再下发"
    : null;
}
