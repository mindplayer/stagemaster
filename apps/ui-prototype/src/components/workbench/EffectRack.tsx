import { useState } from "react";
import type {
  EditOperation,
  FixtureView,
  SceneView,
} from "../../application-host";
import type { SceneEffect } from "../../effect-types";
import {
  createEffect,
  effectTemplates,
  supportsEffect,
  reuseEffect,
} from "../../effect-tools";
import { uniqueName } from "../../editor-tools";
import type { OpenEffect } from "./useEffectSelection";
import { EffectReuseDialog } from "./EffectReuseDialog";
import "./effects.css";
import { isPositionTemplate } from "../../position-effect-tools";

export function EffectRack({
  scene,
  fixtures,
  scenes,
  selected,
  busy,
  error,
  beforeChange,
  onEdit,
  onOpen,
  onToggle,
}: {
  onOpen: OpenEffect;
  onToggle(id: string, enabled: boolean): Promise<boolean>;
  scene: SceneView;
  fixtures: FixtureView[];
  scenes: SceneView[];
  selected: string[];
  busy: boolean;
  error: string;
  beforeChange(): Promise<boolean>;
  onEdit(commands: EditOperation[]): Promise<boolean>;
}) {
  const [importing, setImporting] = useState(false);
  const selectedFixtures = selected.flatMap(
    (id) => fixtures.find((f) => f.id === id) ?? [],
  );
  async function open(effect: SceneEffect, isNew: boolean, copyFrom?: string) {
    if (await beforeChange()) onOpen(effect, isNew, copyFrom);
  }
  return (
    <section className="effect-rack" aria-label="场景动态效果">
      <div className="effect-heading">
        <h2>
          灯光效果 <small>{scene.effects.length}</small>
        </h2>
        <button
          disabled={busy || !scenes.some((s) => s.effects.length)}
          onClick={() => {
            void beforeChange().then((ok) => {
              if (ok) setImporting(true);
            });
          }}
        >
          复用已有
        </button>
        <span>
          {selected.length
            ? `已选 ${selected.length} 台`
            : "选择灯具后添加效果"}
        </span>
      </div>
      <div className="effect-templates">
        {effectTemplates.map((t) => (
          <button
            key={t.key}
            disabled={busy || !supportsEffect(selectedFixtures, t.key)}
            title={
              supportsEffect(selectedFixtures, t.key)
                ? t.detail
                : `请选择全部支持${isPositionTemplate(t.key) || t.key === "worldLine" ? "两轴运动模型" : t.key === "color" || t.key === "multicolor" ? "RGB" : "亮度"}的灯具`
            }
            onClick={() => {
              const effect = createEffect(t.key, crypto.randomUUID(), selected);
              effect.name = uniqueName(
                effect.name,
                scene.effects.map((e) => e.name),
              );
              void open(effect, true);
            }}
          >
            <strong>＋ {t.name}</strong>
            <small>{t.detail}</small>
          </button>
        ))}
      </div>
      <div className="effect-list">
        {scene.effects.map((effect) => (
          <article key={effect.id} className={effect.enabled ? "" : "disabled"}>
            <div>
              <strong>{effect.name}</strong>
              <small>
                {effect.fixtureIds.length} 台 ·{" "}
                {(effect.periodMs / 1000).toLocaleString("zh-CN")} 秒 / 轮 ·{" "}
                {effect.spreadDegrees === 0
                  ? "同步"
                  : `展开 ${effect.spreadDegrees}°`}
                {effect.reverse ? " · 反向" : ""}
                {effect.waveform === "position" && " · 相对位置"}
                {effect.waveform === "worldLine" && " · 共同空间目标"}
                {effect.waveform === "keyframes"
                  ? ` · ${effect.channels[0].keyframes?.length} 帧`
                  : ""}
              </small>
            </div>
            <button
              disabled={busy}
              aria-pressed={effect.enabled}
              onClick={() => void onToggle(effect.id, !effect.enabled)}
            >
              {effect.enabled ? "已启用" : "已停用"}
            </button>
            <button disabled={busy} onClick={() => void open(effect, false)}>
              编辑
            </button>
            <button
              disabled={busy}
              onClick={() =>
                void open(
                  {
                    ...effect,
                    id: crypto.randomUUID(),
                    enabled: false,
                    name: uniqueName(
                      `${effect.name} 副本`,
                      scene.effects.map((e) => e.name),
                    ),
                  },
                  true,
                  effect.id,
                )
              }
            >
              复制
            </button>
            <button
              disabled={busy}
              onClick={() =>
                void onEdit([
                  {
                    op: "effect",
                    command: {
                      kind: "remove",
                      sceneId: scene.id,
                      id: effect.id,
                    },
                  },
                ])
              }
            >
              删除
            </button>
          </article>
        ))}
      </div>
      {importing && (
        <EffectReuseDialog
          scenes={scenes}
          fixtures={fixtures}
          selected={selected}
          onCancel={() => setImporting(false)}
          onChoose={(effect, fixtures) => {
            const copy = reuseEffect(effect, crypto.randomUUID(), fixtures);
            copy.name = uniqueName(
              `${effect.name} 副本`,
              scene.effects.map((e) => e.name),
            );
            onOpen(copy, true);
          }}
        />
      )}
    </section>
  );
}
