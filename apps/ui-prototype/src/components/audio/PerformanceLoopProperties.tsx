import type { RefObject } from "react";
import type { AudioLoopRegion } from "../../audio-performance-types";
import type { PerformanceLoopDraft } from "./performance-loop-draft";
import "./performance-loops.css";

export interface PerformanceLoopActions {
  region?: AudioLoopRegion;
  acting: boolean;
  add(): Promise<void>;
  lock(): Promise<void>;
  enabled(): Promise<void>;
  requestRemove(): Promise<void>;
}
export function PerformanceLoopProperties({
  data, dirty, busy, position, form, onChange, onApply, onCancel, actions,
}: {
  data: PerformanceLoopDraft;
  dirty: boolean;
  busy: boolean;
  position: number;
  form: RefObject<HTMLFormElement | null>;
  onChange(value: PerformanceLoopDraft): void;
  onApply(): void;
  onCancel(): void;
  actions?: PerformanceLoopActions;
}) {
  const locked = actions?.region?.locked ?? false;
  const blocked = busy || locked;
  return (
    <form
      className="audio-inspector performance-loop-properties"
      ref={form}
      noValidate
      onSubmit={(e) => {
        e.preventDefault();
        onApply();
      }}
      onKeyDown={(e) => {
        if (e.key === "Escape" && !busy) {
          e.stopPropagation();
          onCancel();
        }
      }}
    >
      <h3>{data.original ? "循环区段属性" : "新建循环区段"}</h3>
      {locked && <p>区段已锁定，解锁后可修改范围与播放方式。</p>}
      <label>
        名称
        <input name="loopName" aria-label="循环区段名称" value={data.name}
          disabled={blocked}
          onChange={(e) => onChange({ ...data, name: e.target.value })} />
      </label>
      <label>
        开始（秒）
        <input name="loopStart" aria-label="循环区段开始（秒）"
          inputMode="decimal" value={data.start} disabled={blocked}
          onChange={(e) => onChange({ ...data, start: e.target.value })} />
      </label>
      <button type="button" disabled={blocked}
        onClick={() => onChange({ ...data, start: (position / 1000).toFixed(3) })}>
        起点取播放头
      </button>
      <label>
        结束（秒）
        <input name="loopEnd" aria-label="循环区段结束（秒）"
          inputMode="decimal" value={data.end} disabled={blocked}
          onChange={(e) => onChange({ ...data, end: e.target.value })} />
      </label>
      <button type="button" disabled={blocked}
        onClick={() => onChange({ ...data, end: (position / 1000).toFixed(3) })}>
        终点取播放头
      </button>
      <label>
        播放方式
        <select aria-label="循环区段播放方式" value={data.mode} disabled={blocked}
          onChange={(e) => onChange({
            ...data, mode: e.target.value as PerformanceLoopDraft["mode"],
          })}>
          <option value="count">固定总次数</option>
          <option value="untilExit">持续循环</option>
        </select>
      </label>
      {data.mode === "count" ? (
        <label>
          总播放次数
          <input name="loopCount" aria-label="区段总播放次数" inputMode="numeric"
            value={data.count} disabled={blocked}
            onChange={(e) => onChange({ ...data, count: e.target.value })} />
        </label>
      ) : (
        <p>持续重复，执行时可选择本圈结束后继续，也可在圈末前取消退出。</p>
      )}
      <small>总次数包含第一遍；1 表示只经过一次。范围不含终点，相邻区段可以共用端点。</small>
      <div className="performance-loop-actions">
        <button type="submit" className="primary" disabled={blocked || !dirty}>应用区段</button>
        <button type="button" disabled={busy || !dirty} onClick={onCancel}>取消输入</button>
      </div>
      {actions?.region && (
        <div className="performance-loop-actions">
          <button type="button" disabled={busy || locked}
            onClick={() => void actions.enabled()}>
            {actions.region.enabled ? "停用区段" : "启用区段"}
          </button>
          <button type="button" disabled={busy} onClick={() => void actions.lock()}>
            {locked ? "解锁区段" : "锁定区段"}
          </button>
          <button type="button" disabled={busy || locked}
            onClick={() => void actions.requestRemove()}>
            删除区段
          </button>
        </div>
      )}
    </form>
  );
}
