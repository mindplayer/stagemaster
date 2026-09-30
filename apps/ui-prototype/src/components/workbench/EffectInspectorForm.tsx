import type { ReactNode, RefObject } from "react";
import "./effect-inspector.css";

export function EffectInspectorForm({
  form,
  busy,
  dirty,
  isNew,
  error,
  onCancel,
  onChange,
  onApply,
  onPreview,
  children,
}: {
  form: RefObject<HTMLFormElement | null>;
  busy: boolean;
  dirty: boolean;
  isNew: boolean;
  error: string;
  onCancel(): void;
  onChange(): void;
  onApply(): Promise<boolean>;
  onPreview(): Promise<boolean>;
  children: ReactNode;
}) {
  return (
    <form
      ref={form}
      noValidate
      className="effect-inspector"
      aria-label="效果属性"
      onInputCapture={(e) => {
        if (e.target instanceof HTMLInputElement)
          e.target.setCustomValidity("");
        onChange();
      }}
      onKeyDown={(e) => {
        if (e.key === "Escape" && !busy) {
          e.stopPropagation();
          e.preventDefault();
          onCancel();
        }
      }}
      onSubmit={(e) => {
        e.preventDefault();
        void onApply();
      }}
    >
      <header>
        <h2>{isNew ? "添加效果" : "效果属性"}</h2>
        <button type="button" disabled={busy} onClick={onCancel}>
          {dirty ? "取消修改" : "关闭"}
        </button>
      </header>
      <div className="effect-inspector-actions">
        <button
          className="wb-primary"
          type="button"
          disabled={busy}
          title="应用当前修改并重新预演本场景，会替换当前音乐或场景的离线播放"
          onClick={() => void onPreview()}
        >
          应用并预演
        </button>
        <button disabled={busy || !dirty}>应用效果</button>
      </div>
      {error && (
        <p className="wb-library-error" role="alert">
          {error}
        </p>
      )}
      <fieldset disabled={busy}>{children}</fieldset>
    </form>
  );
}
