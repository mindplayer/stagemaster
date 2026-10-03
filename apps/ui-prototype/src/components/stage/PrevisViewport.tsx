import { useEffect, useRef, useState } from "react";
import type { PixelStreaming } from "@epicgames-ps/lib-pixelstreamingfrontend-ue5.8";
import type { PrevisInteractions, PrevisTool } from "../../previs-types";
import { readPrevisMessage } from "../../previs-messages";
import { PrevisInteractionScope } from "../../previs-interaction-scope";
import { sameFixtureSelection } from "../../previs-selection";
import { PrevisMoveTools } from "./PrevisMoveTools";
import { resumeVisibleVideo } from "./resume-visible-video";
import { observePrevisInputGeometry } from "./previs-input-geometry";

/** Video transport adapter only. The document and playback clock stay in Rust. */
export function PrevisViewport({
  url,
  busy,
  allowPlacement = true,
  placementLocked = false,
  contextKey,
  ...interactions
}: {
  url: string | null;
  busy: boolean;
  allowPlacement?: boolean;
  placementLocked?: boolean;
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
    interactionVersion: 0,
    vertical: false,
    tool: "horizontal" as PrevisTool,
  });
  const callbacks = useRef(interactions);
  callbacks.current = interactions;
  const incompatible = viewState.interactionVersion !== 3;
  const tooMany = interactions.selectedIds.length > 256;
  const canPlace =
    allowPlacement && !placementLocked && !tooMany && !incompatible;
  const canMove = useRef(canPlace);
  canMove.current = canPlace;
  const scope = useRef(new PrevisInteractionScope());
  const selectionScope = useRef(new PrevisInteractionScope());
  const selectionKey = JSON.stringify(interactions.selectedIds);
  scope.current.update(`${contextKey}:${selectionKey}`, canPlace);
  selectionScope.current.update(`${contextKey}:${selectionKey}`, true);
  const [retry, setRetry] = useState(0);
  useEffect(() => {
    setPlaying(false);
    setViewState({
      status: "",
      selection: "",
      workLight: "工作照明",
      move: false,
      cutaway: false,
      interactionVersion: 0,
      vertical: false,
      tool: "horizontal" as PrevisTool,
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
          let selectionEpoch = 0;
          player.addResponseEventListener("stagemaster-view", (response) => {
            if (!active) return;
            const value = readPrevisMessage(response);
            if (!value) return;
            if (value.kind === "state") setViewState(value);
            if (value.kind === "selection" || value.kind === "selectionGroup") {
              const ids =
                value.kind === "selectionGroup"
                  ? value.fixtureIds
                  : value.fixtureId
                    ? [value.fixtureId]
                    : [];
              const selection = ++selectionEpoch,
                connection = connectionEpoch;
              const current = selectionScope.current.capture();
              void callbacks.current
                .onSelect(
                  ids,
                  () =>
                    active &&
                    current() &&
                    selection === selectionEpoch &&
                    connection === connectionEpoch,
                )
                .then((ok) => {
                  if (active && selection === selectionEpoch && !ok)
                    player.emitUIInteraction({
                      action: "selectGroup",
                      fixtureIds: callbacks.current.selectedIds,
                    });
                });
            }
            if (value.kind === "placement") {
              player.emitUIInteraction({
                action: "placementResult",
                requestId: value.requestId,
                accepted: false,
              });
            }
            if (
              (value.kind === "translation" || value.kind === "transform") &&
              !proposals.has(value.requestId)
            ) {
              proposals.add(value.requestId);
              if (proposals.size > 32)
                proposals.delete(proposals.values().next().value!);
              const { generation, version, fixtureIds } = value;
              const epoch = connectionEpoch,
                receivedAt = performance.now();
              const permitted = scope.current.capture();
              if (
                !permitted() ||
                !sameFixtureSelection(callbacks.current.selectedIds, fixtureIds)
              ) {
                player.emitUIInteraction({
                  action: "placementResult",
                  requestId: value.requestId,
                  accepted: false,
                });
                return;
              }
              const valid = () =>
                active &&
                permitted() &&
                connectionEpoch === epoch &&
                performance.now() - receivedAt < 2500;
              const proposal = { generation, version, fixtureIds };
              const pending =
                value.kind === "translation"
                  ? callbacks.current.onTranslation(
                      { ...proposal, deltaMeters: value.deltaMeters },
                      valid,
                    )
                  : callbacks.current.onTransform(
                      {
                        ...proposal,
                        yawDegrees: value.yawDegrees,
                        spacingScale: value.spacingScale,
                      },
                      valid,
                    );
              void pending.then((accepted) => {
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
            scope.current.invalidate();
            config.setFlagEnabled(Flags.KeyboardInput, false);
            player.emitUIInteraction({ action: "cancel" });
          };
          container.addEventListener("focusin", focus);
          container.addEventListener("focusout", blur);
          const stopResume = resumeVisibleVideo(container, () => player.play());
          const stopGeometry = observePrevisInputGeometry(container, () => {
            scope.current.invalidate();
            player.emitUIInteraction({ action: "cancel" });
            player.webRtcController.setUpMouseAndFreezeFrame();
          });
          player.connect();
          cleanup = () => {
            stopGeometry();
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
  }, [contextKey, canPlace]);
  useEffect(() => {
    if (playing)
      stream.current?.emitUIInteraction({
        action: "selectGroup",
        fixtureIds: interactions.selectedIds,
      });
  }, [playing, selectionKey, contextKey, viewState.interactionVersion]);
  function view(action: string) {
    if (
      [
        "cancel",
        "inspect",
        "moveHorizontal",
        "moveVertical",
        "rotate",
        "scale",
      ].includes(action)
    )
      scope.current.invalidate();
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
        <button
          disabled={!playing || !interactions.selectedIds.length}
          onClick={() => view("selected")}
        >
          聚焦所选
        </button>
        {allowPlacement && (
          <button
            disabled={!playing || busy || !canPlace}
            title={
              placementLocked
                ? "所选灯位包含锁定对象，请在属性栏解锁"
                : tooMany
                  ? "一次最多移动 256 台灯具"
                  : incompatible
                    ? "三维组件需要更新后才能移动灯位"
                    : undefined
            }
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
            {placementLocked
              ? "所选灯位有锁定"
              : tooMany
                ? "所选超过 256 台"
                : "布置所选灯具"}
          </button>
        )}
        {allowPlacement && (
          <PrevisMoveTools
            moving={viewState.move}
            tool={viewState.tool}
            contextKey={`${contextKey}:${selectionKey}`}
            onExact={(yawDegrees, spacingScale) =>
              stream.current?.emitUIInteraction({
                action: "transformExact",
                fixtureIds: callbacks.current.selectedIds,
                yawDegrees,
                spacingScale,
              })
            }
            disabled={!playing || busy || !canPlace}
            onAction={view}
          />
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
          {allowPlacement && playing && incompatible
            ? "三维组件版本不兼容，请更新后再移动灯位"
            : viewState.move
              ? `${viewState.tool === "rotate" ? "左右拖动整组旋转" : viewState.tool === "scale" ? "左右拖动调整灯间距" : viewState.vertical ? "拖动升降所选灯具" : "水平拖动所选灯具"} · Esc 取消 · 松手整组应用`
              : "Shift 点击增减选择 · 右键或 Option 旋转 · 加 Shift 平移 · 滚动缩放"}
        </span>
      </div>
    </div>
  );
}
