import { execFileSync } from "node:child_process";
import { realpathSync } from "node:fs";
import { sourceRoots, sourceExtras } from "./source-scope.mjs";

export function sourceGit(root) {
  const git = (args) =>
    execFileSync("git", args, {
      cwd: root,
      encoding: "utf8",
      maxBuffer: 8 * 1024 * 1024,
    });
  if (
    realpathSync(git(["rev-parse", "--show-toplevel"]).trim()) !==
    realpathSync(root)
  )
    throw Error("来源目录必须是本 Git 工作区根目录");
  const head = git(["rev-parse", "HEAD"]).trim();
  const objectFormat = git(["rev-parse", "--show-object-format"]).trim();
  if (
    !["sha1", "sha256"].includes(objectFormat) ||
    !/^(?:[a-f0-9]{40}|[a-f0-9]{64})$/.test(head)
  )
    throw Error("Git 来源身份无效");
  const tree = new Map();
  for (const entry of git([
    "ls-tree",
    "-r",
    "-z",
    "HEAD",
    "--",
    ...sourceRoots,
    ...sourceExtras,
  ])
    .split("\0")
    .filter(Boolean)) {
    const index = entry.indexOf("\t"),
      metadata = entry.slice(0, index).split(" ");
    if (index < 0 || metadata[1] !== "blob") throw Error("来源 Git 条目无效");
    tree.set(entry.slice(index + 1), { mode: metadata[0], oid: metadata[2] });
  }
  return { head, objectFormat, tree };
}
