import { useRef, useState } from "react";
import type { EffectAttribute } from "../../effect-types";
import {
  attributeLabels,
  readFrames,
  type FrameDraft,
} from "../../keyframe-tools";
import { curvePath, moveCurveFrame } from "../../keyframe-geometry";
import "./keyframe-curve.css";

const colors = {
  dimmer: "#7de1ca",
  red: "#ff8d92",
  green: "#83d699",
  blue: "#83b8ff",
  pan: "#d5a4ff",
  tilt: "#ffc67d",
};
export function KeyframeCurve({
  frames,
  attributes,
  active,
  onSelect,
  onChange,
}: {
  frames: FrameDraft[];
  attributes: EffectAttribute[];
  active: number;
  onSelect(index: number): void;
  onChange(frames: FrameDraft[]): void;
}) {
  const [channel, setChannel] = useState(attributes[0]);
  const [dragging, setDragging] = useState(false);
  const gesture = useRef<{
    frames: FrameDraft[];
    index: number;
    pointer: number;
  } | null>(null);
  const svg = useRef<SVGSVGElement>(null);
  let valid = true;
  try {
    readFrames(frames, attributes);
  } catch {
    valid = false;
  }
  const cancel = () => {
    const previous = gesture.current;
    gesture.current = null;
    setDragging(false);
    if (previous) onChange(previous.frames);
  };
  return (
    <section className="keyframe-curve" aria-label="关键帧曲线编辑">
      <header>
        <strong>变化曲线</strong>
        {attributes.length > 1 && (
          <select
            aria-label="曲线通道"
            value={channel}
            disabled={dragging}
            onChange={(e) => setChannel(e.target.value as EffectAttribute)}
          >
            {attributes.map((a) => (
              <option key={a} value={a}>
                {attributeLabels[a]}
              </option>
            ))}
          </select>
        )}
        <span>{attributeLabels[channel]} · %</span>
      </header>
      {!valid ? (
        <p role="status">修正下方关键帧数值后显示曲线</p>
      ) : (
        <>
          <div className="keyframe-curve-plot">
            <span className="curve-y-top">100</span>
            <span className="curve-y-bottom">0</span>
            <svg
              ref={svg}
              viewBox="-5 -8 110 116"
              preserveAspectRatio="none"
              aria-label="拖动关键帧调整时间和数值"
              onKeyDown={(e) => {
                if (e.key === "Escape" && gesture.current) {
                  e.preventDefault();
                  e.stopPropagation();
                  cancel();
                }
              }}
            >
              {[0, 25, 50, 75, 100].map((v) => (
                <g key={v} className="curve-grid">
                  <line x1="0" x2="100" y1={v} y2={v} />
                  <line x1={v} x2={v} y1="0" y2="100" />
                </g>
              ))}
              {attributes.map((a) => (
                <path
                  key={a}
                  d={curvePath(frames, a)}
                  fill="none"
                  stroke={colors[a]}
                  opacity={a === channel ? 1 : 0.3}
                  strokeWidth={a === channel ? 2 : 1}
                  vectorEffect="non-scaling-stroke"
                />
              ))}
              {frames.map((frame, index) => (
                <ellipse
                  key={index}
                  cx={Number(frame.position)}
                  cy={100 - Number(frame.values[channel])}
                  rx="2"
                  ry="4"
                  fill={index === active ? colors[channel] : "#12212c"}
                  stroke={colors[channel]}
                  strokeWidth="2"
                  vectorEffect="non-scaling-stroke"
                  role="button"
                  tabIndex={0}
                  aria-pressed={index === active}
                  aria-label={`关键帧 ${index + 1}，时间 ${frame.position}%，${attributeLabels[channel]} ${frame.values[channel]}%`}
                  onFocus={() => onSelect(index)}
                  onPointerDown={(e) => {
                    if (e.button !== 0 || gesture.current) return;
                    e.preventDefault();
                    e.currentTarget.focus();
                    onSelect(index);
                    gesture.current = {
                      frames: structuredClone(frames),
                      index,
                      pointer: e.pointerId,
                    };
                    setDragging(true);
                    e.currentTarget.setPointerCapture(e.pointerId);
                  }}
                  onPointerMove={(e) => {
                    const g = gesture.current;
                    if (!g || g.pointer !== e.pointerId || !svg.current) return;
                    const rect = svg.current.getBoundingClientRect();
                    onChange(
                      moveCurveFrame(
                        g.frames,
                        g.index,
                        channel,
                        ((e.clientX - rect.left) / rect.width) * 110 - 5,
                        108 - ((e.clientY - rect.top) / rect.height) * 116,
                      ),
                    );
                  }}
                  onPointerUp={(e) => {
                    if (gesture.current?.pointer !== e.pointerId) return;
                    gesture.current = null;
                    setDragging(false);
                    e.currentTarget.releasePointerCapture(e.pointerId);
                  }}
                  onPointerCancel={cancel}
                  onLostPointerCapture={cancel}
                  onKeyDown={(e) => {
                    if (!e.key.startsWith("Arrow") || gesture.current) return;
                    e.preventDefault();
                    e.stopPropagation();
                    const step = e.altKey ? 0.01 : e.shiftKey ? 10 : 1;
                    onChange(
                      moveCurveFrame(
                        frames,
                        index,
                        channel,
                        Number(frame.position) +
                          (e.key === "ArrowRight"
                            ? step
                            : e.key === "ArrowLeft"
                              ? -step
                              : 0),
                        Number(frame.values[channel]) +
                          (e.key === "ArrowUp"
                            ? step
                            : e.key === "ArrowDown"
                              ? -step
                              : 0),
                      ),
                    );
                  }}
                />
              ))}
            </svg>
          </div>
          <div className="curve-time-labels">
            <span>0%</span>
            <span>50%</span>
            <span>100% · 循环</span>
          </div>
        </>
      )}
      <small>拖动调整 · 方向键微调 · 按 Esc 取消拖动</small>
    </section>
  );
}
