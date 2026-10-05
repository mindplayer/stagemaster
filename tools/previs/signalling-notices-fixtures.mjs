import { createHash } from "node:crypto";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

export const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
export const write = (file, content) => {
  mkdirSync(dirname(file), { recursive: true });
  writeFileSync(file, content);
};
export function fixture(t) {
  const root = fileURLToPath(new URL("../../", import.meta.url));
  const directory = mkdtempSync(join(root, "tmp/previs-notices-test-"));
  t.after(() => rmSync(directory, { recursive: true, force: true }));
  const bundle = join(directory, "中文 组件"),
    assets = join(directory, "assets");
  const manifest = { dependencies: { a: "1.0.0" } };
  const entry = {
    version: "1.0.0",
    license: "MIT",
    integrity: "sha512-fixture",
    resolved: "https://registry.npmjs.org/a/-/a-1.0.0.tgz",
  };
  const lock = {
    lockfileVersion: 3,
    packages: { "": manifest, "node_modules/a": entry },
  };
  write(join(bundle, "package.json"), JSON.stringify(manifest));
  write(join(bundle, "package-lock.json"), JSON.stringify(lock));
  write(
    join(bundle, "node_modules/a/package.json"),
    JSON.stringify({ name: "a", version: "1.0.0", license: "MIT" }),
  );
  write(join(bundle, "licenses/node-LICENSE"), "fixture Node notice\n");
  write(join(assets, "sources.json"), "[]");
  const text = "fixture copyright and permission\n";
  const catalog = (origin = "embedded-package") => ({
    name: "a",
    version: "1.0.0",
    license: "MIT",
    integrity: entry.integrity,
    origin,
    file: origin === "embedded-package" ? "Readme.md" : "epic-LICENSE.md",
    sha256: hash(text),
    ...(origin === "pinned-upstream"
      ? {
          gitHead: "a".repeat(40),
          sourceUrl: `https://raw.githubusercontent.com/EpicGames/PixelStreamingInfrastructure/${"a".repeat(40)}/LICENSE.md`,
        }
      : {}),
  });
  const setCatalog = (entries) =>
    write(join(assets, "sources.json"), JSON.stringify(entries));
  const embed = () => {
    write(join(bundle, "node_modules/a/Readme.md"), text);
    setCatalog([catalog()]);
  };
  const upstream = () => {
    write(join(assets, "epic-LICENSE.md"), text);
    setCatalog([catalog("pinned-upstream")]);
  };
  return {
    directory,
    bundle,
    assets,
    manifest,
    lock,
    entry,
    text,
    catalog,
    setCatalog,
    embed,
    upstream,
  };
}
