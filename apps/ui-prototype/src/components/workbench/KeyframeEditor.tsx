import {
  forwardRef,
  useEffect,
  useImperativeHandle,
  useRef,
  useState,
} from "react";
import type { EffectChannel } from "../../effect-types";
import {
  appendFrame,
  attributeLabels,
  colorValues,
  evenFrames,
  FrameInputError,
  frameColor,
  frameDrafts,
  readFrames,
  reorderFrames,
} from "../../keyframe-tools";
import { KeyframeCurve } from "./KeyframeCurve";
export interface KeyframeHandle {
  collect(): EffectChannel[];
}
export const KeyframeEditor = forwardRef<
  KeyframeHandle,
  { channels: EffectChannel[] }
>(function KeyframeEditor({ channels }, ref) {
  const [frames, setFrames] = useState(() => frameDrafts(channels));
  const [error, setError] = useState("");
  const [active, setActive] = useState(0);
  const [focusError, setFocusError] = useState<FrameInputError | null>(null);
  const root = useRef<HTMLDivElement>(null);
  const attributes = channels.map((c) => c.attribute);
  const isColor = ["red", "green", "blue"].every((a) =>
    attributes.some((k) => k === a),
  );
  useImperativeHandle(ref, () => ({
    collect() {
      try {
        return readFrames(frames, attributes);
      } catch (reason) {
        if (reason instanceof FrameInputError) {
          setActive(reason.index);
          setFocusError(reason);
        }
        throw reason;
      }
    },
  }));
  useEffect(() => {
    if (!focusError) return;
    const field = root.current?.querySelector<HTMLInputElement>(
      `[data-frame="${focusError.index}"][data-field="${focusError.field}"]`,
    );
    const details = field?.closest("details");
    if (details) details.open = true;
    field?.focus();
    field?.scrollIntoView({ block: "nearest" });
    setFocusError(null);
  }, [focusError]);
  function update(index: number, patch: Partial<(typeof frames)[number]>) {
    setError("");
    setFrames((prev) =>
      prev.map((f, i) => (i === index ? { ...f, ...patch } : f)),
    );
  }
  return (
    <div className="effect-keyframes" ref={root}>
      <KeyframeCurve
        frames={frames}
        attributes={attributes}
        active={active}
        onSelect={setActive}
        onChange={setFrames}
      />
      <div className="effect-frame-toolbar">
        <strong>
          关键帧 <small>{frames.length} / 32</small>
        </strong>
        <button
          type="button"
          onClick={() => {
            setFrames(evenFrames(frames));
            setError("");
          }}
        >
          均分时间
        </button>
        <button
          type="button"
          disabled={frames.length >= 32}
          onClick={() => {
            try {
              setFrames(appendFrame(frames));
              setActive(frames.length);
              setError("");
            } catch (e) {
              setError(String((e as Error).message));
            }
          }}
        >
          添加一帧
        </button>
      </div>
      <div className="effect-frame-strip" aria-label="关键帧顺序">
        {frames.map((f, i) => (
          <button
            type="button"
            key={i}
            aria-label={`选择第 ${i + 1} 帧`}
            aria-pressed={active === i}
            title={`第 ${i + 1} 帧 · ${f.position}%`}
            onClick={() => setActive(i)}
            style={{
              background: isColor
                ? frameColor(f.values)
                : `rgba(108,213,205,${Math.max(0.12, Number(f.values.dimmer ?? 0) / 100)})`,
            }}
          >
            {i + 1}
          </button>
        ))}
      </div>
      {frames.map(
        (frame, i) =>
          i === active && (
            <article className="effect-frame" key={i}>
              <header>
                <strong>第 {i + 1} 帧</strong>
                <div>
                  <button
                    type="button"
                    aria-label={`提前第 ${i + 1} 帧`}
                    disabled={i === 0}
                    onClick={() => {
                      setFrames(reorderFrames(frames, i, -1));
                      setActive(i - 1);
                    }}
                  >
                    ↑
                  </button>
                  <button
                    type="button"
                    aria-label={`延后第 ${i + 1} 帧`}
                    disabled={i === frames.length - 1}
                    onClick={() => {
                      setFrames(reorderFrames(frames, i, 1));
                      setActive(i + 1);
                    }}
                  >
                    ↓
                  </button>
                  <button
                    type="button"
                    aria-label={`删除第 ${i + 1} 帧`}
                    disabled={frames.length <= 2}
                    onClick={() => {
                      setActive(Math.min(i, frames.length - 2));
                      setFrames(
                        frames
                          .filter((_, n) => n !== i)
                          .map((f, n) =>
                            n === 0 ? { ...f, position: "0" } : f,
                          ),
                      );
                    }}
                  >
                    删除
                  </button>
                </div>
              </header>
              <div className="effect-fields">
                <label>
                  时间 · %
                  <input
                    type="number"
                    min={0}
                    max={99.99}
                    step={0.01}
                    required
                    readOnly={i === 0}
                    aria-label={`第 ${i + 1} 帧时间`}
                    data-frame={i}
                    data-field="position"
                    value={frame.position}
                    onChange={(e) => update(i, { position: e.target.value })}
                  />
                </label>
                <label>
                  到下一帧
                  <select
                    aria-label={`第 ${i + 1} 帧过渡`}
                    value={frame.transition}
                    onChange={(e) =>
                      update(i, {
                        transition: e.target.value as typeof frame.transition,
                      })
                    }
                  >
                    <option value="smooth">平滑渐变</option>
                    <option value="linear">匀速渐变</option>
                    <option value="hold">保持后切换</option>
                  </select>
                </label>
              </div>
              {isColor && (
                <label className="effect-frame-color">
                  颜色
                  <input
                    type="color"
                    aria-label={`第 ${i + 1} 帧颜色`}
                    value={frameColor(frame.values)}
                    onChange={(e) =>
                      update(i, {
                        values: {
                          ...frame.values,
                          ...colorValues(e.target.value),
                        },
                      })
                    }
                  />
                </label>
              )}
              <details open={!isColor}>
                <summary>{isColor ? "精确通道" : "亮度"}</summary>
                <div className="effect-frame-values">
                  {attributes.map((attribute) => (
                    <label key={attribute}>
                      {attributeLabels[attribute]} · %
                      <input
                        type="number"
                        min={0}
                        max={100}
                        step="any"
                        required
                        aria-label={`第 ${i + 1} 帧${attributeLabels[attribute]}`}
                        data-frame={i}
                        data-field={attribute}
                        value={frame.values[attribute] ?? ""}
                        onChange={(e) =>
                          update(i, {
                            values: {
                              ...frame.values,
                              [attribute]: e.target.value,
                            },
                          })
                        }
                      />
                    </label>
                  ))}
                </div>
              </details>
            </article>
          ),
      )}
      <p className="wb-dim">
        末帧按所选方式回到第一帧；时间按整个循环的百分比计算。
      </p>
      {error && (
        <p role="alert" className="wb-library-error">
          {error}
        </p>
      )}
    </div>
  );
});
