import { useRef, useState } from "react";
import type {
  AudioLoopRange,
  AudioPosition,
  AudioTimeline,
} from "../../audio-types";
import { audioTime } from "../../audio-tools";
import {
  AudioLoopError,
  audioLoopRange,
  markerLoopRange,
} from "../../audio-loop-tools";
import "./audio-loop.css";

export function AudioLoopControls({
  track,
  selected,
  position,
  disabled,
  configure,
}: {
  track: AudioTimeline;
  selected: string;
  position: AudioPosition;
  disabled: boolean;
  configure(range: AudioLoopRange | null): Promise<boolean>;
}) {
  const [open, setOpen] = useState(false);
  const [draft, setDraft] = useState<{ start: string; end: string } | null>(
    null,
  );
  const [working, setWorking] = useState(false);
  const [problem, setProblem] = useState("");
  const form = useRef<HTMLFormElement>(null);
  const range = position.loopRange;
  const duration = track.outMs - track.inMs;
  const fallback = range ?? { startMs: 0, endMs: Math.min(duration, 10000) };
  const value = draft ?? {
    start: String(fallback.startMs / 1000),
    end: String(fallback.endMs / 1000),
  };
  const blocked = disabled || working || position.playing;
  const segment = markerLoopRange(track, selected);
  function cancel() {
    setDraft(null);
    setProblem("");
  }
  function fill(range: AudioLoopRange) {
    setDraft({
      start: String(range.startMs / 1000),
      end: String(range.endMs / 1000),
    });
    setProblem("");
  }
  async function apply(clear = false) {
    if (blocked) return;
    setWorking(true);
    setProblem("");
    try {
      const next = clear
        ? null
        : audioLoopRange(value.start, value.end, duration);
      if (await configure(next)) cancel();
      else throw new AudioLoopError("start", "循环未更新，请查看音乐错误提示");
    } catch (error) {
      setProblem(error instanceof Error ? error.message : String(error));
      const field = error instanceof AudioLoopError ? error.field : "start";
      requestAnimationFrame(() =>
        form.current
          ?.querySelector<HTMLInputElement>(`[name="${field}"]`)
          ?.focus(),
      );
    } finally {
      setWorking(false);
    }
  }
  return (
    <section className="audio-loop-tools" aria-label="局部循环试听">
      <div className="audio-loop-heading">
        <button aria-expanded={open} onClick={() => setOpen(!open)}>
          局部循环
        </button>
        <span>
          {range
            ? `${audioTime(range.startMs)} — ${audioTime(range.endMs)}`
            : "未启用"}
        </span>
        {range && <strong>{position.playing ? "循环中" : "已启用"}</strong>}
        {working && <span role="status">正在准备循环…</span>}
      </div>
      {open && (
        <form
          ref={form}
          onSubmit={(e) => {
            e.preventDefault();
            void apply();
          }}
          onKeyDown={(e) => {
            if (e.key === "Escape") {
              e.preventDefault();
              e.stopPropagation();
              cancel();
            }
          }}
        >
          <div className="audio-loop-fields">
            {(["start", "end"] as const).map((field) => (
              <label key={field}>
                {field === "start" ? "起点（秒）" : "终点（秒）"}
                <input
                  name={field}
                  aria-label={
                    field === "start" ? "循环起点（秒）" : "循环终点（秒）"
                  }
                  inputMode="decimal"
                  value={value[field]}
                  disabled={blocked}
                  onChange={(e) => {
                    setDraft({ ...value, [field]: e.target.value });
                    setProblem("");
                  }}
                />
                <button
                  type="button"
                  aria-label={
                    field === "start" ? "播放头设为起点" : "播放头设为终点"
                  }
                  disabled={blocked}
                  onClick={() => {
                    setDraft({
                      ...value,
                      [field]: String(position.positionMs / 1000),
                    });
                    setProblem("");
                  }}
                >
                  {field === "start" ? "播放头设为起点" : "播放头设为终点"}
                </button>
              </label>
            ))}
            <div className="audio-loop-actions">
              <button
                type="button"
                disabled={blocked || !segment}
                onClick={() => segment && fill(segment)}
              >
                取所选灯光段落
              </button>
              <button type="submit" disabled={blocked}>
                {range ? "更新循环" : "启用循环"}
              </button>
              <button
                type="button"
                disabled={blocked || !range}
                onClick={() => void apply(true)}
              >
                关闭循环
              </button>
              {draft && (
                <button type="button" disabled={working} onClick={cancel}>
                  取消修改
                </button>
              )}
            </div>
          </div>
          <small>
            {position.playing
              ? "暂停后可调整范围。"
              : "仅用于本次试听，范围 0.100–60 秒。"}
            停止回到循环起点；定位到范围外会退出循环。
          </small>
          {problem && (
            <p role="alert" className="audio-error">
              {problem}
            </p>
          )}
        </form>
      )}
    </section>
  );
}
