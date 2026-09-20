import { test } from "node:test";
import assert from "node:assert/strict";
import {
  createDemoState,
  demoReducer,
  hasChanges,
  formatTime,
} from "../src/demo-session.ts";

test("initial editing, running and next Cue have separate identities", () => {
  const state = createDemoState();
  assert.equal(state.editCueId, 14);
  assert.equal(state.running?.cueId, 12);
  assert.equal(state.nextId, 13);
  assert.equal(hasChanges(state), false);
});
test("editing and saving do not mutate active snapshot, next Cue or other saved cues", () => {
  const initial = createDemoState();
  const running = structuredClone(initial.running);
  let state = demoReducer(initial, {
    type: "patchClip",
    id: "back",
    patch: { intensity: 18, color: "pink" },
  });
  assert.equal(hasChanges(state), true);
  assert.equal(initial.drafts[14].clips[1].intensity, 65);
  assert.equal(state.saved[14].clips[1].intensity, 65);
  state = demoReducer(state, { type: "save" });
  assert.equal(state.saved[14].clips[1].intensity, 18);
  assert.deepEqual(state.running, running);
  assert.equal(state.nextId, 13);
  assert.deepEqual(state.saved[12], initial.saved[12]);
  assert.equal(hasChanges(state), false);
});
test("GO uses saved content, does not consume unsaved draft, and keeps editing selection", () => {
  let state = createDemoState();
  state = demoReducer(state, {
    type: "patchClip",
    id: "back",
    patch: { intensity: 9 },
  });
  state = demoReducer(state, { type: "standby", id: 14 });
  state = demoReducer(state, { type: "go" });
  assert.equal(state.running?.cueId, 14);
  assert.equal(state.running?.snapshot.clips[1].intensity, 65);
  assert.equal(state.editCueId, 14);
  assert.equal(state.selectedClipId, "back");
  assert.equal(hasChanges(state), true);
  state = demoReducer(state, { type: "save" });
  assert.equal(state.running?.snapshot.clips[1].intensity, 65);
  assert.equal(state.saved[14].clips[1].intensity, 9);
});
test("undo changes drafts only; running and saved revisions remain intact", () => {
  let state = createDemoState();
  state = demoReducer(state, {
    type: "patchClip",
    id: "back",
    patch: { intensity: 31 },
  });
  state = demoReducer(state, { type: "save" });
  state = demoReducer(state, { type: "go" });
  state = demoReducer(state, { type: "undo" });
  assert.equal(state.drafts[14].clips[1].intensity, 65);
  assert.equal(state.saved[14].clips[1].intensity, 31);
  assert.equal(state.running?.cueId, 13);
  assert.equal(state.nextId, 14);
  assert.equal(hasChanges(state), true);
  state = demoReducer(state, { type: "redo" });
  assert.equal(state.drafts[14].clips[1].intensity, 31);
  assert.equal(state.running?.cueId, 13);
});
test("switching Cue retains unsaved drafts without changing standby", () => {
  let state = createDemoState();
  state = demoReducer(state, {
    type: "patchClip",
    id: "back",
    patch: { intensity: 52 },
  });
  state = demoReducer(state, { type: "selectCue", id: 11 });
  state = demoReducer(state, { type: "selectCue", id: 14 });
  assert.equal(state.drafts[14].clips[1].intensity, 52);
  assert.equal(hasChanges(state), true);
  assert.equal(state.nextId, 13);
  assert.equal(state.running?.cueId, 12);
});
test("clip editing bounds duration, position and fades, and rejects nonfinite values", () => {
  const state = createDemoState();
  assert.equal(
    demoReducer(state, {
      type: "patchClip",
      id: "back",
      patch: { start: NaN },
    }),
    state,
  );
  assert.equal(
    demoReducer(state, {
      type: "patchClip",
      id: "back",
      patch: { intensity: Infinity },
    }),
    state,
  );
  let changed = demoReducer(state, {
    type: "patchClip",
    id: "back",
    patch: { start: 50, intensity: -9 },
  });
  let clip = changed.drafts[14].clips[1];
  assert.equal(clip.start, 10);
  assert.equal(clip.intensity, 0);
  changed = demoReducer(changed, {
    type: "patchClip",
    id: "back",
    patch: { duration: 0.5 },
  });
  clip = changed.drafts[14].clips[1];
  assert.equal(clip.duration, 0.5);
  assert.ok(clip.fadeIn + clip.fadeOut <= 0.5);
});
test("deleting last selected clip has no dangling selection; undo restores it", () => {
  let state = createDemoState();
  state = demoReducer(state, { type: "deleteClip", id: "back" });
  assert.equal(state.selectedClipId, null);
  state = demoReducer(state, { type: "undo" });
  assert.equal(state.selectedClipId, "back");
  assert.ok(state.drafts[14].clips.some((c) => c.id === "back"));
});
test("add/drop at end remains in range; duplicate id does not overwrite existing clip", () => {
  let state = createDemoState();
  state = demoReducer(state, {
    type: "addClip",
    group: "back",
    at: 20,
    id: "extra",
  });
  const clip = state.drafts[14].clips.at(-1)!;
  assert.equal(clip.start, 16);
  assert.equal(clip.duration, 4);
  assert.equal(
    demoReducer(state, { type: "addClip", group: "wash", at: 0, id: "extra" }),
    state,
  );
});
test("end of sequence does not silently loop; standby selection permits deliberate replay", () => {
  let state = demoReducer(createDemoState(), { type: "standby", id: 15 });
  state = demoReducer(state, { type: "go" });
  assert.equal(state.nextId, null);
  assert.equal(demoReducer(state, { type: "go" }), state);
  state = demoReducer(state, { type: "standby", id: 11 });
  state = demoReducer(state, { type: "go" });
  assert.equal(state.running?.cueId, 11);
  assert.equal(state.nextId, 12);
});
test("display time rounds across seconds and minutes without impossible .10 suffix", () => {
  assert.equal(formatTime(8.4), "00:08.4");
  assert.equal(formatTime(9.99), "00:10.0");
  assert.equal(formatTime(59.99), "01:00.0");
  assert.equal(formatTime(-1), "00:00.0");
});
