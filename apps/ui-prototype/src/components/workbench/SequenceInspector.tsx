import type { RefObject } from "react";
import type { ProjectView } from "../../application-host";
import type { SequenceView } from "../../sequence-types";
import type { SequenceDraft as Draft } from "../../sequence-tools";
export function SequenceInspector({
  form,
  busy,
  index,
  sequence,
  project,
  data,
  pending,
  localError,
  change,
  onApply,
  onCancel,
}: {
  form: RefObject<HTMLFormElement | null>;
  busy: boolean;
  index: number;
  sequence: SequenceView;
  project: ProjectView;
  data: Draft;
  pending: boolean;
  localError: string;
  change(patch: Partial<Draft>): void;
  onApply(): void;
  onCancel(): void;
}) {
  return (
    <form
      noValidate
      ref={form}
      onSubmit={(e) => {
        e.preventDefault();
        onApply();
      }}
      onKeyDown={(e) => {
        if (e.key === "Escape") {
          e.preventDefault();
          onCancel();
        }
      }}
    >
      <fieldset disabled={busy}>
        <div className="wb-section-title">
          <h2>步骤属性</h2>
          <span>
            {index + 1} / {sequence.steps.length}
          </span>
        </div>
        <label>
          步骤名称
          <input
            name="name"
            value={data.name}
            maxLength={256}
            onChange={(e) => change({ name: e.target.value })}
          />
        </label>
        <div className="wb-form-row">
          <label>
            显示编号
            <input
              name="number"
              value={data.number}
              onChange={(e) => change({ number: e.target.value })}
            />
          </label>
          <label>
            执行顺序
            <input
              type="number"
              min={1}
              max={sequence.steps.length}
              name="position"
              value={data.position}
              onChange={(e) => change({ position: e.target.value })}
            />
          </label>
        </div>
        <label>
          引用场景
          <select
            name="sceneId"
            value={data.sceneId}
            onChange={(e) => change({ sceneId: e.target.value })}
          >
            {project.scenes.map((s) => (
              <option key={s.id} value={s.id}>
                {s.name}
              </option>
            ))}
          </select>
        </label>
        <div className="wb-form-row">
          <label>
            延时（秒）
            <input
              inputMode="decimal"
              name="delay"
              value={data.delay}
              onChange={(e) => change({ delay: e.target.value })}
            />
          </label>
          <label>
            渐变（秒）
            <input
              inputMode="decimal"
              name="fade"
              value={data.fade}
              onChange={(e) => change({ fade: e.target.value })}
            />
          </label>
        </div>
        <label>
          推进方式
          <select
            name="advance"
            value={data.advance}
            onChange={(e) =>
              change({ advance: e.target.value as Draft["advance"] })
            }
          >
            <option value="manual">手动执行下一步</option>
            <option value="after">渐变结束后自动推进</option>
          </select>
        </label>
        {data.advance === "after" && (
          <label>
            自动等待（秒）
            <input
              inputMode="decimal"
              name="wait"
              value={data.wait}
              onChange={(e) => change({ wait: e.target.value })}
            />
          </label>
        )}
        <div className="wb-section-title wb-list-properties-title">
          <h2>列表属性</h2>
        </div>
        <label>
          列表名称
          <input
            name="sequenceName"
            value={data.sequenceName}
            maxLength={256}
            onChange={(e) => change({ sequenceName: e.target.value })}
          />
        </label>
        <label>
          未记录的属性
          <select
            name="tracking"
            value={data.tracking}
            onChange={(e) =>
              change({ tracking: e.target.value as Draft["tracking"] })
            }
          >
            <option value="inherited">继承前一步</option>
            <option value="isolated">使用灯具默认值</option>
          </select>
        </label>
        <label>
          播放方式
          <select
            name="repeat"
            value={data.repeat}
            onChange={(e) =>
              change({ repeat: e.target.value as Draft["repeat"] })
            }
          >
            <option value="once">单次</option>
            <option value="loop">循环</option>
          </select>
        </label>
        {localError && (
          <p className="wb-preview-warning" role="alert">
            {localError}
          </p>
        )}
        <div className="wb-form-actions">
          <button className="wb-primary" type="submit" disabled={!pending}>
            应用修改
          </button>
          <button type="button" disabled={!pending} onClick={onCancel}>
            取消
          </button>
        </div>
      </fieldset>
    </form>
  );
}
