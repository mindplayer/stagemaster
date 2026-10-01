import {
  PlusIcon,
  ArrowUpIcon,
  ArrowDownIcon,
  CopyIcon,
  TrashIcon,
} from "@phosphor-icons/react";
import type { ProjectView } from "../../application-host";
export function SequenceEditToolbar({
  scenes,
  busy,
  addSceneId,
  setAddSceneId,
  stepQuery,
  setStepQuery,
  index,
  stepCount,
  hasStep,
  onInsert,
  onMove,
  onDuplicate,
  onDelete,
}: {
  scenes: ProjectView["scenes"];
  busy: boolean;
  addSceneId: string;
  setAddSceneId(id: string): void;
  stepQuery: string;
  setStepQuery(query: string): void;
  index: number;
  stepCount: number;
  hasStep: boolean;
  onInsert(): void;
  onMove(index: number): void;
  onDuplicate(): void;
  onDelete(): void;
}) {
  return (
    <>
      <div className="wb-sequence-add">
        <select
          aria-label="要加入的场景"
          value={
            scenes.some((s) => s.id === addSceneId)
              ? addSceneId
              : (scenes[0]?.id ?? "")
          }
          onChange={(e) => setAddSceneId(e.target.value)}
          disabled={busy}
        >
          {scenes.map((s) => (
            <option value={s.id} key={s.id}>
              {s.name}
            </option>
          ))}
        </select>
        <button disabled={busy || !scenes.length} onClick={() => onInsert()}>
          <PlusIcon />
          加入步骤
        </button>
      </div>
      <div className="wb-sequence-actions">
        <input
          aria-label="搜索步骤"
          placeholder="搜索步骤、幕场或台词"
          value={stepQuery}
          onChange={(e) => setStepQuery(e.target.value)}
        />
        <button
          aria-label="上移步骤"
          disabled={busy || index <= 0}
          onClick={() => onMove(index - 1)}
        >
          <ArrowUpIcon />
        </button>
        <button
          aria-label="下移步骤"
          disabled={busy || index < 0 || index >= stepCount - 1}
          onClick={() => onMove(index + 1)}
        >
          <ArrowDownIcon />
        </button>
        <button
          aria-label="复制步骤"
          disabled={busy || !hasStep}
          onClick={() => onDuplicate()}
        >
          <CopyIcon />
        </button>
        <button
          aria-label="删除步骤"
          disabled={busy || !hasStep || stepCount === 1}
          title={stepCount === 1 ? "列表至少保留一个步骤" : "删除步骤"}
          onClick={onDelete}
        >
          <TrashIcon />
        </button>
      </div>
    </>
  );
}
