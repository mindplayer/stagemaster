import { useEffect, useRef, useState } from "react";
import { ArrowClockwiseIcon, XIcon } from "@phosphor-icons/react";
import type { ApplicationHost } from "../../application-host";
import type { RecoveryCatalog, RecoveryEntry } from "../../recovery-types";
import "./recovery.css";

const labels = { ready: "可以恢复", active: "窗口使用中", damaged: "无法恢复" };
export function RecoveryCenter({
  host,
  onClose,
  onRestore,
  operationError,
}: {
  host: ApplicationHost;
  onClose(): void;
  onRestore(entry: RecoveryEntry): Promise<boolean>;
  operationError: string;
}) {
  const dialog = useRef<HTMLDialogElement>(null);
  const mounted = useRef(true);
  const inFlight = useRef(false);
  const [catalog, setCatalog] = useState<RecoveryCatalog | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const [query, setQuery] = useState("");
  const [confirm, setConfirm] = useState<RecoveryEntry | null>(null);
  const [restoring, setRestoring] = useState(false);
  async function load(discard?: RecoveryEntry) {
    if (inFlight.current) return;
    inFlight.current = true;
    setBusy(true);
    setError("");
    setNotice("");
    setRestoring(false);
    try {
      const next = await host.recovery(
        discard
          ? { kind: "discard", id: discard.id, token: discard.token }
          : { kind: "list" },
      );
      if (!mounted.current) return;
      setCatalog(next);
      setConfirm(null);
      if (discard) setNotice("恢复副本已丢弃，原工程文件未改动");
    } catch (reason) {
      if (mounted.current)
        setError(String(reason instanceof Error ? reason.message : reason));
    } finally {
      inFlight.current = false;
      if (mounted.current) setBusy(false);
    }
  }
  async function restore(entry: RecoveryEntry) {
    if (inFlight.current) return;
    inFlight.current = true;
    setBusy(true);
    setError("");
    setNotice("");
    setRestoring(true);
    try {
      if (!(await onRestore(entry)) && mounted.current)
        setNotice("恢复未完成，当前工程和恢复副本保留。");
    } finally {
      inFlight.current = false;
      if (mounted.current) setBusy(false);
    }
  }
  useEffect(() => {
    mounted.current = true;
    dialog.current?.showModal();
    void load();
    return () => {
      mounted.current = false;
    };
  }, []);
  const entries = (catalog?.entries ?? []).filter((entry) =>
    `${entry.projectName ?? ""} ${entry.sourceFile ?? ""} ${labels[entry.state]}`
      .toLocaleLowerCase()
      .includes(query.trim().toLocaleLowerCase()),
  );
  function close() {
    if (!busy) onClose();
  }
  return (
    <dialog
      ref={dialog}
      className="wb-dialog wb-recovery"
      aria-labelledby="recovery-title"
      onCancel={(event) => {
        event.preventDefault();
        if (confirm && !busy) setConfirm(null);
        else close();
      }}
      onKeyDown={(event) => {
        if (event.key !== "Escape") return;
        event.preventDefault();
        event.stopPropagation();
        if (busy) return;
        if (confirm) setConfirm(null);
        else if (event.target instanceof HTMLInputElement && query)
          setQuery("");
        else close();
      }}
    >
      <header className="wb-recovery-heading">
        <div>
          <h2 id="recovery-title">恢复工程</h2>
          <p>从已应用的编辑恢复为未保存副本</p>
        </div>
        <button aria-label="关闭恢复中心" disabled={busy} onClick={close}>
          <XIcon />
        </button>
      </header>
      <p className="wb-recovery-caption">
        恢复后需要另行保存，不会自动开始播放。尚未应用的输入框草稿不在恢复副本中。
      </p>
      {(error || (restoring && operationError)) && (
        <p className="wb-recovery-warning" role="alert">
          {error || operationError}
        </p>
      )}
      {notice && <p role="status">{notice}</p>}
      {confirm ? (
        <section
          className="wb-recovery-confirm"
          aria-labelledby="recovery-discard-title"
        >
          <h3 id="recovery-discard-title">
            丢弃“{confirm.projectName || "无效恢复副本"}”？
          </h3>
          <p>此恢复副本将永久删除，原工程文件保留。</p>
          <p>{formatDate(confirm.capturedAtMs)}</p>
          <div className="wb-dialog-actions">
            <button
              autoFocus
              disabled={busy}
              onClick={() => {
                setConfirm(null);
                setError("");
              }}
            >
              取消丢弃
            </button>
            <button
              className="wb-danger"
              disabled={busy}
              onClick={() => void load(confirm)}
            >
              确认丢弃
            </button>
          </div>
        </section>
      ) : (
        <>
          <div className="wb-recovery-toolbar">
            <input
              type="search"
              autoFocus
              aria-label="搜索恢复副本"
              placeholder="搜索工程或来源"
              value={query}
              onChange={(event) => setQuery(event.target.value)}
            />
            <button disabled={busy} onClick={() => void load()}>
              <ArrowClockwiseIcon />
              {busy ? "处理中…" : "刷新"}
            </button>
          </div>
          <div className="wb-recovery-list" aria-busy={busy}>
            {entries.map((entry) => (
              <article key={entry.id}>
                <div className="wb-recovery-heading">
                  <strong>{entry.projectName || "无效恢复副本"}</strong>
                  <span className={`wb-recovery-badge ${entry.state}`}>
                    {labels[entry.state]}
                  </span>
                </div>
                <p>
                  {formatDate(entry.capturedAtMs)}
                  {entry.older && " · 超过 30 天，请核对版本"}
                </p>
                <p
                  className="wb-recovery-source"
                  title={entry.sourceFile ?? undefined}
                >
                  {entry.sourceFile ||
                    (entry.state === "damaged"
                      ? "工程来源未知"
                      : "尚未保存到工程文件")}
                </p>
                {entry.problem && (
                  <p className="wb-recovery-warning">{entry.problem}</p>
                )}
                {entry.state === "active" && (
                  <p>副本由正在编辑的窗口保护，请回到该窗口继续编辑或保存。</p>
                )}
                <div className="wb-dialog-actions">
                  <button
                    disabled={busy || !entry.canDiscard}
                    onClick={() => {
                      setConfirm(entry);
                      setError("");
                      setNotice("");
                    }}
                  >
                    丢弃副本
                  </button>
                  <button
                    className="wb-primary"
                    disabled={busy || entry.state !== "ready"}
                    onClick={() => void restore(entry)}
                  >
                    恢复为副本
                  </button>
                </div>
              </article>
            ))}
            {catalog && !entries.length && (
              <p className="wb-recovery-empty">
                {catalog.entries.length
                  ? "没有符合搜索的恢复副本"
                  : "暂无恢复副本"}
              </p>
            )}
            {!catalog && busy && <p>正在读取恢复副本…</p>}
          </div>
          {!!catalog?.omitted && (
            <p className="wb-recovery-warning">
              另有 {catalog.omitted}{" "}
              份记录超出本次显示上限；处理部分副本后刷新，其余记录仍保留。
            </p>
          )}
          <footer className="wb-recovery-heading">
            <span>{catalog ? `${catalog.entries.length} 份恢复副本` : ""}</span>
            <button disabled={busy} onClick={close}>
              完成
            </button>
          </footer>
        </>
      )}
    </dialog>
  );
}
function formatDate(value: number | null): string {
  if (value === null) return "恢复时间未知";
  const date = new Date(value);
  return Number.isFinite(date.getTime())
    ? date.toLocaleString("zh-CN", { hour12: false })
    : "恢复时间无效";
}
