import type { ExecutionAudioOutput } from "../../execution-media-types";
import { useState } from "react";
import type { ProjectView } from "../../application-host";
import type { ExecutionSelection } from "../../execution-types";

export function PrepareSources({
  project,
  disabled,
  onPrepare,
}: {
  project: ProjectView;
  disabled: boolean;
  onPrepare(
    selection: ExecutionSelection[],
    output?: ExecutionAudioOutput,
  ): void;
}) {
  const [selected, setSelected] = useState<string[]>([]);
  const [output, setOutput] = useState<ExecutionAudioOutput>("systemDefault");
  const [query, setQuery] = useState("");
  const choices = [
    ...(project.audio
      ? [
          {
            id: "music",
            name: project.audio.asset.fileName,
            kind: "audioTimeline" as const,
            label: "音乐编排",
          },
        ]
      : []),
    ...project.sequences.map((s) => ({
      ...s,
      kind: "sequence" as const,
      label: "列表",
    })),
    ...project.scenes.map((s) => ({
      ...s,
      kind: "scene" as const,
      label: "场景",
    })),
  ];
  const valid = choices.filter((s) => selected.includes(`${s.kind}:${s.id}`));
  const music = valid.some((s) => s.kind === "audioTimeline");
  const looped = !!project.audio?.loopRegions?.some((r) => r.enabled);
  const visible = choices.filter((s) =>
    s.name.toLocaleLowerCase().includes(query.trim().toLocaleLowerCase()),
  );
  return (
    <section className="execution-prepare" aria-label="载入后台节目">
      <header>
        <h3>载入后台节目</h3>
        <span>已选 {valid.length} / 63</span>
      </header>
      <input
        aria-label="搜索待载入节目"
        placeholder="搜索场景、场景列表或音乐"
        value={query}
        onChange={(e) => setQuery(e.target.value)}
      />
      <div className="execution-choices">
        {visible.map((s) => {
          const key = `${s.kind}:${s.id}`;
          return (
            <label key={key}>
              <input
                type="checkbox"
                checked={selected.includes(key)}
                disabled={
                  disabled ||
                  (s.kind === "audioTimeline" &&
                    looped &&
                    !selected.includes(key)) ||
                  (!selected.includes(key) && valid.length >= 63)
                }
                onChange={(e) =>
                  setSelected((old) =>
                    e.target.checked
                      ? [...old, key]
                      : old.filter((k) => k !== key),
                  )
                }
              />
              <span>{s.name}</span>
              <small>{s.label}</small>
            </label>
          );
        })}
        {!visible.length && <p>没有匹配的节目</p>}
      </div>
      {looped && project.audio && (
        <p role="status">音乐含已启用的循环区段，请先停用循环再载入后台。</p>
      )}
      {music && (
        <label className="execution-audio-route">
          声音输出
          <select
            aria-label="后台声音输出"
            value={output}
            disabled={disabled}
            onChange={(e) => setOutput(e.target.value as ExecutionAudioOutput)}
          >
            <option value="systemDefault">本机声音输出</option>
            <option value="software">静音预演</option>
          </select>
        </label>
      )}
      <footer>
        <button
          disabled={disabled || !valid.length}
          onClick={() => setSelected([])}
        >
          清空选择
        </button>
        <button
          className="wb-primary"
          disabled={disabled || !valid.length || (music && looped)}
          onClick={() =>
            onPrepare(
              valid.map(({ kind, id }) =>
                kind === "audioTimeline" ? { kind } : { kind, id },
              ),
              music ? output : undefined,
            )
          }
        >
          载入所选节目
        </button>
      </footer>
    </section>
  );
}
