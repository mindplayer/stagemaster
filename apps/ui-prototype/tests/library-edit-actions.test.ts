import test from "node:test";
import assert from "node:assert/strict";
import { libraryEditActions } from "../src/library-edit-actions.ts";
import { stageProject } from "./stage-organization-fixture.ts";
import type { EditCommand, Snapshot } from "../src/application-host.ts";
import type { LibraryEdit } from "../src/library-types.ts";

const command: LibraryEdit = {
  kind: "renamePreset",
  id: "preset",
  name: "暖金",
};
function fixture() {
  let current: Snapshot = {
    generation: 42,
    project: stageProject(),
    fileName: "fixture.json",
    dirty: true,
    canUndo: true,
    canRedo: true,
    recovery: { state: "clean", capturedAtMs: null, problem: null },
  };
  const notices: string[] = [],
    errors: string[] = [],
    calls: EditCommand[] = [];
  let prepare = () => {},
    mutation = () => {};
  const action = libraryEditActions(
    async (work) => {
      prepare();
      try {
        await work();
        return true;
      } catch (error) {
        errors.push(String(error));
        return false;
      }
    },
    () => current,
    async (value) => {
      calls.push(value);
      mutation();
    },
    (value) => notices.push(value),
  );
  return {
    action,
    notices,
    errors,
    calls,
    current: () => current,
    set: (next: Snapshot) => {
      current = next;
    },
    mutate: (value: () => void) => {
      mutation = value;
    },
    prepare: (value: () => void) => {
      prepare = value;
    },
  };
}

test("unchanged resource does not borrow existing dirty undo or redo history", async () => {
  const f = fixture(),
    before = f.current();
  assert.equal(await f.action(command), before.project);
  assert.match(f.notices[0], /资源未变化/);
  assert.doesNotMatch(f.notices[0], /可撤销恢复/);
  assert.equal(f.current(), before);
  assert.deepEqual(f.calls, [{ op: "library", command }]);
});
test("host generation change reports a new resource edit and returns its project", async () => {
  const f = fixture();
  f.mutate(() => f.set({ ...f.current(), generation: 43, canRedo: false }));
  assert.equal(await f.action(command), f.current().project);
  assert.equal(f.notices[0], "资源已更新，可撤销恢复");
  assert.equal(f.calls.length, 1);
});
test("preceding draft commit is not mistaken for the following unchanged resource", async () => {
  const f = fixture();
  f.prepare(() => f.set({ ...f.current(), generation: 43 }));
  await f.action(command);
  assert.match(f.notices[0], /资源未变化/);
  assert.equal(f.current().generation, 43);
});
test("host rejection reports no success and never replays the edit", async () => {
  const f = fixture();
  f.mutate(() => {
    throw Error("资源已更换");
  });
  assert.equal(await f.action(command), null);
  assert.deepEqual(f.notices, []);
  assert.match(f.errors[0], /资源已更换/);
  assert.equal(f.calls.length, 1);
});
test("project replacement cannot lend an unrelated generation or project as success", async () => {
  const f = fixture();
  f.mutate(() =>
    f.set({
      ...f.current(),
      project: { ...f.current().project!, id: "another" },
      generation: 43,
    }),
  );
  assert.equal(await f.action(command), null);
  assert.deepEqual(f.notices, []);
  assert.match(f.errors[0], /工程已更换/);
  assert.equal(f.calls.length, 1);
});
test("absent project is rejected before editing or reporting history", async () => {
  const f = fixture();
  f.set({ ...f.current(), project: null });
  assert.equal(await f.action(command), null);
  assert.equal(f.calls.length, 0);
  assert.deepEqual(f.notices, []);
});
