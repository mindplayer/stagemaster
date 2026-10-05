import { useEffect, useRef, useState } from "react";
import { audioTime } from "../../audio-tools";
import { MediaLoopControls } from "./MediaLoopControls";
import { MediaControlStatus } from "./MediaControlStatus";
import { MediaRequestDetails } from "./MediaRequestDetails";
import type { ExecutionView } from "../../execution-types";
import type { ExecutionMediaAction } from "../../execution-media-types";
import {
  mediaSeekReceipt,
  type MediaRequestIdentity,
} from "../../media-seek-receipt";
const names = {
  ready: "待播放",
  preparing: "准备中",
  playing: "播放中",
  paused: "已暂停",
  stopped: "已停止",
  ended: "已结束",
  failed: "播放失败",
};
export function MediaControls({
  runtime,
  disabled,
  sourceId,
  onDraftChange,
  onAction,
}: {
  runtime: ExecutionView;
  disabled: boolean;
  sourceId?: string;
  onDraftChange?(id: string, dirty: boolean): void;
  onAction(action: ExecutionMediaAction): Promise<MediaRequestIdentity | null>;
}) {
  const [draft, setDraft] = useState<{ text: string } | null>(null);
  const [submitted, setSubmitted] = useState<{
    draft: { text: string };
    request: MediaRequestIdentity;
  } | null>(null);
  const [error, setError] = useState("");
  const cancelled = useRef(false);
  const config = runtime.catalog.audio;
  const state = runtime.observation.snapshot?.state;
  const media = state?.media?.find((m) => m.id === config?.group);
  const audio = state?.audio;
  useEffect(() => {
    if (!submitted || !config) return;
    const receipt = mediaSeekReceipt(submitted.request, runtime, config.group);
    if (receipt === "waiting") return;
    if (receipt === "applied")
      setDraft((current) => (current === submitted.draft ? null : current));
    setSubmitted(null);
  }, [submitted, runtime, config]);
  const dirty = draft !== null;
  useEffect(() => {
    if (sourceId) onDraftChange?.(sourceId, dirty);
  }, [sourceId, dirty, onDraftChange]);
  if (!config || !media || !audio)
    return <p role="status">正在读取后台音乐状态</p>;
  const failed = audio.status === "failed";
  const unavailable = disabled || failed;
  const position = audio.positionMs;
  const playing = media.status === "Following";
  const max = Math.max(0, config.durationMs - (config.seekIncludesEnd ? 0 : 1));
  const validDraft =
    draft !== null && /^\d+(\.\d{1,3})?$/.test(draft.text.trim());
  const desired = validDraft ? Math.round(Number(draft.text) * 1000) : position;
  async function seek(ms: number) {
    if (unavailable) return;
    if (!Number.isSafeInteger(ms) || ms < 0 || ms > max) {
      setError(
        config?.seekIncludesEnd
          ? "请输入音乐范围内的秒数，最多三位小数。"
          : "请输入音乐范围内的秒数，最多三位小数；当前不支持直接定位到末尾。",
      );
      return;
    }
    setError("");
    const original = draft;
    const request = await onAction({ kind: "seek", positionMs: ms, playing });
    if (request && original) setSubmitted({ draft: original, request });
  }
  async function recover() {
    if (disabled) return;
    setError("");
    const original = draft;
    const request = await onAction({ kind: "recover", positionMs: 0 });
    if (request && original) setSubmitted({ draft: original, request });
  }
  return (
    <article
      className="execution-source execution-music"
      aria-label="后台音乐编排"
    >
      <header>
        <h3>音乐编排</h3>
        <span>{names[audio.status]}</span>
      </header>
      <p>{config.output === "software" ? "静音预演" : "本机声音输出"}</p>
      <div className="execution-music-time">
        <output aria-label="后台音乐位置">{audioTime(position)}</output>
        <span>/ {audioTime(config.durationMs)}</span>
      </div>
      <input
        type="range"
        aria-label="后台音乐进度"
        min={0}
        max={max}
        step={1}
        value={Math.min(max, Math.max(0, desired))}
        disabled={unavailable}
        onPointerDown={() => {
          cancelled.current = false;
        }}
        onChange={(e) => {
          if (!cancelled.current) {
            setDraft({ text: (Number(e.target.value) / 1000).toFixed(3) });
            setError("");
          }
        }}
        onPointerUp={(e) => {
          if (!cancelled.current) void seek(Number(e.currentTarget.value));
          cancelled.current = false;
        }}
        onPointerCancel={() => {
          cancelled.current = false;
          setDraft(null);
          setError("");
        }}
        onKeyDown={(e) => {
          if (e.key === "Escape") {
            cancelled.current = true;
            setDraft(null);
            setError("");
            e.preventDefault();
          } else cancelled.current = false;
        }}
        onKeyUp={(e) => {
          if (
            [
              "ArrowLeft",
              "ArrowRight",
              "ArrowUp",
              "ArrowDown",
              "Home",
              "End",
              "PageUp",
              "PageDown",
            ].includes(e.key)
          )
            void seek(Number(e.currentTarget.value));
        }}
      />
      <div className="execution-buttons">
        <button
          className="wb-primary"
          disabled={unavailable || (playing && audio.status !== "preparing")}
          onClick={() => void onAction({ kind: "play" })}
        >
          {audio.status === "ended"
            ? "重新播放"
            : media.status === "Paused"
              ? "继续播放"
              : "播放音乐"}
        </button>
        <button
          disabled={unavailable || (!playing && audio.status !== "preparing")}
          onClick={() => void onAction({ kind: "pause" })}
        >
          暂停音乐
        </button>
        <button
          disabled={unavailable}
          onClick={() => void onAction({ kind: "stop" })}
        >
          停止音乐
        </button>
      </div>
      {config.performanceLoops && (
        <MediaLoopControls
          audio={audio}
          disabled={unavailable}
          onAction={onAction}
        />
      )}
      <form
        onSubmit={(e) => {
          e.preventDefault();
          if (!validDraft) setError("请输入非负秒数，最多三位小数。");
          else void seek(desired);
        }}
      >
        <label>
          定位到{" "}
          <input
            aria-label="后台音乐定位秒数"
            inputMode="decimal"
            value={draft?.text ?? (position / 1000).toFixed(3)}
            disabled={unavailable}
            onChange={(e) => {
              setDraft({ text: e.target.value });
              setError("");
            }}
            onKeyDown={(e) => {
              if (e.key === "Escape") {
                setDraft(null);
                setError("");
                e.preventDefault();
              }
            }}
          />{" "}
          秒
        </label>
        <button disabled={unavailable || draft === null}>定位</button>
        {draft !== null && (
          <button
            type="button"
            onClick={() => {
              setDraft(null);
              setError("");
            }}
          >
            取消定位
          </button>
        )}
      </form>
      {failed && config.providerRecovery && (
        <section aria-label="音乐故障恢复">
          <button disabled={disabled} onClick={() => void recover()}>
            重新准备音乐
          </button>
          <p>从头重新准备并保持暂停，准备完成后再继续播放。</p>
        </section>
      )}
      {error && <p role="alert">{error}</p>}
      {audio.problem && <p role="alert">{audio.problem}</p>}
      <MediaControlStatus runtime={runtime} group={config.group} />
      <MediaRequestDetails runtime={runtime} group={config.group} />
    </article>
  );
}
