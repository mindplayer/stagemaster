import { useState } from "react";
import type { ComponentProps } from "react";
import type { SequenceReportExport as Receipt } from "../../report-types";
import type { PatchReportExport } from "./PatchReportExport";
import { ResourcePicker } from "../resources/ResourcePicker";
import { useReportExport } from "./use-report-export";
import "./patch-report.css";

export function SequenceReportExport({
  host,
  project,
  generation,
  hasDrafts,
  busy,
  visible,
  capture,
}: ComponentProps<typeof PatchReportExport>) {
  const [selectedId, setSelectedId] = useState("");
  const sequence =
    project.sequences.find((s) => s.id === selectedId) ?? project.sequences[0];
  const { working, receipt, message, error, start } = useReportExport<Receipt>({
    busy,
    capture,
  });
  return (
    <section
      className="patch-report-export"
      hidden={!visible}
      aria-label="场景列表节目单"
    >
      <div className="patch-report-heading">
        <div>
          <h2>节目单</h2>
          <p>
            {sequence
              ? `${sequence.steps.length} 个步骤 · 按执行顺序导出`
              : "尚无场景列表"}
          </p>
        </div>
        <button
          disabled={busy || working || !sequence}
          onClick={() => {
            if (sequence)
              void start((version) =>
                host.exportSequenceReport(version, sequence.id),
              );
          }}
        >
          {working ? "正在导出…" : "导出节目单（CSV）"}
        </button>
      </div>
      {visible && (
        <ResourcePicker
          label="节目单场景列表"
          placeholder="选择场景列表"
          value={sequence?.id}
          options={project.sequences.map((s) => ({
            id: s.id,
            label: s.name,
            detail: `${s.steps.length} 个步骤`,
          }))}
          disabled={busy || working || !visible}
          onSelect={setSelectedId}
        />
      )}
      <p>
        包含幕场、台词或动作提示、备注、灯光场景和时间设置，可用表格软件打开。
      </p>
      <p>使用当前工程内容；手动推进保留人工等待，自动等待从渐变完成后开始。</p>
      {error && (
        <p role="alert" className="wb-error">
          {error}
        </p>
      )}
      {message && <p role="status">{message}</p>}
      {receipt && (
        <div role="status" className="patch-report-receipt">
          <strong>
            已导出「{receipt.sequenceName}」· {receipt.stepCount} 个步骤
          </strong>
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
