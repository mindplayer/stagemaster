import { useEffect, useState } from "react";
import type {
  ExecutionAction,
  ExecutionSource,
  ExecutionStatus,
  ExecutionView,
} from "../../execution-types";
import {
  commonManualAttributes,
  manualAvailable,
  manualChanges,
  manualResult,
  type ManualDraft,
} from "../../execution-manual";
import { ManualFixturePicker } from "./ManualFixturePicker";
import { ManualValueEditor } from "./ManualValueEditor";
import { SourceControls } from "./SourceControls";
import "./manual-controls.css";
export function ManualControls({
  source,
  runtime,
  disabled,
  active,
  observed,
  onDraftChange,
  onAction,
}: {
  source: ExecutionSource;
  runtime: ExecutionView;
  disabled: boolean;
  active: boolean;
  observed: boolean;
  onDraftChange(id: string, dirty: boolean): void;
  onAction(action: ExecutionAction): Promise<ExecutionStatus | undefined>;
}) {
  const [open, setOpen] = useState(false);
  const [selected, setSelected] = useState<string[]>([]);
  const [attributeKey, setAttribute] = useState("");
  const [draft, setDraft] = useState<ManualDraft | null>(null);
  const [levelDirty, setLevelDirty] = useState(false);
  const [problem, setProblem] = useState("");
  const [working, setWorking] = useState(false);
  const [confirm, setConfirm] = useState(false);
  const [submitted, setSubmitted] = useState<{
    identity: NonNullable<ReturnType<typeof manualResult>>;
    draft: ManualDraft | null;
  } | null>(null);
  const fixtures = runtime.catalog.fixtures ?? [];
  const held = runtime.observation.snapshot?.state.sources.find(
    (s) => s.id === source.id,
  )?.held;
  const available = manualAvailable(runtime) && held !== undefined;
  const attrs = commonManualAttributes(fixtures, selected);
  const attr = attrs.find((a) => a.key === attributeKey) ?? attrs[0];
  const input = draft ?? {
    targets: selected,
    attribute: attr?.key ?? "",
    value: "",
    functionKey: "",
  };
  const unavailable = disabled || !active || !observed || working || !available;
  useEffect(() => {
    onDraftChange(source.id, !!draft || levelDirty);
  }, [draft, levelDirty, onDraftChange, source.id]);
  useEffect(() => {
    if (!active) setConfirm(false);
  }, [active]);
  useEffect(() => {
    if (!submitted) return;
    const record = runtime.record;
    if (
      runtime.hostId !== submitted.identity.hostId ||
      runtime.sessionId !== submitted.identity.sessionId
    ) {
      setProblem("控制连接已变化，请核对实际持有属性后重试");
      setSubmitted(null);
      return;
    }
    if (
      record?.serial !== submitted.identity.serial ||
      record.status !== "complete"
    )
      return;
    if (record.outcome?.kind === "applied") {
      setDraft((current) => (current === submitted.draft ? null : current));
      setProblem("");
    } else
      setProblem(record.outcome?.message || "后台未确认应用，待设置输入已保留");
    setSubmitted(null);
  }, [submitted, runtime]);
  async function submit(action: ExecutionAction) {
    if (unavailable) return;
    const sent = draft;
    setWorking(true);
    setProblem("");
    setConfirm(false);
    try {
      const result = await onAction(action);
      const identity = manualResult(result);
      if (identity) setSubmitted({ identity, draft: sent });
      else setProblem("操作尚未确认，请查看后台状态；待设置输入已保留");
    } catch (e) {
      setProblem(e instanceof Error ? e.message : String(e));
    } finally {
      setWorking(false);
    }
  }
  function apply(release = false) {
    try {
      void submit({
        kind: "patch",
        changes: manualChanges(runtime, source.id, input, release),
      });
    } catch (e) {
      setProblem(e instanceof Error ? e.message : String(e));
    }
  }
  return (
    <div className="execution-manual">
      <SourceControls
        source={source}
        runtime={runtime}
        disabled={disabled || working || !!submitted}
        active={active}
        observed={observed}
        hideManualRelease
        onDraftChange={(_id, dirty) => setLevelDirty(dirty)}
        onAction={(a) => void onAction(a)}
      />
      <div className="execution-manual-body">
        <div className="execution-buttons">
          <span>
            {observed ? "实际持有" : "最后已知持有"} {held?.length ?? "—"}{" "}
            项属性
          </span>
          <button
            aria-expanded={open}
            disabled={!available}
            onClick={() => setOpen((v) => !v)}
          >
            {open ? "收起手动编程" : "打开手动编程"}
          </button>
          <button
            disabled={unavailable || !held?.length || !!draft}
            onClick={() => setConfirm(true)}
          >
            释放手动层
          </button>
        </div>
        {!available && <p>当前后台未提供完整手动编程信息，请重新载入节目。</p>}
        {confirm && (
          <div role="alert" className="execution-confirm">
            释放这层全部 {held?.length} 项属性，让原节目接管？
            <button
              disabled={unavailable}
              onClick={() => void submit({ kind: "stop" })}
            >
              确认释放
            </button>
            <button onClick={() => setConfirm(false)}>取消</button>
          </div>
        )}
        {problem && <p role="alert">{problem}</p>}
        {submitted && <p role="status">正在核对后台操作回执</p>}
        {open && available && (
          <div className="execution-manual-grid">
            <ManualFixturePicker
              fixtures={fixtures}
              selected={selected}
              held={held}
              disabled={!!draft || working}
              onSelect={(ids) => {
                setSelected(ids);
                setAttribute("");
                setProblem("");
              }}
            />
            <section aria-label="手动属性设置">
              <p>
                使用后台固定版本的灯具。应用只改变现场手动层，取消输入不影响正在运行的节目。
              </p>
              {draft && <p>有未应用输入；取消或应用后可更换灯具和属性。</p>}
              {attr ? (
                <>
                  <label>
                    共同属性
                    <select
                      aria-label="手动共同属性"
                      value={attr.key}
                      disabled={!!draft || working}
                      onChange={(e) => {
                        setAttribute(e.target.value);
                        setProblem("");
                      }}
                    >
                      {attrs.map((a) => (
                        <option key={a.key} value={a.key}>
                          {a.label}
                        </option>
                      ))}
                    </select>
                  </label>
                  <ManualValueEditor
                    attribute={attr}
                    draft={input}
                    disabled={unavailable}
                    onChange={(next) => {
                      setDraft(next);
                      setProblem("");
                    }}
                  />
                  <div className="execution-buttons">
                    <button
                      className="wb-primary"
                      disabled={unavailable || !draft}
                      onClick={() => apply()}
                    >
                      应用到 {selected.length} 台灯
                    </button>
                    <button
                      disabled={!draft || working}
                      onClick={() => {
                        setDraft(null);
                        setProblem("");
                      }}
                    >
                      取消输入
                    </button>
                    <button
                      disabled={
                        unavailable ||
                        !!draft ||
                        !held.some(
                          (t) =>
                            selected.includes(t.fixtureId) &&
                            t.attribute === attr.key,
                        )
                      }
                      onClick={() => apply(true)}
                    >
                      释放所选属性
                    </button>
                  </div>
                </>
              ) : (
                <p>
                  {selected.length
                    ? "这些灯没有完全一致的共同属性，请缩小选灯范围"
                    : "先选择要手动控制的灯具"}
                </p>
              )}
              <details className="execution-manual-held">
                <summary>持有属性明细（{held.length}）</summary>
                <ul>
                  {held.map((t) => (
                    <li key={`${t.fixtureId}:${t.attribute}`}>
                      {fixtures.find((f) => f.id === t.fixtureId)?.name} ·{" "}
                      {fixtures
                        .find((f) => f.id === t.fixtureId)
                        ?.attributes.find((a) => a.key === t.attribute)
                        ?.label ?? t.attribute}
                    </li>
                  ))}
                </ul>
              </details>
            </section>
          </div>
        )}
      </div>
    </div>
  );
}
