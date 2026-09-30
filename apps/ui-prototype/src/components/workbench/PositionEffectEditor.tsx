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
import {
  positionAxisDrafts,
  readPositionAxes,
  type PositionEffectAxisDraft,
} from "../../position-effect-tools";
import { seconds, secondsToMs } from "../../sequence-tools";
import { EffectInspectorForm } from "./EffectInspectorForm";
import { EffectTiming } from "./EffectTiming";
import { EffectFixtureOrder } from "./EffectFixtureOrder";
import { validateEditorForm } from "./form-validation";

export const PositionEffectEditor = forwardRef<EffectHandle, EffectEditorProps>(
  function PositionEffectEditor(
    {
      effect,
      sceneId,
      fixtures,
      selected,
      isNew,
      busy,
      error,
      onCancel,
      onPending,
      onApply,
      onPreview,
    },
    ref,
  ) {
    const form = useRef<HTMLFormElement>(null);
    const [draft, setDraft] = useState(effect);
    const [axes, setAxes] = useState(() => positionAxisDrafts(effect.channels));
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
      dirtyRef.current = true;
      setDirty(true);
      pending.current(true);
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
      setAxes(positionAxisDrafts(effect.channels));
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
    function collect(): EditOperation[] {
      if (!dirtyRef.current) return [];
      try {
        validateEditorForm(form.current);
        const periodMs = secondsToMs(period, "循环周期");
        if (periodMs < 100 || periodMs > 3_600_000)
          throw new Error("循环周期应在 0.1–3600 秒之间");
        return effectCommands(
          sceneId,
          {
            ...draft,
            periodMs,
            channels: readPositionAxes(axes),
            spreadDegrees: Number(timing.spread),
            phaseDegrees: Number(timing.phase),
          },
          fixtures,
          false,
        );
      } catch (e) {
        setLocalError(e instanceof Error ? e.message : String(e));
        throw e;
      }
    }
    function axisChange(
      attribute: PositionEffectAxisDraft["attribute"],
      patch: Partial<PositionEffectAxisDraft>,
    ) {
      mark();
      setAxes((current) =>
        current.map((a) =>
          a.attribute === attribute ? { ...a, ...patch } : a,
        ),
      );
    }
    function scaleSpeed(factor: number) {
      try {
        const value = Math.round(secondsToMs(period, "循环周期") * factor);
        if (value < 100 || value > 3_600_000)
          throw new Error("调整后的周期须在 0.1–3600 秒之间");
        mark();
        setPeriod(seconds(value));
      } catch (e) {
        setLocalError((e as Error).message);
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
        <label>
          循环周期 · 秒
          <input
            type="number"
            required
            min={0.1}
            max={3600}
            step={0.001}
            aria-label="循环周期"
            value={period}
            onChange={(e) => setPeriod(e.target.value)}
          />
        </label>
        <div className="effect-speed-actions">
          <button type="button" onClick={() => scaleSpeed(2)}>
            半速
          </button>
          <button type="button" onClick={() => scaleSpeed(0.5)}>
            倍速
          </button>
        </div>
        <p className="position-effect-note">
          围绕每台灯的静态位置运动；中心偏移和单侧幅度均使用角度。
        </p>
        {(["pan", "tilt"] as const).map((attribute) => {
          const label = attribute === "pan" ? "水平" : "垂直";
          const axis = axes.find((a) => a.attribute === attribute);
          return (
            <section
              key={attribute}
              className="position-effect-axis"
              aria-label={`${label}运动`}
            >
              <label className="position-effect-toggle">
                <input
                  type="checkbox"
                  checked={!!axis}
                  onChange={(e) => {
                    mark();
                    setAxes((current) =>
                      e.target.checked
                        ? [
                            ...current,
                            {
                              attribute,
                              amplitude: "15",
                              offset: "0",
                              phase: attribute === "tilt" ? "90" : "0",
                            },
                          ]
                        : current.filter((a) => a.attribute !== attribute),
                    );
                  }}
                />
                {label}轴
              </label>
              {axis && (
                <>
                  <div className="effect-fields">
                    <label>
                      单侧幅度 · 度
                      <input
                        type="number"
                        required
                        min={0}
                        max={3600}
                        step={0.000001}
                        aria-label={`${label}幅度`}
                        value={axis.amplitude}
                        onChange={(e) =>
                          axisChange(attribute, { amplitude: e.target.value })
                        }
                      />
                    </label>
                    <label>
                      中心偏移 · 度
                      <input
                        type="number"
                        required
                        min={-3600}
                        max={3600}
                        step={0.000001}
                        aria-label={`${label}中心偏移`}
                        value={axis.offset}
                        onChange={(e) =>
                          axisChange(attribute, { offset: e.target.value })
                        }
                      />
                    </label>
                  </div>
                  <label>
                    轴相位 · 度
                    <input
                      type="number"
                      required
                      min={0}
                      max={359}
                      step={1}
                      aria-label={`${label}轴相位`}
                      value={axis.phase}
                      onChange={(e) =>
                        axisChange(attribute, { phase: e.target.value })
                      }
                    />
                  </label>
                </>
              )}
            </section>
          );
        })}
        <EffectTiming
          waveform="position"
          timing={timing}
          onChange={setTiming}
        />
        <div className="effect-checks">
          <label>
            <input
              type="checkbox"
              checked={draft.reverse}
              onChange={(e) =>
                setDraft({ ...draft, reverse: e.target.checked })
              }
            />
            反向灯序
          </label>
          <label>
            <input
              type="checkbox"
              checked={draft.enabled}
              onChange={(e) =>
                setDraft({ ...draft, enabled: e.target.checked })
              }
            />
            启用效果
          </label>
        </div>
        <EffectFixtureOrder
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
  },
);
