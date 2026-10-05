import { mkdirSync } from "node:fs";
import { resolve } from "node:path";
import { plainAncestors } from "../previs/desktop-assembly-files.mjs";
import { projectTarget } from "./build-plan.mjs";

export function prepareWorkspaceFolders(root, additional = []) {
  const common = ["tmp", "tmp/cargo-home", "logs/DESKTOP-001"].map(
    (directory) => resolve(root, directory),
  );
  const folders = [
    ...common,
    ...additional.map((directory) => projectTarget(root, directory)),
  ];
  for (const folder of folders) plainAncestors(folder);
  for (const folder of common) mkdirSync(folder, { recursive: true });
}
