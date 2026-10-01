import { packageTargetReason } from "../../installation-tools";
import { useEffect, useRef, useState } from "react";
import type { ApplicationHost, ProjectView } from "../../application-host";
import type { CheckLocation } from "../../check-types";
import type {
  PackageCandidate,
  PackageResult,
  PackageSelection,
} from "../../package-types";
import {
  filterPrograms,
  packageCurrent,
  packageKey,
  selectFiltered,
  selectionKey,
} from "../../package-tools";
import "./package-panel.css";
import { PackageResults } from "./PackageResults";
const PAGE = 12;
export function PackagePanel({
  host,
  project,
  generation,
  hasDrafts,
  visible,
  busy,
  capture,
  onLocate,
  onInstall,
  installReason: destinationReason,
  targetExecutionSemantics,
}: {
  host: ApplicationHost;
  project: ProjectView;
  generation: number;
  hasDrafts: boolean;
  visible: boolean;
  busy: boolean;
  capture(): Promise<number | null>;
  onLocate(location: CheckLocation, generation: number): Promise<boolean>;
  onInstall(generation: number, token: string): Promise<void>;
  installReason: string | null;
  targetExecutionSemantics?: number;
}) {
  const [selected, setSelected] = useState<PackageSelection[]>([]);
  const [query, setQuery] = useState("");
  const [kind, setKind] = useState<"all" | PackageSelection["kind"]>("all");
  const [page, setPage] = useState(0);
  const [resultPage, setResultPage] = useState(0);
  const [result, setResult] = useState<PackageResult | null>(null);
  const [builtSelection, setBuiltSelection] = useState("");
  const [working, setWorking] = useState<"build" | "export" | "install" | null>(
    null,
  );
  const [cancelled, setCancelled] = useState(false);
  const [message, setMessage] = useState("");
  const [error, setError] = useState("");
  const inFlight = useRef(false);
  const discard = useRef(false);
  const candidates: PackageCandidate[] = [
    ...project.scenes.map((s) => ({ ...s, kind: "scene" as const })),
    ...project.sequences.map((s) => ({ ...s, kind: "sequence" as const })),
  ];
  const candidateKeys = candidates.map(packageKey).join("|");
  useEffect(() => {
    const keys = new Set(candidateKeys.split("|"));
    setSelected((old) => old.filter((p) => keys.has(packageKey(p))));
  }, [candidateKeys]);
  useEffect(
    () => () => {
      discard.current = true;
    },
    [],
  );
  const filtered = filterPrograms(candidates, query, kind);
  const selectedKeys = new Set(selected.map(packageKey));
  const current = packageCurrent(
    result,
    project.id,
    generation,
    hasDrafts,
    builtSelection,
    selected,
  );
  const pages = Math.max(1, Math.ceil(filtered.length / PAGE));
  const activePage = Math.min(page, pages - 1);
  const report = result?.report;
  const installReason =
    destinationReason ||
    (report
      ? packageTargetReason(report.executionSemantics, targetExecutionSemantics)
      : null);
  const programOrder = new Map(candidates.map((p, i) => [packageKey(p), i]));
  const reportPrograms = [...(report?.programs ?? [])].sort((a, b) => {
    const index = (p: typeof a) =>
      "id" in p.location
        ? (programOrder.get(`${p.location.kind}:${p.location.id}`) ??
          candidates.length)
        : candidates.length;
    return index(a) - index(b);
  });
  const disabled = busy || working !== null;
  async function build() {
    if (inFlight.current) return;
    inFlight.current = true;
    discard.current = false;
    setWorking("build");
    setCancelled(false);
    setError("");
    setMessage("");
    setResult(null);
    const selection = selected.map(({ kind, id }) => ({ kind, id }));
    try {
      const version = await capture();
      if (version === null || discard.current) return;
      const next = await host.buildPackage(version, selection);
      if (!discard.current) {
        setResult(next);
        setBuiltSelection(selectionKey(selection));
        setResultPage(0);
      }
    } catch (reason) {
      if (!discard.current)
        setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      inFlight.current = false;
      setWorking(null);
    }
  }
  async function exportFile() {
    if (!current || !result?.token || inFlight.current) return;
    inFlight.current = true;
    setWorking("export");
    setError("");
    setMessage("");
    try {
      const version = await capture();
      if (version === null) return;
      if (version !== result.generation)
        throw new Error("工程已变化，请重新生成播放包");
      const exported = await host.exportPackage(version, result.token);
      if (!discard.current)
        setMessage(
          exported.path
            ? exported.warning || `已导出：${exported.path}`
            : "已取消导出",
        );
    } catch (reason) {
      if (!discard.current)
        setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      inFlight.current = false;
      setWorking(null);
    }
  }
  if (!visible) return null;
  return (
    <section className="wb-package" aria-label="播放包">
      <div className="wb-package-heading">
        <div>
          <h2>播放包</h2>
          <p>选择场景或完整列表，导出独立节目文件</p>
        </div>
        <span className="wb-package-badge">单路输出</span>
      </div>
      <div className="wb-package-toolbar">
        <input
          type="search"
          aria-label="搜索待导出节目"
          placeholder="搜索节目名称"
          value={query}
          onChange={(e) => {
            setQuery(e.target.value);
            setPage(0);
          }}
        />
        <select
          aria-label="待导出节目类型"
          value={kind}
          onChange={(e) => {
            setKind(e.target.value as typeof kind);
            setPage(0);
          }}
        >
          <option value="all">全部类型</option>
          <option value="scene">场景</option>
          <option value="sequence">场景列表</option>
        </select>
        <button
          disabled={
            disabled ||
            filtered.length === 0 ||
            selectFiltered(selected, filtered) === null
          }
          onClick={() => {
            const next = selectFiltered(selected, filtered);
            if (next) setSelected(next);
          }}
        >
          选择筛选结果
        </button>
        <button
          disabled={disabled || selected.length === 0}
          onClick={() => setSelected([])}
        >
          清空选择
        </button>
      </div>
      {project.audio && (
        <p className="wb-package-caption">
          设备包仅包含所选灯光场景／列表；音乐和音频卡点目前在电脑端试听，不随此包下发。
        </p>
      )}
      <p className="wb-package-caption">
        已选 {selected.length} / 64 项
        {selected.filter(
          (p) => !filtered.some((f) => packageKey(f) === packageKey(p)),
        ).length > 0
          ? " · 包含筛选外节目"
          : ""}
      </p>
      {selectFiltered(selected, filtered) === null && (
        <p className="wb-package-caption">
          合并选择后超过 64 项，请缩小筛选范围或逐项选择。
        </p>
      )}
      <div className="wb-package-list">
        {filtered.slice(activePage * PAGE, (activePage + 1) * PAGE).map((p) => (
          <label
            key={packageKey(p)}
            className={selectedKeys.has(packageKey(p)) ? "selected" : ""}
          >
            <input
              type="checkbox"
              checked={selectedKeys.has(packageKey(p))}
              disabled={
                disabled ||
                (!selectedKeys.has(packageKey(p)) && selected.length >= 64)
              }
              onChange={(e) =>
                setSelected((old) =>
                  e.target.checked
                    ? [...old, { kind: p.kind, id: p.id }]
                    : old.filter((v) => packageKey(v) !== packageKey(p)),
                )
              }
            />
            <span>{p.name}</span>
            <small>{p.kind === "scene" ? "场景" : "列表"}</small>
          </label>
        ))}
        {filtered.length === 0 && (
          <p>
            {candidates.length === 0
              ? "先在编排中创建场景或场景列表"
              : "没有符合筛选条件的节目"}
          </p>
        )}
      </div>
      {pages > 1 && (
        <div className="wb-package-pager">
          <button
            disabled={activePage === 0}
            onClick={() => setPage(activePage - 1)}
          >
            上一页
          </button>
          <span>
            节目 {activePage + 1} / {pages} 页
          </span>
          <button
            disabled={activePage + 1 >= pages}
            onClick={() => setPage(activePage + 1)}
          >
            下一页
          </button>
        </div>
      )}
      <div className="wb-package-toolbar wb-package-actions">
        <button
          className="wb-primary"
          disabled={disabled || selected.length === 0}
          onClick={() => void build()}
        >
          {working === "build"
            ? "正在生成并验证…"
            : result
              ? "重新生成"
              : "生成播放包"}
        </button>
        {working === "build" && (
          <button
            disabled={cancelled}
            onClick={() => {
              discard.current = true;
              setCancelled(true);
              setMessage("已取消本次结果，后台计算结束后可重试");
            }}
          >
            取消生成
          </button>
        )}
        <button
          disabled={disabled || !current || !result?.token}
          onClick={() => void exportFile()}
        >
          {working === "export" ? "正在导出…" : "另存播放包"}
        </button>
        <button
          disabled={disabled || !current || !result?.token || !!installReason}
          title={installReason || undefined}
          onClick={() => {
            if (!current || !result?.token || inFlight.current) return;
            inFlight.current = true;
            setWorking("install");
            setError("");
            void (async () => {
              try {
                const version = await capture();
                if (version === null) return;
                if (version !== result.generation)
                  throw new Error("工程已变化，请重新生成播放包");
                await onInstall(version, result.token!);
              } catch (reason) {
                if (!discard.current)
                  setError(
                    reason instanceof Error ? reason.message : String(reason),
                  );
              } finally {
                inFlight.current = false;
                if (!discard.current) setWorking(null);
              }
            })();
          }}
        >
          {working === "install" ? "正在开始安装…" : "安装到设备"}
        </button>
        {result && (
          <strong className={current && report ? "passed" : ""}>
            {!current ? "结果已过期" : report ? "软件校验通过" : "需要修复"}
          </strong>
        )}
      </div>
      {error && (
        <p className="wb-package-error" role="alert">
          {error}
        </p>
      )}
      {report && current && installReason && (
        <p className="wb-package-caption">{installReason}</p>
      )}
      {message && (
        <p className="wb-package-message" role="status">
          {message}
        </p>
      )}
      {result && !current && (
        <p className="wb-package-caption">
          工程、草稿或选择已变化，请重新生成。
        </p>
      )}
      {result && (
        <PackageResults
          result={result}
          reportPrograms={reportPrograms}
          current={current}
          disabled={disabled}
          resultPage={resultPage}
          setResultPage={setResultPage}
          onLocate={onLocate}
        />
      )}
      <p className="wb-package-caption">
        场景／列表包含已应用的未保存编辑；音乐、卡点和时间线渐变不随此包导出。安装使用生成时的固定内容；安装不会开始播放。
      </p>
    </section>
  );
}
