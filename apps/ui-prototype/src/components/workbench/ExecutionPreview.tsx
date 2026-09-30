import { useEffect, useState } from "react";
import {
  PlayIcon,
  PauseIcon,
  StopIcon,
  SkipForwardIcon,
} from "@phosphor-icons/react";
import type { SequenceView } from "../../sequence-types";
import type { PreviewController } from "./usePreviewController";
import { executionPhase, executionPosition } from "./execution-position";
import { seconds } from "../../sequence-tools";
import { PreviewOutput } from "./PreviewOutput";
import "./execution-view.css";

export function ExecutionPreview({
  controller,
  sequence,
  stepId,
  busy,
}: {
  controller: PreviewController;
  sequence?: SequenceView;
  stepId: string;
  busy: boolean;
}) {
  const { snapshot, act, error, working } = controller;
  const loaded = snapshot.loaded;
  const position = executionPosition(loaded);
  const same = !!sequence && position.sequenceId === sequence.id;
  const ready = same && !loaded?.stale;
  const current = loaded?.steps.find((s) => s.id === loaded.stepId);
  const next = loaded?.steps.find((s) => s.id === position.nextId);
  const selected = sequence?.steps.find((s) => s.id === stepId);
  const phase = executionPhase(loaded);
  const [jump, setJump] = useState<string | null>(null);
  useEffect(() => setJump(null), [snapshot.epoch, stepId, loaded?.stale]);
  const controlsBusy = busy || working;
  const status = loaded
    ? {
        idle: "待执行",
        running: "正在执行",
        paused: "已暂停",
        finished: "已结束",
      }[loaded.status]
    : "未载入";
  async function executeSelected() {
    if (!ready || !selected) return;
    if (loaded?.status === "running" || loaded?.status === "paused") {
      setJump(selected.id);
    } else await act({ kind: "execute", stepId: selected.id });
  }
  return (
    <section className="wb-preview execution-preview" aria-label="列表执行台">
      <header className="execution-title">
        <span className="wb-eyebrow">离线执行预演</span>
        <strong>{loaded?.name ?? "尚未载入列表"}</strong>
        <span className={`execution-status ${loaded?.status ?? "empty"}`}>
          {status}
        </span>
      </header>
      <article className="execution-current" aria-label="当前执行步骤">
        <span>{loaded?.sceneId ? "当前场景" : "当前步骤"}</span>
        <strong>
          {current ? `${current.number} · ${current.name}` : "尚未开始"}
        </strong>
        {current && (
          <>
            <div className="execution-phase">
              <span>
                {loaded?.status === "finished" ? "本轮结束" : phase.label}
              </span>
              <output>
                {seconds(phase.elapsed)} 秒
                {phase.total ? ` / ${seconds(phase.total)} 秒` : ""}
              </output>
            </div>
            {phase.total > 0 && (
              <progress
                aria-label="当前阶段进度"
                max={1}
                value={phase.progress}
              />
            )}
          </>
        )}
      </article>
      <article className="execution-next" aria-label="下一执行步骤">
        <span>下一步骤</span>
        <strong>
          {next
            ? `${next.number} · ${next.name}`
            : loaded
              ? "没有后续步骤"
              : "载入后显示"}
        </strong>
      </article>
      <button
        className="wb-primary execution-go"
        disabled={!ready || !loaded?.canNext || controlsBusy}
        onClick={() => {
          setJump(null);
          void act({ kind: "next" });
        }}
      >
        <SkipForwardIcon weight="fill" />
        {loaded?.status === "idle" ? "开始执行" : "执行下一步"}
      </button>
      <div className="execution-transport">
        <button
          disabled={
            !loaded ||
            working ||
            (loaded.status !== "running" && loaded.status !== "paused") ||
            (loaded.status === "paused" && (!ready || busy))
          }
          onClick={() =>
            void act({ kind: loaded?.status === "paused" ? "resume" : "pause" })
          }
        >
          {loaded?.status === "paused" ? <PlayIcon /> : <PauseIcon />}
          {loaded?.status === "paused" ? "继续" : "暂停"}
        </button>
        <button
          disabled={!loaded || working || loaded.status === "idle"}
          onClick={() => {
            setJump(null);
            void act({ kind: "stop" });
          }}
        >
          <StopIcon />
          停止
        </button>
      </div>
      <div className="execution-selected" aria-label="所选步骤操作">
        <span>所选步骤</span>
        <strong>
          {selected ? `${selected.number} · ${selected.name}` : "未选择"}
        </strong>
        {jump === selected?.id ? (
          <div className="execution-jump" role="group" aria-label="确认跳转">
            <p>将按所选步骤的延时与渐变执行，替换当前播放。</p>
            <button
              disabled={controlsBusy || !ready}
              onClick={() => {
                setJump(null);
                void act({ kind: "execute", stepId: jump! });
              }}
            >
              确认执行所选
            </button>
            <button disabled={working} onClick={() => setJump(null)}>
              取消跳转
            </button>
          </div>
        ) : (
          <button
            disabled={!ready || !selected || controlsBusy}
            onClick={() => void executeSelected()}
          >
            执行所选
          </button>
        )}
      </div>
      {(!same || loaded?.stale) && (
        <p className="wb-preview-warning" role="status">
          {loaded?.stale
            ? "工程已修改。当前播放保留旧版本，载入后才能执行新编排。"
            : loaded
              ? loaded.sceneId
                ? "当前播放的是单个场景。载入所选列表后可执行。"
                : "正在查看另一个列表，当前播放保持不变。"
              : "选择列表并载入后开始执行。"}
        </p>
      )}
      <button
        disabled={!sequence || controlsBusy}
        title="载入所选列表并回到灯具默认值，会停止当前音乐或列表播放"
        onClick={() => {
          setJump(null);
          void act("load");
        }}
      >
        {same ? "重新载入列表" : "载入所选列表"}
      </button>
      {error && (
        <p className="wb-preview-warning" role="alert">
          {error}
        </p>
      )}
      {loaded && <PreviewOutput loaded={loaded} />}
    </section>
  );
}
