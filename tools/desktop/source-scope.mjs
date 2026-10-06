export const sourceRoots = Object.freeze([
  "crates",
  "apps/desktop",
  "apps/execution-host",
  "apps/ui-prototype",
  "tools/desktop",
  "tools/previs",
]);
export const sourceExtras = Object.freeze([
  "Cargo.toml",
  "Cargo.lock",
  "rust-toolchain.toml",
  ".cargo/config.toml",
  "docs/project-format/schemas/project.schema.json",
  "docs/project-format/schemas/common.schema.json",
]);
export const requiredInputs = [
  "Cargo.toml",
  "Cargo.lock",
  "apps/desktop/tauri.conf.json",
  "apps/ui-prototype/package.json",
  "apps/ui-prototype/package-lock.json",
  ...sourceExtras.filter((p) => p.startsWith("docs/")),
];
export const sourceLimits = Object.freeze({
  files: 4096,
  entries: 16384,
  fileBytes: 64 * 1024 * 1024,
  totalBytes: 256 * 1024 * 1024,
});
const excludedFolders = new Set([
  "node_modules",
  "target",
  "dist",
  "tests",
  "examples",
  "benches",
  "gen",
  "notices",
]);
export function sourcePath(path) {
  if (sourceExtras.includes(path)) return true;
  if (!sourceRoots.some((root) => path.startsWith(root + "/"))) return false;
  const pieces = path.split("/");
  return (
    !pieces.some((p) => p.startsWith(".") || excludedFolders.has(p)) &&
    !/\.(md|test\.mjs|test\.ts|test\.tsx)$/.test(path) &&
    !/(?:^|\/)(?:tests|.*_tests)\.rs$/.test(path) &&
    !/-fixtures?\.mjs$/.test(path)
  );
}
