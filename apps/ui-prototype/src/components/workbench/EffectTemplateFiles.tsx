import { useEffect, useRef, useState } from "react";
import type {
  ApplicationHost,
  FixtureView,
  SceneView,
} from "../../application-host";
import type { ImportedEffectTemplate } from "../../effect-template-types";
import {
  canExportEffectTemplate,
  effectTemplateContext,
} from "../../effect-template-tools";
import { EffectTemplateReview } from "./EffectTemplateReview";
import "./effect-template-files.css";
export interface EffectTemplateFileProps {
  host: ApplicationHost;
  generation: number;
  visible: boolean;
  busy: boolean;
  scene: SceneView;
  fixtures: FixtureView[];
  selected: string[];
  operationError?: string;
  capture(): Promise<number | null>;
  onApply(generation: number, token: string): Promise<boolean>;
}
export function EffectTemplateFiles({
  host,
  generation,
  visible,
  busy,
  scene,
  fixtures,
  selected,
  capture,
  onApply,
  operationError = "",
}: EffectTemplateFileProps) {
  const [exportId, setExportId] = useState("");
  const [working, setWorking] = useState(false),
    [error, setError] = useState(""),
    [message, setMessage] = useState("");
  const [file, setFile] = useState<ImportedEffectTemplate | null>(null);
  const pending = useRef<ImportedEffectTemplate | null>(null),
    active = useRef(false),
    mounted = useRef(true);
  const button = useRef<HTMLButtonElement>(null),
    root = useRef<HTMLElement>(null);
  const signature = effectTemplateContext(scene.id, selected, visible);
  const current = useRef({ signature, epoch: 0, generation, visible });
  current.current = {
    signature,
    epoch:
      current.current.epoch + (signature !== current.current.signature ? 1 : 0),
    generation,
    visible,
  };
  function release(token: string) {
    void host.cancelEffectTemplate(token).catch(() => {});
  }
  function close() {
    if (pending.current) release(pending.current.token);
    pending.current = null;
    setFile(null);
    setError("");
    requestAnimationFrame(() => {
      if (
        mounted.current &&
        current.current.visible &&
        root.current?.isConnected
      )
        button.current?.focus();
    });
  }
  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
      if (pending.current) release(pending.current.token);
    };
  }, [host]);
  useEffect(() => {
    if (pending.current) {
      release(pending.current.token);
      pending.current = null;
      setFile(null);
      setError("");
    }
  }, [generation, signature]);
  const eligible = scene.effects.filter(canExportEffectTemplate);
  const exporting = eligible.find((e) => e.id === exportId) ?? eligible[0];
  async function start(kind: "import" | "export") {
    if (active.current || busy || !visible || !root.current?.isConnected)
      return;
    if (kind === "import" && !selected.length) return;
    if (kind === "export" && !exporting) return;
    active.current = true;
    setWorking(true);
    setError("");
    setMessage("");
    const epoch = current.current.epoch,
      target = scene.id,
      ids = [...selected];
    try {
      const version = await capture();
      if (version === null || !mounted.current) return;
      if (
        epoch !== current.current.epoch ||
        !current.current.visible ||
        !root.current?.isConnected
      )
        throw new Error("编辑上下文已变化，请返回动态效果后重试");
      if (kind === "import") {
        const imported = await host.importEffectTemplate(version, target, ids);
        if (!imported) {
          if (mounted.current) setMessage("已取消导入");
          return;
        }
        if (
          !mounted.current ||
          epoch !== current.current.epoch ||
          imported.generation !== current.current.generation ||
          !root.current?.isConnected
        ) {
          release(imported.token);
          if (mounted.current)
            throw new Error("场景、选择或工程已变化，模板未应用，请重新导入");
          return;
        }
        pending.current = imported;
        setFile(imported);
      } else {
        const result = await host.exportEffectTemplate(
          version,
          target,
          exporting.id,
        );
        if (mounted.current)
          setMessage(
            result.path
              ? `已导出：${result.path}${result.warning ? ` · ${result.warning}` : ""}`
              : "已取消导出",
          );
      }
    } catch (reason) {
      if (mounted.current)
        setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      active.current = false;
      if (mounted.current) setWorking(false);
    }
  }
  return (
    <section
      ref={root}
      className="effect-template-files"
      aria-label="灯效模板文件"
    >
      <div className="effect-template-actions">
        <button
          ref={button}
          disabled={busy || working || !selected.length}
          title="把灯效模板绑定到当前所选灯具，先检查再应用"
          onClick={() => void start("import")}
        >
          导入灯效模板
        </button>
        <select
          aria-label="导出亮度效果"
          disabled={busy || working || !eligible.length}
          value={exporting?.id ?? ""}
          onChange={(e) => setExportId(e.target.value)}
        >
          {!eligible.length && <option value="">暂无可导出的亮度效果</option>}
          {eligible.map((e) => (
            <option value={e.id} key={e.id}>
              {e.name}
            </option>
          ))}
        </select>
        <button
          disabled={busy || working || !exporting}
          onClick={() => void start("export")}
        >
          导出模板
        </button>
      </div>
      <p className="wb-dim">
        {working
          ? "正在处理模板文件…"
          : "支持亮度呼吸、往返和脉冲；先选择目标灯具，再导入模板。"}
      </p>
      {error && !file && (
        <p role="alert" className="wb-error">
          {error}
        </p>
      )}
      {message && <p role="status">{message}</p>}
      {file && (
        <EffectTemplateReview
          file={file}
          fixtures={fixtures}
          sceneName={scene.name}
          busy={busy || working}
          error={error || operationError}
          onCancel={close}
          onApply={async () => {
            if (active.current || !pending.current) return false;
            if (
              file.generation !== current.current.generation ||
              !current.current.visible
            )
              throw new Error("检查已失效，请重新导入");
            active.current = true;
            setWorking(true);
            try {
              const ok = await onApply(file.generation, file.token);
              if (ok) {
                pending.current = null;
                if (mounted.current) setMessage("灯效模板已应用，可撤销恢复");
              }
              return ok;
            } finally {
              active.current = false;
              if (mounted.current) setWorking(false);
            }
          }}
        />
      )}
    </section>
  );
}
