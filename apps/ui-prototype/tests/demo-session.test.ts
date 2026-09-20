import { test } from "node:test";
import assert from "node:assert/strict";
import {
  createDemoState,
  demoReducer,
  hasChanges,
  formatTime,
} from "../src/demo-session.ts";
import { dragClip, pointerTime } from "../src/timeline-interaction.ts";

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

test("small pointer motion stays continuous and is retained after committing", () => {
  const initial = createDemoState();
  const clip = initial.drafts[14].clips[1];
  const first = dragClip(clip, "move", 3, 24.35, 20, [], false).clip;
  const second = dragClip(clip, "move", 5, 24.35, 20, [], false).clip;
  assert.equal(first.start, 4.123);
  assert.equal(second.start, 4.205);
  const state = demoReducer(initial, {
    type: "patchClip",
    id: clip.id,
    patch: second,
  });
  assert.equal(state.drafts[14].clips[1].start, 4.205);
  assert.equal(state.history.length, 1);
  assert.deepEqual(state.running, initial.running);
  assert.deepEqual(demoReducer(state, { type: "undo" }).drafts, initial.drafts);
});

test("magnetic snapping applies only within six screen pixels, at either clip edge", () => {
  const clip = createDemoState().drafts[14].clips[1];
  assert.equal(dragClip(clip, "move", 30, 50, 20, [5], true).clip.start, 4.6);
  const near = dragClip(clip, "move", 47, 50, 20, [5], true);
  assert.equal(near.clip.start, 5);
  assert.equal(near.snapTarget, 5);
  assert.equal(dragClip(clip, "move", 47, 50, 20, [5], false).clip.start, 4.94);
  const rightEdge = dragClip(clip, "move", 47, 50, 20, [15], true);
  assert.equal(rightEdge.clip.start, 5);
  assert.equal(rightEdge.clip.duration, 10);
  // The same 0.1 s distance is outside the magnet at a higher zoom.
  assert.equal(dragClip(clip, "move", 45, 50, 20, [5], true).clip.start, 5);
  assert.equal(dragClip(clip, "move", 135, 150, 20, [5], true).clip.start, 4.9);
});

test("trimming keeps the opposite edge fixed and preserves legal fade lengths", () => {
  const clip = createDemoState().drafts[14].clips[1];
  const left = dragClip(clip, "left", 3, 24.35, 20, [], false).clip;
  assert.equal(left.start, 4.123);
  assert.equal(left.duration, 9.877);
  assert.equal(left.start + left.duration, 14);
  const right = dragClip(clip, "right", -3, 24.35, 20, [], false).clip;
  assert.equal(right.start, 4);
  assert.equal(right.duration, 9.877);
  const tiny = dragClip(clip, "right", -9999, 50, 20, [0], true).clip;
  assert.equal(tiny.start, 4);
  assert.equal(tiny.duration, 0.2);
  assert.ok(tiny.fadeIn + tiny.fadeOut <= tiny.duration);
});

test("dragging beyond the timeline is clamped without changing clip length", () => {
  const clip = createDemoState().drafts[14].clips[1];
  assert.equal(dragClip(clip, "move", -9999, 50, 20, [0], true).clip.start, 0);
  const right = dragClip(clip, "move", 9999, 50, 20, [20], true).clip;
  assert.equal(right.start, 10);
  assert.equal(right.duration, 10);
});

test("scrubbing uses the visible lane geometry including scroll and zoom", () => {
  assert.equal(pointerTime(450, 124, 487, 20), (326 / 487) * 20);
  assert.equal(pointerTime(150, -350, 1000, 20), 10);
  assert.equal(pointerTime(-10, 124, 487, 20), 0);
  assert.equal(pointerTime(900, 124, 487, 20), 20);
});
