import { execFileSync } from "node:child_process";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { sourceRoots, requiredInputs } from "./source-scope.mjs";
const project = fileURLToPath(new URL("../../", import.meta.url));
export function sourceFixture(t) {
  const root = mkdtempSync(join(project, "tmp/desktop-013-source-test-"));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const write = (path, text) => {
    mkdirSync(dirname(join(root, path)), { recursive: true });
    writeFileSync(join(root, path), text);
  };
  const git = (args) =>
    execFileSync("git", args, { cwd: root, encoding: "utf8" }).trim();
  for (const folder of sourceRoots)
    mkdirSync(join(root, folder), { recursive: true });
  for (const file of requiredInputs) write(file, "fixture input\n");
  write("crates/sample/src/lib.rs", "pub const VALUE: u8 = 1;\n");
  write("apps/ui-prototype/src/界面.ts", "export const value = 1;\n");
  git(["init", "-q"]);
  git(["config", "user.name", "Source fixture"]);
  git(["config", "user.email", "fixture@invalid.example"]);
  const commit = () => {
    git(["add", "--all"]);
    git(["-c", "commit.gpgsign=false", "commit", "-q", "-m", "fixture"]);
  };
  commit();
  return { root, write, git, commit };
}
