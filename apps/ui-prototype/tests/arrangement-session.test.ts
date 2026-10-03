import test from "node:test";
import assert from "node:assert/strict";
import { stageProject } from "./stage-organization-fixture.ts";
import {
  beginArrangement,
  arrangementOperations,
  arrangementPlacements,
  arrangementStage,
} from "../src/components/stage/arrangement-session.ts";

test("opening does not commit; zero movement preserves history and numeric formatting", () => {
  const p = stageProject();
  p.stage.placements[0].positionMeters.x = "0.000";
  const s = beginArrangement(p, ["front"], undefined, true);
  assert.deepEqual(arrangementOperations(p, s), []);
  assert.deepEqual(arrangementOperations(p, { ...s, dirty: true }), []);
});

test("draft movement is from the source, produces one batch's operations, and never mutates the source", () => {
  const p = stageProject(),
    original = structuredClone(p);
  const s = beginArrangement(p, ["front", "audience"], undefined, true);
  s.dirty = true;
  s.draft.dx = "1.25";
  s.draft.dz = "-0.5";
  const values = arrangementPlacements(p, s);
  assert.deepEqual(
    values.map((v) => v.positionMeters.x),
    ["1.25", "21.25"],
  );
  assert.equal(values[0].positionMeters.z, "5.376543");
  assert.deepEqual(arrangementPlacements(p, s), values);
  const ops = arrangementOperations(p, s);
  assert.equal(ops.length, 2);
  assert.deepEqual(ops[0], {
    op: "stage",
    command: { op: "putPlacement", placement: values[0] },
  });
  assert.deepEqual(p, original);
});

test("unplaced draft appears inside the stage and cancellation restores its exact source", () => {
  const p = stageProject();
  p.stage.placements = p.stage.placements.slice(0, 1);
  const s = beginArrangement(
    p,
    ["audience", "loose"],
    p.stage.spaces[0],
    false,
  );
  assert.equal(s.draft.z, "6.5");
  assert.deepEqual(arrangementOperations(p, s), []);
  const values = arrangementPlacements(p, s);
  const drawn = arrangementStage(p.stage, values);
  assert.equal(drawn.placements.length, 3);
  assert.equal(p.stage.placements.length, 1);
  assert.equal(drawn.constructions, p.stage.constructions);
  assert.equal(arrangementStage(p.stage, null), p.stage);
  assert.equal(arrangementOperations(p, { ...s, dirty: true }).length, 2);
});

test("all five arrangement modes retain ordered targets and exact base orientation", () => {
  const p = stageProject();
  for (const mode of ["line", "grid", "circle", "move", "align"] as const) {
    const s = beginArrangement(p, ["audience", "front"], undefined, true);
    s.draft.mode = mode;
    const values = arrangementPlacements(p, s);
    assert.deepEqual(
      values.map((v) => v.fixtureId),
      ["audience", "front"],
    );
    assert.deepEqual(
      values[0].rotationDegreesXYZ,
      p.stage.placements[1].rotationDegreesXYZ,
    );
  }
});

test("invalid or locked membership rejects the whole group without changing source", () => {
  const p = stageProject();
  p.stage.editLocks = [{ kind: "placement", targetId: "audience" }];
  const s = beginArrangement(p, ["front", "audience"], undefined, true);
  assert.throws(() => arrangementPlacements(p, s), /锁定/);
  s.ids = [];
  assert.throws(() => arrangementPlacements(p, s), /请选择/);
  s.ids = ["front", "front"];
  assert.throws(() => arrangementPlacements(p, s), /重复/);
  s.ids = ["missing"];
  assert.throws(() => arrangementPlacements(p, s), /不存在/);
  s.ids = ["front"];
  s.draft.dx = "";
  assert.throws(() => arrangementPlacements(p, s), /有效数字/);
});

test("source geometry, locks, fixture deletion, and project identity cannot silently rebase a pending edit", () => {
  const p = stageProject();
  const s = beginArrangement(p, ["front"], undefined, true);
  s.dirty = true;
  s.draft.dx = "2";
  for (const change of [
    (v: typeof p) => {
      v.stage.placements[0].positionMeters.x = "50";
    },
    (v: typeof p) => {
      v.stage.editLocks = [{ kind: "placement", targetId: "front" }];
    },
    (v: typeof p) => {
      v.fixtures.shift();
    },
    (v: typeof p) => {
      v.id = "another-project";
    },
  ]) {
    const next = structuredClone(p);
    change(next);
    assert.throws(() => arrangementOperations(next, s), /已变化/);
  }
  const nameOnly = { ...p, name: "重新命名" };
  assert.equal(arrangementOperations(nameOnly, s).length, 1);
});
