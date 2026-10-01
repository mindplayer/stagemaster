import { useEffect, useRef, useState } from "react";
import type { ReactNode } from "react";
import { validateEditorForm } from "./form-validation";
export function LibraryDialog({
  title,
  busy,
  error,
  onCancel,
  onSubmit,
  submit = "保存",
  submitDisabled = false,
  children,
}: {
  title: string;
  busy: boolean;
  error: string;
  onCancel(): void;
  onSubmit(): Promise<boolean>;
  submit?: string;
  submitDisabled?: boolean;
  children: ReactNode;
}) {
  const dialog = useRef<HTMLDialogElement>(null);
  const [localError, setLocalError] = useState("");
  useEffect(() => {
    dialog.current?.showModal();
  }, []);
  return (
    <dialog
      ref={dialog}
      className="wb-dialog wb-library-dialog"
      aria-label={title}
      onCancel={(e) => {
        e.preventDefault();
        if (!busy) onCancel();
      }}
    >
      <form
        noValidate
        onSubmit={async (e) => {
          e.preventDefault();
          const form = e.currentTarget;
          setLocalError("");
          try {
            validateEditorForm(form);
            if (await onSubmit()) onCancel();
          } catch (reason) {
            setLocalError(
              reason instanceof Error ? reason.message : String(reason),
            );
          }
        }}
        onInputCapture={(e) => {
          if (e.target instanceof HTMLInputElement)
            e.target.setCustomValidity("");
          setLocalError("");
        }}
      >
        <h2>{title}</h2>
        <fieldset disabled={busy}>{children}</fieldset>
        {(localError || error) && (
          <p className="wb-library-error" role="alert">
            {localError || error}
          </p>
        )}
        <div className="wb-dialog-actions">
          <button type="button" disabled={busy} onClick={onCancel}>
            取消
          </button>
          <button className="wb-primary" disabled={busy || submitDisabled}>
            {submit}
          </button>
        </div>
      </form>
    </dialog>
  );
}
