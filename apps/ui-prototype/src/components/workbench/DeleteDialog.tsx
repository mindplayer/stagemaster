import { useEffect, useRef } from "react";
export function DeleteDialog({
  name,
  error,
  busy,
  onCancel,
  onDelete,
}: {
  name: string;
  error?: string;
  busy: boolean;
  onCancel: () => void;
  onDelete: () => void;
}) {
  const dialog = useRef<HTMLDialogElement>(null);
  useEffect(() => {
    dialog.current?.showModal();
  }, []);
  return (
    <dialog
      ref={dialog}
      className="wb-dialog"
      aria-labelledby="delete-title"
      onCancel={onCancel}
    >
      <h2 id="delete-title">删除“{name}”？</h2>
      {error && <p role="alert">{error}</p>}
      <div className="wb-dialog-actions">
        <button autoFocus onClick={onCancel}>
          取消
        </button>
        <button className="wb-danger" disabled={busy} onClick={onDelete}>
          删除
        </button>
      </div>
    </dialog>
  );
}
