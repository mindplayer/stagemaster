import "./scene-preview.css";
import { useEffect, useRef, useState } from "react";
import {
  PlayIcon,
  PauseIcon,
  StopIcon,
  ArrowClockwiseIcon,
  SkipForwardIcon,
} from "@phosphor-icons/react";
import type { ApplicationHost, SceneView } from "../../application-host";
import type {
  PreviewCommand,
  PreviewSnapshot,
  SequenceView,
} from "../../sequence-types";
import { seconds } from "../../sequence-tools";
import { PreviewOutput } from "./PreviewOutput";
import { startScenePreview } from "./scene-preview-action";

export function PreviewPanel({
  host,
  sequence,
  scene,
  stepId,
  generation,
  busy,
  beforeAction,
  visible,
  onView3d,
}: {
  host: ApplicationHost;
  sequence?: SequenceView;
  scene?: SceneView;
  stepId: string;
  generation: number;
  busy: boolean;
  beforeAction(): Promise<boolean>;
  visible: boolean;
  onView3d?(): void;
}) {
  const [snapshot, setSnapshot] = useState<PreviewSnapshot>({
    epoch: 0,
    controlSerial: 0,
    loaded: null,
  });
  const [error, setError] = useState("");
  const [working, setWorking] = useState(false);
  const current = useRef(snapshot);
  const controlBusy = useRef(false);
  const epochRequest = useRef(0);
  const alive = useRef(true);
  const target = useRef(scene?.id);
  target.current = visible ? scene?.id : undefined;
  const publish = (value: PreviewSnapshot) => {
    if (alive.current) {
      current.current = value;
      setSnapshot(value);
    }
  };
  useEffect(() => {
    alive.current = true;
    return () => {
      alive.current = false;
    };
  }, []);
  useEffect(() => {
    let disposed = false;
    let timer: ReturnType<typeof setTimeout>;
    const poll = async () => {
      const version = epochRequest.current;
      if (!controlBusy.current) {
        try {
          const next = await host.preview({ kind: "snapshot" });
          if (!disposed && version === epochRequest.current) publish(next);
        } catch (reason) {
          if (!disposed && version === epochRequest.current)
            setError(String(reason));
        }
      }
      if (!disposed) timer = setTimeout(poll, visible ? 80 : 500);
    };
    void poll();
    return () => {
      disposed = true;
      clearTimeout(timer);
    };
  }, [host, visible]);
  async function act(command: PreviewCommand | "load" | "startScene") {
    if (controlBusy.current) return;
    controlBusy.current = true;
    epochRequest.current++;
    setWorking(true);
    setError("");
    try {
      // Stop/pause must remain available even when an unrelated editor draft is invalid.
      if (
        (typeof command === "string" ||
          !["stop", "pause"].includes(command.kind)) &&
        !(await beforeAction())
      )
        return;
      // Flush can change the project generation, so obtain the host's authoritative snapshot.
      if (command === "startScene" && scene) {
        const result = await startScenePreview(
          host,
          scene.id,
          () => alive.current && target.current === scene.id,
        );
        publish(result);
        if (alive.current && target.current === scene.id) onView3d?.();
      } else if (command === "load") {
        if (!sequence && !scene) return;
        const project = await host.request({ kind: "snapshot" });
        publish(
          await host.preview(
            scene
              ? {
                  kind: "loadScene",
                  generation: project.generation,
                  sceneId: scene.id,
                }
              : {
                  kind: "load",
                  generation: project.generation,
                  sequenceId: sequence!.id,
                },
          ),
        );
      } else if (typeof command !== "string") {
        // Several views share one player. Read the current serial, never restart a local counter.
        const latest = await host.preview({ kind: "snapshot" });
        if (latest.epoch !== current.current.epoch) {
          publish(latest);
          throw new Error("预览内容已更换，请确认后重试");
        }
        publish(
          await host.preview({
            kind: "control",
            epoch: current.current.epoch,
            serial: latest.controlSerial + 1,
            command,
          }),
        );
      }
    } catch (reason) {
      if (alive.current)
        setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      controlBusy.current = false;
      if (alive.current) setWorking(false);
    }
  }
  const loaded = snapshot.loaded;
  const same =
    !!loaded &&
    (scene
      ? loaded.sceneId === scene.id
      : !loaded.sceneId && loaded.sequenceId === sequence?.id);
  const ready = same && !loaded?.stale;
  const active = loaded?.steps.find((s) => s.id === loaded.stepId);
  const status = loaded
    ? {
        idle: "待执行",
        running: "运行中",
        paused: "已暂停",
        finished: "已结束",
      }[loaded.status]
    : "未载入";
  const phase = !loaded?.stepId
    ? ""
    : loaded.elapsedMs < loaded.delayMs
      ? "延时"
      : loaded.elapsedMs < loaded.delayMs + loaded.fadeMs
        ? "渐变"
        : loaded.waitMs === null
          ? loaded.sceneId
            ? "场景持续播放"
            : "等待手动推进"
          : "自动等待";
  const duration = loaded
    ? loaded.delayMs + loaded.fadeMs + (loaded.waitMs ?? 0)
    : 0;
  const progress =
    duration === 0 ? 0 : Math.min(1, (loaded?.elapsedMs ?? 0) / duration);
  // generation triggers a render after edits; authoritative stale status comes from the host.
  return (
    <section
      className="wb-preview"
      aria-label="离线预览"
      data-generation={generation}
    >
      <div className="wb-preview-heading">
        <div>
          <span className="wb-eyebrow">离线预览</span>
          <h2>{scene ? "场景预演" : (loaded?.name ?? "列表预览")}</h2>
        </div>
        <span
          className={`wb-preview-status ${loaded?.status === "running" ? "running" : ""}`}
        >
          {status}
        </span>
      </div>
      <div className="wb-preview-toolbar">
        {scene ? (
          <button
            className="wb-primary"
            disabled={working || busy}
            title="用当前场景替换离线播放，从头预演并切至三维监看；会停止当前音乐预览"
            onClick={() => void act("startScene")}
          >
            <PlayIcon weight="fill" />
            {same && loaded?.stale
              ? "更新并预演"
              : same && loaded?.status === "running"
                ? "重新预演"
                : "预演当前场景"}
          </button>
        ) : (
          <>
            <button
              disabled={(!sequence && !scene) || working || busy}
              onClick={() => void act("load")}
              title="将当前编辑内容载入预览，并回到默认值"
            >
              <ArrowClockwiseIcon />
              {same ? "重新载入" : scene ? "载入场景" : "载入列表"}
            </button>
            <button
              className="wb-primary"
              disabled={!ready || !stepId || working || busy}
              onClick={() => void act({ kind: "execute", stepId })}
            >
              <PlayIcon weight="fill" />
              {scene ? "播放场景" : "执行所选"}
            </button>
          </>
        )}
        {!scene && (
          <button
            aria-label="执行下一步"
            title="执行下一步"
            disabled={!ready || working || busy || !loaded?.canNext}
            onClick={() => void act({ kind: "next" })}
          >
            <SkipForwardIcon />
          </button>
        )}
        <button
          disabled={
            !loaded ||
            working ||
            busy ||
            (loaded.status !== "running" && loaded.status !== "paused") ||
            (loaded.status === "paused" && !ready)
          }
          onClick={() =>
            void act({ kind: loaded?.status === "paused" ? "resume" : "pause" })
          }
        >
          {loaded?.status === "paused" ? <PlayIcon /> : <PauseIcon />}
          {loaded?.status === "paused" ? "继续" : "暂停"}
        </button>
        <button
          disabled={!loaded || working || loaded.status === "idle"}
          onClick={() => void act({ kind: "stop" })}
        >
          <StopIcon />
          停止
        </button>
        {onView3d && (
          <button disabled={!ready || working || busy} onClick={onView3d}>
            三维监看
          </button>
        )}
      </div>
      {scene && (
        <div className="scene-preview-context" aria-label="编辑与预演对象">
          <span>
            正在编辑：<strong>{scene.name}</strong>
          </span>
          <span>
            播放内容：
            <strong>
              {!loaded
                ? "尚未载入"
                : loaded.status === "idle"
                  ? `灯具默认值（${loaded.name} 已载入）`
                  : loaded.name}
            </strong>
          </span>
          {loaded && (
            <span>
              预演版本：{loaded.stale ? "工程修改前" : "与已应用工程一致"}
            </span>
          )}
        </div>
      )}
      {loaded?.stale && (
        <p className="wb-preview-warning" role="status">
          {scene
            ? "工程已修改，当前播放保留的是修改前的版本。"
            : "工程已修改，重新载入后可执行新编排。"}
        </p>
      )}
      {loaded && !same && <p className="wb-dim">当前预览：{loaded.name}</p>}
      {error && (
        <p className="wb-preview-warning" role="alert">
          {error}
        </p>
      )}
      {loaded ? (
        <>
          <div className="wb-playback-position">
            <strong>
              {active
                ? `${active.number} · ${active.name}`
                : scene
                  ? "场景已就绪"
                  : "选择步骤后执行"}
            </strong>
            <span>
              {phase}{" "}
              {active
                ? `${seconds(Math.min(loaded.elapsedMs, 86_400_000_000))} 秒`
                : ""}
            </span>
          </div>
          <progress max={1} value={progress} aria-label="当前步骤进度" />
          <PreviewOutput loaded={loaded} />
        </>
      ) : (
        <div className="wb-preview-empty">
          {scene ? "预演当前场景，查看灯光效果" : "载入场景列表，预览灯光变化"}
        </div>
      )}
    </section>
  );
}
