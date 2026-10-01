import test from "node:test";
import assert from "node:assert/strict";
import {
  appendGroupMembers,
  removeGroupMembers,
  groupMembersIssue,
} from "../src/group-members.ts";

test("灯组批量加入保持已有灯序、追加顺序和幂等，筛选移出不影响隐藏成员", () => {
  const ids = ["b", "a", "d"],
    add = ["a", "c", "e", "c"];
  assert.deepEqual(appendGroupMembers(ids, add), ["b", "a", "d", "c", "e"]);
  assert.deepEqual(appendGroupMembers(appendGroupMembers(ids, add), add), [
    "b",
    "a",
    "d",
    "c",
    "e",
  ]);
  assert.deepEqual(removeGroupMembers(ids, ["a", "missing", "a"]), ["b", "d"]);
  assert.deepEqual(removeGroupMembers(ids, []), ids);
  assert.deepEqual(ids, ["b", "a", "d"]);
  assert.deepEqual(add, ["a", "c", "e", "c"]);
});
test("灯组保存保护不静默过滤无效成员，边界与去重必须在提交前明确处理", () => {
  const valid = Array.from({ length: 10001 }, (_, i) => ({ id: String(i) }));
  assert.match(groupMembersIssue([], valid), /至少/);
  assert.equal(
    groupMembersIssue(
      valid.slice(0, 10000).map((f) => f.id),
      valid,
    ),
    "",
  );
  assert.match(
    groupMembersIssue(
      valid.map((f) => f.id),
      valid,
    ),
    /10000/,
  );
  assert.match(groupMembersIssue(["missing"], valid), /已删除/);
  assert.match(groupMembersIssue(["1", "1"], valid), /重复/);
  assert.equal(groupMembersIssue(["2", "1"], valid), "");
});
