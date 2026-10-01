import {
  forwardRef,
  useEffect,
  useImperativeHandle,
  useRef,
  useState,
} from "react";
import type { EditOperation } from "../../application-host";
import type { EffectEditorProps, EffectHandle } from "./EffectEditor";
import { effectCommands } from "../../effect-tools";
import { readWorldLine, WorldLineInputError } from "../../world-line-tools";
import { seconds } from "../../sequence-tools";
import { effectPeriodMs } from "../../effect-tempo";
import { EffectPeriodControls } from "./EffectPeriodControls";
import { EffectInspectorForm } from "./EffectInspectorForm";
import { EffectTiming } from "./EffectTiming";
import { EffectFixtureOrder } from "./EffectFixtureOrder";
import { validateEditorForm } from "./form-validation";
import { WorldLineFields } from "./WorldLineFields";
import "./effects.css";

export const WorldLineEffectEditor = forwardRef<
  EffectHandle,
  EffectEditorProps
>(function WorldLineEffectEditor(
  {
    project,
    effect,
    sceneId,
    fixtures,
    placements,
    selected,
    isNew,
    busy,
    error,
    onCancel,
    onPending,
    onApply,
    onPreview,
    audition,
    onDraftChange,
  },
  ref,
) {
  const form = useRef<HTMLFormElement>(null);
  const semanticInput = useRef<HTMLInputElement | null>(null);
  const [draft, setDraft] = useState(effect);
  const [period, setPeriod] = useState(seconds(effect.periodMs));
  const [timing, setTiming] = useState({
    spread: String(effect.spreadDegrees),
    phase: String(effect.phaseDegrees),
    duty: String(effect.dutyPercent),
  });
  const dirtyRef = useRef(isNew);
  const [dirty, setDirty] = useState(isNew);
  const [localError, setLocalError] = useState("");
  const pending = useRef(onPending);
  pending.current = onPending;
  function mark() {
    semanticInput.current?.setCustomValidity("");
    semanticInput.current = null;
    dirtyRef.current = true;
    setDirty(true);
    pending.current(true);
    onDraftChange?.();
    setLocalError("");
  }
  function accept() {
    dirtyRef.current = false;
    setDirty(false);
    pending.current(false);
    setLocalError("");
  }
  const source = JSON.stringify(effect);
  useEffect(() => {
    setDraft(effect);
    setPeriod(seconds(effect.periodMs));
    setTiming({
      spread: String(effect.spreadDegrees),
      phase: String(effect.phaseDegrees),
      duty: String(effect.dutyPercent),
    });
    dirtyRef.current = isNew;
    setDirty(isNew);
    pending.current(isNew);
    setLocalError("");
  }, [source, isNew]);
  useEffect(() => () => pending.current(false), []);
  useImperativeHandle(ref, () => ({ collect, accept }));
  function collect(interactive = true, force = false): EditOperation[] {
    if (!force && !dirtyRef.current) return [];
    try {
      validateEditorForm(form.current, interactive);
      const targetPath = readWorldLine(draft.targetPath);
      const unplaced = draft.fixtureIds.find(
        (id) => !placements.some((p) => p.fixtureId === id),
      );
      if (draft.enabled && unplaced)
        throw new Error(
          `灯具“${fixtures.find((f) => f.id === unplaced)?.name ?? unplaced}”尚未布置灯位，请先设置安装位置`,
        );
      return effectCommands(
        sceneId,
        {
          ...draft,
          targetPath,
          periodMs: effectPeriodMs(period),
          spreadDegrees: Number(timing.spread),
          phaseDegrees: Number(timing.phase),
        },
        fixtures,
        false,
      );
    } catch (e) {
      if (interactive && e instanceof WorldLineInputError) {
        const input = form.current?.querySelector<HTMLInputElement>(
          `[aria-label="${e.field}"]`,
        );
        input?.setCustomValidity(e.message);
        input?.focus();
        semanticInput.current = input ?? null;
      }
      if (interactive)
        setLocalError(e instanceof Error ? e.message : String(e));
      throw e;
    }
  }
  return (
    <EffectInspectorForm
      form={form}
      busy={busy}
      dirty={dirty}
      isNew={isNew}
      error={localError || error}
      onCancel={onCancel}
      onChange={mark}
      onApply={onApply}
      onPreview={onPreview}
      audition={audition}
    >
      <label>
        效果名称
        <input
          autoFocus
          required
          maxLength={256}
          aria-label="效果名称"
          value={draft.name}
          onChange={(e) => setDraft({ ...draft, name: e.target.value })}
        />
      </label>
      <EffectPeriodControls
        value={period}
        onChange={(value) => {
          mark();
          setPeriod(value);
        }}
      />
      {draft.targetPath && (
        <WorldLineFields
          project={project}
          fixtureIds={draft.fixtureIds}
          busy={busy}
          path={draft.targetPath}
          onChange={(targetPath) => {
            mark();
            setDraft({ ...draft, targetPath });
          }}
        />
      )}
      <EffectTiming waveform="worldLine" timing={timing} onChange={setTiming} />
      <div className="effect-checks">
        <label>
          <input
            type="checkbox"
            checked={draft.reverse}
            onChange={(e) => setDraft({ ...draft, reverse: e.target.checked })}
          />
          反向灯序
        </label>
        <label>
          <input
            type="checkbox"
            checked={draft.enabled}
            onChange={(e) => setDraft({ ...draft, enabled: e.target.checked })}
          />
          启用效果
        </label>
      </div>
      <EffectFixtureOrder
        placements={placements}
        ids={draft.fixtureIds}
        fixtures={fixtures}
        selected={selected}
        channels={draft.channels}
        onChange={(fixtureIds) => {
          mark();
          setDraft({ ...draft, fixtureIds });
        }}
      />
    </EffectInspectorForm>
  );
});
