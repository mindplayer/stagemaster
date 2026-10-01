import { useReportExport } from "./use-report-export";
import type { ApplicationHost, ProjectView } from "../../application-host";
import type { PatchReportExport as Receipt } from "../../report-types";
import "./patch-report.css";

export function PatchReportExport({
  host,
  project,
  generation,
  hasDrafts,
  busy,
  visible,
  capture,
}: {
  host: ApplicationHost;
  project: ProjectView;
  generation: number;
  hasDrafts: boolean;
  busy: boolean;
  visible: boolean;
  capture(): Promise<number | null>;
}) {
  const { working, receipt, message, error, start } = useReportExport<Receipt>({
    busy,
    capture,
  });
  const placed = new Set(project.stage.placements.map((p) => p.fixtureId));
  const missingPatch = project.fixtures.filter(
    (f) => f.address === null || f.universe === null,
  ).length;
  const missingPlacement = project.fixtures.filter(
    (f) => !placed.has(f.id),
  ).length;
  return (
    <section
      className="patch-report-export"
      hidden={!visible}
      aria-label="工程交接资料"
    >
      <div className="patch-report-heading">
        <div>
          <h2>配灯表</h2>
          <p>{project.fixtures.length} 台灯具 · 全部导出</p>
        </div>
        <button
          disabled={busy || working}
          onClick={() =>
            void start((version) => host.exportPatchReport(version))
          }
        >
          {working ? "正在导出…" : "导出配灯表（CSV）"}
        </button>
      </div>
      <p>包含型号、模式、地址、空间和安装位置，可用表格软件打开。</p>
      {(missingPatch > 0 || missingPlacement > 0) && (
        <p>
          {missingPatch} 台未配适 · {missingPlacement}{" "}
          台未布置，仍会保留在表中。
        </p>
      )}
      <p>使用当前工程内容；已应用但尚未保存的修改也会导出。</p>
      {error && (
        <p role="alert" className="wb-error">
          {error}
        </p>
      )}
      {message && <p role="status">{message}</p>}
      {receipt && (
        <div role="status" className="patch-report-receipt">
          <strong>已导出 {receipt.fixtureCount} 台灯具</strong>
          <span>{receipt.path}</span>
          {(receipt.generation !== generation || hasDrafts) && (
            <span>工程已变化，如需最新资料请重新导出。</span>
          )}
          {receipt.warning && <span>{receipt.warning}</span>}
        </div>
      )}
    </section>
  );
}
