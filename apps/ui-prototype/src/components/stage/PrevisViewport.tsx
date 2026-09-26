import { useEffect, useRef, useState } from "react";
import type { PixelStreaming } from "@epicgames-ps/lib-pixelstreamingfrontend-ue5.8";

/** Video transport adapter only. The document and playback clock stay in Rust. */
export function PrevisViewport({ url }: { url: string | null }) {
  const parent = useRef<HTMLDivElement>(null);
  const stream = useRef<PixelStreaming | null>(null);
  const [message, setMessage] = useState("");
  const [playing, setPlaying] = useState(false);
  const [viewState, setViewState] = useState({ status: "", selection: "", workLight: "工作照明" });
  const [retry, setRetry] = useState(0);
  useEffect(() => {
    setPlaying(false);
    setViewState({ status: "", selection: "", workLight: "工作照明" });
    setMessage(url ? "正在连接三维画面…" : "");
    if (!url || !parent.current) return;
    let active = true;
    const container = parent.current;
    let cleanup = () => {};
    void import("@epicgames-ps/lib-pixelstreamingfrontend-ue5.8").then(({ Config, Flags, NumericParameters, PixelStreaming, TextParameters }) => {
      if (!active) return;
      const config = new Config({ useUrlParams: false, initialSettings: {
        [TextParameters.SignallingServerUrl]: url,
        [Flags.AutoConnect]: false,
        [Flags.AutoPlayVideo]: true,
        [Flags.StartVideoMuted]: true,
        [Flags.HoveringMouseMode]: true,
        [Flags.KeyboardInput]: false,
        [Flags.GamepadInput]: false,
        [Flags.XRControllerInput]: false,
        [Flags.UseMic]: false,
        [Flags.UseCamera]: false,
        [Flags.SuppressBrowserKeys]: false,
        [Flags.WaitForStreamer]: true,
        [Flags.MatchViewportResolution]: false,
        [NumericParameters.MaxReconnectAttempts]: 5,
      } });
      const player = new PixelStreaming(config, { videoElementParent: container });
      stream.current = player;
      player.addResponseEventListener("stagemaster-view", (response) => {
        if (!active || response.length > 8192) return;
        try {
          const value: unknown = JSON.parse(response);
          if (value && typeof value === "object" && "status" in value && "selection" in value && "workLight" in value &&
            typeof value.status === "string" && typeof value.selection === "string" && typeof value.workLight === "string")
            setViewState({ status: value.status, selection: value.selection, workLight: value.workLight });
        } catch { /* A malformed renderer response cannot mutate the document. */ }
      });
      const status = (text: string) => { if (active) { setPlaying(false); setMessage(text); } };
      player.addEventListener("playStream", () => { if (active) { setPlaying(true); setMessage(""); } });
      player.addEventListener("webRtcDisconnected", () => status("三维画面已断开，可以重新连接"));
      player.addEventListener("webRtcFailed", () => status("三维画面连接失败，可以重试"));
      player.addEventListener("playStreamRejected", () => status("点击播放三维画面"));
      player.addEventListener("playStreamError", () => status("三维画面暂时无法播放"));
      // Official keyboard input listens to the document. Enable it only while the viewport owns focus.
      const focus = () => config.setFlagEnabled(Flags.KeyboardInput, true);
      const blur = () => config.setFlagEnabled(Flags.KeyboardInput, false);
      container.addEventListener("focusin", focus);
      container.addEventListener("focusout", blur);
      player.connect();
      cleanup = () => {
        container.removeEventListener("focusin", focus);
        container.removeEventListener("focusout", blur);
        config.setFlagEnabled(Flags.KeyboardInput, false);
        player.removeResponseEventListener("stagemaster-view");
        player.disconnect();
        stream.current = null;
        container.replaceChildren();
      };
    }).catch(() => { if (active) setMessage("三维视窗组件加载失败，请重新打开"); });
    return () => { active = false; cleanup(); };
  }, [url, retry]);
  function view(action: string) { stream.current?.emitUIInteraction({ action }); }
  return <div className="previs-live">
    <div className="previs-tools" aria-label="三维视图操作">
      <button disabled={!playing} onClick={() => view("perspective")}>透视</button>
      <button disabled={!playing} onClick={() => view("top")}>俯视</button>
      <button disabled={!playing} onClick={() => view("all")}>查看全场</button>
      <button disabled={!playing} onClick={() => view("selected")}>聚焦所选</button>
      <button disabled={!playing} onClick={() => view("workLight")}>{viewState.workLight}</button>
    </div>
    <div className="previs-viewport" aria-label="三维舞台视窗">
    <div ref={parent} className="previs-video" tabIndex={0} aria-label="三维舞台操作区域"
      onPointerDownCapture={() => parent.current?.focus({ preventScroll: true })} />
    {!playing && <div className="previs-overlay" role="status">
      <p>{url ? message : "开启三维预演，查看当前工程的灯光与空间"}</p>
      {url && <div>
        <button onClick={() => stream.current?.play()}>播放画面</button>
        <button onClick={() => setRetry((value) => value + 1)}>重新连接</button>
      </div>}
    </div>}
    </div>
    <div className="previs-footer">
      <span role="status">{playing ? `${viewState.status} · ${viewState.selection}` : "三维画面未就绪"}</span>
      <span>右键或 Option 拖动旋转 · 加 Shift 平移 · 滚动缩放</span>
    </div>
  </div>;
}
