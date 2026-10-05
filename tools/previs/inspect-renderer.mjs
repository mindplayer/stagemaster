import { fileURLToPath, pathToFileURL } from "node:url";
import { inspectMacBundle } from "./mac-bundle-inspection.mjs";

export function inspectRenderer(args = process.argv.slice(2)) {
  if (args.length !== 1)
    throw new Error("请只指定一份项目内独立预演 Game 程序路径");
  if (process.platform !== "darwin" || process.arch !== "arm64")
    throw new Error("此资格检查仅支持 Mac ARM64");
  return inspectMacBundle(
    fileURLToPath(new URL("../../", import.meta.url)),
    args[0],
  );
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href)
  try {
    console.log(JSON.stringify(inspectRenderer(), null, 2));
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
