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
import { EffectEditor } from "./EffectEditor";
import { EffectReuseDialog } from "./EffectReuseDialog";
import "./effects.css";

export function EffectRack({
  scene,
  fixtures,
  scenes,
  selected,
  busy,
  error,
  beforeChange,
  onEdit,
}: {
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
  const [dialog, setDialog] = useState<{
    effect: SceneEffect;
    isNew: boolean;
  } | null>(null);
  const selectedFixtures = selected.flatMap(
    (id) => fixtures.find((f) => f.id === id) ?? [],
  );
  async function open(effect: SceneEffect, isNew: boolean) {
    if (await beforeChange()) setDialog({ effect, isNew });
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
                : `请选择全部支持${t.key === "color" || t.key === "multicolor" ? "RGB" : "亮度"}的灯具`
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
                {effect.waveform === "keyframes"
                  ? ` · ${effect.channels[0].keyframes?.length} 帧`
                  : ""}
              </small>
            </div>
            <button
              disabled={busy}
              aria-pressed={effect.enabled}
              onClick={() =>
                void onEdit([
                  {
                    op: "effect",
                    command: {
                      kind: "put",
                      sceneId: scene.id,
                      effect: { ...effect, enabled: !effect.enabled },
                    },
                  },
                ])
              }
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
          selected={selected}
          onCancel={() => setImporting(false)}
          onChoose={(effect, fixtures) => {
            const copy = reuseEffect(effect, crypto.randomUUID(), fixtures);
            copy.name = uniqueName(
              `${effect.name} 副本`,
              scene.effects.map((e) => e.name),
            );
            setDialog({ effect: copy, isNew: true });
          }}
        />
      )}
      {dialog && (
        <EffectEditor
          effect={dialog.effect}
          isNew={dialog.isNew}
          sceneId={scene.id}
          fixtures={fixtures}
          selected={selected}
          busy={busy}
          error={error}
          onCancel={() => setDialog(null)}
          onApply={onEdit}
        />
      )}
    </section>
  );
}
