import test from "node:test";
import assert from "node:assert/strict";
import { referenceCapture } from "../src/position-reference.ts";
import type { FixtureView, ProjectView } from "../src/application-host.ts";
function context() {
  const fixture: FixtureView = {
    id: "a",
    name: "光束灯",
    profileId: "p",
    profileName: "模式",
    domainId: "d",
    domainName: "灯光",
    universe: 1,
    address: 1,
    footprint: 5,
    attributes: [],
    positioning: {
      kind: "intersectingOrthogonal",
      pan: { minDegrees: "-270", maxDegrees: "270", reversed: false },
      tilt: { minDegrees: "-135", maxDegrees: "135", reversed: false },
    },
  };
  const project = {
    stage: { placements: [{ fixtureId: "a" }] },
  } as ProjectView;
  return { fixture, project };
}
const draft = { name: " 台口 ", x: "1.5", y: "-2", z: "0.5" };
test("记录只提交目标与身份，轴设定由 Rust 读取", () => {
  const { fixture, project } = context();
  assert.deepEqual(referenceCapture(project, fixture, "scene", draft), {
    op: "position",
    command: {
      op: "captureReference",
      fixtureId: "a",
      sceneId: "scene",
      name: "台口",
      targetMeters: { x: "1.5", y: "-2", z: "0.5" },
    },
  });
});
test("拒绝无模型、无灯位、过期档案与无效草稿", () => {
  const { fixture, project } = context();
  for (const patch of [{ name: " " }, { x: "NaN" }, { z: "100001" }])
    assert.throws(() =>
      referenceCapture(project, fixture, "scene", { ...draft, ...patch }),
    );
  assert.throws(() =>
    referenceCapture(
      project,
      { ...fixture, positioning: null },
      "scene",
      draft,
    ),
  );
  assert.throws(() =>
    referenceCapture(
      { stage: { placements: [] } } as unknown as ProjectView,
      fixture,
      "scene",
      draft,
    ),
  );
  fixture.positionReference = {
    profileId: "old",
    profileRevision: "1",
    compatible: false,
    points: [],
  };
  assert.throws(
    () => referenceCapture(project, fixture, "scene", draft),
    /档案已改变/,
  );
});
