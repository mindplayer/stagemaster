import {
  forwardRef,
  useEffect,
  useImperativeHandle,
  useRef,
  useState,
} from "react";
import type { EditOperation, FixtureView } from "../../application-host";
import type { SceneEffect } from "../../effect-types";
import { effectCommands } from "../../effect-tools";
import { seconds } from "../../sequence-tools";
import { effectPeriodMs } from "../../effect-tempo";
import { EffectPeriodControls } from "./EffectPeriodControls";
import { EffectInspectorForm } from "./EffectInspectorForm";
import { validateEditorForm } from "./form-validation";
import { KeyframeEditor, type KeyframeHandle } from "./KeyframeEditor";
import { toKeyframes, valuePercent } from "../../keyframe-tools";

import { EffectTiming } from "./EffectTiming";
import { EffectFixtureOrder } from "./EffectFixtureOrder";

import { attributeLabels as labels } from "../../keyframe-tools";
export interface EffectHandle {
  collect(interactive?: boolean, force?: boolean): EditOperation[];
  accept(): void;
}
export interface EffectEditorProps {
  effect: SceneEffect;
  sceneId: string;
  fixtures: FixtureView[];
  selected: string[];
  isNew: boolean;
  busy: boolean;
  error: string;
  onCancel(): void;
  onPending(pending: boolean): void;
  onApply(): Promise<boolean>;
  onPreview(): Promise<boolean>;
  audition?: import("./useEffectDraftPreview").EffectAuditionControls;
  onDraftChange?(): void;
}
export const EffectEditor = forwardRef<EffectHandle, EffectEditorProps>(
  function EffectEditor(
    {
      effect,
      sceneId,
      fixtures,
      selected,
      isNew,
      busy,
      error,
      onCancel,
      onApply,
      onPending,
      onPreview,
      audition,
      onDraftChange,
    },
    ref,
  ) {
    const form = useRef<HTMLFormElement>(null);
    const dirtyRef = useRef(isNew);
    const [dirty, setDirty] = useState(isNew);
    const [localError, setLocalError] = useState("");
    const pending = useRef(onPending);
    pending.current = onPending;
    function mark() {
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
    const [draft, setDraft] = useState(effect);
    const keyframes = useRef<KeyframeHandle>(null);
    const [speedError, setSpeedError] = useState("");
    const [period, setPeriod] = useState(seconds(effect.periodMs));
    const [timing, setTiming] = useState({
      spread: String(effect.spreadDegrees),
      phase: String(effect.phaseDegrees),
      duty: String(effect.dutyPercent),
    });
    const isColor = ["red", "green", "blue"].every((key) =>
      effect.channels.some((c) => c.attribute === key),
    );
    const [illuminate, setIlluminate] = useState(
      isNew && isColor && effect.enabled,
    );
    const [ends, setEnds] = useState(() =>
      effect.channels.map((c) => ({
        low: valuePercent(c.low ?? 0),
        high: valuePercent(c.high ?? 65535),
      })),
    );
    const source = JSON.stringify(effect);
    useEffect(() => {
      setDraft(effect);
      setPeriod(seconds(effect.periodMs));
      setTiming({
        spread: String(effect.spreadDegrees),
        phase: String(effect.phaseDegrees),
        duty: String(effect.dutyPercent),
      });
      setEnds(
        effect.channels.map((c) => ({
          low: valuePercent(c.low ?? 0),
          high: valuePercent(c.high ?? 65535),
        })),
      );
      setIlluminate(isNew && isColor && effect.enabled);
      dirtyRef.current = isNew;
      setDirty(isNew);
      pending.current(isNew);
      setLocalError("");
    }, [source, isNew]);
    useEffect(() => () => pending.current(false), []);
    const change = (patch: Partial<SceneEffect>) => {
      mark();
      setDraft((d) => ({ ...d, ...patch }));
    };
    useImperativeHandle(ref, () => ({ collect, accept }));
    function collect(interactive = true, force = false): EditOperation[] {
      if (!force && !dirtyRef.current) return [];
      try {
        validateEditorForm(form.current, interactive);
        const periodMs = effectPeriodMs(period);
        const channels =
          draft.waveform === "keyframes"
            ? keyframes.current!.collect(interactive)
            : readRanges();
        return effectCommands(
          sceneId,
          {
            ...draft,
            periodMs,
            channels,
            spreadDegrees: Number(timing.spread),
            phaseDegrees: Number(timing.phase),
            dutyPercent: Number(timing.duty),
          },
          fixtures,
          illuminate,
        );
      } catch (e) {
        if (interactive)
          setLocalError(e instanceof Error ? e.message : String(e));
        throw e;
      }
    }
    const color = (end: "low" | "high") =>
      "#" +
      ["red", "green", "blue"]
        .map((key) => {
          const index = effect.channels.findIndex((c) => c.attribute === key);
          return Math.round((Number(ends[index]?.[end] ?? 0) * 255) / 100)
            .toString(16)
            .padStart(2, "0");
        })
        .join("");
    function putColor(end: "low" | "high", hex: string) {
      mark();
      setEnds((previous) =>
        previous.map((v, i) => {
          const index = ["red", "green", "blue"].indexOf(
            effect.channels[i].attribute,
          );
          return index < 0
            ? v
            : {
                ...v,
                [end]: String(
                  (parseInt(hex.slice(1 + index * 2, 3 + index * 2), 16) /
                    255) *
                    100,
                ),
              };
        }),
      );
    }
    function readRanges() {
      return draft.channels.map((c, i) => ({
        attribute: c.attribute,
        ...Object.fromEntries(
          ["low", "high"].map((key) => {
            const raw = ends[i][key as "low" | "high"];
            const value = Number(raw);
            if (
              !raw.trim() ||
              !Number.isFinite(value) ||
              value < 0 ||
              value > 100
            )
              throw new Error("效果两端值应在 0–100% 之间");
            return [key, Math.round((value * 65535) / 100)];
          }),
        ),
      })) as SceneEffect["channels"];
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
            onChange={(e) => change({ name: e.target.value })}
          />
        </label>
        <div className="effect-fields">
          <EffectPeriodControls
            value={period}
            onChange={(value) => {
              mark();
              setPeriod(value);
            }}
          />
          <label>
            变化方式
            <select
              disabled={draft.waveform === "keyframes"}
              aria-label="变化方式"
              value={draft.waveform}
              onChange={(e) =>
                change({ waveform: e.target.value as SceneEffect["waveform"] })
              }
            >
              {draft.waveform === "keyframes" && (
                <option value="keyframes">逐帧编辑</option>
              )}
              <option value="smooth">平滑往返</option>
              <option value="triangle">线性往返</option>
              <option value="pulse">脉冲切换</option>
            </select>
          </label>
        </div>
        <div className="effect-speed-actions">
          {draft.waveform !== "keyframes" && (
            <button
              type="button"
              onClick={() => {
                try {
                  change(
                    toKeyframes({
                      ...draft,
                      dutyPercent: Number(timing.duty),
                      channels: readRanges(),
                    }),
                  );
                  setSpeedError("");
                } catch (e) {
                  setSpeedError((e as Error).message);
                }
              }}
            >
              转换为关键帧
            </button>
          )}
        </div>
        {speedError && (
          <p role="alert" className="wb-library-error">
            {speedError}
          </p>
        )}
        {draft.waveform === "keyframes" && (
          <KeyframeEditor
            ref={keyframes}
            channels={draft.channels}
            onChange={mark}
          />
        )}
        {isColor && draft.waveform !== "keyframes" && (
          <div className="effect-fields effect-colors">
            <label>
              颜色一
              <input
                type="color"
                aria-label="颜色一"
                value={color("low")}
                onChange={(e) => putColor("low", e.target.value)}
              />
            </label>
            <label>
              颜色二
              <input
                type="color"
                aria-label="颜色二"
                value={color("high")}
                onChange={(e) => putColor("high", e.target.value)}
              />
            </label>
          </div>
        )}
        {draft.waveform !== "keyframes" && (
          <details open={!isColor}>
            <summary>{isColor ? "精确颜色通道" : "亮度范围"}</summary>
            {draft.channels.map((c, i) => (
              <div className="effect-fields" key={c.attribute}>
                {(["low", "high"] as const).map((end, j) => (
                  <label key={end}>
                    {labels[c.attribute]} · 数值{j ? "二" : "一"} %
                    <input
                      type="number"
                      required
                      min={0}
                      max={100}
                      step="any"
                      aria-label={`${labels[c.attribute]}数值${j ? "二" : "一"}`}
                      value={ends[i][end]}
                      onChange={(e) =>
                        setEnds((prev) =>
                          prev.map((v, n) =>
                            n === i ? { ...v, [end]: e.target.value } : v,
                          ),
                        )
                      }
                    />
                  </label>
                ))}
              </div>
            ))}
          </details>
        )}
        <EffectTiming
          waveform={draft.waveform}
          timing={timing}
          onChange={setTiming}
        />
        <div className="effect-checks">
          <label>
            <input
              type="checkbox"
              checked={draft.reverse}
              onChange={(e) => change({ reverse: e.target.checked })}
            />
            反向灯序
          </label>
          <label>
            <input
              type="checkbox"
              checked={draft.enabled}
              onChange={(e) => change({ enabled: e.target.checked })}
            />
            启用效果
          </label>
          {isColor && (
            <label>
              <input
                type="checkbox"
                checked={illuminate}
                onChange={(e) => setIlluminate(e.target.checked)}
              />
              同时将所选灯具亮度设为 100%
            </label>
          )}
        </div>
        <EffectFixtureOrder
          ids={draft.fixtureIds}
          fixtures={fixtures}
          selected={selected}
          channels={draft.channels}
          onChange={(fixtureIds) => change({ fixtureIds })}
        />
      </EffectInspectorForm>
    );
  },
);
