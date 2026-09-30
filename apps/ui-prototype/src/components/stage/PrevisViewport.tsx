import { useEffect, useRef, useState } from "react";
import type { PixelStreaming } from "@epicgames-ps/lib-pixelstreamingfrontend-ue5.8";
import type { PrevisInteractions } from "../../previs-types";
import { readPrevisMessage } from "../../previs-messages";
import { PrevisInteractionScope } from "../../previs-interaction-scope";
import { resumeVisibleVideo } from "./resume-visible-video";

/** Video transport adapter only. The document and playback clock stay in Rust. */
export function PrevisViewport({
  url,
  busy,
  allowPlacement = true,
  contextKey,
  ...interactions
}: {
  url: string | null;
  busy: boolean;
  allowPlacement?: boolean;
  contextKey: string;
} & PrevisInteractions) {
  const parent = useRef<HTMLDivElement>(null);
  const stream = useRef<PixelStreaming | null>(null);
  const [message, setMessage] = useState("");
  const [playing, setPlaying] = useState(false);
  const [viewState, setViewState] = useState({
    status: "",
    selection: "",
    workLight: "工作照明",
    move: false,
    cutaway: false,
  });
  const callbacks = useRef(interactions);
  callbacks.current = interactions;
  const canMove = useRef(allowPlacement);
  canMove.current = allowPlacement;
  const scope = useRef(new PrevisInteractionScope());
  scope.current.update(contextKey, allowPlacement);
  const [retry, setRetry] = useState(0);
  useEffect(() => {
    setPlaying(false);
    setViewState({
      status: "",
      selection: "",
      workLight: "工作照明",
      move: false,
      cutaway: false,
    });
    setMessage(url ? "正在连接三维画面…" : "");
    if (!url || !parent.current) return;
    let active = true;
    const container = parent.current;
    let cleanup = () => {};
    void import("@epicgames-ps/lib-pixelstreamingfrontend-ue5.8")
      .then(
        ({
          Config,
          Flags,
          NumericParameters,
          PixelStreaming,
          TextParameters,
        }) => {
          if (!active) return;
          const config = new Config({
            useUrlParams: false,
            initialSettings: {
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
            },
          });
          const player = new PixelStreaming(config, {
            videoElementParent: container,
          });
          stream.current = player;
          const proposals = new Set<string>();
          let connectionEpoch = 0;
          player.addResponseEventListener("stagemaster-view", (response) => {
            if (!active) return;
            const value = readPrevisMessage(response);
            if (!value) return;
            if (value.kind === "state") setViewState(value);
            if (value.kind === "selection") {
              void callbacks.current.onSelect(value.fixtureId).then((ok) => {
                if (active && !ok)
                  player.emitUIInteraction({
                    action: "select",
                    fixtureId: callbacks.current.selectedId,
                  });
              });
            }
            if (value.kind === "placement" && !proposals.has(value.requestId)) {
              proposals.add(value.requestId);
              if (proposals.size > 32)
                proposals.delete(proposals.values().next().value!);
              const { generation, version, placement } = value;
              const epoch = connectionEpoch,
                receivedAt = performance.now();
              const permitted = scope.current.capture();
              if (!permitted()) {
                player.emitUIInteraction({
                  action: "placementResult",
                  requestId: value.requestId,
                  accepted: false,
                });
                return;
              }
              void callbacks.current
                .onPlacement(
                  { generation, version, placement },
                  () =>
                    active &&
                    permitted() &&
                    connectionEpoch === epoch &&
                    performance.now() - receivedAt < 2500,
                )
                .then((accepted) => {
                  if (active)
                    player.emitUIInteraction({
                      action: "placementResult",
                      requestId: value.requestId,
                      accepted,
                    });
                });
            }
          });
          const status = (text: string) => {
            connectionEpoch++;
            if (active) {
              setPlaying(false);
              setMessage(text);
            }
          };
          player.addEventListener("playStream", () => {
            if (active) {
              setPlaying(true);
              setMessage("");
              if (!canMove.current)
                player.emitUIInteraction({ action: "inspect" });
            }
          });
          player.addEventListener("webRtcDisconnected", () =>
            status("三维画面已断开，可以重新连接"),
          );
          player.addEventListener("webRtcFailed", () =>
            status("三维画面连接失败，可以重试"),
          );
          player.addEventListener("playStreamRejected", () =>
            status("点击播放三维画面"),
          );
          player.addEventListener("playStreamError", () =>
            status("三维画面暂时无法播放"),
          );
          // Official keyboard input listens to the document. Enable it only while the viewport owns focus.
          const focus = () => config.setFlagEnabled(Flags.KeyboardInput, true);
          const blur = () => {
            config.setFlagEnabled(Flags.KeyboardInput, false);
            player.emitUIInteraction({ action: "cancel" });
          };
          container.addEventListener("focusin", focus);
          container.addEventListener("focusout", blur);
          const stopResume = resumeVisibleVideo(container, () =>
            player.play(),
          );
          player.connect();
          cleanup = () => {
            stopResume();
            container.removeEventListener("focusin", focus);
            container.removeEventListener("focusout", blur);
            config.setFlagEnabled(Flags.KeyboardInput, false);
            player.emitUIInteraction({ action: "inspect" });
            player.removeResponseEventListener("stagemaster-view");
            player.disconnect();
            stream.current = null;
            container.replaceChildren();
          };
        },
      )
      .catch(() => {
        if (active) setMessage("三维视窗组件加载失败，请重新打开");
      });
    return () => {
      active = false;
      cleanup();
    };
  }, [url, retry]);
  useEffect(() => {
    stream.current?.emitUIInteraction({ action: "cancel" });
    stream.current?.emitUIInteraction({ action: "inspect" });
    setViewState((state) => ({ ...state, move: false }));
  }, [contextKey, allowPlacement]);
  useEffect(() => {
    if (playing)
      stream.current?.emitUIInteraction({
        action: "select",
        fixtureId: interactions.selectedId,
      });
  }, [playing, interactions.selectedId]);
  function view(action: string) {
    stream.current?.emitUIInteraction({ action });
  }
  return (
    <div className="previs-live">
      <div className="previs-tools" aria-label="三维视图操作">
        <button disabled={!playing} onClick={() => view("perspective")}>
          透视
        </button>
        <button disabled={!playing} onClick={() => view("top")}>
          俯视
        </button>
        <button disabled={!playing} onClick={() => view("all")}>
          查看全场
        </button>
        <button disabled={!playing} onClick={() => view("selected")}>
          聚焦所选
        </button>
        {allowPlacement && (
          <button
            disabled={!playing || busy}
            aria-pressed={viewState.move}
            onClick={() => {
              const player = stream.current;
              if (viewState.move) view("inspect");
              else {
                const permitted = scope.current.capture();
                void callbacks.current.onPrepareMove().then((ok) => {
                  if (ok && permitted() && player && player === stream.current)
                    player.emitUIInteraction({ action: "move" });
                });
              }
            }}
          >
            移动灯位
          </button>
        )}
        <button
          disabled={!playing}
          aria-pressed={viewState.cutaway}
          onClick={() => view("cutaway")}
        >
          剖视
        </button>
        <button disabled={!playing} onClick={() => view("workLight")}>
          {viewState.workLight}
        </button>
      </div>
      <div className="previs-viewport" aria-label="三维舞台视窗">
        <div
          ref={parent}
          className="previs-video"
          tabIndex={0}
          aria-label="三维舞台操作区域"
          onPointerDownCapture={() =>
            parent.current?.focus({ preventScroll: true })
          }
          onPointerLeave={(event) => {
            if (event.buttons) view("cancel");
          }}
          onKeyDown={(event) => {
            if (event.key === "Escape") view("cancel");
          }}
        />
        {!playing && (
          <div className="previs-overlay" role="status">
            <p>{url ? message : "开启三维预演，查看当前工程的灯光与空间"}</p>
            {url && (
              <div>
                <button onClick={() => stream.current?.play()}>播放画面</button>
                <button onClick={() => setRetry((value) => value + 1)}>
                  重新连接
                </button>
              </div>
            )}
          </div>
        )}
      </div>
      <div className="previs-footer">
        <span role="status">
          {playing
            ? `${viewState.status} · ${viewState.selection}`
            : "三维画面未就绪"}
        </span>
        <span>
          {viewState.move
            ? "拖动灯具调整水平位置 · Esc 取消 · 松手应用"
            : "右键或 Option 拖动旋转 · 加 Shift 平移 · 滚动缩放"}
        </span>
      </div>
    </div>
  );
}
