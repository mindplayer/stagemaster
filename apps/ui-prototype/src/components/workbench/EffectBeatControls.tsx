import { useRef, useState } from "react";
import { cycleBeats, EffectTapTempo, tempoPeriodMs } from "../../effect-tempo";
import { seconds } from "../../sequence-tools";
import "./effect-period.css";
export function EffectBeatControls({
  onPeriod,
}: {
  onPeriod(value: string): void;
}) {
  const [bpm, setBpm] = useState("");
  const [beats, setBeats] = useState(1);
  const [count, setCount] = useState(0);
  const [message, setMessage] = useState("");
  const [error, setError] = useState("");
  const tapper = useRef(new EffectTapTempo());
  const details = useRef<HTMLDetailsElement>(null);
  const summary = useRef<HTMLElement>(null);
  let preview: number | null = null;
  try {
    preview = tempoPeriodMs(bpm, beats);
  } catch {
    /* Auxiliary input may be incomplete. */
  }
  function adopt() {
    try {
      onPeriod(seconds(tempoPeriodMs(bpm, beats)));
      setError("");
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    }
  }
  function reset() {
    tapper.current.reset();
    setCount(0);
    setMessage("");
    setError("");
  }
  return (
    <details
      className="effect-beat-controls"
      data-editor-navigation="true"
      ref={details}
      onToggle={(e) => {
        if (!e.currentTarget.open) reset();
      }}
      onKeyDown={(e) => {
        if (e.nativeEvent.isComposing) return;
        if (e.key === "Escape" && details.current?.open) {
          e.preventDefault();
          e.stopPropagation();
          if (details.current) details.current.open = false;
          summary.current?.focus();
        }
      }}
    >
      <summary ref={summary}>按拍设置</summary>
      <label>
        每分钟拍数
        <input
          aria-label="每分钟拍数"
          inputMode="decimal"
          placeholder="例如 120"
          maxLength={5}
          value={bpm}
          onChange={(e) => {
            setBpm(e.target.value);
            reset();
          }}
          onKeyDown={(e) => {
            if (e.key === "Enter" && !e.nativeEvent.isComposing) {
              e.preventDefault();
              e.stopPropagation();
              adopt();
            }
          }}
        />
      </label>
      <span>每轮拍数</span>
      <div className="effect-beat-values" role="group" aria-label="每轮拍数">
        {cycleBeats.map((value) => (
          <button
            key={value}
            type="button"
            aria-pressed={beats === value}
            onClick={() => {
              setBeats(value);
              setError("");
            }}
          >
            {value} 拍
          </button>
        ))}
      </div>
      <div className="effect-beat-tap">
        <button
          type="button"
          onKeyDown={(e) => {
            if (e.repeat && (e.key === "Enter" || e.key === " "))
              e.preventDefault();
          }}
          onClick={() => {
            const next = tapper.current.tap(performance.now());
            setCount(next.count);
            setError("");
            if (next.state !== "tooFast")
              setBpm(next.bpm === null ? "" : String(next.bpm));
            setMessage(
              next.state === "tooFast"
                ? "间隔过短，请按节奏继续。"
                : next.state === "invalid"
                  ? "计拍已重置，请重新敲拍。"
                  : next.bpm === null
                    ? "再敲一次以估计拍速。"
                    : "拍速已估计，采用后更新周期。",
            );
          }}
        >
          敲拍取速
        </button>
        <button
          type="button"
          onClick={() => {
            reset();
            setBpm("");
          }}
        >
          重新敲拍
        </button>
        {!!count && <small>最近 {count} 拍</small>}
      </div>
      {message && <small role="status">{message}</small>}
      {preview !== null && <output>待用周期 {seconds(preview)} 秒</output>}
      {error && (
        <p className="wb-library-error" role="alert">
          {error}
        </p>
      )}
      <button type="button" disabled={!bpm.trim()} onClick={adopt}>
        采用此节拍
      </button>
      <small>只设置本效果周期，不绑定音乐或现场节拍。</small>
    </details>
  );
}
