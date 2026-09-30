import { DockPane } from "../layout/DockPane";
import { AudioMarkerList } from "./AudioMarkerList";
import { AudioTransportBar } from "./AudioTransportBar";
import { forwardRef, useImperativeHandle, useRef, useState } from "react";
import type {
  ApplicationHost,
  EditCommand,
  EditOperation,
  ProjectView,
} from "../../application-host";
import type { AudioEdit, AudioMarker } from "../../audio-types";
import { audioMilliseconds, validateMarker } from "../../audio-tools";
import { DeleteDialog } from "../workbench/DeleteDialog";
import { WorkspaceSurface } from "../workbench/WorkspaceSurface";
import { AudioInspector, type AudioDraft } from "./AudioInspector";
import { AudioWaveform } from "./AudioWaveform";
import type { useAudio } from "./useAudio";
import { useMarkerActions } from "./useMarkerActions";
import "./audio.css";
export interface AudioHandle {
  collect(): EditOperation[];
  accept(): void;
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
  const track = project.audio;
  const [selected, setSelected] = useState("");
  const [query, setQuery] = useState("");
  const [problem, setProblem] = useState("");
  const [draft, setDraft] = useState<AudioDraft | null>(null);
  const draftRef = useRef<AudioDraft | null>(null);
  const form = useRef<HTMLFormElement>(null);
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
  const blocked = busy || audio.preparing || markerActions.acting;
  function change(value: AudioDraft) {
    draftRef.current = value;
    setDraft(value);
    onPending(true);
    setProblem("");
  }
  function cancel() {
    draftRef.current = null;
    setDraft(null);
    onPending(false);
    setProblem("");
  }
  function collect(): EditOperation[] {
    const value = draftRef.current;
    if (!value || !track) return [];
    let field = value.kind === "marker" ? "markerName" : "trimStart";
    try {
      if (!form.current?.reportValidity())
        throw new Error("请修正音频属性中的输入");
      if (value.kind === "marker") {
        if (!value.name.trim()) throw new Error("卡点名称不能为空");
        field = "markerTime";
        const next = validateMarker(
          {
            id: value.id,
            name: value.name.trim(),
            timeMs: audioMilliseconds(value.time, "卡点时间"),
            sceneId: value.sceneId || null,
          },
          track,
        );
        return [{ op: "audio", command: { kind: "putMarker", marker: next } }];
      }
      const inMs = audioMilliseconds(value.start, "裁切开始");
      field = "trimEnd";
      const outMs = audioMilliseconds(value.end, "裁切结束");
      if (inMs >= outMs || outMs > track.asset.durationMs)
        throw new Error("裁切范围必须在源文件内，结束晚于开始");
      if (track.markers.some((m) => m.timeMs >= outMs - inMs))
        throw new Error("裁切后部分卡点超出音乐，请先移动或删除这些卡点");
      return [{ op: "audio", command: { kind: "trim", inMs, outMs } }];
    } catch (error) {
      setProblem(error instanceof Error ? error.message : String(error));
      requestAnimationFrame(() =>
        form.current
          ?.querySelector<HTMLInputElement>(`[name="${field}"]`)
          ?.focus(),
      );
      throw error;
    }
  }
  useImperativeHandle(ref, () => ({ collect, accept: cancel }));
  async function edit(command: AudioEdit) {
    return onEdit({ op: "audio", command });
  }
  async function choose(id: string) {
    if (await beforeChange()) {
      setSelected(id);
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
            e.target.closest("input,select,textarea,[contenteditable=true]")
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
          <section className="audio-resources">
            <header className="audio-header">
              <div>
                <h2>音乐与卡点</h2>
                <span>{track?.asset.fileName ?? "用音乐安排灯光节奏"}</span>
              </div>
              <div className="wb-actions">
                {track ? (
                  <>
                    <button disabled={blocked} onClick={() => choose("")}>
                      裁切范围
                    </button>
                    <button
                      disabled={blocked}
                      onClick={() => audio.prepare("locate")}
                    >
                      重新定位音乐
                    </button>
                    <button
                      disabled={blocked}
                      onClick={() => setRemoveMusic(true)}
                    >
                      移除音乐
                    </button>
                  </>
                ) : (
                  <button
                    className="primary"
                    disabled={blocked || host.kind !== "desktop"}
                    onClick={importMusic}
                  >
                    导入音乐
                  </button>
                )}
              </div>
            </header>
            {(problem ||
              markerActions.problem ||
              audio.problem ||
              audio.position.problem) && (
              <div role="alert" className="audio-error">
                {problem ||
                  markerActions.problem ||
                  audio.problem ||
                  audio.position.problem}
              </div>
            )}
            {audio.preparing && (
              <div role="status" className="audio-progress">
                正在准备音乐波形…
                <button onClick={() => audio.cancel()}>取消准备</button>
              </div>
            )}
            {track && (
              <AudioMarkerList
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
                  command={audio.command}
                  ready={!!audio.waveform}
                  duration={track.outMs - track.inMs}
                  markerCount={track.markers.length}
                  blocked={blocked}
                  addMarker={addMarker}
                  editingOnly={sharedTransport}
                />
                <AudioWaveform
                  track={track}
                  scenes={project.scenes}
                  compact={sharedTransport}
                  waveform={audio.waveform}
                  sample={audio.playingSample}
                  selected={selected}
                  disabled={blocked || !!draft}
                  onSeek={seek}
                  onSelect={(id) => {
                    setSelected(id);
                    cancel();
                  }}
                  onMove={moveMarker}
                />
              </>
            )}
          </section>
        </DockPane>
        <DockPane region="inspector" visible={visible}>
          {track ? (
            <AudioInspector
              track={track}
              marker={marker}
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
                if (marker) void onEditScene?.(marker.id);
              }}
              onChange={change}
              onApply={() => void beforeChange()}
              onCancel={cancel}
              onRemove={() => {
                if (marker) void edit({ kind: "removeMarker", id: marker.id });
              }}
            />
          ) : (
            <div className="editor-empty-properties">
              选择音乐或卡点查看属性
            </div>
          )}
        </DockPane>
        {removeMusic && (
          <DeleteDialog
            name="音乐及全部卡点"
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
