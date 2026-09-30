import { useState, type ReactNode } from "react";
import { WorkspaceSurface } from "./WorkspaceSurface";
import "./scene-inspector.css";

/** Each editor keeps its own draft; switching uses the workbench transaction guard. */
export function SceneInspector({
  active,
  busy,
  hasPosition,
  beforeChange,
  light,
  position,
  scene,
}: {
  active: boolean;
  busy: boolean;
  hasPosition: boolean;
  beforeChange(): Promise<boolean>;
  light: ReactNode;
  position: ReactNode;
  scene: ReactNode;
}) {
  const [tab, setTab] = useState("light");
  if (!active)
    return (
      <>
        {light}
        {position}
        {scene}
      </>
    );
  const panels = [
    { id: "light", name: "灯光", content: light },
    {
      id: "position",
      name: "位置",
      content: hasPosition ? (
        position
      ) : (
        <p className="wb-dim">选择摇头灯后编辑灯头位置</p>
      ),
    },
    { id: "scene", name: "场景", content: scene },
  ];
  return (
    <div className="scene-inspector">
      <nav aria-label="属性分类">
        {panels.map((p) => (
          <button
            key={p.id}
            disabled={busy}
            aria-pressed={tab === p.id}
            onClick={async () => {
              if (tab !== p.id && (await beforeChange())) setTab(p.id);
            }}
          >
            {p.name}
          </button>
        ))}
      </nav>
      {panels.map((p) => (
        <WorkspaceSurface
          key={p.id}
          visible={tab === p.id}
          className="scene-inspector-panel"
          label={`${p.name}属性`}
        >
          {p.content}
        </WorkspaceSurface>
      ))}
    </div>
  );
}
