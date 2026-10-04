import type { PixelStreaming } from "@epicgames-ps/lib-pixelstreamingfrontend-ue5.8";

type Phase =
  | "disabled"
  | "connecting"
  | "waiting"
  | "playing"
  | "gesture"
  | "disconnected"
  | "failed"
  | "playError"
  | "loadError";

export function previsConnectionView(phase: Phase) {
  const messages: Record<Phase, string> = {
    disabled: "",
    connecting: "正在连接三维画面…",
    waiting: "正在等待三维渲染就绪，首次启动可能较慢…",
    playing: "",
    gesture: "点击播放三维画面",
    disconnected: "三维画面已断开，可以重新连接",
    failed: "三维画面连接失败，可以重试",
    playError: "三维画面暂时无法播放，可以重新连接",
    loadError: "三维视窗组件加载失败，请重新打开",
  };
  return {
    playing: phase === "playing",
    message: messages[phase],
    canPlay: phase === "gesture",
  };
}
export type PrevisConnectionView = ReturnType<typeof previsConnectionView>;

/** Official video/discovery events only; this never owns a show clock or another connection. */
export function observePrevisConnection(
  player: Pick<PixelStreaming, "addEventListener" | "removeEventListener">,
  report: (view: PrevisConnectionView) => void,
  invalidate: () => void,
) {
  let active = true;
  let phase: Phase = "connecting";
  const publish = (next: Phase, broken = false) => {
    if (!active) return;
    if (broken) invalidate();
    phase = next;
    report(previsConnectionView(next));
  };
  const ready = () => publish("playing");
  const disconnected = () => publish("disconnected", true);
  const failed = () => publish("failed", true);
  const rejected = () => publish("gesture", true);
  const playError = () => publish("playError", true);
  // An explicit event type keeps the upstream payload rather than interpreting raw protocol JSON.
  const listed = (event: {
    data: { messageStreamerList: { ids: string[] } };
  }) => {
    if (active && (phase === "connecting" || phase === "waiting"))
      publish(
        event.data.messageStreamerList.ids.length ? "connecting" : "waiting",
      );
  };
  player.addEventListener("playStream", ready);
  player.addEventListener("webRtcDisconnected", disconnected);
  player.addEventListener("webRtcFailed", failed);
  player.addEventListener("playStreamRejected", rejected);
  player.addEventListener("playStreamError", playError);
  player.addEventListener("streamerListMessage", listed);
  return () => {
    active = false;
    player.removeEventListener("playStream", ready);
    player.removeEventListener("webRtcDisconnected", disconnected);
    player.removeEventListener("webRtcFailed", failed);
    player.removeEventListener("playStreamRejected", rejected);
    player.removeEventListener("playStreamError", playError);
    player.removeEventListener("streamerListMessage", listed);
  };
}
