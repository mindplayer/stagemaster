import { useState } from "react";
import type { ProjectView } from "../../application-host";
import { OrderedFixturePicker } from "./OrderedFixturePicker";
import { RigAttachmentFields } from "./RigAttachmentFields";
import type { useRiggingDraft } from "./useRiggingDraft";
import "./arrangement-inspector.css";
import "./rigging-inspector.css";

export function RigAttachmentInspector({
  project,
  draft,
  busy,
  error,
  onApply,
}: {
  project: ProjectView;
  draft: ReturnType<typeof useRiggingDraft>;
  busy: boolean;
  error: string;
  onApply(): void;
}) {
  const [membersOpen, setMembersOpen] = useState(!draft.session?.ids.length);
  const session = draft.session;
  if (!session) return null;
  return (
    <form
      ref={draft.form}
      noValidate
      className="arrangement-inspector"
      aria-label="灯具挂接属性"
      onSubmit={(e) => {
        e.preventDefault();
        if (!busy && !draft.computing) onApply();
      }}
      onKeyDown={(e) => {
        if (e.key === "Escape" && !busy) {
          e.preventDefault();
          e.stopPropagation();
          draft.cancel();
        }
        if (
          e.key === "Enter" &&
          e.target instanceof Element &&
          e.target.closest(".placement-picker")
        )
          e.preventDefault();
      }}
    >
      <header>
        <h2>挂接／换挂</h2>
        <button type="button" disabled={busy} onClick={draft.cancel}>
          {session.dirty ? "取消修改" : "关闭挂接"}
        </button>
      </header>
      <div className="arrangement-actions">
        <button className="wb-primary" disabled={busy || draft.computing}>
          应用挂接 · {session.ids.length} 台
        </button>
        <span>
          {draft.computing
            ? "正在计算位置"
            : draft.projection?.changed === false
              ? "挂接与位置未改变"
              : "平面草稿 · 未应用"}
        </span>
      </div>
      {(draft.problem || error) && (
        <p role="alert" className="wb-library-error">
          {draft.problem || error}
        </p>
      )}
      {draft.problem && (
        <button type="button" disabled={busy} onClick={draft.retry}>
          重新预览
        </button>
      )}
      <fieldset disabled={busy}>
        <RigAttachmentFields
          project={project}
          session={session}
          update={draft.update}
        />
        <details
          className="arrangement-members"
          open={membersOpen}
          onToggle={(e) => setMembersOpen(e.currentTarget.open)}
        >
          <summary>灯具与顺序 · {session.ids.length} 台</summary>
          <OrderedFixturePicker
            project={project}
            ids={session.ids}
            setIds={draft.setIds}
          />
        </details>
      </fieldset>
    </form>
  );
}
