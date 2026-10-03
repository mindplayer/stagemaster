import { test } from "node:test";
import assert from "node:assert/strict";
import { stageProject } from "./stage-organization-fixture.ts";
import {
  selectTarget,
  validSelection,
} from "../src/components/stage/stage-selection.ts";
import {
  affectedTargets,
  translatedObject,
  translationCommand,
  translationPreview,
  translationProblem,
} from "../src/components/stage/object-translation.ts";
import {
  objectTranslationOperations,
  translationSource,
} from "../src/components/stage/object-translation-session.ts";
import { selectedStage } from "../src/stage-tools.ts";
import { outlineSelectionStatus } from "../src/components/stage/stage-outline-navigation.ts";
import { outlineMembers } from "../src/components/stage/stage-outliner-model.ts";
import type { StageSelection } from "../src/stage-types.ts";
const rig: StageSelection = { kind: "construction", id: "rig" };
const lamp: StageSelection = { kind: "placement", id: "front" };
const platform: StageSelection = { kind: "construction", id: "piece-0" };
const targets = [rig, lamp, platform];
test("mixed selection preserves order, toggles and counts search-hidden objects", () => {
  const p = stageProject();
  assert.deepEqual(selectTarget([rig], lamp, true), [rig, lamp]);
  assert.deepEqual(selectTarget(targets, rig, true), [lamp, platform]);
  assert.deepEqual(selectTarget(targets, rig, false, true), targets);
  assert.deepEqual(selectTarget(targets, lamp), [lamp]);
  assert.deepEqual(
    validSelection(p.stage, [
      ...targets,
      rig,
      { kind: "placement", id: "missing" },
    ]),
    targets,
  );
  const status = outlineSelectionStatus(
    p,
    outlineMembers(p),
    "前桁架",
    platform,
    ["front"],
    targets,
  );
  assert.equal(status.count, 3);
  assert.equal(status.hidden, 1);
  assert.deepEqual(status.active, platform);
});
test("preview expands attachments exactly once and preserves unchanged axes and objects", () => {
  const p = stageProject(),
    original = structuredClone(p);
  assert.deepEqual(affectedTargets(p.stage, [rig]), [rig, lamp]);
  assert.deepEqual(affectedTargets(p.stage, targets), targets);
  const preview = translationPreview(p.stage, targets, {
    x: "1.25",
    y: "0",
    z: "-0.25",
  });
  const before = selectedStage(p.stage, lamp)!;
  const after = preview(before);
  assert.equal(after.kind, "placement");
  if (after.kind === "placement")
    assert.deepEqual(after.value.positionMeters, {
      x: "1.25",
      y: "2",
      z: "5.626543",
    });
  const unchanged = selectedStage(p.stage, {
    kind: "placement",
    id: "audience",
  })!;
  assert.equal(preview(unchanged), unchanged);
  const moved = preview(selectedStage(p.stage, platform)!);
  if (moved.kind === "construction" && moved.value.shape.kind === "platform") {
    assert.equal(moved.value.shape.baseElevationMeters, "0.15");
    assert.equal(moved.value.shape.heightMeters, "0.05");
  }
  assert.deepEqual(p, original);
  assert.deepEqual(
    translatedObject(before, { x: "0", y: "-0.000000", z: "0" }),
    before,
  );
});
test("indirect locks, unsupported objects and malformed deltas never produce a partial command", () => {
  const p = stageProject();
  p.stage.editLocks = [{ kind: "placement", targetId: "front" }];
  assert.match(translationProblem(p.stage, [rig, platform]), /锁定/);
  assert.throws(
    () =>
      translationCommand(p.stage, [rig, platform], { x: "1", y: "0", z: "0" }),
    /锁定/,
  );
  p.stage.editLocks = [];
  for (const value of ["", "NaN", "1e2", "200001", "0.0000001", "+1"]) {
    assert.throws(
      () => translationCommand(p.stage, targets, { x: value, y: "0", z: "0" }),
      /X 位移/,
    );
  }
  assert.throws(
    () =>
      translationCommand(p.stage, [{ kind: "space", id: "stage" }, rig], {
        x: "1",
        y: "0",
        z: "0",
      }),
    /空间/,
  );
  assert.throws(
    () => translationCommand(p.stage, [rig, rig], { x: "1", y: "0", z: "0" }),
    /不重复/,
  );
  assert.equal(
    translationCommand(p.stage, targets, { x: "1", y: "0", z: "0" }).targets
      .length,
    3,
  );
});
test("fixed mixed draft commits one command, zero has no edit and changed source is rejected", () => {
  const p = stageProject();
  const draft = {
    source: translationSource(p),
    targets,
    delta: { x: "1.25", y: "0", z: "0" },
  };
  const ops = objectTranslationOperations(p, draft);
  assert.equal(ops.length, 1);
  assert.equal(ops[0].op, "stage");
  assert.deepEqual(
    objectTranslationOperations(p, {
      ...draft,
      delta: { x: "0", y: "0", z: "-0" },
    }),
    [],
  );
  const changed = structuredClone(p);
  changed.stage.placements[0].positionMeters.x = "5";
  assert.throws(
    () => objectTranslationOperations(changed, draft),
    /场地已变化/,
  );
  assert.throws(
    () => objectTranslationOperations({ ...p, id: "another" }, draft),
    /场地已变化/,
  );
});
