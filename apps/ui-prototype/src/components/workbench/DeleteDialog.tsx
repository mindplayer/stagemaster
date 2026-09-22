import { useEffect, useRef } from "react";
export function DeleteDialog({
  name,
  busy,
  onCancel,
  onDelete,
}: {
  name: string;
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
      <div>
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
