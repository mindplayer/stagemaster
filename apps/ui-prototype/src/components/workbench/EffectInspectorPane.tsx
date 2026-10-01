import type { ReactNode, Ref } from "react";
import type { FixtureView } from "../../application-host";
import { EffectEditor, type EffectHandle } from "./EffectEditor";
import { PositionEffectEditor } from "./PositionEffectEditor";
import type { EffectSelection } from "./useEffectSelection";
import { WorkspaceSurface } from "./WorkspaceSurface";

export function EffectInspectorPane({
  selection,
  editor,
  fixtures,
  placements,
  selected,
  busy,
  error,
  onCancel,
  onPending,
  onApply,
  onPreview,
  audition,
  onDraftChange,
  children,
}: {
  selection: EffectSelection | null;
  editor: Ref<EffectHandle>;
  fixtures: FixtureView[];
  placements: import("../../stage-types").FixturePlacement[];
  selected: string[];
  busy: boolean;
  error: string;
  onCancel(): void;
  onPending(pending: boolean): void;
  onApply(): Promise<boolean>;
  onPreview(): Promise<boolean>;
  audition?: import("./useEffectDraftPreview").EffectAuditionControls;
  onDraftChange?(): void;
  children: ReactNode;
}) {
  const Editor =
    selection?.effect.waveform === "position"
      ? PositionEffectEditor
      : EffectEditor;
  return (
    <>
      {selection && (
        <Editor
          key={selection.token}
          ref={editor}
          effect={selection.effect}
          sceneId={selection.sceneId}
          isNew={selection.isNew}
          fixtures={fixtures}
          placements={placements}
          selected={selected}
          busy={busy}
          error={error}
          onCancel={onCancel}
          onPending={onPending}
          onApply={onApply}
          onPreview={onPreview}
          audition={audition}
          onDraftChange={onDraftChange}
        />
      )}
      <WorkspaceSurface
        visible={!selection}
        className="scene-base-inspector"
        label="场景常用属性"
      >
        {children}
      </WorkspaceSurface>
    </>
  );
}
