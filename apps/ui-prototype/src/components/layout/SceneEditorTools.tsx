import { useState, type ReactNode } from "react";
import { WorkspaceSurface } from "../workbench/WorkspaceSurface";
import "./scene-editor-tools.css";

export function SceneEditorTools({
  visible,
  busy,
  beforeChange,
  fixtures,
  effects,
  resources,
  preview,
}: {
  visible: boolean;
  busy: boolean;
  beforeChange(): Promise<boolean>;
  fixtures: ReactNode;
  effects: ReactNode;
  resources: ReactNode;
  preview: ReactNode;
}) {
  const [tool, setTool] = useState("fixtures");
  const items = [
    { id: "fixtures", name: "选择灯具", content: fixtures },
    { id: "effects", name: "动态效果", content: effects },
    { id: "resources", name: "灯组与预设", content: resources },
  ];
  return (
    <WorkspaceSurface
      visible={visible}
      className="scene-editor-tools"
      label="场景编辑工具"
    >
      <div className="scene-tools-main">
        <nav aria-label="场景工具">
          {items.map((item) => (
            <button
              key={item.id}
              disabled={busy}
              aria-pressed={tool === item.id}
              onClick={() => {
                void beforeChange().then((ok) => {
                  if (ok) setTool(item.id);
                });
              }}
            >
              {item.name}
            </button>
          ))}
        </nav>
        {items.map((item) => (
          <WorkspaceSurface
            key={item.id}
            visible={tool === item.id}
            className="scene-tool-panel"
            label={item.name}
          >
            {item.content}
          </WorkspaceSurface>
        ))}
      </div>
      {preview}
    </WorkspaceSurface>
  );
}
