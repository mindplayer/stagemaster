import { useEffect, useState } from "react";
import { PrevisViewport } from "./PrevisViewport";
import { CubeIcon } from "@phosphor-icons/react";
import type { ApplicationHost, SceneView } from "../../application-host";
import type { PrevisSource, PrevisStatus, PrevisInteractions } from "../../previs-types";

export function PrevisPanel({ host, scenes, busy, generation, run, ...interactions }: {
  host: ApplicationHost;
  scenes: SceneView[];
  busy: boolean;
  generation: () => number;
  run: (work: () => Promise<void>) => Promise<boolean>;
} & PrevisInteractions) {
  const [status, setStatus] = useState<PrevisStatus | null>(null);
  const [problem, setProblem] = useState("");
  useEffect(() => {
    let active = true;
    let timer: ReturnType<typeof setTimeout>;
    async function poll() {
      try {
        const next = await host.previs({ kind: "status" });
        if (active) { setStatus(next); setProblem(""); }
      } catch (reason) {
        if (active) setProblem(String(reason));
      } finally {
        if (active) timer = setTimeout(() => void poll(), 1000);
      }
    }
    if (host.kind === "desktop") void poll();
    return () => { active = false; clearTimeout(timer); };
  }, [host]);
  const source = status?.source;
  const value = source?.kind === "scene" ? `scene:${source.sceneId}` : source?.kind ?? "defaults";
  function select(value: string) {
    const source: PrevisSource = value.startsWith("scene:")
      ? { kind: "scene", sceneId: value.slice(6) }
      : { kind: value === "playback" ? "playback" : "defaults" };
    void run(async () => setStatus(await host.previs({ kind: "source", generation: generation(), source })));
  }
  return <div className="previs-workspace"><section className="wb-previs" aria-label="三维预演">
    <button disabled={busy || host.kind !== "desktop"} onClick={() => void run(async () => {
      setStatus(await host.previs({ kind: status?.enabled ? "disable" : "enable" }));
    })}><CubeIcon />{status?.enabled ? "停止预演" : "开启预演"}</button>
    <label>灯光来源
      <select aria-label="三维灯光来源" value={value} disabled={busy || host.kind !== "desktop"} onChange={(event) => select(event.target.value)}>
        <option value="defaults">灯具默认值</option>
        <option value="playback">跟随播放预览</option>
        {scenes.map((scene) => <option key={scene.id} value={`scene:${scene.id}`}>{scene.name}{scene.effects.length ? " · 静态值" : ""}</option>)}
        {source?.kind === "scene" && !scenes.some((scene) => scene.id === source.sceneId) && <option value={value}>原场景已删除</option>}
      </select>
    </label>
    <span role="status">{problem || status?.problem || (status?.connected ? "三维已连接" : status?.enabled ? "三维正在启动" : "三维已关闭")}</span>
  </section><PrevisViewport url={status?.viewerUrl ?? null} busy={busy} {...interactions} /></div>;
}
