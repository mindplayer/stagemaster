import { useEffect, useRef } from "react";
import type { ApplicationHost } from "../../application-host";
import { RecentProjects } from "./RecentProjects";

export function RecentProjectsDialog({
  host,
  busy,
  error,
  onOpen,
  onBrowse,
  onClose,
}: {
  host: ApplicationHost;
  busy: boolean;
  error: string;
  onOpen(id: string): Promise<boolean>;
  onBrowse(): void;
  onClose(): void;
}) {
  const dialog = useRef<HTMLDialogElement>(null);
  useEffect(() => {
    const previous = document.activeElement;
    const element = dialog.current;
    element?.showModal();
    return () => {
      element?.close();
      if (previous instanceof HTMLElement && previous.isConnected)
        previous.focus();
    };
  }, []);
  return (
    <dialog
      ref={dialog}
      className="wb-dialog recent-dialog"
      aria-label="最近工程目录"
      data-navigation-dialog="true"
      onKeyDown={(e) => {
        if (e.key === "Escape") {
          e.preventDefault();
          e.stopPropagation();
          if (!busy) onClose();
        }
      }}
      onCancel={(e) => {
        e.preventDefault();
        if (!busy) onClose();
      }}
    >
      <div className="recent-dialog-actions">
        <button disabled={busy} onClick={onBrowse}>
          打开其他工程
        </button>
        <button disabled={busy} onClick={onClose}>
          关闭
        </button>
      </div>
      {error && (
        <p role="alert" className="recent-error">
          {error}
        </p>
      )}
      <RecentProjects
        host={host}
        busy={busy}
        onBrowse={onBrowse}
        onOpen={async (id) => {
          const opened = await onOpen(id);
          if (opened) onClose();
          return opened;
        }}
      />
    </dialog>
  );
}
