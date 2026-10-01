import {
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
  type CSSProperties,
  type ReactNode,
} from "react";
import {
  DockContext,
  ViewportRevealContext,
  type DockRegion,
  type DockTargets,
} from "./DockPane";
import {
  defaultLayout,
  readLayout,
  layoutTasks,
  type LayoutTask,
} from "./layout-preferences";
import { LayoutActions } from "./LayoutActions";
import { PanelDivider } from "./PanelDivider";
import "./performance-layout.css";
import "./performance-panels.css";
import "./focused-layout.css";

function Target({
  target,
  region,
  visible = true,
  label,
}: {
  target: HTMLElement;
  region: DockRegion;
  visible?: boolean;
  label: string;
}) {
  const slot = useRef<HTMLDivElement>(null);
  const attached = visible || region === "viewport";
  useLayoutEffect(() => {
    if (attached) slot.current?.appendChild(target);
    return () => target.remove();
  }, [target, attached]);
  return (
    <section
      className={`editor-region region-${region}`}
      aria-label={label}
      hidden={!visible}
      inert={!visible}
      ref={slot}
    />
  );
}

export function PerformanceLayout({
  mode,
  toolbar,
  children,
  beforeChange,
  busy = false,
  revealInspector,
  revealEditing,
}: {
  revealInspector?: string;
  revealEditing?: number;
  mode: string;
  toolbar: ReactNode;
  children: ReactNode;
  beforeChange?(): Promise<boolean>;
  busy?: boolean;
}) {
  const [targets] = useState(() => {
    const make = () => {
      const node = document.createElement("div");
      node.className = "dock-target";
      return node;
    };
    return {
      library: make(),
      viewport: make(),
      inspector: make(),
      editor: make(),
      full: make(),
    } satisfies DockTargets;
  });
  const [layout, setLayout] = useState(() => {
    try {
      return readLayout(localStorage.getItem("stagemaster.layout.v1"));
    } catch {
      return { ...defaultLayout, focusedTasks: [] };
    }
  });
  useEffect(() => {
    const timer = setTimeout(() => {
      try {
        localStorage.setItem("stagemaster.layout.v1", JSON.stringify(layout));
      } catch {
        /* Optional UI preference. */
      }
    }, 250);
    return () => clearTimeout(timer);
  }, [layout]);
  useEffect(() => {
    if (revealInspector) setLayout((v) => ({ ...v, showInspector: true }));
  }, [revealInspector]);
  useLayoutEffect(() => {
    if (revealEditing)
      setLayout((v) => ({ ...v, showLibrary: true, showInspector: true }));
  }, [revealEditing]);
  async function toggle(side: "showLibrary" | "showInspector") {
    if (!layout[side] || !beforeChange || (await beforeChange()))
      setLayout((v) => ({ ...v, [side]: !v[side] }));
  }
  const full = ["fixtures", "profiles", "settings", "execution"].includes(mode);
  const lower = layoutTasks.includes(mode as LayoutTask);
  const focused = lower && layout.focusedTasks.includes(mode as LayoutTask);
  function revealViewport() {
    setLayout((v) =>
      v.focusedTasks.includes(mode as LayoutTask)
        ? {
            ...v,
            focusedTasks: v.focusedTasks.filter((task) => task !== mode),
          }
        : v,
    );
  }
  async function toggleFocus() {
    if (!lower || (beforeChange && !(await beforeChange()))) return;
    setLayout((v) => ({
      ...v,
      focusedTasks: v.focusedTasks.includes(mode as LayoutTask)
        ? v.focusedTasks.filter((task) => task !== mode)
        : [...v.focusedTasks, mode as LayoutTask],
    }));
  }
  return (
    <DockContext.Provider value={targets}>
      <ViewportRevealContext.Provider value={revealViewport}>
        <div className="editor-navigation">
          {toolbar}
          <LayoutActions
            full={full}
            lower={lower}
            focused={focused}
            busy={busy}
            library={layout.showLibrary}
            inspector={layout.showInspector}
            onFocus={() => void toggleFocus()}
            onToggle={(side) => void toggle(side)}
            onRestore={() => setLayout({ ...defaultLayout, focusedTasks: [] })}
          />
        </div>
        <div
          className="performance-layout"
          data-mode={mode}
          data-full={full}
          data-lower={lower}
          data-focus={focused ? "editor" : "balanced"}
          data-library={layout.showLibrary}
          data-inspector={layout.showInspector}
          style={
            {
              "--library-width": `${layout.library}px`,
              "--inspector-width": `${layout.inspector}px`,
              "--editor-height": `${layout.editor}px`,
            } as CSSProperties
          }
        >
          <Target
            region="library"
            label="资源区"
            target={targets.library}
            visible={!full && layout.showLibrary}
          />
          <Target
            region="viewport"
            label="舞台画布"
            target={targets.viewport}
            visible={!full}
          />
          <Target
            region="inspector"
            label="属性区"
            target={targets.inspector}
            visible={!full && layout.showInspector}
          />
          <Target
            region="editor"
            label="编排区"
            target={targets.editor}
            visible={!full && lower}
          />
          <Target
            region="full"
            label={mode === "execution" ? "执行工作区" : "管理工作区"}
            target={targets.full}
            visible={full}
          />
          {!full && layout.showLibrary && (
            <PanelDivider
              panel="library"
              value={layout.library}
              onChange={(library) => setLayout((v) => ({ ...v, library }))}
            />
          )}
          {!full && layout.showInspector && (
            <PanelDivider
              panel="inspector"
              value={layout.inspector}
              onChange={(inspector) => setLayout((v) => ({ ...v, inspector }))}
            />
          )}
          {!full && lower && !focused && (
            <PanelDivider
              panel="editor"
              value={layout.editor}
              onChange={(editor) => setLayout((v) => ({ ...v, editor }))}
            />
          )}
          <div className="editor-module-roots">{children}</div>
        </div>
      </ViewportRevealContext.Provider>
    </DockContext.Provider>
  );
}
