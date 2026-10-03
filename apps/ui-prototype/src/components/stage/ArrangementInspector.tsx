import { useState } from "react";
import type { ProjectView } from "../../application-host";
import { OrderedFixturePicker } from "./OrderedFixturePicker";
import { ArrangementFields } from "./ArrangementFields";
import type { useStageArrangement } from "./useStageArrangement";
import "./arrangement-inspector.css";

export function ArrangementInspector({
  project,
  arrangement,
  busy,
  error,
  onApply,
}: {
  project: ProjectView;
  arrangement: ReturnType<typeof useStageArrangement>;
  busy: boolean;
  error: string;
  onApply(): void;
}) {
  const session = arrangement.session;
  const [membersOpen, setMembersOpen] = useState(!session?.ids.length);
  if (!session) return null;
  const problem = arrangement.problem || error;
  return (
    <form
      ref={arrangement.form}
      noValidate
      className="arrangement-inspector"
      aria-label="灯位排列属性"
      onSubmit={(e) => {
        e.preventDefault();
        onApply();
      }}
      onKeyDown={(e) => {
        if (e.key === "Escape" && !busy) {
          e.preventDefault();
          e.stopPropagation();
          arrangement.cancel();
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
        <h2>灯位排列</h2>
        <button type="button" disabled={busy} onClick={arrangement.cancel}>
          {session.dirty ? "取消修改" : "关闭排列"}
        </button>
      </header>
      <div className="arrangement-actions">
        <button className="wb-primary" disabled={busy}>
          应用到 {session.ids.length} 台灯具
        </button>
        <span>平面草稿 · 未应用</span>
      </div>
      {problem && (
        <p className="wb-library-error" role="alert">
          {problem}
        </p>
      )}
      <fieldset disabled={busy}>
        <ArrangementFields
          project={project}
          draft={session.draft}
          update={arrangement.update}
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
            setIds={arrangement.setIds}
          />
        </details>
      </fieldset>
    </form>
  );
}
