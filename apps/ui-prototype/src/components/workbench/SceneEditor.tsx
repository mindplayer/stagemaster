import { StackIcon } from "@phosphor-icons/react";
import type {
  ProjectView,
  SceneView,
  EditCommand,
} from "../../application-host";
import { SceneParameter } from "./SceneParameter";
export function SceneEditor({
  project,
  selected,
  busy,
  onSelect,
  onEdit,
}: {
  project: ProjectView;
  selected: string;
  busy: boolean;
  onSelect: (scene: SceneView) => void;
  onEdit: (command: EditCommand) => Promise<boolean>;
}) {
  const scene = project.scenes.find((item) => item.id === selected);
  return (
    <>
      <div className="wb-scenes">
        {project.scenes.map((item, i) => (
          <button
            aria-pressed={selected === item.id}
            key={item.id}
            disabled={busy}
            onClick={() => onSelect(item)}
          >
            <span>{String(i + 1).padStart(2, "0")}</span>
            <strong>{item.name}</strong>
            <small>{item.values.length} 项属性</small>
          </button>
        ))}
      </div>
      {!project.scenes.length && (
        <div className="wb-empty">
          <StackIcon size={32} />
          <p>
            {project.fixtures.length
              ? "尚未创建场景"
              : "添加灯具后即可创建场景"}
          </p>
        </div>
      )}
      {scene && (
        <section className="wb-scene-values">
          <h2>{scene.name}</h2>
          {project.fixtures.map((fixture) => (
            <div className="wb-parameter-group" key={fixture.id}>
              <h3>{fixture.name}</h3>
              <div>
                {fixture.attributes.map((attribute) => {
                  const value = scene.values.find(
                    (v) =>
                      v.fixtureId === fixture.id &&
                      v.attribute === attribute.key,
                  );
                  return (
                    <SceneParameter
                      key={`${scene.id}:${fixture.id}:${attribute.key}`}
                      label={attribute.label}
                      value={value?.value ?? 0}
                      mode={value?.mode ?? "none"}
                      preset={value?.presetName ?? null}
                      disabled={busy}
                      onCommit={(amount, mode) =>
                        onEdit({
                          op: "setSceneValue",
                          sceneId: scene.id,
                          fixtureId: fixture.id,
                          attribute: attribute.key,
                          value: amount,
                          mode,
                        })
                      }
                    />
                  );
                })}
              </div>
            </div>
          ))}
        </section>
      )}
    </>
  );
}
