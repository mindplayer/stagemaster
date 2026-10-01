import test from "node:test";
import assert from "node:assert/strict";
import { stageProject } from "./stage-organization-fixture.ts";
import {
  isStageLocked,
  lockTargets,
  movementBlocker,
  placementTargets,
} from "../src/stage-locks.ts";
import { visibleStage } from "../src/components/stage/stage-display.ts";
import { PrevisInteractionScope } from "../src/previs-interaction-scope.ts";
test("空间锁不隐式锁住成员，混合选灯不能部分移动", () => {
  const { stage } = stageProject();
  assert.equal(isStageLocked(stage, { kind: "space", id: "stage" }), false);
  stage.editLocks = lockTargets([
    { kind: "space", id: "stage" },
    { kind: "placement", id: "front" },
  ]);
  assert.equal(
    movementBlocker(stage, placementTargets(["audience", "loose"])),
    null,
  );
  assert.deepEqual(
    movementBlocker(stage, placementTargets(["loose", "front"])),
    { kind: "placement", id: "front" },
  );
  assert.deepEqual(
    movementBlocker(stage, [{ kind: "construction", id: "rig" }]),
    { kind: "placement", id: "front" },
  );
  assert.equal(
    isStageLocked(stage, { kind: "construction", id: "rig" }),
    false,
  );
});
test("隐藏挂灯或围护不能绕过间接移动保护", () => {
  const { stage } = stageProject();
  stage.constructions.push({
    id: "walls",
    name: "围护",
    shape: {
      kind: "enclosure",
      spaceId: "stage",
      wallThicknessMeters: "0.2",
      floorThicknessMeters: "0.1",
      ceilingThicknessMeters: null,
    },
  });
  stage.editLocks = lockTargets([
    { kind: "construction", id: "walls" },
    { kind: "placement", id: "front" },
  ]);
  const shown = visibleStage(stage, {
    hiddenSpaces: [],
    hiddenLayers: ["fixtures", "constructions"],
  });
  assert.equal(shown.editLocks, stage.editLocks);
  assert.deepEqual(movementBlocker(stage, [{ kind: "space", id: "stage" }]), {
    kind: "construction",
    id: "walls",
  });
  assert.deepEqual(
    movementBlocker(stage, [{ kind: "construction", id: "rig" }]),
    { kind: "placement", id: "front" },
  );
});
test("锁定或换选后旧三维提案失效，解锁不能重放旧拖动", () => {
  const scope = new PrevisInteractionScope();
  scope.update("stage:front", true);
  const before = scope.capture();
  scope.update("stage:front", false);
  assert.equal(before(), false);
  scope.update("stage:front", true);
  assert.equal(before(), false);
  const next = scope.capture();
  scope.update("stage:loose", true);
  assert.equal(next(), false);
  assert.equal(scope.capture()(), true);
});
