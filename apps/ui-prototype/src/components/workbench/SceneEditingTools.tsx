import { useState, type ReactNode } from "react";
import { StackIcon } from "@phosphor-icons/react";
import type {
  ApplicationHost,
  EditCommand,
  ProjectView,
  SceneView,
} from "../../application-host";
import type { LibraryEdit } from "../../library-types";
import type { RecallMode } from "../../library-tools";
import { SceneEditorTools } from "../layout/SceneEditorTools";
import { SceneSelectionBar } from "./SceneSelectionBar";
import { FixtureBrowser } from "./FixtureBrowser";
import type { OpenEffect } from "./useEffectSelection";
import { EffectRack } from "./EffectRack";
import { PreviewPanel } from "./PreviewPanel";
import { QuickPresets } from "./QuickPresets";
import { ResourcePool } from "./ResourcePool";

export function SceneEditingTools({
  host,
  project,
  scene,
  selected,
  fixtureQuery,
  onlySelected,
  generation,
  visible,
  busy,
  error,
  beforeChange,
  onQuery,
  onFilter,
  onSelect,
  onEdit,
  onLibraryEdit,
  onView3d,
  onAddScene,
  onAddFixtures,
  context,
  onOpenEffect,
  onToggleEffect,
  captureTemplate,
  onApplyTemplate,
}: {
  onOpenEffect: OpenEffect;
  captureTemplate(): Promise<number | null>;
  onApplyTemplate(generation: number, token: string): Promise<boolean>;
  onToggleEffect(id: string, enabled: boolean): Promise<boolean>;
  context?: ReactNode;
  host: ApplicationHost;
  project: ProjectView;
  scene?: SceneView;
  selected: string[];
  fixtureQuery: string;
  onlySelected: boolean;
  generation: number;
  visible: boolean;
  busy: boolean;
  error: string;
  beforeChange(): Promise<boolean>;
  onQuery(value: string): void;
  onFilter(value: boolean): void;
  onSelect(ids: string[]): Promise<boolean>;
  onEdit(command: EditCommand): Promise<boolean>;
  onLibraryEdit(command: LibraryEdit): Promise<ProjectView | null>;
  onView3d(): void;
  onAddScene(): void;
  onAddFixtures(): void;
}) {
  const [tool, setTool] = useState("fixtures");
  const [presetId, setPresetId] = useState("");
  const [mask, setMask] = useState<string[] | null>(null);
  const [recall, setRecall] = useState<RecallMode>("replace");
  return (
    <SceneEditorTools
      visible={visible}
      onToolChange={setTool}
      selection={
        <>
          {context}
          <SceneSelectionBar
            project={project}
            selected={selected}
            busy={busy}
            recall={recall}
            onRecall={setRecall}
            onSelect={onSelect}
          />
          <QuickPresets
            project={project}
            scene={scene}
            selected={selected}
            busy={busy}
            presetId={presetId}
            mask={mask}
            onPreset={setPresetId}
            onMask={setMask}
            onEdit={onLibraryEdit}
          />
        </>
      }
      busy={busy}
      beforeChange={beforeChange}
      fixtures={
        scene ? (
          <FixtureBrowser
            fixtures={project.fixtures}
            selected={selected}
            scene={scene}
            query={fixtureQuery}
            onlySelected={onlySelected}
            busy={busy}
            onQuery={onQuery}
            onFilter={onFilter}
            onSelect={(ids) => void onSelect(ids)}
          />
        ) : (
          <div className="wb-empty">
            <StackIcon size={40} />
            <h2>{project.scenes.length ? "选择一个场景" : "创建第一个场景"}</h2>
            {project.fixtures.length ? (
              <button
                className="wb-primary"
                disabled={busy}
                onClick={onAddScene}
              >
                新建场景
              </button>
            ) : (
              <button disabled={busy} onClick={onAddFixtures}>
                添加灯具
              </button>
            )}
          </div>
        )
      }
      effects={
        scene && (
          <EffectRack
            key={scene.id}
            files={{
              host,
              generation,
              visible: visible && tool === "effects",
              capture: captureTemplate,
              onApply: onApplyTemplate,
            }}
            scene={scene}
            fixtures={project.fixtures}
            scenes={project.scenes}
            selected={selected}
            busy={busy}
            error={error}
            onOpen={onOpenEffect}
            onToggle={onToggleEffect}
            beforeChange={beforeChange}
            onEdit={(commands) => onEdit({ op: "batch", commands })}
          />
        )
      }
      preview={
        scene && (
          <PreviewPanel
            host={host}
            scene={scene}
            stepId={scene.id}
            generation={generation}
            busy={busy}
            beforeAction={beforeChange}
            visible={visible}
            onView3d={onView3d}
          />
        )
      }
      resources={
        <ResourcePool
          key={project.id}
          project={project}
          scene={scene}
          selected={selected}
          busy={busy}
          error={error}
          visible={visible}
          beforeChange={beforeChange}
          recall={recall}
          onRecall={setRecall}
          presetId={presetId}
          onPreset={setPresetId}
          mask={mask}
          onMask={setMask}
          onEdit={onLibraryEdit}
          onSelect={onSelect}
        />
      }
    />
  );
}
