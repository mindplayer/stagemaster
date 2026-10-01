import { useEffect, useRef, useState } from "react";
import type { ApplicationHost } from "../../application-host";
import {
  OutputControlQueue,
  type OutputControlView,
} from "./output-control-queue";
import "./output-controls.css";
const empty: OutputControlView = {
  actual: null,
  desired: null,
  working: false,
  recovering: false,
  error: "",
};
export function PreviewOutputControls({ host }: { host: ApplicationHost }) {
  const [view, setView] = useState(empty);
  const queue = useRef<OutputControlQueue | null>(null);
  const [text, setText] = useState("100");
  const [editing, setEditing] = useState(false);
  const [invalid, setInvalid] = useState(false);
  const value = view.desired;
  useEffect(() => {
    const control = new OutputControlQueue(
      (request) => host.output(request),
      setView,
    );
    queue.current = control;
    void control.poll();
    const timer = setInterval(() => void control.poll(), 500);
    return () => {
      clearInterval(timer);
      control.dispose();
      queue.current = null;
    };
  }, [host]);
  useEffect(() => {
    if (!editing && value) setText(String(value.percent));
  }, [value?.percent, editing]);
  useEffect(() => {
    setEditing(false);
    setInvalid(false);
  }, [view.actual?.epoch]);
  function apply() {
    const percent = Number(text);
    if (!/^\d{1,3}$/.test(text) || percent > 100) {
      setInvalid(true);
      return;
    }
    setInvalid(false);
    setEditing(false);
    queue.current?.set({ percent });
  }
  const suppressed = value?.blackout || value?.percent === 0;
  return (
    <section
      className={`output-controls${suppressed ? " suppressed" : ""}`}
      aria-label="预演输出总控"
    >
      <label htmlFor="preview-master">预演亮度</label>
      <input
        id="preview-master"
        aria-label="预演亮度总控"
        type="range"
        min="0"
        max="100"
        step="1"
        disabled={!value || view.recovering}
        value={value?.percent ?? 100}
        onChange={(event) =>
          queue.current?.set({ percent: Number(event.target.value) })
        }
      />
      <input
        className="output-percent"
        type="text"
        inputMode="numeric"
        aria-label="预演亮度百分比"
        aria-invalid={invalid}
        disabled={!value || view.recovering}
        value={text}
        onFocus={() => setEditing(true)}
        onChange={(event) => {
          setEditing(true);
          setText(event.target.value);
          setInvalid(false);
        }}
        onBlur={() => {
          if (editing) apply();
        }}
        onKeyDown={(event) => {
          if (event.key === "Enter") {
            event.preventDefault();
            apply();
          }
          if (event.key === "Escape") {
            event.preventDefault();
            event.stopPropagation();
            setInvalid(false);
            setEditing(false);
            setText(String(value?.percent ?? 100));
          }
        }}
      />
      <span>%</span>
      <button
        aria-pressed={value?.blackout ?? false}
        disabled={!value || view.recovering}
        onClick={() => queue.current?.set({ blackout: !value?.blackout })}
      >
        {value?.blackout ? "解除熄灯" : "预演熄灯"}
      </button>
      <details
        className="output-detail"
        open={invalid || !!view.error || undefined}
      >
        <summary aria-label="预演总控状态与范围">
          {view.error || invalid
            ? "!"
            : view.actual?.uncontrolledFixtures
              ? `${view.actual.uncontrolledFixtures} 台未覆盖`
              : view.working
                ? "…"
                : suppressed
                  ? "已抑制"
                  : "范围"}
        </summary>
        <div>
          <p>总控只改变预演亮度，节目与音乐继续运行。停止播放不会解除熄灯。</p>
          <p>
            {value?.blackout
              ? `已熄灯；解除后恢复 ${value.percent}% 总控。`
              : `当前总控 ${value?.percent ?? "—"}%。`}
          </p>
          {!!view.actual?.uncontrolledFixtures && (
            <p role="status">
              {view.actual.uncontrolledFixtures}{" "}
              台灯具没有可识别的亮度属性，总控无法保证其熄灯。
            </p>
          )}
          <p>总控不写入工程或设备播放包。</p>
          {invalid && <p role="alert">请输入 0—100 的整数百分比。</p>}
          {view.error && <p role="alert">{view.error}</p>}
        </div>
      </details>
    </section>
  );
}
