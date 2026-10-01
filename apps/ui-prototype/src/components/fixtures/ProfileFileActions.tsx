import { useEffect, useRef, useState, type Ref } from "react";
import type { ApplicationHost } from "../../application-host";
import type { ProfileView } from "../../fixture-types";
import type {
  ExportedProfile,
  ImportedProfile,
} from "../../profile-file-types";
import "./profile-file.css";

export function ProfileFileActions({
  host,
  generation,
  profile,
  hasDrafts,
  busy,
  visible,
  capture,
  onImport,
  importButtonRef,
}: {
  host: ApplicationHost;
  generation: number;
  profile: ProfileView | undefined;
  hasDrafts: boolean;
  busy: boolean;
  visible: boolean;
  capture(): Promise<number | null>;
  onImport(file: ImportedProfile): void;
  importButtonRef?: Ref<HTMLButtonElement>;
}) {
  const [working, setWorking] = useState(false);
  const [error, setError] = useState("");
  const [message, setMessage] = useState("");
  const [receipt, setReceipt] = useState<
    (ExportedProfile & { name: string }) | null
  >(null);
  const active = useRef(false),
    mounted = useRef(true);
  const current = useRef({ generation, hasDrafts, visible, visibility: 0 });
  current.current = {
    generation,
    hasDrafts,
    visible,
    visibility:
      current.current.visibility +
      (current.current.visible !== visible ? 1 : 0),
  };
  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
    };
  }, []);
  async function start(kind: "import" | "export") {
    if (
      active.current ||
      busy ||
      !visible ||
      (kind === "export" && (hasDrafts || !profile?.authorable))
    )
      return;
    active.current = true;
    setWorking(true);
    setError("");
    setMessage("");
    setReceipt(null);
    const context = current.current.visibility;
    try {
      const version = await capture();
      if (version === null || !mounted.current) return;
      if (!current.current.visible || context !== current.current.visibility)
        throw new Error("已离开工程灯库，请返回后重试");
      if (kind === "import") {
        const file = await host.importProfile(version);
        if (!mounted.current) return;
        if (!file) {
          setMessage("已取消导入");
          return;
        }
        if (
          file.generation !== current.current.generation ||
          current.current.hasDrafts ||
          !current.current.visible ||
          context !== current.current.visibility
        ) {
          throw new Error("编辑上下文已变化，文件未加入草稿，请重新导入");
        }
        onImport(file);
      } else if (profile) {
        const result = await host.exportProfile(version, profile.id);
        if (!mounted.current) return;
        if (result.path) setReceipt({ ...result, name: profile.name });
        else setMessage("已取消导出");
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
    <section className="profile-file-actions" aria-label="灯具模式文件">
      <div className="profile-file-buttons">
        <button
          ref={importButtonRef}
          disabled={busy || working}
          onClick={() => void start("import")}
        >
          导入模式文件
        </button>
        <button
          disabled={busy || working || hasDrafts || !profile?.authorable}
          title={
            hasDrafts
              ? "请先保存或取消当前模式草稿"
              : "导出所选模式，供其他工程使用"
          }
          onClick={() => void start("export")}
        >
          导出所选模式
        </button>
        <span>
          {working
            ? "正在处理模式文件…"
            : "StageMaster 模式文件 · .smfixture.json"}
        </span>
      </div>
      {hasDrafts && <p>完成或取消当前草稿后，可导出模式文件。</p>}
      {error && (
        <p role="alert" className="wb-error">
          {error}
        </p>
      )}
      {message && <p role="status">{message}</p>}
      {receipt && (
        <div role="status" className="profile-file-receipt">
          <strong>已导出：{receipt.name}</strong>
          <span>{receipt.path}</span>
          {receipt.generation !== generation && (
            <span>工程已变化；此文件保留导出时的模式。</span>
          )}
          {receipt.warning && <span>{receipt.warning}</span>}
        </div>
      )}
    </section>
  );
}

export function ImportedProfileNotice({
  file,
  name,
  sameNameCount,
}: {
  file: ImportedProfile;
  name: string;
  sameNameCount: number;
}) {
  return (
    <section className="profile-import-notice" aria-label="导入模式检查">
      <strong>待检查：{file.fileName}</strong>
      <p>检查通道后“保存到工程”，或取消导入。现有灯具继续使用原模式。</p>
      {sameNameCount > 0 && (
        <p>
          工程内已有 {sameNameCount} 个“{name}
          ”；保存会新增独立模式，可在下面改名。
        </p>
      )}
      <details>
        <summary>文件来源修订</summary>
        <span>{file.source.revision}</span>
      </details>
    </section>
  );
}
