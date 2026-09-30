import {
  PlusIcon,
  FolderOpenIcon,
  FloppyDiskIcon,
  ArrowCounterClockwiseIcon,
  ArrowClockwiseIcon,
  LightbulbIcon,
} from "@phosphor-icons/react";
import type { ApplicationHost, Snapshot } from "../../application-host";
import type { InstallationController } from "../installation/useInstallation";
import { DeviceTools } from "../devices/DeviceTools";
export function ProjectHeader({
  host,
  snapshot,
  dirty,
  busy,
  hasDrafts,
  installation,
  onNew,
  onOpen,
  onRecent,
  onRecover,
  onHistory,
  onSave,
}: {
  host: ApplicationHost;
  snapshot: Snapshot;
  dirty: boolean;
  busy: boolean;
  hasDrafts: boolean;
  installation: InstallationController;
  onNew(): void;
  onOpen(): void;
  onRecent(): void;
  onRecover(): void;
  onHistory(redo: boolean): void;
  onSave(saveAs: boolean): void;
}) {
  const project = snapshot.project;
  return (
    <header className="workbench-header">
      <div className="wb-brand">
        <span className="wb-logo">
          <LightbulbIcon weight="fill" size={22} />
        </span>
        舞台大师
      </div>
      {project && (
        <div className="wb-project-title">
          <strong>{project.name}</strong>
          <span className={dirty ? "wb-unsaved" : ""}>
            {dirty ? "未保存" : "已保存"}
          </span>
        </div>
      )}
      <div className="wb-file-actions">
        <DeviceTools host={host} installation={installation} />
        <button
          title="新建工程（⌘N / Ctrl+N）"
          disabled={busy || host.kind !== "desktop"}
          onClick={onNew}
        >
          <PlusIcon />
          新建
        </button>
        <button
          title="打开工程（⌘O / Ctrl+O）"
          disabled={busy || host.kind !== "desktop"}
          onClick={onOpen}
        >
          <FolderOpenIcon />
          打开
        </button>
        {project && (
          <button disabled={busy || host.kind !== "desktop"} onClick={onRecent}>
            最近
          </button>
        )}
        <button disabled={busy || host.kind !== "desktop"} onClick={onRecover}>
          恢复
        </button>
        <span className="wb-toolbar-separator" />
        <button
          aria-label="撤销"
          title="撤销（⌘Z / Ctrl+Z）"
          disabled={busy || (!snapshot.canUndo && !hasDrafts)}
          onClick={() => onHistory(false)}
        >
          <ArrowCounterClockwiseIcon />
        </button>
        <button
          aria-label="重做"
          title="重做（⇧⌘Z / Ctrl+Shift+Z）"
          disabled={busy || !snapshot.canRedo || hasDrafts}
          onClick={() => onHistory(true)}
        >
          <ArrowClockwiseIcon />
        </button>
        <button disabled={busy || !project} onClick={() => onSave(true)}>
          另存为
        </button>
        <button
          className="wb-primary"
          title="保存工程（⌘S / Ctrl+S）"
          disabled={busy || !project || !dirty}
          onClick={() => onSave(false)}
        >
          <FloppyDiskIcon />
          保存
        </button>
      </div>
    </header>
  );
}
