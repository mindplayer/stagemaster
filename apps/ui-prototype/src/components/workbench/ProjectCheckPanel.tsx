import { CheckTargets } from "./CheckTargets";
import { CheckAudioResources } from "./CheckAudioResources";
import { useEffect, useRef, useState } from "react";
import { ArrowClockwiseIcon } from "@phosphor-icons/react";
import type { ApplicationHost } from "../../application-host";
import type {
  CheckLocation,
  PlanLimits,
  ProjectCheck,
} from "../../check-types";
import { filterIssues, isCheckCurrent } from "../../check-tools";
import "./project-check.css";

const labels: [keyof PlanLimits, string][] = [
  ["attributes", "属性"],
  ["steps", "步骤"],
  ["targetValues", "目标数值"],
  ["effectChannels", "效果通道"],
  ["keyframes", "关键帧"],
];
const statusLabels = {
  passed: "编译通过",
  failed: "编译失败",
  blocked: "受配适阻断",
};
const PAGE_SIZE = 20;

export function ProjectCheckPanel({
  host,
  projectId,
  generation,
  hasDrafts,
  visible,
  busy,
  capture,
  onLocate,
  onSaveResources,
}: {
  host: ApplicationHost;
  projectId: string;
  generation: number;
  hasDrafts: boolean;
  visible: boolean;
  busy: boolean;
  capture(): Promise<number | null>;
  onLocate(location: CheckLocation, generation: number): Promise<boolean>;
  onSaveResources(generation: number): Promise<boolean>;
}) {
  const [check, setCheck] = useState<ProjectCheck | null>(null);
  const [checking, setChecking] = useState(false);
  const [cancelled, setCancelled] = useState(false);
  const inFlight = useRef(false);
  const discard = useRef(false);
  const [error, setError] = useState("");
  const [query, setQuery] = useState("");
  const [severity, setSeverity] = useState<"all" | "error" | "warning">("all");
  const [issuePage, setIssuePage] = useState(0);
  const [programQuery, setProgramQuery] = useState("");
  const [programFilter, setProgramFilter] = useState<"all" | "failed">("all");
  const [programPage, setProgramPage] = useState(0);
  const [completedAt, setCompletedAt] = useState("");
  useEffect(
    () => () => {
      discard.current = true;
    },
    [],
  );
  async function start() {
    if (inFlight.current) return;
    inFlight.current = true;
    discard.current = false;
    setChecking(true);
    setCancelled(false);
    setError("");
    try {
      const version = await capture();
      if (version === null || discard.current) return;
      const result = await host.check(version);
      if (!discard.current) {
        setCheck(result);
        setCompletedAt(
          new Date().toLocaleTimeString("zh-CN", { hour12: false }),
        );
        setIssuePage(0);
        setProgramPage(0);
      }
    } catch (reason) {
      if (!discard.current)
        setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      inFlight.current = false;
      setChecking(false);
    }
  }
  const current = isCheckCurrent(check, projectId, generation, hasDrafts);
  const report = check?.report;
  const errors =
    report?.issues.filter((i) => i.severity === "error").length ?? 0;
  const warnings =
    report?.issues.filter((i) => i.severity === "warning").length ?? 0;
  const issues = filterIssues(report?.issues ?? [], query, severity);
  const programs = (report?.programs ?? []).filter(
    (p) =>
      p.name
        .toLocaleLowerCase()
        .includes(programQuery.trim().toLocaleLowerCase()) &&
      (programFilter === "all" || p.status !== "passed"),
  );
  if (!visible) return null;
  return (
    <section className="wb-check" aria-label="工程检查">
      <div className="wb-check-heading">
        <div>
          <h2>工程检查</h2>
          <p>检查灯光编译、计划容量与音乐文件完整性</p>
        </div>
        <div className="wb-check-actions">
          {checking && (
            <button
              disabled={cancelled}
              onClick={() => {
                discard.current = true;
                setCancelled(true);
              }}
            >
              取消检查
            </button>
          )}
          <button
            className="wb-primary"
            disabled={busy || checking}
            onClick={() => void start()}
          >
            <ArrowClockwiseIcon />
            {checking
              ? cancelled
                ? "正在结束检查…"
                : "正在检查…"
              : check
                ? "重新检查"
                : "检查工程"}
          </button>
        </div>
      </div>
      {error && (
        <p className="wb-check-alert" role="alert">
          {error}
        </p>
      )}
      {cancelled && (
        <p role="status">本次结果已取消；已开始的后台计算结束后可重新检查。</p>
      )}
      <CheckTargets
        check={check}
        current={current}
        errors={errors}
        warnings={warnings}
      />
      {check && report && (
        <>
          <p className="wb-check-caption">
            {completedAt} 完成 ·{" "}
            {current ? "当前工程" : "以下为历史结果，定位已停用"} ·{" "}
            {report.programs.length} 个节目
          </p>
          <CheckAudioResources
            resource={check.audioResource}
            current={current}
            disabled={busy || checking}
            onLocate={() => void onLocate({ kind: "audio" }, check.generation)}
            onSave={() => void onSaveResources(check.generation)}
          />
          <div className="wb-check-heading">
            <h3>
              问题 <small>{errors + warnings}</small>
            </h3>
            <div className="wb-check-actions">
              <input
                type="search"
                aria-label="搜索检查问题"
                placeholder="搜索灯具或问题"
                value={query}
                onChange={(e) => {
                  setQuery(e.target.value);
                  setIssuePage(0);
                }}
              />
              <select
                aria-label="检查问题级别"
                value={severity}
                onChange={(e) => {
                  setSeverity(e.target.value as typeof severity);
                  setIssuePage(0);
                }}
              >
                <option value="all">全部级别</option>
                <option value="error">仅错误</option>
                <option value="warning">仅提醒</option>
              </select>
            </div>
          </div>
          <ul className="wb-check-issues">
            {issues
              .slice(issuePage * PAGE_SIZE, (issuePage + 1) * PAGE_SIZE)
              .map((issue, index) => (
                <li key={`${issuePage}:${index}`}>
                  <span className={`wb-check-badge ${issue.severity}`}>
                    {issue.severity === "error" ? "错误" : "提醒"}
                  </span>
                  <p>{issue.message}</p>
                  <button
                    disabled={!current || busy || checking}
                    aria-label={`定位：${issue.message}`}
                    onClick={() =>
                      void onLocate(issue.location, check.generation)
                    }
                  >
                    定位
                  </button>
                </li>
              ))}
          </ul>
          {!issues.length && (
            <p className="wb-check-empty">
              {report.issues.length
                ? "没有符合筛选的问题"
                : "未发现编译与配适问题"}
            </p>
          )}
          <Pages
            page={issuePage}
            total={issues.length}
            onChange={setIssuePage}
            label="问题"
          />
          <div className="wb-check-heading">
            <h3>节目与容量</h3>
            <div className="wb-check-actions">
              <input
                type="search"
                aria-label="搜索检查节目"
                placeholder="搜索场景或列表"
                value={programQuery}
                onChange={(e) => {
                  setProgramQuery(e.target.value);
                  setProgramPage(0);
                }}
              />
              <select
                aria-label="节目检查结果"
                value={programFilter}
                onChange={(e) => {
                  setProgramFilter(e.target.value as typeof programFilter);
                  setProgramPage(0);
                }}
              >
                <option value="all">全部节目</option>
                <option value="failed">未通过</option>
              </select>
            </div>
          </div>
          <div className="wb-check-programs">
            {programs
              .slice(programPage * PAGE_SIZE, (programPage + 1) * PAGE_SIZE)
              .map((program) => (
                <article
                  key={`${program.location.kind}:${"id" in program.location ? program.location.id : program.name}`}
                >
                  <div className="wb-check-heading">
                    <div>
                      <span className="wb-check-caption">
                        {program.location.kind === "sequence" ? "列表" : "场景"}
                      </span>
                      <h4>{program.name}</h4>
                    </div>
                    <span>{statusLabels[program.status]}</span>
                    <button
                      disabled={!current || busy || checking}
                      onClick={() =>
                        void onLocate(program.location, check.generation)
                      }
                      aria-label={`查看节目 ${program.name}`}
                    >
                      查看
                    </button>
                  </div>
                  {program.usage && (
                    <details>
                      <summary>
                        计划容量 · {program.usage.steps} 步 ·{" "}
                        {program.usage.attributes} 个属性
                      </summary>
                      <dl className="wb-check-usage">
                        {labels.map(([key, label]) => (
                          <div key={key}>
                            <dt>{label}</dt>
                            <dd>
                              {program.usage![key].toLocaleString()} /{" "}
                              {report.limits[key].toLocaleString()}
                            </dd>
                          </div>
                        ))}
                      </dl>
                      <p className="wb-check-caption">
                        数值缓冲{" "}
                        {program.usage.valueBufferBytes.toLocaleString()}{" "}
                        字节；本机效果数据{" "}
                        {program.usage.effectBufferBytes.toLocaleString()}{" "}
                        字节；功能切换索引{" "}
                        {(program.usage.snapBufferBytes ?? 0).toLocaleString()}{" "}
                        字节。均不含完整运行开销，不代表设备文件体积或可用内存。
                      </p>
                    </details>
                  )}
                </article>
              ))}
          </div>
          {!programs.length && (
            <p className="wb-check-empty">
              {report.programs.length
                ? "没有符合筛选的节目"
                : "尚无可检查的节目"}
            </p>
          )}
          <Pages
            page={programPage}
            total={programs.length}
            onChange={setProgramPage}
            label="节目"
          />
        </>
      )}
    </section>
  );
}
function Pages({
  page,
  total,
  onChange,
  label,
}: {
  page: number;
  total: number;
  onChange(page: number): void;
  label: string;
}) {
  if (total <= PAGE_SIZE) return null;
  return (
    <nav className="wb-check-pages" aria-label={`${label}分页`}>
      <button disabled={!page} onClick={() => onChange(page - 1)}>
        上一页
      </button>
      <span>
        {page + 1} / {Math.ceil(total / PAGE_SIZE)}
      </span>
      <button
        disabled={(page + 1) * PAGE_SIZE >= total}
        onClick={() => onChange(page + 1)}
      >
        下一页
      </button>
    </nav>
  );
}
