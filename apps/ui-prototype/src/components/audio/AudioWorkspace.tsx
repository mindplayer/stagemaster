import { useRevealItem, type RevealItem } from "../layout/useRevealItem";
import { AudioClipGroupFadeInspector } from "./AudioClipGroupFadeInspector";
import { groupFadeDraft } from "./clip-group-fade";
import { clipMotionCommand } from "./clip-trim-tools";
import { useClipSelection } from "./useClipSelection";
import { clipsInRange } from "./clip-selection";
import { AudioResourceHeader } from "./AudioResourceHeader";
import { AudioClipLibrary } from "./AudioClipLibrary";
import { useAudioClipActions } from "./useAudioClipActions";
import { AudioLoopControls } from "./AudioLoopControls";
import { DockPane } from "../layout/DockPane";
import { AudioMarkerLibrary } from "./AudioMarkerBatch";
import { AudioTransportBar } from "./AudioTransportBar";
import { forwardRef, useEffect, useImperativeHandle, useState } from "react";
import type {
  ApplicationHost,
  EditCommand,
  EditOperation,
  ProjectView,
} from "../../application-host";
import type { AudioEdit, AudioMarker } from "../../audio-types";
import { validateMarker } from "../../audio-tools";
import { DeleteDialog } from "../workbench/DeleteDialog";
import { WorkspaceSurface } from "../workbench/WorkspaceSurface";
import { AudioInspector } from "./AudioInspector";
import { useAudioWorkspaceDraft } from "./useAudioWorkspaceDraft";
import { AudioWaveform } from "./AudioWaveform";
import type { useAudio } from "./useAudio";
import { useMarkerActions } from "./useMarkerActions";
import "./audio.css";
export interface AudioHandle {
  collect(): EditOperation[];
  accept(): void;
  reveal(kind: "clip" | "marker", id: string): boolean;
}
export const AudioWorkspace = forwardRef<
  AudioHandle,
  {
    project: ProjectView;
    host: ApplicationHost;
    generation: () => number;
    session: ReturnType<typeof useAudio>;
    visible: boolean;
    sharedTransport?: boolean;
    busy: boolean;
    onEdit(command: EditCommand): Promise<ProjectView | null>;
    beforeChange(): Promise<boolean>;
    onPending(value: boolean): void;
    onEditScene?(markerId: string): Promise<boolean>;
    onView3d?(): void;
  }
>(function AudioWorkspace(
  {
    project,
    host,
    generation,
    session: audio,
    visible,
    sharedTransport = false,
    busy,
    onEdit,
    beforeChange,
    onPending,
    onEditScene,
    onView3d,
  },
  ref,
) {
  const [revealRequest, setRevealRequest] = useState<RevealItem | null>(null);
  const track = project.audio;
  const trackIdentity = track
    ? `${project.id}:${track.asset.digest}:${track.inMs}:${track.outMs}`
    : "";
  const [batchKey, setBatchKey] = useState("");
  const [clipGroupPending, setClipGroupPending] = useState(false);
  const batch = !!trackIdentity && batchKey === `${trackIdentity}:markers`;
  const clipBatch =
    !!track?.lightingClips && batchKey === `${trackIdentity}:clips`;
  useEffect(() => setBatchKey(""), [trackIdentity]);
  const [selected, setSelected] = useState("");
  const clipSelection = useClipSelection(trackIdentity, track?.lightingClips);
  const [query, setQuery] = useState("");
  const [problem, setProblem] = useState("");
  const { draft, form, change, cancel, collect } = useAudioWorkspaceDraft(
    track,
    onPending,
    setProblem,
    setSelected,
  );
  const [removeMusic, setRemoveMusic] = useState(false);
  const marker = track?.markers.find((m) => m.id === selected);
  const markerActions = useMarkerActions({
    project,
    selected,
    host,
    generation,
    audio,
    beforeChange,
    onView3d,
  });
  const clips = useAudioClipActions({
    track,
    selected,
    position: audio.position.positionMs,
    readPosition: async () =>
      (await host.audio(generation(), { kind: "snapshot" })).positionMs,
    scenes: project.scenes,
    beforeChange,
    edit,
    onDraft: change,
    onSelect: (id) => {
      setSelected(id);
      setBatchKey("");
    },
    onProblem: setProblem,
  });
  const blocked = busy || audio.preparing || markerActions.acting;
  const revealRoot = useRevealItem(revealRequest, visible, blocked);
  useImperativeHandle(ref, () => ({
    collect,
    accept: cancel,
    reveal(kind, id) {
      const items = kind === "clip" ? track?.lightingClips : track?.markers;
      if (!items?.some((item) => item.id === id)) return false;
      cancel();
      setSelected(id);
      setBatchKey("");
      setQuery("");
      setRevealRequest((previous) => ({
        id,
        serial: (previous?.serial ?? 0) + 1,
      }));
      return true;
    },
  }));
  async function edit(command: AudioEdit) {
    return onEdit({ op: "audio", command });
  }
  async function choose(id: string) {
    if (await beforeChange()) {
      setSelected(id);
      setBatchKey("");
      cancel();
    }
  }
  async function addMarker() {
    if (!track || blocked) return;
    if (!(await beforeChange())) return;
    try {
      const state = await host.audio(generation(), { kind: "snapshot" });
      const next = {
        id: crypto.randomUUID(),
        name: `卡点 ${track.markers.length + 1}`,
        timeMs: Math.min(track.outMs - track.inMs - 1, state.positionMs),
        sceneId: null,
      };
      validateMarker(next, track);
      if (await edit({ kind: "putMarker", marker: next })) {
        setSelected(next.id);
        setBatchKey("");
        setQuery("");
      }
    } catch (error) {
      setProblem(error instanceof Error ? error.message : String(error));
    }
  }
  async function importMusic() {
    if (!(await beforeChange())) return;
    const result = await audio.prepare("import");
    if (result) await edit({ kind: "setAsset", asset: result.asset });
  }
  async function moveMarker(next: AudioMarker) {
    if (!track) return;
    try {
      validateMarker(next, track);
      await edit({ kind: "putMarker", marker: next });
    } catch (error) {
      setProblem(error instanceof Error ? error.message : String(error));
    }
  }
  const seek = (time: number) => {
    void audio.command({ kind: "seek", positionMs: time });
  };
  return (
    <WorkspaceSurface
      visible={visible}
      className="audio-workspace"
      label="音频卡点"
    >
      <div
        className="audio-shell"
        onKeyDown={(e) => {
          if (
            e.target instanceof HTMLElement &&
            e.target.closest(
              "input,select,textarea,button,[contenteditable=true]",
            )
          )
            return;
          if (blocked || removeMusic || e.repeat) return;
          if (e.code === "Space") {
            e.preventDefault();
            void audio.command({
              kind: audio.position.playing ? "pause" : "play",
            });
          }
          if (e.key.toLowerCase() === "m" && !e.metaKey && !e.ctrlKey) {
            e.preventDefault();
            void addMarker();
          }
        }}
      >
        <DockPane region="library" visible={visible}>
          <section ref={revealRoot} className="audio-resources">
            <AudioResourceHeader
              track={track}
              blocked={blocked}
              host={host}
              choose={choose}
              locate={() => void audio.prepare("locate")}
              remove={() => setRemoveMusic(true)}
              importMusic={importMusic}
              problem={
                problem ||
                markerActions.problem ||
                audio.problem ||
                audio.position.problem
              }
              preparing={audio.preparing}
              cancelPrepare={() => void audio.cancel()}
            />
            {track && (
              <AudioClipLibrary
                revealRequest={revealRequest}
                key={trackIdentity}
                selectionState={clipSelection}
                onGroupPending={setClipGroupPending}
                groupEditor={{
                  beforeChange,
                  begin: () => {
                    try {
                      change(groupFadeDraft(track, clipSelection.ids));
                    } catch (error) {
                      setProblem(
                        error instanceof Error ? error.message : String(error),
                      );
                    }
                  },
                  content:
                    draft?.kind === "clipGroupFade" ? (
                      <AudioClipGroupFadeInspector
                        draft={draft}
                        track={track}
                        form={form}
                        busy={blocked}
                        problem={problem}
                        onChange={change}
                        onApply={() => void beforeChange()}
                        onCancel={cancel}
                      />
                    ) : null,
                }}
                batch={clipBatch}
                visible={visible}
                onEdit={edit}
                onBatch={async () => {
                  if (await beforeChange()) {
                    if (!clipBatch && !clipSelection.ids.length)
                      clipSelection.replace([selected]);
                    setBatchKey(clipBatch ? "" : `${trackIdentity}:clips`);
                  }
                }}
                track={track}
                scenes={project.scenes}
                selected={selected}
                busy={blocked}
                onSelect={choose}
                onSeek={seek}
                onAdd={() => void clips.add()}
                onConvert={() => void clips.convert()}
              />
            )}
            {track && (
              <AudioMarkerLibrary
                key={trackIdentity}
                batch={batch}
                onBatch={(value) =>
                  setBatchKey(value ? `${trackIdentity}:markers` : "")
                }
                workspaceVisible={visible}
                onEdit={edit}
                beforeChange={beforeChange}
                track={track}
                scenes={project.scenes}
                selected={selected}
                query={query}
                busy={blocked}
                onQuery={setQuery}
                onSelect={choose}
                onSeek={seek}
              />
            )}
          </section>
        </DockPane>
        <DockPane region="editor" visible={visible}>
          <section className="audio-sequencer">
            {!track ? (
              <div className="audio-empty">
                <strong>尚未添加音乐</strong>
                <p>导入音乐后，在这里查看波形和安排卡点。</p>
              </div>
            ) : (
              <>
                <AudioTransportBar
                  position={audio.position}
                  requestedPosition={audio.requestedPosition}
                  requestedVolume={audio.requestedVolume}
                  command={audio.command}
                  ready={!!audio.waveform}
                  duration={track.outMs - track.inMs}
                  markerCount={track.markers.length}
                  blocked={blocked}
                  addMarker={addMarker}
                  editingOnly={sharedTransport}
                />
                <AudioLoopControls
                  key={trackIdentity}
                  track={track}
                  selected={batch || clipBatch ? "" : selected}
                  position={audio.position}
                  disabled={
                    blocked ||
                    !audio.waveform ||
                    audio.position.durationMs !== track.outMs - track.inMs
                  }
                  configure={audio.configureLoop}
                />
                <AudioWaveform
                  track={track}
                  scenes={project.scenes}
                  loopRange={audio.position.loopRange}
                  compact={sharedTransport}
                  waveform={audio.waveform}
                  sample={audio.playingSample}
                  requestedPosition={audio.requestedPosition}
                  selected={batch || clipBatch ? "" : selected}
                  disabled={blocked || !!draft || !visible}
                  onSeek={seek}
                  onSelect={(id) => {
                    setSelected(id);
                    setBatchKey("");
                    cancel();
                  }}
                  onMove={moveMarker}
                  clipSelection={{
                    active: clipBatch,
                    movementBlocked: clipGroupPending
                      ? "请先应用或取消右侧目标输入／删除确认"
                      : "",
                    ids: clipSelection.ids,
                    onMode: () => {
                      if (!clipBatch && !clipSelection.ids.length)
                        clipSelection.replace([selected]);
                      setBatchKey(clipBatch ? "" : `${trackIdentity}:clips`);
                    },
                    onPick: (id, range) =>
                      clipSelection.toggle(
                        id,
                        track.lightingClips ?? [],
                        range,
                      ),
                    onRange: (start, end, append) =>
                      clipSelection.replace([
                        ...(append ? clipSelection.ids : []),
                        ...clipsInRange(track.lightingClips ?? [], start, end),
                      ]),
                    onClear: () => clipSelection.replace([]),
                    onMove: (ids, destinationMs) =>
                      void edit({
                        kind: "editLightingClips",
                        ids,
                        action: { kind: "move", destinationMs },
                      }),
                  }}
                  onClipMove={(clip, mode) =>
                    void edit(clipMotionCommand(clip, mode))
                  }
                />
              </>
            )}
          </section>
        </DockPane>
        <DockPane region="inspector" visible={visible && !batch && !clipBatch}>
          {track ? (
            <AudioInspector
              track={track}
              marker={marker}
              clip={clips.clip}
              splitActions={clips.splitActions}
              onCopy={clips.copy}
              onLock={() => void clips.lock()}
              onEnabled={() => void clips.enabled()}
              draft={draft}
              scenes={project.scenes}
              busy={blocked}
              form={form}
              ready={
                !!audio.waveform &&
                audio.position.durationMs === track.outMs - track.inMs
              }
              onPreview={() => void markerActions.preview()}
              onEditScene={() => {
                if (selected) void onEditScene?.(selected);
              }}
              onChange={change}
              onApply={() => void beforeChange()}
              onCancel={cancel}
              onRemove={() => {
                if (clips.clip) clips.requestRemove();
                else if (marker)
                  void edit({ kind: "removeMarker", id: marker.id });
              }}
            />
          ) : (
            <div className="editor-empty-properties">
              选择音乐或卡点查看属性
            </div>
          )}
        </DockPane>
        {clips.removing && (
          <DeleteDialog
            name={clips.removing.name}
            description="保留相邻片段和节奏标记，删除处成为默认值空隙；可以撤销。"
            busy={blocked}
            onCancel={clips.cancelRemove}
            onDelete={() => void clips.remove()}
          />
        )}
        {removeMusic && (
          <DeleteDialog
            name="音乐、卡点及灯光片段"
            description="灯光场景会保留，此操作可以撤销。"
            busy={blocked}
            onCancel={() => setRemoveMusic(false)}
            onDelete={async () => {
              if (await edit({ kind: "clear" })) {
                setRemoveMusic(false);
                setSelected("");
                cancel();
              }
            }}
          />
        )}
      </div>
    </WorkspaceSurface>
  );
});
