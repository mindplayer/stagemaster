import test from "node:test";
import assert from "node:assert/strict";
import { stageProject } from "./stage-organization-fixture.ts";
import { outlineMembers } from "../src/components/stage/stage-outliner-model.ts";
import {
  outlineRooms,
  outlineSelectionStatus,
} from "../src/components/stage/stage-outline-navigation.ts";
const project = stageProject(),
  members = outlineMembers(project);
test("空间名称命中保留所属成员，成员名称命中保留祖先，空结果不伪装失去选择", () => {
  const room = outlineRooms(project, members, "表演区").filter(
    (r) => r.visible,
  );
  assert.equal(room.length, 1);
  assert.deepEqual(
    room[0].children.map((m) => m.target.id),
    ["rig", "front"],
  );
  const child = outlineRooms(project, members, "FRONT").filter(
    (r) => r.visible,
  );
  assert.equal(child[0].space?.id, "stage");
  assert.deepEqual(
    child[0].children.map((m) => m.target.id),
    ["front"],
  );
  assert.equal(
    outlineRooms(project, members, "没有这个对象").filter((r) => r.visible)
      .length,
    0,
  );
  assert.deepEqual(
    outlineSelectionStatus(
      project,
      members,
      "没有这个对象",
      { kind: "space", id: "stage" },
      [],
    ),
    { count: 1, hidden: 1, active: { kind: "space", id: "stage" } },
  );
});
test("多灯选择按稳定身份去重，隐藏数量只看筛选且忽略已失效项", () => {
  const selection = { kind: "placement" as const, id: "front" };
  assert.deepEqual(
    outlineSelectionStatus(project, members, "观众区", selection, [
      "front",
      "audience",
      "loose",
      "front",
      "gone",
    ]),
    { count: 3, hidden: 2, active: selection },
  );
  assert.deepEqual(
    outlineSelectionStatus(
      project,
      members,
      "",
      { kind: "construction", id: "gone" },
      [],
    ),
    { count: 0, hidden: 0, active: null },
  );
  assert.equal(
    outlineSelectionStatus(project, members, "前桁架", selection, ["front"])
      .hidden,
    0,
  );
  assert.equal(
    outlineSelectionStatus(
      project,
      members,
      "未归属空间",
      { kind: "placement", id: "loose" },
      ["loose"],
    ).hidden,
    0,
  );
});
