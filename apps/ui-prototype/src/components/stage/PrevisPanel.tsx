import { useEffect, useRef, useState } from "react";
import { PrevisViewport } from "./PrevisViewport";
import { CubeIcon } from "@phosphor-icons/react";
import type {
  ApplicationHost,
  SceneView,
  FixtureView,
} from "../../application-host";
import type {
  PrevisSource,
  PrevisStatus,
  PrevisInteractions,
} from "../../previs-types";

/** A view onto the host's single renderer. Source changes never execute playback. */
export function PrevisPanel({
  host,
  limitedFixtures = [],
  scenes,
  busy,
  generation,
  run,
  currentScene,
  followCurrent = false,
  onFollowCurrent,
  allowPlacement = true,
  placementLocked = false,
  contextKey,
  ...interactions
}: {
  host: ApplicationHost;
  scenes: SceneView[];
  limitedFixtures?: FixtureView[];
  busy: boolean;
  generation: () => number;
  run: (work: () => Promise<void>) => Promise<boolean>;
  currentScene?: SceneView;
  followCurrent?: boolean;
  onFollowCurrent?(follow: boolean): void;
  allowPlacement?: boolean;
  placementLocked?: boolean;
  contextKey: string;
} & PrevisInteractions) {
  const [status, setStatus] = useState<PrevisStatus | null>(null);
  const [problem, setProblem] = useState("");
  const revision = useRef(0);
  const commands = useRef({ run, generation });
  commands.current = { run, generation };
  const follow = useRef({
    followCurrent,
    sceneId: currentScene?.id,
    onFollowCurrent,
  });
  follow.current = {
    followCurrent,
    sceneId: currentScene?.id,
    onFollowCurrent,
  };
  useEffect(() => {
    let active = true;
    let timer: ReturnType<typeof setTimeout>;
    async function poll() {
      const version = revision.current;
      try {
        const next = await host.previs({ kind: "status" });
        if (active && version === revision.current) {
          setStatus(next);
          setProblem("");
          if (
            follow.current.followCurrent &&
            (next.source.kind !== "scene" ||
              next.source.sceneId !== follow.current.sceneId)
          )
            follow.current.onFollowCurrent?.(false);
        }
      } catch (reason) {
        if (active && version === revision.current) setProblem(String(reason));
      } finally {
        if (active) timer = setTimeout(() => void poll(), 1000);
      }
    }
    if (host.kind === "desktop") void poll();
    return () => {
      active = false;
      clearTimeout(timer);
    };
  }, [host]);
  const sceneId = currentScene?.id;
  useEffect(() => {
    if (!followCurrent || !sceneId || host.kind !== "desktop") return;
    let active = true;
    void commands.current.run(async () => {
      if (!active) return;
      revision.current++;
      const next = await host.previs({
        kind: "source",
        generation: commands.current.generation(),
        source: { kind: "scene", sceneId },
      });
      if (active) setStatus(next);
    });
    return () => {
      active = false;
    };
  }, [host, followCurrent, sceneId]);
  const source = status?.source;
  const value =
    followCurrent && currentScene
      ? "current"
      : source?.kind === "scene"
        ? `scene:${source.sceneId}`
        : (source?.kind ?? "defaults");
  function select(value: string) {
    const follow = value === "current" && !!currentScene;
    const source: PrevisSource = follow
      ? { kind: "scene", sceneId: currentScene!.id }
      : value.startsWith("scene:")
        ? { kind: "scene", sceneId: value.slice(6) }
        : { kind: value === "playback" ? "playback" : "defaults" };
    void run(async () => {
      revision.current++;
      setStatus(
        await host.previs({ kind: "source", generation: generation(), source }),
      );
      onFollowCurrent?.(follow);
    });
  }
  return (
    <div className="previs-workspace">
      <section className="wb-previs" aria-label="三维预演">
        <button
          disabled={busy || host.kind !== "desktop"}
          onClick={() =>
            void run(async () => {
              revision.current++;
              setStatus(
                await host.previs({
                  kind: status?.enabled ? "disable" : "enable",
                }),
              );
            })
          }
        >
          <CubeIcon />
          {status?.enabled ? "停止预演" : "开启预演"}
        </button>
        <label>
          灯光来源
          <select
            aria-label="三维灯光来源"
            value={value}
            disabled={busy || host.kind !== "desktop"}
            onChange={(event) => select(event.target.value)}
          >
            {currentScene && <option value="current">当前场景 · 静态值</option>}
            <option value="defaults">灯具默认值</option>
            <option value="playback">跟随播放预览</option>
            {scenes.map((scene) => (
              <option key={scene.id} value={`scene:${scene.id}`}>
                {scene.name}
                {scene.effects.length ? " · 静态值" : ""}
              </option>
            ))}
            {source?.kind === "scene" &&
              !scenes.some((scene) => scene.id === source.sceneId) && (
                <option value={value}>原场景已删除</option>
              )}
          </select>
        </label>
        <span role="status">
          {problem ||
            status?.problem ||
            (status?.connected
              ? "三维已连接"
              : status?.enabled
                ? "三维正在启动"
                : "三维已关闭")}
        </span>
      </section>
      {!!limitedFixtures.length && (
        <details className="previs-limitations">
          <summary>
            {limitedFixtures.length} 台灯具仅显示灯位与朝向，功能光束暂未模拟
          </summary>
          <p>{limitedFixtures.map((f) => f.name).join("、")}</p>
          <p>
            色盘、图案、快门和棱镜尚无光学模型，因此隐藏这些灯具的光束。实际通道值以播放监看为准。
          </p>
        </details>
      )}
      <PrevisViewport
        url={status?.viewerUrl ?? null}
        busy={busy}
        allowPlacement={allowPlacement}
        placementLocked={placementLocked}
        contextKey={contextKey}
        {...interactions}
      />
    </div>
  );
}
