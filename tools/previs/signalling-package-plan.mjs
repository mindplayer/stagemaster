import { dirname, join } from "node:path";
import { withoutLoaderOverrides } from "./packaging-tools.mjs";

export const signallingTests = [
  "signalling.test.mjs",
  "signalling-discovery.test.mjs",
  "signalling-lifecycle.test.mjs",
];
const expectedNames = [
  "viewer credentials cannot authorize another origin or the renderer",
  "authorized paths are removed before upstream logging",
  "actual loopback service rejects unauthorized upgrades and negotiates official config",
  "a waiting viewer discovers a late renderer without another list request or connection",
  "a renderer ready before the initial directory request cannot double-discover a viewer",
  "a cancelled waiting viewer stays closed; a new viewer discovers the current renderer",
  "父进程 EOF 结束所属信令服务并释放两个真实回环端口",
];

export function signallingEnvironment(environment) {
  return Object.fromEntries(
    Object.entries(withoutLoaderOverrides(environment)).filter(
      ([key]) => !key.startsWith("NODE_") && !/^npm_config_/i.test(key),
    ),
  );
}

export function signallingPackagePlan(
  root,
  node,
  id,
  platform = process.platform,
  arch = process.arch,
) {
  if (platform !== "darwin" || arch !== "arm64")
    throw new Error("本组件组装只支持 Mac ARM64");
  if (!/^previs-signalling-[a-zA-Z0-9]+$/.test(id))
    throw new Error("信令组件实例名称无效");
  const temporary = join(root, "tmp", id);
  const archive = join(root, "data/PREVIS-005", id);
  const logs = join(root, "logs/PREVIS-005", id);
  const bundle = join(archive, "previs");
  const npm = join(dirname(node), "../lib/node_modules/npm/bin/npm-cli.js");
  const license = join(dirname(node), "../LICENSE");
  const config = join(temporary, "user.npmrc");
  const globalConfig = join(temporary, "global.npmrc");
  const env = {
    TMPDIR: temporary,
    NODE_ENV: "production",
    NODE_COMPILE_CACHE: join(temporary, "node-cache"),
  };
  const install = {
    program: node,
    args: [
      npm,
      "ci",
      "--offline",
      "--ignore-scripts",
      "--omit=dev",
      "--global=false",
      "--workspaces=false",
      "--audit=false",
      "--fund=false",
      "--update-notifier=false",
      `--prefix=${bundle}`,
      `--cache=${join(root, "tmp/npm-cache")}`,
      `--logs-dir=${logs}`,
      `--userconfig=${config}`,
      `--globalconfig=${globalConfig}`,
    ],
    log: join(logs, "npm-ci.log"),
  };
  return {
    root,
    node,
    license,
    npm,
    temporary,
    archive,
    logs,
    bundle,
    config,
    globalConfig,
    env,
    install,
  };
}

export function successfulSignallingReport(text) {
  const value = (key) => {
    const values = [...text.matchAll(new RegExp(`^# ${key} (\\d+)$`, "gm"))];
    return values.length === 1 ? Number(values[0][1]) : undefined;
  };
  const names = [...text.matchAll(/^# Subtest: (.+)$/gm)]
    .map((match) => match[1])
    .sort();
  if (
    value("tests") !== 7 ||
    value("pass") !== 7 ||
    ["fail", "cancelled", "skipped", "todo"].some((key) => value(key) !== 0) ||
    names.length !== 7 ||
    names.some((name, i) => name !== [...expectedNames].sort()[i])
  )
    throw new Error("包内信令报告缺项、空或存在未通过测试");
  return names;
}
