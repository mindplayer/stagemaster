import { createPrevisResponder } from "../../previs-responses";
import { useEffect, useRef, useState } from "react";
import type { PixelStreaming } from "@epicgames-ps/lib-pixelstreamingfrontend-ue5.8";
import type {
  PrevisInteractions,
  PrevisTool,
  MarqueeMode,
} from "../../previs-types";

import { PrevisInteractionScope } from "../../previs-interaction-scope";

import { PrevisSelectionTools } from "./PrevisSelectionTools";
import { PrevisMoveTools } from "./PrevisMoveTools";
import { resumeVisibleVideo } from "./resume-visible-video";
import { observePrevisInputGeometry } from "./previs-input-geometry";
import {
  observePrevisConnection,
  previsConnectionView,
} from "./previs-connection";

/** Video transport adapter only. The document and playback clock stay in Rust. */
export function PrevisViewport({
  url,
  busy,
  allowPlacement = true,
  placementLocked = false,
  placementProblem = "",
  contextKey,
  ...interactions
}: {
  url: string | null;
  busy: boolean;
  allowPlacement?: boolean;
  placementLocked?: boolean;
  placementProblem?: string;
  contextKey: string;
} & PrevisInteractions) {
  const parent = useRef<HTMLDivElement>(null);
  const stream = useRef<PixelStreaming | null>(null);
  const [connection, setConnection] = useState(
    previsConnectionView("disabled"),
  );
  const { message, playing, canPlay } = connection;
  const [viewState, setViewState] = useState({
    status: "",
    selection: "",
    workLight: "工作照明",
    move: false,
    cutaway: false,
    interactionVersion: 0,
    marqueeSupported: false,
    allObjects: false,
    marqueeMode: "replace" as MarqueeMode,
    selectionThrough: false,
    vertical: false,
    tool: "horizontal" as PrevisTool,
  });
  const callbacks = useRef(interactions);
  callbacks.current = interactions;
  const incompatible = viewState.interactionVersion !== 4;
  const selectionCount = (
    interactions.selectedTargets ?? interactions.selectedIds
  ).length;
  const mixed = !!interactions.selectedTargets?.some(
    (t) => t.kind !== "placement",
  );
  const tooMany = selectionCount > 256;
  const canPlace =
    allowPlacement &&
    !placementLocked &&
    !placementProblem &&
    !tooMany &&
    !incompatible;
  const canMove = useRef(canPlace);
  canMove.current = canPlace;
  const scope = useRef(new PrevisInteractionScope());
  const selectionScope = useRef(new PrevisInteractionScope());
  const selectionKey = JSON.stringify(
    interactions.selectedTargets ?? interactions.selectedIds,
  );
  scope.current.update(`${contextKey}:${selectionKey}`, canPlace);
  selectionScope.current.update(`${contextKey}:${selectionKey}`, true);
  const [retry, setRetry] = useState(0);
  useEffect(() => {
    setConnection(previsConnectionView(url ? "connecting" : "disabled"));
    setViewState({
      status: "",
      selection: "",
      workLight: "工作照明",
      move: false,
      cutaway: false,
      interactionVersion: 0,
      marqueeSupported: false,
      allObjects: false,
      marqueeMode: "replace" as MarqueeMode,
      selectionThrough: false,
      vertical: false,
      tool: "horizontal" as PrevisTool,
    });
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
              // The local server publishes readiness after an empty directory. Upstream
              // polling would otherwise resubscribe when its already-scheduled query returns.
              [Flags.WaitForStreamer]: false,
              [Flags.MatchViewportResolution]: false,
              [NumericParameters.MaxReconnectAttempts]: 5,
            },
          });
          const player = new PixelStreaming(config, {
            videoElementParent: container,
          });
          stream.current = player;
          let connectionEpoch = 0;
          player.addResponseEventListener(
            "stagemaster-view",
            createPrevisResponder({
              active: () => active,
              callbacks: () => callbacks.current,
              state: setViewState,
              send: (value) => player.emitUIInteraction(value),
              scope: scope.current,
              selectionScope: selectionScope.current,
              connection: () => connectionEpoch,
            }),
          );
          const stopConnection = observePrevisConnection(
            player,
            (next) => {
              if (!active) return;
              setConnection(next);
              if (next.playing && !canMove.current)
                player.emitUIInteraction({ action: "inspect" });
            },
            () => connectionEpoch++,
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
            stopConnection();
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
        if (active) setConnection(previsConnectionView("loadError"));
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
      stream.current?.emitUIInteraction(
        interactions.selectedTargets && viewState.interactionVersion === 4
          ? { action: "selectTargets", targets: interactions.selectedTargets }
          : { action: "selectGroup", fixtureIds: interactions.selectedIds },
      );
  }, [playing, selectionKey, contextKey, viewState.interactionVersion]);
  function view(action: string) {
    if (
      [
        "cancel",
        "inspect",
        "selectionThrough",
        "selectAllObjects",
        "selectFixturesOnly",
        "marqueeReplace",
        "marqueeAdd",
        "marqueeRemove",
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
          disabled={!playing || !selectionCount}
          onClick={() => view("selected")}
        >
          聚焦所选
        </button>
        {allowPlacement && (
          <button
            disabled={!playing || busy || !canPlace}
            title={
              placementProblem ||
              (placementLocked
                ? "所选灯位包含锁定对象，请在属性栏解锁"
                : tooMany
                  ? "一次最多移动 256 个对象"
                  : incompatible
                    ? "三维组件需要更新后才能移动对象"
                    : undefined)
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
            {placementProblem
              ? "所选对象不可移动"
              : placementLocked
                ? "所选灯位有锁定"
                : tooMany
                  ? "所选超过 256 个"
                  : interactions.selectedTargets
                    ? "移动所选对象"
                    : "布置所选灯具"}
          </button>
        )}
        {allowPlacement && (
          <PrevisMoveTools
            moving={viewState.move}
            fixturesOnly={!mixed}
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
        <PrevisSelectionTools
          objectsSupported={
            interactions.selectedTargets !== undefined && !incompatible
          }
          allObjects={viewState.allObjects}
          supported={viewState.marqueeSupported}
          disabled={!playing || viewState.move}
          through={viewState.selectionThrough}
          mode={viewState.marqueeMode}
          onAction={view}
        />
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
                {canPlay && (
                  <button onClick={() => stream.current?.play()}>
                    播放画面
                  </button>
                )}
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
            ? "三维组件版本不兼容，请更新后再移动对象"
            : viewState.move
              ? `${viewState.tool === "rotate" ? "左右拖动整组旋转" : viewState.tool === "scale" ? "左右拖动调整灯间距" : viewState.vertical ? "拖动升降所选对象" : "水平拖动所选对象"} · Esc 取消 · 松手整组应用`
              : viewState.marqueeSupported
                ? `${viewState.allObjects ? "拖框选择对象" : "拖框选灯"} · Shift 加选 · ⌘/Ctrl 减选 · 右键或 Option 旋转 · 滚动缩放`
                : "Shift 点击增减选择 · 右键或 Option 旋转 · 加 Shift 平移 · 滚动缩放"}
        </span>
      </div>
    </div>
  );
}
